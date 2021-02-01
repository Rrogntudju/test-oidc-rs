use druid::widget::{prelude::*, Flex, Label};
use druid::{theme, Color, WidgetExt, WidgetPod};
use druid::{Insets, KeyOrValue, TextLayout};
use std::sync::Arc;
use std::iter;

const SPACING: f64 = 12.0;
const LAST_SPACING: f64 = SPACING / 2.0;
const SHADING: f64 = 0.1;

pub type TableColumns = Vec<String>;
pub type TableRows = Vec<TableColumns>;
pub type TableHeader = Vec<String>;
pub struct TableData {
    pub header: TableHeader,
    pub rows: TableRows,
}

// Find out the maximum layout width of each column
fn layout_columns_width(ctx: &mut UpdateCtx, data: &Arc<TableData>, env: &Env) -> Option<Vec<f64>> {
    let mut columns_width = Vec::new();
    for j in 0_usize.. {
        let mut end_of_cols = true;
        let mut max_width = 0.0;

        data.rows.iter().chain(iter::once(&data.header)).for_each(|row| {
            if let Some(text) = row.get(j) {
                end_of_cols = false;
                if !text.is_empty() {
                    let mut layout = TextLayout::<String>::from_text(text.clone());
                    layout.rebuild_if_needed(ctx.text(), env);
                    let width = layout.size().width;
                    if width > max_width {
                        max_width = width;
                    }
                }
            }
        });

        if end_of_cols {
            break;
        } else {
            columns_width.push(max_width);
        }
    }

    if columns_width.len() > 0 {
        Some(columns_width)
    } else {
        None
    }
}

pub struct Table {
    header_text_color: Option<KeyOrValue<Color>>,
    inner: WidgetPod<Arc<TableData>, Box<dyn Widget<Arc<TableData>>>>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            header_text_color: None,
            inner: WidgetPod::new(Flex::row().boxed()),
        }
    }

    fn build(&mut self, ctx: &mut UpdateCtx, data: &Arc<TableData>, env: &Env) {
        let mut table = Flex::<Arc<TableData>>::column();

        if let Some(widths) = layout_columns_width(ctx, data, env) {
            let last_col = widths.len() - 1;
            let (r, g, b, a) = env.get(theme::WINDOW_BACKGROUND_COLOR).as_rgba();
            let shade = if r + g + b < 1.5 {
                Color::rgba(
                    (r + SHADING).clamp(0.0, 1.0),
                    (g + SHADING).clamp(0.0, 1.0),
                    (b + SHADING).clamp(0.0, 1.0),
                    a,
                )
            } else {
                Color::rgba(
                    (r - SHADING).clamp(0.0, 1.0),
                    (g - SHADING).clamp(0.0, 1.0),
                    (b - SHADING).clamp(0.0, 1.0),
                    a,
                )
            };

            let mut header = Flex::<Arc<TableData>>::row();
            data.header.iter().enumerate().for_each(|(j, col_name)| {
                let mut label = Label::new(col_name.clone());
                if let Some(color) = &self.header_text_color {
                    label.set_text_color(color.clone());
                }
                header.add_child(label.fix_width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
            });
            table.add_child(header.padding(Insets::new(0.0, 0.0, 0.0, 5.0)));

            data.rows.iter().enumerate().for_each(|(i, row)| {
                let mut table_row = Flex::<Arc<TableData>>::row();
                row.iter().enumerate().for_each(|(j, text)| {
                    table_row.add_child(Label::new(text.clone()).fix_width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
                });
                if i % 2 == 0 {
                    table.add_child(table_row.background(Color::from(shade.clone())))
                } else {
                    table.add_child(table_row)
                };
            });
        }

        self.inner = WidgetPod::new(Box::new(table));
    }

    pub fn set_header_text_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.header_text_color = Some(color.into());
    }

    pub fn with_header_text_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.set_header_text_color(color);
        self
    }
}

// If this widget has any child widgets it should call its event, update and layout
// (and lifecycle) methods as well to make sure it works. Some things can be filtered,
// but a general rule is to just pass it through unless you really know you don't want it.
impl Widget<Arc<TableData>> for Table {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Arc<TableData>, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Arc<TableData>, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Arc<TableData>, data: &Arc<TableData>, env: &Env) {
        if !old_data.same(data) {
            self.build(ctx, data, env);
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Arc<TableData>, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<TableData>, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
