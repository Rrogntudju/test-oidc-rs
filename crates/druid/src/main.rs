#![windows_subsystem = "windows"]
use druid::im::Vector;
use druid::widget::{Button, CrossAxisAlignment, Either, Flex, Image, Label, MainAxisAlignment, RadioGroup, Spinner};
use druid::{
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink, Handled, ImageBuf, Lens, Selector, Target, Widget, WidgetExt,
    WindowDesc,
};
use static_init::dynamic;

mod table;
use serde_json::value::Value;
use std::sync::Arc;
use std::{fmt, thread};
use table::{Table, TableData, TableRows};

mod pkce;
use pkce::Pkce;

mod seticon;

const FINISH_GET_USERINFOS: Selector<Result<TableRows, String>> = Selector::new("finish_get_userinfos");
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

#[dynamic]
static mut TOKEN: Option<(Fournisseur, Pkce)> = None;

#[derive(Clone, Data, Lens)]
struct AppData {
    radio_fournisseur: Fournisseur,
    label_fournisseur: String,
    infos: Arc<TableData>,
    en_traitement: bool,
    erreur: String,
}

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

fn request_userinfos(f: &Fournisseur) -> Result<Value, anyhow::Error> {
    let token = TOKEN.read();
    if token.is_some() {
        let (fournisseur, secret) = token.as_ref().unwrap();
        if f != fournisseur || secret.is_expired() {
            drop(token);
            TOKEN.write().replace((f.to_owned(), Pkce::new(f)?));
        }
    } else {
        drop(token);
        TOKEN.write().replace((f.to_owned(), Pkce::new(f)?));
    }

    Ok(ureq::get(f.userinfos())
        .set("Authorization", &format!("Bearer {}", TOKEN.read().as_ref().unwrap().1.secret()))
        .call()?
        .into_json::<Value>()?)
}

fn get_userinfos(sink: ExtEventSink, fournisseur: Fournisseur) {
    thread::spawn(move || {
        let result = match request_userinfos(&fournisseur) {
            Ok(value) => match value {
                Value::Object(map) => {
                    let table = map
                        .iter()
                        .map(|(k, v)| vec![k.to_owned(), v.to_string().replace('"', "")])
                        .collect::<TableRows>();
                    Ok(table)
                }
                _ => Err("La valeur doit être un map".to_string()),
            },
            Err(e) => Err(e.to_string()),
        };

        sink.submit_command(FINISH_GET_USERINFOS, result, Target::Auto)
            .expect("command failed to submit");
    });
}

struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(&mut self, _ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut AppData, _env: &Env) -> Handled {
        match cmd.get(FINISH_GET_USERINFOS) {
            Some(Ok(infos)) => {
                data.en_traitement = false;
                data.infos = Arc::new(TableData {
                    rows: infos.to_owned(),
                    header: vec!["Propriété".to_owned(), "Valeur".to_owned()],
                });
                Handled::Yes
            }
            Some(Err(e)) => {
                data.en_traitement = false;
                data.erreur = e.to_owned();
                Handled::Yes
            }
            None => Handled::No,
        }
    }
}

fn ui_builder() -> impl Widget<AppData> {
    let mut oidc = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center);

    let png_data = ImageBuf::from_data(include_bytes!("openid-icon-100x100.png")).unwrap();
    oidc.add_child(Image::new(png_data));
    oidc.add_child(Label::new("OpenID Connect").with_text_size(25.));
    oidc.add_default_spacer();

    oidc.add_child(Label::new("Fournisseur:"));
    oidc.add_default_spacer();
    let mut fournisseurs = Vector::new();
    fournisseurs.push_back((Fournisseur::Microsoft.to_string(), Fournisseur::Microsoft));
    fournisseurs.push_back((Fournisseur::Google.to_string(), Fournisseur::Google));
    oidc.add_child(RadioGroup::row(fournisseurs).lens(AppData::radio_fournisseur));
    oidc.add_default_spacer();

    let bouton = Button::new("UserInfos")
        .on_click(|ctx, data: &mut AppData, _| {
            data.erreur = String::new();
            data.label_fournisseur = data.radio_fournisseur.to_string();
            data.en_traitement = true;
            get_userinfos(ctx.get_external_handle(), data.radio_fournisseur.clone());
        })
        .fix_height(30.0);

    oidc.add_child(Either::new(|data, _env| data.en_traitement, Spinner::new(), bouton));

    let infos = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            Label::new(|data: &AppData, _env: &_| format!("UserInfos {}", data.label_fournisseur))
                .with_text_size(18.)
                .with_text_color(Color::from_hex_str("FFA500").unwrap()),
        )
        .with_default_spacer()
        .with_child(
            Table::new()
                .with_header_text_color(Color::from_hex_str("FFA500").unwrap())
                .lens(AppData::infos),
        );

    let main = Flex::row().with_default_spacer().with_child(oidc).with_spacer(40.).with_child(infos);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(main)
        .with_default_spacer()
        .with_child(
            Flex::row().with_default_spacer().with_child(
                Label::new(|data: &AppData, _env: &_| data.erreur.clone())
                    .with_text_color(Color::rgb(1., 0., 0.))
                    .expand_width(),
            ),
        )
    //    .debug_paint_layout()
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder()).title("UserInfos").window_size((1100., 600.));
    let data = AppData {
        radio_fournisseur: Fournisseur::Microsoft,
        label_fournisseur: String::new(),
        infos: Arc::new(TableData::default()),
        en_traitement: false,
        erreur: String::new(),
    };

    seticon::set_window_icon(1, "druid", "Userinfos"); // Temporary workaround for title bar icon issue

    AppLauncher::with_window(main_window)
        .delegate(Delegate {})
        .launch(data)
        .expect("launch failed");
}
