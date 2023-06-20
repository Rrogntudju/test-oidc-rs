#![windows_subsystem = "windows"]
use anyhow::{anyhow, Result};
use iced::theme::Container;
use iced::widget::{button, column, container, radio, row, text, Image};
use iced::{executor, window, Font, Renderer};
use iced::{Application, Color, Command, Element, Settings, Theme};
use iced_native::command::Action;
use iced_native::image::Handle;
use iced_native::window::Action as WAction;
use iced_native::Subscription;
use mode_couleur::{stream_event_mode_couleur, ModeCouleur};
use serde_json::value::Value;
use std::{fmt, iter};
use window::icon;

mod pkce;
use pkce::Pkce;

mod mode_couleur;

const ID_MS: &str = include_str!("../../../secrets/clientid.microsoft");
const SECRET_MS: &str = include_str!("../../../secrets/secret.microsoft");
const ID_GG: &str = include_str!("../../../secrets/clientid.google");
const SECRET_GG: &str = include_str!("../../../secrets/secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const ICON: &[u8; 1612] = include_bytes!("../openid.png");
const MONO: &[u8; 111108] = include_bytes!("../lucon.ttf");

type TableColumns = Vec<String>;
type TableRows = Vec<TableColumns>;
type TableHeader = Vec<String>;

#[derive(Debug, Clone)]
struct TableData {
    header: TableHeader,
    rows: TableRows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fournisseur {
    Microsoft,
    Google,
}

impl fmt::Display for Fournisseur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fournisseur = match self {
            Fournisseur::Microsoft => "Microsoft",
            Fournisseur::Google => "Google",
        };
        f.write_str(fournisseur)
    }
}

impl Fournisseur {
    fn endpoints(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (AUTH_MS, TOKEN_MS),
            Self::Google => (AUTH_GG, TOKEN_GG),
        }
    }

    fn secrets(&self) -> (&str, &str) {
        match self {
            Self::Microsoft => (ID_MS, SECRET_MS),
            Self::Google => (ID_GG, SECRET_GG),
        }
    }

    fn userinfos(&self) -> &str {
        match self {
            Self::Microsoft => INFOS_MS,
            Self::Google => INFOS_GG,
        }
    }
}

fn main() -> iced::Result {
    let icon = icon::from_file_data(ICON, None).unwrap();
    let settings = Settings {
        window: window::Settings {
            size: (950, 400),
            icon: Some(icon),
            ..Default::default()
        },
        ..Default::default()
    };
    App::run(settings)
}

