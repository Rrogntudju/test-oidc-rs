use iced::widget::{button, column, container, radio, row, text, Image};
use iced::{executor, window, Renderer};
use iced::{Application, Color, Command, Element, Settings, Theme};
use serde_json::value::Value;
use std::fmt;

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
    App::run(Settings::default())
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
    Infos((Option<TableData>, Option<Pkce>, String)),
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
            window::resize(1000, 400),
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
                let pkce = self.secret.clone();
                let task = async move { get_userinfos(fournisseur, pkce) };
                self.en_traitement = true;
                Command::perform(task, |i| Message::Infos(i))
            }
            Message::Infos((infos, secret, erreur)) => {
                self.infos = infos;
                self.secret = secret;
                self.erreur = erreur;
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
                    ))
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
}

fn request_userinfos(f: &Fournisseur, secret: Option<Pkce>) -> Result<(Value, Option<Pkce>), anyhow::Error> {
    let secret = match secret {
        Some(pkce) if pkce.is_expired() => Some(Pkce::new(f)?),
        Some(pkce) => Some(pkce),
        None => Some(Pkce::new(f)?),
    };

    Ok((
        ureq::get(f.userinfos())
            .set("Authorization", &format!("Bearer {}", secret.as_ref().unwrap().secret()))
            .call()?
            .into_json::<Value>()?,
        secret,
    ))
}

fn get_userinfos(fournisseur: Fournisseur, secret: Option<Pkce>) -> (Option<TableData>, Option<Pkce>, String) {
    match request_userinfos(&fournisseur, secret) {
        Ok((value, secret)) => match value {
            Value::Object(map) => {
                let infos = map
                    .iter()
                    .map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")])
                    .collect::<TableRows>();
                let table = TableData {
                    rows: infos.to_owned(),
                    header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
                };
                (Some(table), secret, String::new())
            }
            _ => (None, secret, "La valeur doit être un map".to_owned()),
        },
        Err(e) => (None, None, e.to_string()),
    }
}
