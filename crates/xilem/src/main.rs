use std::sync::Arc;

use accesskit::{DefaultActionVerb, Role};
use masonry::app_driver::{AppDriver, DriverCtx};
use masonry::vello::Scene;
use masonry::widget::{Align, CrossAxisAlignment, MainAxisAlignment, Flex, Label, RootWidget, SizedBox, WidgetRef};
use masonry::{
    AccessCtx, AccessEvent, Action, BoxConstraints, Color, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, PointerEvent, Size,
    StatusChange, TextEvent, Widget, WidgetId, WidgetPod,
};
use anyhow::{anyhow, Result};
use winit::dpi::LogicalSize;
use winit::window::Window;

mod table;
use serde_json::value::Value;
use std::{fmt, thread};
use table::{Table, TableData};

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

#[derive(Clone)]
struct AppData {
    radio_fournisseur: Fournisseur,
    label_fournisseur: String,
    infos: TableData,
    en_traitement: bool,
    erreur: String,
}

impl AppData {
    async fn get_infos(fournisseur: Fournisseur, secret: Option<Pkce>) -> Result<(Option<TableData>, Option<Pkce>)> {
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
}

#[derive(Clone, PartialEq)]
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

fn ui_builder() -> impl Widget {
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
                .lens(AppData:: infos),
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

impl AppDriver for AppData {
    fn on_action(&mut self, ctx: &mut DriverCtx<'_>, _widget_id: WidgetId, action: Action) {
        match action {
            Action::Other(payload) => match payload.downcast_ref::<CalcAction>().unwrap() {
                CalcAction::Digit(digit) => self.digit(*digit),
                CalcAction::Op(op) => self.op(*op),
            },
            _ => unreachable!(),
        }

        ctx.get_root::<RootWidget<Flex>>()
            .get_element()
            .child_mut(1)
            .unwrap()
            .downcast::<Label>()
            .set_text(&*self.value);
    }
}



pub fn main() {
    let window_size = LogicalSize::new(223., 300.);

    let window_attributes = Window::default_attributes()
        .with_title("Simple Calculator")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let calc_state = CalcState {
        value: "0".to_string(),
        operand: 0.0,
        operator: 'C',
        in_num: false,
    };

    masonry::event_loop_runner::run(window_attributes, RootWidget::new(build_calc()), calc_state).unwrap();
}
