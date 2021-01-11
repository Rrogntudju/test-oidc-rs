use druid::im::Vector;
use druid::lens::LensExt;
use druid::widget::{Button, CrossAxisAlignment, Either, Flex, Image, Label, List, MainAxisAlignment, RadioGroup, Scroll};
use druid::Key;
use druid::{
    lens, theme, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, ExtEventSink, FontDescriptor, FontFamily, Handled, ImageBuf, Lens,
    Selector, Target, Widget, WidgetExt, WindowDesc,
};
use std::thread;

const LIST_TEXT_COLOR: Key<Color> = Key::new("rrogntudju.list-text-color");
const FINISH_GET_USERINFOS: Selector<Vector<Info>> = Selector::new("finish_get_userinfos");
const SESSION: &str = "";
const CSRF: &str = "";

#[derive(Clone, Data, Lens)]
struct AppData {
    fournisseur: Fournisseur,
    infos: Vector<Info>,
    en_traitement: bool,
    erreur: String,
}

#[derive(Clone, PartialEq, Data)]
enum Fournisseur {
    Microsoft,
    Google,
}

#[derive(Clone, PartialEq, Data)]
struct Info {
    propriete: String,
    valeur: String,
}

fn get_userinfos(sink: ExtEventSink, number: u32) {
    thread::spawn(move || {
       

        sink.submit_command(FINISH_GET_USERINFOS, number, Target::Auto)
            .expect("command failed to submit");
    });
}

struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(&mut self, _ctx: &mut DelegateCtx, _target: Target, cmd: &Command, data: &mut AppData, _env: &Env) -> Handled {
        if let Some(number) = cmd.get(FINISH_GET_USERINFOS) {
             data.processing = false;
            data.infos = *number;
            Handled::Yes
        } else {
            Handled::No
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
    fournisseurs.push_back(("Microsoft".to_string(), Fournisseur::Microsoft));
    fournisseurs.push_back(("Google".to_string(), Fournisseur::Google));
    oidc.add_child(RadioGroup::new(fournisseurs).lens(AppData::fournisseur));
    oidc.add_default_spacer();

    oidc.add_child(
        Button::new("UserInfos")
            .on_click(|_, data: &mut AppData, _| {
                data.erreur = "LOOOO OOOOOOOOO OOOOOOOOO OOOOOOOL".into();
            })
            .fix_height(30.0),
    );

    let infos = Flex::column()
        .must_fill_main_axis(true)
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .main_axis_alignment(MainAxisAlignment::Center)
        .with_child(
            Label::new(|data: &AppData, _env: &_| {
                let f = match data.fournisseur {
                    Fournisseur::Microsoft => "Microsoft",
                    Fournisseur::Google => "Google",
                };
                format!("UserInfos {}", f)
            })
            .with_text_size(18.)
            .with_text_color(Color::from_hex_str("FFA500").unwrap()),
        )
        .with_default_spacer()
        .with_child(
            Scroll::new(List::new(|| {
                Label::new(|(infos, info): &(Vector<Info>, Info), _: &Env| {
                    let propriete_col_len = infos.into_iter().map(|info| info.propriete.len()).max().unwrap();
                    format!("{:2$}    {}", info.propriete, info.valeur, propriete_col_len)
                })
                .with_font(FontDescriptor::new(FontFamily::MONOSPACE))
                .with_text_size(16.)
                .with_text_color(LIST_TEXT_COLOR)
                .env_scope(|env: &mut druid::Env, (infos, info): &(Vector<Info>, Info)| {
                    let label_color = env.get(theme::LABEL_COLOR);
                    if (infos.index_of(info).unwrap() % 2) == 0 {
                        env.set(LIST_TEXT_COLOR, label_color.with_alpha(0.75));
                    } else {
                        env.set(LIST_TEXT_COLOR, label_color);
                    }
                })
            }))
            .vertical()
            .lens(lens::Identity.map(
                |data: &AppData| (data.infos.clone(), data.infos.clone()),
                |_: &mut AppData, _: (Vector<Info>, Vector<Info>)| (),
            )),
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
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder).title("UserInfos").window_size((1000., 200.));
    let mut infos = Vector::new();
    
    let data = AppData {
        fournisseur: Fournisseur::Microsoft,
        infos,
        en_traitement: false,
        erreur: String::new(),
    };
    AppLauncher::with_window(main_window).delegate(Delegate {}).launch(data).expect("launch failed");
}
