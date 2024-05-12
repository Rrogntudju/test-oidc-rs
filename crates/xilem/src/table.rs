use masonry::widget::{prelude::*, CrossAxisAlignment, Flex, Label, SizedBox};
use masonry::{Insets, theme, Color, WidgetPod};
use masonry::text2::TextLayout;

use std::iter;
use std::sync::Arc;

const SPACING: f64 = 12.0;
const LAST_SPACING: f64 = SPACING / 2.0;
const SHADING: f64 = 0.1;

pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Default for TableData {
    fn default() -> Self {
        TableData {
            header: vec![],
            rows: vec![vec![]],
        }
    }
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
                    let mut layout = TextLayout::<String>::new(text.clone());
                    layout.rebuild(ctx.text(), env);
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

    if !columns_width.is_empty() {
        Some(columns_width)
    } else {
        None
    }
}

pub struct Table {
    header_text_color: Option<Color>,
    data: TableData,
    inner: WidgetPod<Flex>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            header_text_color: None,
            data: TableData::default(),
            inner: WidgetPod::new(Flex::row()),
        }
    }

    fn build(&mut self, ctx: &mut UpdateCtx, data: &Arc<TableData>, env: &Env) {
        let mut table = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);

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

            let mut header = Flex::row();
            data.header.iter().enumerate().for_each(|(j, col_name)| {
                let mut label = Label::new(col_name.clone());
                if let Some(color) = &self.header_text_color {
                    label.with_text_brush(color.clone());
                }
                header.with_child(SizedBox::new(label).width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
            });
            table.with_child(header);

            data.rows.iter().enumerate().for_each(|(i, row)| {
                let mut table_row = Flex::row();
                row.iter().enumerate().for_each(|(j, text)| {
                    table_row.with_child(SizedBox::new(Label::new(text.clone())).width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
                });
                if i % 2 == 0 {
                    table.with_child(table_row)
                } else {
                    table.with_child(table_row)
                };
            });
        }

        self.inner = WidgetPod::new(table);
    }

    pub fn set_header_text_color(&mut self, color: Color) {
        self.header_text_color = Some(color.into());
    }

    pub fn with_header_text_color(mut self, color: Color>>) -> Self {
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

    fn on_pointer_event(&mut self, ctx: &mut EventCtx<'_>, event: &PointerEvent);
    fn on_text_event(&mut self, ctx: &mut EventCtx<'_>, event: &TextEvent);
    fn on_access_event(&mut self, ctx: &mut EventCtx<'_>, event: &AccessEvent);
    fn on_status_change(
        &mut self,
        ctx: &mut LifeCycleCtx<'_>,
        event: &StatusChange
    );
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx<'_>, event: &LifeCycle);
    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, bc: &BoxConstraints) -> Size;
    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene);
    fn accessibility_role(&self) -> Role;
    fn accessibility(&mut self, ctx: &mut AccessCtx<'_>);
    fn children(&self) -> SmallVec<[WidgetRef<'_, dyn Widget>; 16]>;
}
