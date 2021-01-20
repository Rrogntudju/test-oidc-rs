use druid::im::Vector;
use druid::lens::LensExt;
use druid::widget::{Button, CrossAxisAlignment, Either, Flex, Image, Label, List, MainAxisAlignment, RadioGroup, Scroll, Spinner, Painter, ClipBox};
use druid::Key;
use druid::{
    lens, theme, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink, Handled, ImageBuf, Lens,
    Selector, Target, Widget, WidgetExt, WindowDesc, RenderContext,
};
use std::{fmt, thread};
use std::error::Error;
use minreq;
use serde_json::value::Value;

const LIST_TEXT_COLOR: Key<Color> = Key::new("rrogntudju.list-text-color");
const FINISH_GET_USERINFOS: Selector<Result<Vector<Info>, String>> = Selector::new("finish_get_userinfos");
const ORIGINE: &str = "http://localhost";
const SESSION: &str = "a8aadY8XEGSKz8QtXRqOMPugDbwqI2cf";
const CSRF: &str = "YdMTXDrWbHbBoW4daYoexory1Q4RtzuJYpBZgSg0VaaCU07ubDGp4JbI9wRD8QaO";

#[derive(Clone, Data, Lens)]
struct AppData {
    radio_fournisseur: Fournisseur,
    label_fournisseur: String,
    infos: Vector<Info>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Clone, PartialEq, Data)]
enum Fournisseur {
    Microsoft,
    Google,
}

impl fmt::Display for Fournisseur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fournisseur = match &self {
            Fournisseur::Microsoft => "Microsoft",
            Fournisseur::Google => "Google",
        };
        f.write_str(fournisseur)
    }
}

#[derive(Clone, PartialEq, Data)]
struct Info {
    propriete: String,
    valeur: String,
}

fn request_userinfos(fournisseur: &Fournisseur) -> Result<Value, Box<dyn Error>> {
    Ok(minreq::post(format!("{}{}", ORIGINE, "/userinfos"))
        .with_header("Content-Type", "application/json")
        .with_header("Cookie", format!("Session-Id={}; Csrf-Token={}", SESSION, CSRF))
        .with_header("X-Csrf-Token", CSRF)
        .with_body(format!(r#"{{ "fournisseur": "{}", "origine": "{}" }}"#, fournisseur, ORIGINE))
        .with_timeout(10)
        .send()?
        .json()?
    )
}
fn get_userinfos(sink: ExtEventSink, fournisseur: Fournisseur) {
    thread::spawn(move || {
        let result = match request_userinfos(&fournisseur) {
            Ok(value) => {
               let infos: Vector<Info> = value.as_array().unwrap_or(&Vec::<Value>::new()).iter()
                    .map(|value| {
                        Info {
                            propriete: value["propriété"].as_str().unwrap_or_default().to_owned(),
                            valeur: value["valeur"].to_string().trim_matches('"').to_owned(),
                        }
                    })
                    .collect();
                Ok(infos)
            },
            Err(e) => Err(e.to_string())
        };

        sink.submit_command(FINISH_GET_USERINFOS, result, Target::Auto)
            .expect("command failed to submit");
    });
}

fn set_list_text_color(env: &mut Env, infos: &Vector<Info>, info: &Info) {
    let label_color = env.get(theme::LABEL_COLOR);
    if (infos.index_of(info).unwrap() % 2) == 0 {
        env.set(LIST_TEXT_COLOR, label_color.with_alpha(0.75));
    } else {
        env.set(LIST_TEXT_COLOR, label_color);
    }
}
struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(&mut self, _ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut AppData, _env: &Env) -> Handled {
        match cmd.get(FINISH_GET_USERINFOS) {
            Some(Ok(infos)) => {
                data.en_traitement = false;
                data.infos = infos.to_owned();
                Handled::Yes
            },
            Some(Err(e)) => {
                data.en_traitement = false;
                data.erreur = e.to_string();
                Handled::Yes
            },
            None => Handled::No
        }
    }
}

fn ui_builder() -> impl Widget<AppData> {
    let mut oidc = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center);

    let png_data = ImageBuf::from_data(include_bytes!("openid-icon-100x100.png")).unwrap();
    oidc.add_child(Image::new(png_data.clone()));
    oidc.add_child(Label::new("OpenID Connect").with_text_size(25.));
    oidc.add_default_spacer();

    oidc.add_child(Label::new("Fournisseur:"));
    oidc.add_default_spacer();
    let mut fournisseurs = Vector::new();
    fournisseurs.push_back((Fournisseur::Microsoft.to_string(), Fournisseur::Microsoft));
    fournisseurs.push_back((Fournisseur::Google.to_string(), Fournisseur::Google));
    oidc.add_child(RadioGroup::new(fournisseurs).lens(AppData::radio_fournisseur));
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
    
    /* let my_painter = Painter::new(|ctx, _data: &_, env| {
        let bounds = ctx.size().to_rect();

    
            ctx.fill(bounds, &Color::from_hex_str("FFA500").unwrap());
        }
    ); */

    let table = /* Scroll::new( */Flex::row()
        .with_child(List::new(|| {
            Label::new(|(_infos, info): &(Vector<Info>, Info), _: &Env| info.propriete.clone())
                .background(Color::from_hex_str("FFA500").unwrap())
                .env_scope(|env: &mut Env, (infos, info): &(Vector<Info>, Info)| {
                    set_list_text_color(env, infos, info);
                })
            })
        )
        .with_default_spacer()
        .with_child(List::new(|| {
            Label::new(|(_infos, info): &(Vector<Info>, Info), _: &Env| info.valeur.clone())
                .with_text_color(LIST_TEXT_COLOR)
                .env_scope(|env: &mut Env, (infos, info): &(Vector<Info>, Info)| {
                    set_list_text_color(env, infos, info);
                })
            })
        )
//    )
//    .vertical()
    .lens(lens::Identity.map(
        |data: &AppData| (data.infos.clone(), data.infos.clone()),
        |_: &mut AppData, _: (Vector<Info>, Vector<Info>)| (),
    ));

    let infos = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            Label::new(|data: &AppData, _env: &_| {
                format!("UserInfos {}", data.label_fournisseur)
            })
            .with_text_size(18.)
            .with_text_color(Color::from_hex_str("FFA500").unwrap()),
        )
        .with_default_spacer()
        .with_child(Either::new(|data, _env| data.en_traitement, Spinner::new(), table));

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
        ).debug_paint_layout()
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder).title("UserInfos").window_size((1100., 200.));
    let infos = Vector::new();
    
    let data = AppData {
        radio_fournisseur: Fournisseur::Microsoft,
        label_fournisseur: String::new(),
        infos,
        en_traitement: false,
        erreur: String::new(),
    };
    AppLauncher::with_window(main_window).delegate(Delegate {}).launch(data).expect("launch failed");
}
