#![windows_subsystem = "windows"]
use anyhow::{anyhow, Result};
use cosmic_time::{anim, chain, id, Duration, Exponential, Instant, Timeline};
use iced::advanced::image::Handle;
use iced::widget::{button, column, container, radio, row, text, Image};
use iced::window::icon;
use iced::{executor, window, Event, Renderer};
use iced::{Application, Color, Command, Element, Settings, Subscription, Theme};
use mode_couleur::{stream_event_mode_couleur, ModeCouleur};
use once_cell::sync::Lazy;
use serde_json::value::Value;
use std::fmt;
use table::{Table, TableData};

mod table;
mod pkce;
use pkce::Pkce;

#[cfg_attr(target_os = "linux", path = "linux_mode_couleur.rs")]
#[cfg_attr(windows, path = "windows_mode_couleur.rs")]
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

static CONTAINER: Lazy<id::Container> = Lazy::new(id::Container::unique);

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
            size: (880, 380),
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
    fournisseur: String,
    secret: Option<Pkce>,
    infos: Option<TableData>,
    en_traitement: bool,
    erreur: String,
    mode: ModeCouleur,
    timeline: Timeline,
}

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    GetInfos,
    Infos(Result<(Option<TableData>, Option<Pkce>), String>),
    ModeCouleurChanged(Result<ModeCouleur, String>),
    Tick(Instant),
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
                fournisseur: String::new(),
                secret: None,
                infos: None,
                en_traitement: false,
                erreur: String::new(),
                mode: ModeCouleur::Clair,
                timeline: Timeline::new(),
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
                Command::none()
            }
            Message::GetInfos => {
                let fournisseur = self.radio_fournisseur.to_string();
                if self.fournisseur != fournisseur {
                    self.secret = None;
                    self.fournisseur = fournisseur;
                }
                let fournisseur = self.radio_fournisseur;
                let secret = self.secret.clone();
                let task = async move { get_infos(fournisseur, secret) };
                self.erreur = String::new();
                self.en_traitement = true;
                Command::perform(task, |i| Message::Infos(i.map_err(|e| format!("{e:#}"))))
            }
            Message::Infos(result) => {
                match result {
                    Ok(infos) => {
                        let prec = self.infos.clone();
                        (self.infos, self.secret) = infos;
                        self.timeline = Timeline::new();
                        let animation = if prec != self.infos {
                            chain![
                                CONTAINER,
                                cosmic_time::container(Duration::ZERO).padding([15, 0, 0, 200]),
                                cosmic_time::container(Duration::from_millis(600))
                                    .padding([15, 0, 0, 20])
                                    .ease(Exponential::Out),
                            ]
                        } else {
                            chain![CONTAINER, cosmic_time::container(Duration::ZERO).padding([15, 0, 0, 20]),]
                        };
                        self.timeline.set_chain(animation).start();
                    }
                    Err(e) => self.erreur = e,
                }
                self.en_traitement = false;
                Command::single(iced_runtime::command::Action::Window(window::Action::GainFocus))
            }
            Message::ModeCouleurChanged(mode) => {
                match mode {
                    Ok(mode) => self.mode = mode,
                    Err(e) => self.erreur = e,
                }
                Command::none()
            }
            Message::Tick(now) => {
                self.timeline.now(now);
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
                let titre = text(format!("Userinfos {}", &self.fournisseur))
                    .size(24)
                    .style(Color::from_rgb8(255, 165, 0));

                column![titre, Table::new(data).size(16)]
            }
            _ => column![""],
        };

        let erreur = text(&self.erreur).style(Color::from([1.0, 0.0, 0.0]));

        container(
            row![
                column![image, titre, fournisseur, bouton, erreur].spacing(10),
                anim!(CONTAINER, &self.timeline, infos)
            ]
            .spacing(10),
        )
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
        Subscription::batch([
            stream_event_mode_couleur().map(Message::ModeCouleurChanged),
            self.timeline.as_subscription::<Event>().map(Message::Tick),
        ])
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
            let infos: Vec<Vec<String>> = map.iter().map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")]).collect();
            let table = TableData {
                rows: infos,
                header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
            };
            Ok((Some(table), secret))
        }
        _ => Err(anyhow!("La valeur doit être un map")),
    }
}
