use std::fmt;
use std::sync::Arc;
use iced::widget::{container, button, radio, text, column, row};
use iced::{executor, Renderer};
use iced::{Color, Element, Length, Application, Settings, Theme, Command, alignment};
use iced_native::widget::image::Image;
use iced_native::image::Handle;
use iced_native::widget::Column;
use static_init::dynamic;

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

#[derive(Debug)]
struct TableData {
    header: TableHeader,
    rows: TableRows,
}

use table::Table;

fn infos<Message>(
    table: Option<TableData>,
    on_change: impl Fn(Option<TableData>) -> Message + 'static,
) -> Table<Message> {
    Table::new(table, on_change)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fournisseur {
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

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Debug, Clone)]
struct App {
    radio_fournisseur: Fournisseur,
    infos: Arc<Option<TableData>>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Debug, Clone)]
enum Message {
    FournisseurChanged(Fournisseur),
    TableChanged(Option<TableData>),
    Userinfos,
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { radio_fournisseur: Fournisseur::Microsoft, infos: None, en_traitement: false, erreur: String::new()}, Command::none())
    }

    fn title(&self) -> String {
        "Userinfos".to_owned()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FournisseurChanged(fournisseur) => {
                self.radio_fournisseur = fournisseur;
            }
            Message::TableChanged(table) => {
                self.infos = table;
            }

        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let image = Image::<Handle>::new("openid-icon-100x100.png");

        let titre = text("OpenID Connect")
                    .width(Length::Fill)
                    .size(100)
                    .style(Color::from([0.5, 0.5, 0.5]))
                    .horizontal_alignment(alignment::Horizontal::Center);

        let fournisseur =  column![
                    text("Fournisseur:").size(24),
                    column(
                        [Fournisseur::Microsoft, Fournisseur::Google]
                            .iter()
                            .map(|fournisseur| {
                                radio(
                                    format!("{fournisseur}"),
                                    fournisseur,
                                    Some(fournisseur),
                                    |f: &Fournisseur| Message::FournisseurChanged(*f),
                                )
                            })
                            .map(Element::from)
                            .collect()
                    )
                    .spacing(10)
                ]
                .padding(20)
                .spacing(10);

        let mut bouton = button("Userinfos");
        if !self.en_traitement {
            bouton.on_press(Message::Userinfos);
        }

        let infos = column![
            text(self.radio_fournisseur).size(24),
            table(self.infos)
        ];

        let erreur = text(self.erreur)
                .width(Length::Fill)
                .size(100);

        container(row![ column![image, titre, fournisseur, bouton], column![infos], erreur])
            .padding(20)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

fn table(data: Option<TableData>) -> Column<'static, Message, Renderer> {
    "".into()
}

mod table {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, text };
    use iced::{Element, Length};
    use iced_lazy::{self, Component};
    use super::TableData;

    type Event = ();

    pub struct Table<Message> {
        data: Option<TableData>,
    }

    impl<Message> Table<Message> {
        pub fn new(
            data: Option<TableData>,
        ) -> Self {
            Self {
                data,
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for Table<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            "".into()
        }
    }

    impl<'a, Message, Renderer> From<Table<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(table: Table<Message>) -> Self {
            iced_lazy::component(table)
        }
    }
}
