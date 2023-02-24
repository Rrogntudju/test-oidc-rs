use iced::widget::{button, column, container, radio, row, text, Image, Column};
use iced::{alignment, Application, Color, Command, Element, Length, Settings, Theme};
use iced::{executor, Renderer};
use static_init::dynamic;
use std::fmt;
use anyhow::Error;

mod pkce;
use pkce::Pkce;

const ID_MS: &str = include_str!("../../secrets/clientid.microsoft");
const SECRET_MS: &str = include_str!("../../secrets/secret.microsoft");
const ID_GG: &str = include_str!("../../secrets/clientid.google");
const SECRET_GG: &str = include_str!("../../secrets/secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[dynamic]
static mut TOKEN: Option<(Fournisseur, Pkce)> = None;

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
    infos: Option<TableData>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    GetInfos,
    Infos((Option<TableData>, String)),
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
                infos: None,
                en_traitement: false,
                erreur: "ipsum lorem".to_owned(),
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
                let task = async { (None, "DOH!".to_owned()) };
                Command::perform(task, |i| Message::Infos(i))
            }
            Message::Infos((infos, erreur)) => {
                self.infos = infos;
                self.erreur = erreur;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let image = Image::new("openid-icon-100x100.png");

        let titre = text("OpenID Connect")
            .size(48)
            .style(Color::from([1.0, 0.5, 0.2]));

        let fournisseur = column![
            text("Fournisseur:"),
            column(
                [Fournisseur::Microsoft, Fournisseur::Google]
                    .into_iter()
                    .map(|fournisseur| radio(format!("{fournisseur}"), fournisseur, Some(self.radio_fournisseur), Message::FournisseurChanged))
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

        let infos = column![text(&self.radio_fournisseur).size(24), table(&self.infos)].spacing(10);
        let erreur = text(&self.erreur).style(Color::from([1.0, 0.0, 0.0]));

        container(row![column![image, titre, fournisseur, bouton, erreur].spacing(10), infos].spacing(10))
        .padding(20)
        .center_y()
        .into()
    }
}

fn table<'a>(_data: &Option<TableData>) -> Column<'a, Message, Renderer> {
    column![text("LOL"), text("LOL"), text("LOL"), text("LOL"), text("LOL")]
}
