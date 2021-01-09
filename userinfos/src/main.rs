use druid::im::Vector;
use druid::widget::{Button, CrossAxisAlignment, Flex, Image, Label, List, MainAxisAlignment, RadioGroup, Scroll};
use druid::{AppLauncher, Color, Data, ImageBuf, Lens, Widget, WidgetExt, WindowDesc, Env, theme, lens, FontFamily, FontDescriptor};
use druid::lens::{LensExt};

#[derive(Clone, Data, Lens)]
struct AppData {
    fournisseur: Fournisseur,
    erreur: String,
    infos: Vector<Info>,
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
        )
        .with_default_spacer()
        .with_child(
            Scroll::new(
                List::new(||
                    Label::new(|(infos, info): &(Vector<Info>, Info), env: &Env| {
                        let propriete_col_len = infos.into_iter().map(|info| info.propriete.len()).max().unwrap() + 3;
                        let even_item = (infos.index_of(info).unwrap() % 2) == 0;
                        if even_item {
                            if let Ok(dark) = env.try_get(theme::BACKGROUND_DARK) {

                            }

                            if let Ok(light) = env.try_get(theme::BACKGROUND_LIGHT) {
                                
                            }

                        }
                        format!("{:2$}{}", info.propriete, info.valeur, propriete_col_len)
                    })
                    .with_font(FontDescriptor::new(FontFamily::MONOSPACE))
                    .with_text_size(16.)
                )
//                    .background(Color::rgb(0.3, 0.3, 0.3))
            )
            .vertical()
            .lens(lens::Identity.map(
                |data: &AppData| (data.infos.clone(), data.infos.clone()),
                |_: &mut AppData, _: (Vector<Info>, Vector<Info>)| 
                    ()
                )
            )
        );

    let main = Flex::row()
        .with_default_spacer()
        .with_child(oidc)
        .with_spacer(40.)
        .with_child(infos);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(main)
        .with_default_spacer()
        .with_child(
            Flex::row()
                .with_default_spacer()
                .with_child(
                    Label::new(|data: &AppData, _env: &_| data.erreur.clone())
                        .with_text_color(Color::rgb(1., 0., 0.))
                        .expand_width()
                )
        )
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder).title("UserInfos").window_size((1000., 200.));
    let mut infos = Vector::new();
    let info1 = Info {
        valeur: "Name".to_string(),
        propriete: "LOOOOOOoooOOOOOO   OOOOOOOL!".to_string(),
    };
    let info2 = Info {
        valeur: "Address".to_string(),
        propriete: "lOOOO   OOOOOOOO  iiOOOOOOL!".to_string(),
    };
    infos.push_back(info1);
    infos.push_back(info2);
    let data = AppData {
        fournisseur: Fournisseur::Microsoft,
        erreur: String::new(),
        infos,
    };
    AppLauncher::with_window(main_window).launch(data).expect("launch failed");
}
