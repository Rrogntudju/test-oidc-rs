use druid::widget::{prelude::*, Flex, Label};
use druid::TextLayout;
use druid::{theme, Color, WidgetExt, WidgetPod};
use std::sync::Arc;

const SPACING: f64 = 6.0;
const LAST_SPACING: f64 = SPACING / 3.0;
const SHADING: f64 = 0.1;

pub type TableColumns = Vec<String>;
pub type TableRows = Vec<TableColumns>;
pub type TableHeader = Vec<String>;
pub struct TableData {
    pub header: TableHeader,
    pub rows: TableRows,
}
pub struct Table {
    columns_width: Vec<f64>,
    cw_layout_pass: bool,
    inner: WidgetPod<Arc<TableData>, Box<dyn Widget<Arc<TableData>>>>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            columns_width: Vec::new(),
            cw_layout_pass: false,
            inner: WidgetPod::new(Label::new("LOL")).boxed(),
        }
    }

    fn build_table(&mut self, data: &Arc<TableData>, env: &Env) {
        let last_col = self.columns_width.len() - 1;
        let (r, g, b, a) = env.get(theme::WINDOW_BACKGROUND_COLOR).as_rgba();
        let shade = if r + g + b > 1.5 {
            Color::rgba(r - SHADING, g - SHADING, b - SHADING, a)
        } else {
            Color::rgba(r + SHADING, g + SHADING, b + SHADING, a)
        };

        let mut table = Flex::<Arc<TableData>>::column();

        let mut header = Flex::<Arc<TableData>>::row();
        let mut idx_col = 0_usize;
        for col_name in &data.header {
            header.add_child(
                Label::new(col_name.to_owned()).fix_width(self.columns_width[idx_col] + (if idx_col == last_col { LAST_SPACING } else { SPACING })),
            );
            idx_col += 1;
        }
        table.add_child(header);

        let mut idx_row = 0_usize;
        for row in &data.rows {
            let mut idx_col = 0_usize;
            for text in row {
                let mut table_row = Flex::<Arc<TableData>>::row();
                table_row.add_child(
                    Label::new(text.to_owned()).fix_width(self.columns_width[idx_col] + (if idx_col == last_col { LAST_SPACING } else { SPACING })),
                );
                if idx_row % 2 == 0 {
                    table.add_child(table_row.background(Color::from(shade.clone())))
                } else {
                    table.add_child(table_row)
                };
                idx_col += 1;
            }
            idx_row += 1;
        }

        self.inner = WidgetPod::new(Box::new(table));
    }

    fn set_columns_width(&mut self, ctx: &mut LayoutCtx, data: &Arc<TableData>, env: &Env) {
        self.columns_width = Vec::new();
        for col in 0.. {
            let mut end_of_cols = true;
            let mut max_width = 0.0;

            for row in &data.rows {
                if let Some(text) = row.get(col) {
                    end_of_cols = false;
                    if !text.is_empty() {
                        let mut layout = TextLayout::<String>::from_text(text.to_owned());
                        layout.rebuild_if_needed(ctx.text(), env);
                        let width = layout.size().width;
                        if width > max_width {
                            max_width = width;
                        }
                    }
                } else {
                    continue;
                }
            }

            if let Some(text) = data.header.get(col) {
                end_of_cols = false;
                if !text.is_empty() {
                    let mut layout = TextLayout::<String>::from_text(text.to_owned());
                    layout.rebuild_if_needed(ctx.text(), env);
                    let width = layout.size().width;
                    if width > max_width {
                        max_width = width;
                    }
                }
            }

            if end_of_cols {
                break;
            } else {
                self.columns_width.push(max_width);
            }
        }
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
            self.cw_layout_pass = true;
            ctx.request_layout();
            self.build_table(data, env);
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Arc<TableData>, env: &Env) -> Size {
        if self.cw_layout_pass {
            self.set_columns_width(ctx, data, env);
            self.cw_layout_pass = false;
        }

        self.inner.layout(ctx, bc, data, env)
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<TableData>, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
