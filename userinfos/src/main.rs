use druid::{im::{Vector}, widget::LabelText};
use druid::widget::{Button, CrossAxisAlignment, Flex, Image, Label, List, Scroll, RadioGroup};
use druid::{AppLauncher, Color, Data, ImageBuf, Lens, UnitPoint, Widget, WidgetExt, WindowDesc};

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

#[derive(Clone, Data)]
struct Info {
    valeur: String,
    propriete: String,
}

fn ui_builder() -> impl Widget<AppData> {
    let mut oidc = Flex::column();

    let png_data = ImageBuf::from_data(include_bytes!("openid-icon-100x100.png")).unwrap();
    oidc.add_child(Image::new(png_data.clone()));
    let mut titre = Label::new("OpenID Connect");
    titre.set_text_size(30.);
    oidc.add_child(titre);
    oidc.add_default_spacer();

    oidc.add_child(Label::new("Fournisseur:"));
    let mut fournisseurs = Vector::new();
    fournisseurs.push_back(("Microsoft".to_string(), Fournisseur::Microsoft));
    fournisseurs.push_back(("Google".to_string(), Fournisseur::Google));
    oidc.add_child(RadioGroup::new(fournisseurs).lens(AppData::fournisseur));

    oidc.add_child(
        Button::new("UserInfos")
            .on_click(|_, data: &mut AppData, _| {
               data.erreur = "LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOL".into();
            })
            .fix_height(30.0)
    );

    oidc.add_child(Label::new(|data: &AppData, _env: &_| data.erreur.clone()));

    let mut infos = Flex::column();
    infos.add_child(
        Label::new(|data: &AppData, _env: &_| {
            let f = match data.fournisseur {
                Fournisseur::Microsoft => "Microsoft",
                Fournisseur::Google => "Google",
            };
            format!("UserInfos {}", f)
        })
    );

    infos.add_flex_child(
        Scroll::new(
            List::new(|| {
                    Label::new(|info: &Info,  _env: &_| format!("{}    {}", info.valeur, info.propriete))
            })
            .with_spacing(10.),
        )
        .vertical()
        .lens(AppData::infos),
        1.0,
    );

    let root = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);
    root.with_spacer(20.).with_child(oidc).with_spacer(40.).with_child(infos).debug_paint_layout()
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder).title("UserInfos");
    let mut infos = Vector::new();
    let info  = Info {
        valeur: "Name".to_string(),
        propriete: "LOOOOOOOOOOOOOOOOOOOOOOOOOL!".to_string(),
    };
    infos.push_back(info);
    let data = AppData {
        fournisseur: Fournisseur::Microsoft,
        erreur: String::new(),
        infos,
    };
    AppLauncher::with_window(main_window)
        .launch(data)
        .expect("launch failed");
}

