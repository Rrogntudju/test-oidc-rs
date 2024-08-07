//use masonry::vello::peniko::{Format, Image as ImageBuf};
//use masonry::widget::{CrossAxisAlignment, FillStrat, Image, MainAxisAlignment};
use masonry::Color;
use xilem::view::{button, checkbox, flex, label};
use xilem::Axis;

use anyhow::{anyhow, Result};
use winit::dpi::LogicalSize;
use winit::window::Window;
use xilem::{MasonryView, Xilem};

mod table;
use serde_json::value::Value;
use std::{fmt, sync::Arc};
use table::{table, TableData};

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
    secret: Option<Pkce>,
    infos: Arc<TableData>,
    //    en_traitement: bool,
    erreur: String,
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

fn app_logic(data: &mut AppData) -> impl MasonryView<AppData> {
    let oidc = flex((
        label("OpenID Connect").color(Color::ORANGE),
        label("Fournisseurs:").color(Color::ORANGE),
        checkbox(
            "Microsoft / Google",
            data.radio_fournisseur == Fournisseur::Microsoft,
            |data: &mut AppData, checked| {
                if checked {
                    data.radio_fournisseur = Fournisseur::Microsoft;
                    data.label_fournisseur = "Microsoft".to_string();
                } else {
                    data.radio_fournisseur = Fournisseur::Google;
                    data.label_fournisseur = "Google".to_string();
                }
            },
        ),
        button("Userinfos", |data: &mut AppData| {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let infos = rt.block_on(get_infos(data.radio_fournisseur.clone(), data.secret.clone()));
            match infos {
                Ok((infos, secret)) => {
                    data.infos = Arc::new(infos.expect("infos absentes"));
                    data.secret = secret;
                }
                Err(err) => {
                    data.erreur = err.to_string();
                    data.infos = Arc::new(TableData::default());
                }
            }
        }),
    ))
    .direction(Axis::Vertical);

    let infos = flex((
        label(format!("Userinfos {}", data.label_fournisseur)).color(Color::ORANGE),
        table(data.infos.clone()).header_text_brush(Color::ORANGE),
    ))
    .direction(Axis::Vertical);

    flex((
        flex((oidc, infos)).direction(Axis::Horizontal),
        label(data.erreur.clone()).color(Color::RED),
    ))
    .direction(Axis::Vertical)
}

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

pub fn main() {
    let window_size = LogicalSize::new(1100., 600.);

    let window_attributes = Window::default_attributes()
        .with_title("Userinfos")
        .with_resizable(true)
        .with_min_inner_size(window_size);

    let data = AppData {
        radio_fournisseur: Fournisseur::Microsoft,
        label_fournisseur: String::new(),
        secret: None,
        infos: Arc::new(TableData::default()),
        //        en_traitement: false,
        erreur: String::new(),
    };

    let app = Xilem::new(data, app_logic);
    app.run_windowed_in(window_attributes).unwrap();
}
