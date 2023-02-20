use iced::widget::container;
use iced::{executor, Renderer};
use iced::{Element, Length, Application, Settings, Theme, Command};

use numeric_input::numeric_input;

mod pkce;
use pkce::Pkce;

const FINISH_GET_USERINFOS: Selector<Result<TableRows, String>> = Selector::new("finish_get_userinfos");
const ID_MS: &str = include_str!("clientid.microsoft");
const SECRET_MS: &str = include_str!("secret.microsoft");
const ID_GG: &str = include_str!("clientid.google");
const SECRET_GG: &str = include_str!("secret.google");
const AUTH_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const AUTH_GG: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_MS: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const TOKEN_GG: &str = "https://oauth2.googleapis.com/token";
const INFOS_MS: &str = "https://graph.microsoft.com/oidc/userinfo";
const INFOS_GG: &str = "https://openidconnect.googleapis.com/v1/userinfo";

#[dynamic]
static mut TOKEN: Option<(Fournisseur, Pkce)> = None;

#[derive(Clone, PartialEq, Data)]
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

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

#[derive(Default)]
struct App {
    radio_fournisseur: Fournisseur,
    label_fournisseur: String,
    infos: Arc<TableData>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    NumericInputChanged(Option<u32>),
}

impl Application for App {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { value: None }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Userinfos")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::NumericInputChanged(value) => {
                self.value = value;
            }
        };

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        container(numeric_input(self.value, Message::NumericInputChanged))
            .padding(20)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

mod numeric_input {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::{self, Component};

    pub struct NumericInput<Message> {
        value: Option<u32>,
        on_change: Box<dyn Fn(Option<u32>) -> Message>,
    }

    pub fn numeric_input<Message>(
        value: Option<u32>,
        on_change: impl Fn(Option<u32>) -> Message + 'static,
    ) -> NumericInput<Message> {
        NumericInput::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumericInput<Message> {
        pub fn new(
            value: Option<u32>,
            on_change: impl Fn(Option<u32>) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumericInput<Message>
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

    impl<'a, Message, Renderer> From<NumericInput<Message>>
        for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme: widget::button::StyleSheet
            + widget::text_input::StyleSheet
            + widget::text::StyleSheet,
    {
        fn from(numeric_input: NumericInput<Message>) -> Self {
            iced_lazy::component(numeric_input)
        }
    }
}
