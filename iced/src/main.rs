use std::fmt;
use iced::widget::{container, button, radio, text, column, row};
use iced::{executor, Renderer};
use iced::{Color, Element, Length, Application, Settings, Theme, Command, alignment};
use iced_native::widget::image::Image;
use iced_native::image::Handle;
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

#[derive(Debug, Clone)]
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


#[derive(Debug, Clone, PartialEq, Eq)]
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
    infos: Option<TableData>,
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
            Table::new(self.infos, Message::TableChanged)
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

mod table {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};
    use super::TableData;


    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    pub struct Table<Message> {
        data: Option<TableData>,
        on_change: Box<dyn Fn(Option<TableData>) -> Message>,
    }

    impl<Message> Table<Message> {
        pub fn new(
            data: Option<TableData>,
            on_change: impl Fn(Option<TableData>) -> Message + 'static,
        ) -> Self {
            Self {
                data,
                on_change: Box::new(on_change),
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
        type State = ();
        type Event = Event;

        fn update(
            &mut self,
            _state: &mut Self::State,
            event: Event,
        ) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_add(1),
                ))),
                Event::DecrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_sub(1),
                ))),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(None))
                    } else {
                        value
                            .parse()
                            .ok()
                            .map(Some)
                            .map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(Length::Fixed(50.0))
                .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u32::to_string)
                        .as_deref()
                        .unwrap_or(""),
                    Event::InputChanged,
                )
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_items(Alignment::Fill)
            .spacing(10)
            .into()
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
