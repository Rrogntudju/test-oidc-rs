#![windows_subsystem = "windows"]
use anyhow::{anyhow, Result};
use cosmic_time::{anim, chain, id, Duration, Exponential, Instant, Timeline};
use iced::advanced::image::Handle;
use iced::widget::{button, column, container, radio, row, text, Image};
use iced::window::icon;
use iced::{application, Color, Element, Padding, Subscription, Task, Theme};
use iced::{window, Event, Renderer};
use mode_couleur::{stream_event_mode_couleur, ModeCouleur};
use serde_json::value::Value;
use std::{fmt, iter};
use table::Table;

mod pkce;
mod table;
use pkce::Pkce;

#[cfg_attr(target_os = "linux", path = "nix_mode_couleur.rs")]
#[cfg_attr(target_os = "windows", path = "win_mode_couleur.rs")]
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

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    GetInfos,
    Infos(Result<(Option<Vec<Vec<String>>>, Option<Pkce>), String>),
    ModeCouleurChanged(Result<ModeCouleur, String>),
    Tick(Instant),
}

fn main() -> iced::Result {
    let icon = icon::from_file_data(ICON, None).unwrap();
    let window = window::Settings {
        size: iced_core::Size { width: 900.0, height: 380.0 },
        icon: Some(icon),
        ..Default::default()
    };
    application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(window)
        .run_with(App::new)
}

#[derive(Debug)]
struct App {
    radio_fournisseur: Fournisseur,
    fournisseur: String,
    secret: Option<Pkce>,
    infos: Option<Vec<Vec<String>>>,
    en_traitement: bool,
    erreur: String,
    theme: Theme,
    timeline: Timeline,
    container: id::Container,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                radio_fournisseur: Fournisseur::Microsoft,
                fournisseur: String::new(),
                secret: None,
                infos: None,
                en_traitement: false,
                erreur: String::new(),
                theme: Theme::Light,
                timeline: Timeline::new(),
                container: id::Container::unique(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        "Userinfos".to_owned()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FournisseurChanged(fournisseur) => {
                self.radio_fournisseur = fournisseur;
                Task::none()
            }
            Message::GetInfos => {
                let fournisseur = self.radio_fournisseur.to_string();
                if self.fournisseur != fournisseur {
                    self.secret = None;
                    self.fournisseur = fournisseur;
                }
                let fournisseur = self.radio_fournisseur;
                let secret = self.secret.clone();
                let task = get_infos(fournisseur, secret);
                self.infos = None;
                self.erreur = String::new();
                self.en_traitement = true;
                Task::perform(task, |i| Message::Infos(i.map_err(|e| format!("{e:#}"))))
            }
            Message::Infos(result) => {
                match result {
                    Ok(infos) => {
                        let (infos, secret) = infos;
                        self.secret = secret;
                        self.timeline = Timeline::new();
                        self.infos = infos;
                        let animation = chain![
                            self.container,
                            cosmic_time::container(Duration::ZERO).padding(from([15, 0, 400, 20])),
                            cosmic_time::container(Duration::from_secs(1))
                                .padding(from([15, 0, 0, 20]))
                                .ease(Exponential::Out),
                        ];
                        self.timeline.set_chain(animation).start();
                    }
                    Err(e) => self.erreur = e,
                }
                self.en_traitement = false;
                iced_runtime::window::get_oldest().and_then(|id| window::request_user_attention(id, Some(window::UserAttention::Informational)))
            }
            Message::ModeCouleurChanged(mode) => {
                match mode {
                    Ok(mode) => match mode {
                        ModeCouleur::Clair => self.theme = Theme::Light,
                        ModeCouleur::Sombre => self.theme = Theme::Dark,
                    },
                    Err(e) => self.erreur = e,
                }
                Task::none()
            }
            Message::Tick(now) => {
                self.timeline.now(now);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let image = Image::new(Handle::from_bytes(ICON.as_slice()));

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
                    .collect::<Vec<_>>()
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
                let fournisseur = text(format!("Userinfos {}", &self.fournisseur)).size(24);

                if self.en_traitement {
                    column![fournisseur]
                } else {
                    column![
                        fournisseur,
                        Table::new(data)
                            .font_size(16)
                            .header_color(Color::from_rgb8(255, 165, 0))
                            .cell_padding(Padding::new(5.).left(7).right(3))
                    ]
                }
            }
            _ => column![""],
        };

        let erreur = text(&self.erreur).color([1.0, 0.0, 0.0]);

        container(
            row![
                column![image, titre, fournisseur, bouton, erreur].spacing(10),
                anim!(self.container, &self.timeline, infos)
            ]
            .spacing(20),
        )
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        let mut palette = self.theme.palette();
        palette.primary = Color::from_rgb(1.0_f32, 165.0_f32 / 255.0, 0.0_f32); // orange
        Theme::custom(String::new(), palette)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            stream_event_mode_couleur().map(Message::ModeCouleurChanged),
            self.timeline.as_subscription::<Event>().map(Message::Tick),
        ])
    }
}

async fn get_infos(fournisseur: Fournisseur, secret: Option<Pkce>) -> Result<(Option<Vec<Vec<String>>>, Option<Pkce>)> {
    let secret = match secret {
        Some(pkce) if pkce.is_expired() => Some(Pkce::new(&fournisseur).await?),
        Some(pkce) => Some(pkce),
        None => Some(Pkce::new(&fournisseur).await?),
    };

    let value = ureq::get(fournisseur.userinfos())
        .set("Authorization", &format!("Bearer {}", secret.clone().unwrap().secret()))
        .timeout(std::time::Duration::from_secs(20))
        .call()?
        .into_json::<Value>()?;

    match value {
        Value::Object(map) => {
            let infos: Vec<Vec<String>> = iter::once(vec!["Propriété".to_owned(), "Valeur".to_owned()])
                .chain(map.iter().map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")]))
                .collect();
            Ok((Some(infos), secret))
        }
        _ => Err(anyhow!("La valeur doit être un map")),
    }
}

fn from(p: [u16; 4]) -> Padding {
    Padding {
        top: f32::from(p[0]),
        right: f32::from(p[1]),
        bottom: f32::from(p[2]),
        left: f32::from(p[3]),
    }
}