#[derive(Debug)]
struct App {
    radio_fournisseur: Fournisseur,
    secret: Option<Pkce>,
    infos: Option<TableData>,
    en_traitement: bool,
    erreur: String,
    mode: ModeCouleur,
    mono: Font,
}

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    GetInfos,
    Infos(Result<(Option<TableData>, Option<Pkce>), String>),
    ModeCouleurChanged(Result<ModeCouleur, String>),
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                radio_fournisseur: Fournisseur::Microsoft,
                secret: None,
                infos: None,
                en_traitement: false,
                erreur: String::new(),
                mode: ModeCouleur::Clair,
                mono: Font::External {
                    name: "Lucida Console",
                    bytes: MONO,
                },
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Userinfos".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FournisseurChanged(fournisseur) => {
                self.radio_fournisseur = fournisseur;
                self.secret = None;
                Command::none()
            }
            Message::GetInfos => {
                let fournisseur = self.radio_fournisseur;
                let secret = self.secret.clone();
                let task = async move { get_infos(fournisseur, secret) };
                self.erreur = String::new();
                self.en_traitement = true;
                Command::perform(task, |i| Message::Infos(i.map_err(|e| format!("{e:#}"))))
            }
            Message::Infos(result) => {
                match result {
                    Ok(infos) => (self.infos, self.secret) = infos,
                    Err(e) => self.erreur = e,
                }
                self.en_traitement = false;
                Command::single(Action::Window(WAction::GainFocus))
            }
            Message::ModeCouleurChanged(mode) => {
                match mode {
                    Ok(mode) => self.mode = mode,
                    Err(e) => self.erreur = e,
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let image = Image::new(Handle::from_memory(ICON));

        let titre = text("OpenID Connect").size(26);

        let fournisseur = column![
            text("Fournisseur:"),
            column(
                [Fournisseur::Microsoft, Fournisseur::Google]
                    .into_iter()
                    .map(|fournisseur| radio(
                        format!("{fournisseur}"),
                        fournisseur,
                        Some(self.radio_fournisseur),
                        Message::FournisseurChanged
                    )
                    .size(18))
                    .map(Element::from)
                    .collect()
            )
            .spacing(5)
        ]
        .spacing(10);

        let bouton = if !self.en_traitement {
            button("Userinfos").on_press(Message::GetInfos)
        } else {
            button("Userinfos")
        };

        let infos = match &self.infos {
            Some(data) => {
                let titre = text(format!("Userinfos {}", &self.radio_fournisseur))
                    .size(24)
                    .style(Color::from_rgb8(255, 165, 0));

                let count = data
                    .rows
                    .iter()
                    .chain(iter::once(&data.header))
                    .fold(vec![0; data.header.len()], |acc, row| {
                        acc.iter()
                            .zip(row.iter())
                            .map(|(max, s)| {
                                let count = s.chars().count();
                                if count > *max {
                                    count
                                } else {
                                    *max
                                }
                            })
                            .collect()
                    });

                let entêtes = row![
                    container(text(stretch(&data.header[0], count[0] + 1)).font(self.mono).size(12)),
                    container(text(stretch(&data.header[1], count[1])).font(self.mono).size(12)),
                ]
                .padding([5, 0, 5, 0]);

                let mut infos = column![];
                let mut flip = false;

                for row in &data.rows {
                    let info = row![
                        container(text(stretch(&row[0], count[0] + 1)).size(12).font(self.mono)).style(style(flip)),
                        container(text(stretch(&row[1], count[1])).size(12).font(self.mono)).style(style(flip)),
                    ]
                    .padding([5, 0, 0, 0]);
                    infos = infos.push(info);
                    flip = !flip;
                }
                column![titre, entêtes, infos]
            }
            _ => column![""],
        };

        let erreur = text(&self.erreur).style(Color::from([1.0, 0.0, 0.0]));

        container(row![
            column![image, titre, fournisseur, bouton, erreur].spacing(10),
            infos.padding([15, 0, 0, 20])
        ])
        .padding([10, 0, 0, 10])
        .into()
    }

    fn theme(&self) -> Self::Theme {
        let mut palette = match self.mode {
            ModeCouleur::Sombre => Theme::Dark.palette(),
            ModeCouleur::Clair => Theme::Light.palette(),
        };
        palette.primary = Color::from_rgb(1.0_f32, 165.0_f32 / 255.0, 0.0_f32);
        Theme::custom(palette)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        stream_event_mode_couleur().map(Message::ModeCouleurChanged)
    }
}

fn get_infos(fournisseur: Fournisseur, secret: Option<Pkce>) -> Result<(Option<TableData>, Option<Pkce>)> {
    let secret = match secret {
        Some(pkce) if pkce.is_expired() => Some(Pkce::new(&fournisseur)?),
        Some(pkce) => Some(pkce),
        None => Some(Pkce::new(&fournisseur)?),
    };
    let value = ureq::get(fournisseur.userinfos())
        .set("Authorization", &format!("Bearer {}", secret.clone().unwrap().secret()))
        .call()?
        .into_json::<Value>()?;

    match value {
        Value::Object(map) => {
            let infos = map
                .iter()
                .map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")])
                .collect::<TableRows>();
            let table = TableData {
                rows: infos,
                header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
            };
            Ok((Some(table), secret))
        }
        _ => Err(anyhow!("La valeur doit être un map")),
    }
}

fn stretch(s: &str, w: usize) -> String {
    format!("{}{}", s, " ".repeat(w - s.chars().count()))
}

fn style(flip: bool) -> iced::theme::Container {
    if flip {
        Container::Box
    } else {
        Container::default()
    }
}
