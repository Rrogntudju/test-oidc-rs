use masonry::vello::peniko::{Format, Image as ImageBuf};
use masonry::widget::{CrossAxisAlignment, FillStrat, Image, MainAxisAlignment};
use masonry::Color;
use xilem::view::{button, checkbox, flex, label};
use xilem::Axis;

use anyhow::{anyhow, Result};
use winit::dpi::LogicalSize;
use winit::window::Window;
use xilem::{MasonryView, Xilem};

mod table;
use serde_json::value::Value;
use std::fmt;
use table::TableData;

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
    infos: TableData,
    en_traitement: bool,
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
    let mut oidc = flex((
        label("OpenID Connect").color(Color::ORANGE),
        label("Fournisseurs:").color(Color::ORANGE),
        checkbox("Microsoft / Google", true, |data: &mut AppData, checked| {
            if checked {
                data.radio_fournisseur = Fournisseur::Microsoft;
                data.label_fournisseur = "Microsoft".to_string();
            } else {
                data.radio_fournisseur = Fournisseur::Google;
                data.label_fournisseur = "Google".to_string();
            }
         }),
        button("Userinfos", |data: &mut AppData| {

        })
    )).direction(Axis::Vertical);
    oidc
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
        infos: TableData::default(),
        en_traitement: false,
        erreur: String::new(),
    };

    let app = Xilem::new(data, app_logic);
    app.run_windowed_in(window_attributes).unwrap();
}