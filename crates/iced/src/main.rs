use anyhow::{anyhow, Result};
use iced::widget::{button, column, container, radio, row, text, Image};
use iced::{executor, window, Renderer};
use iced::{Application, Color, Command, Element, Settings, Theme};
use serde_json::value::Value;
use std::fmt;
use window::icon;

mod pkce;
use pkce::Pkce;

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
const ICON: &[u8; 41662] = include_bytes!("../icon.ico");

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
            size: (1000, 400),
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
}

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    GetInfos,
    Secret(Result<Option<Pkce>, String>),
    Infos(Result<Option<TableData>, String>),
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
                let fournisseur = self.radio_fournisseur.clone();
                let secret = self.secret.clone();
                let task = async move { get_secret(fournisseur, secret) };
                self.erreur = String::new();
                self.en_traitement = true;
                Command::perform(task, |s| Message::Secret(s.map_err(|e| e.to_string())))
            }
            Message::Secret(result) => match result {
                Ok(secret) => {
                    let fournisseur = self.radio_fournisseur.clone();
                    self.secret = secret.clone();
                    let task = async move { get_userinfos(fournisseur, secret) };
                    Command::perform(task, |i| Message::Infos(i.map_err(|e| e.to_string())))
                }
                Err(erreur) => {
                    self.erreur = erreur;
                    self.en_traitement = false;
                    Command::none()
                }
            },
            Message::Infos(result) => {
                match result {
                    Ok(infos) => self.infos = infos,
                    Err(erreur) => self.erreur = erreur,
                }
                self.en_traitement = false;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let image = Image::new("openid-icon-100x100.png");

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
                let r1 = text(format!("Userinfos {}", &self.radio_fournisseur))
                    .size(24)
                    .style(Color::from_rgb8(255, 165, 0));
                let r2 = {
                    let mut c1 = column![text(&data.header[0]).style(Color::from_rgb8(255, 165, 0))];
                    let mut c2 = column![text(&data.header[1]).style(Color::from_rgb8(255, 165, 0))];

                    for row in &data.rows {
                        c1 = c1.push(text(row[0].to_owned()).size(18));
                        c2 = c2.push(text(row[1].to_owned()).size(18));
                    }

                    row![c1.spacing(5).padding([0, 10, 0, 0]), c2.spacing(5)]
                };

                column![r1, r2].spacing(10)
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
        let mut palette = Theme::Dark.palette();
        palette.primary = Color::from_rgb(255.0 as f32 / 255.0, 165 as f32 / 255.0, 0.0 as f32 / 255.0);
        Theme::custom(palette)
    }
}

fn get_secret(fournisseur: Fournisseur, secret: Option<Pkce>) -> Result<Option<Pkce>> {
    let secret = match secret {
        Some(pkce) if pkce.is_expired() => Some(Pkce::new(&fournisseur)?),
        Some(pkce) => Some(pkce),
        None => Some(Pkce::new(&fournisseur)?),
    };
    Ok(secret)
}

fn get_userinfos(fournisseur: Fournisseur, secret: Option<Pkce>) -> Result<Option<TableData>> {
    let value = ureq::get(fournisseur.userinfos())
        .set("Authorization", &format!("Bearer {}", secret.unwrap().secret()))
        .call()?
        .into_json::<Value>()?;

    match value {
        Value::Object(map) => {
            let infos = map
                .iter()
                .map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")])
                .collect::<TableRows>();
            let table = TableData {
                rows: infos.to_owned(),
                header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
            };
            Ok(Some(table))
        }
        _ => Err(anyhow!("La valeur doit être un map")),
    }
}
