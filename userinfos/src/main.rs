use druid::im::{vector, Vector};
use druid::lens::{self, LensExt};
use druid::widget::{Button, CrossAxisAlignment, Flex, Label, List, Scroll};
use druid::{
    AppLauncher, Color, Data, Lens, UnitPoint, Widget, WidgetExt, WindowDesc,
};

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

pub fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .title("UserInfos");
    let data = AppData {
        fournisseur: Fournisseur::Microsoft,
        erreur: String::new(),
        infos: Vector::new(),
    };
    AppLauncher::with_window(main_window)
    //    .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppData> {
    let mut root = Flex::column();

    // Build a button to add children to both lists
    root.add_child(
        Button::new("Add")
            .on_click(|_, data: &mut AppData, _| {
                // Add child to left list
                data.l_index += 1;
                data.left.push_back(data.l_index as u32);

                // Add child to right list
                data.r_index += 1;
                data.right.push_back(data.r_index as u32);
            })
            .fix_height(30.0)
            .expand_width(),
    );

    let mut lists = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);

    // Build a simple list
    lists.add_flex_child(
        Scroll::new(List::new(|| {
            Label::new(|item: &u32, _env: &_| format!("List item #{}", item))
                .align_vertical(UnitPoint::LEFT)
                .padding(10.0)
                .expand()
                .height(50.0)
                .background(Color::rgb(0.5, 0.5, 0.5))
        }))
        .vertical()
        .lens(AppData::left),
        1.0,
    );

    // Build a list with shared data
    lists.add_flex_child(
        Scroll::new(
            List::new(|| {
                Flex::row()
                    .with_child(
                        Label::new(|(_, item): &(Vector<u32>, u32), _env: &_| {
                            format!("List item #{}", item)
                        })
                        .align_vertical(UnitPoint::LEFT),
                    )
                    .with_flex_spacer(1.0)
                    .with_child(
                        Button::new("Delete")
                            .on_click(|_ctx, (shared, item): &mut (Vector<u32>, u32), _env| {
                                // We have access to both child's data and shared data.
                                // Remove element from right list.
                                shared.retain(|v| v != item);
                            })
                            .fix_size(80.0, 30.0)
                            .align_vertical(UnitPoint::CENTER),
                    )
                    .padding(10.0)
                    .background(Color::rgb(0.5, 0.0, 0.5))
                    .fix_height(50.0)
            })
            .with_spacing(10.),
        )
        .vertical()
        .lens(lens::Identity.map(
            // Expose shared data with children data
            |d: &AppData| (d.right.clone(), d.right.clone()),
            |d: &mut AppData, x: (Vector<u32>, Vector<u32>)| {
                // If shared data was changed reflect the changes in our AppData
                d.right = x.0
            },
        )),
        1.0,
    );

    root.add_flex_child(lists, 1.0);

    root.with_child(Label::new("horizontal list"))
        .with_child(
            Scroll::new(
                List::new(|| {
                    Label::new(|item: &u32, _env: &_| format!("List item #{}", item))
                        .padding(10.0)
                        .background(Color::rgb(0.5, 0.5, 0.0))
                        .fix_height(50.0)
                })
                .horizontal()
                .with_spacing(10.)
                .lens(AppData::left),
            )
            .horizontal(),
        )
    //    .debug_paint_layout()
}
// let png_data = ImageBuf::from_data(include_bytes!("./assets/PicWithAlpha.png")).unwrap();

