use druid::{TextLayout, piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder, dwrite::TextLayout}};
use druid::widget::{prelude::*, Label, LabelText,};
use druid::{theme, Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, WidgetPod, WindowDesc,};
use std::boxed;
use std::sync::Arc;

pub type TableColumns = Vec<String>;
pub type TableRows = Vec<TableColumns>;
pub type TableHeaders = Vec<String>;
pub struct TableData {
    pub headers: TableHeaders,
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

    fn build_table(&mut self, data: &Arc<TableData>) {}

    fn set_columns_width(&mut self, ctx: &mut LayoutCtx, data: &Arc<TableData>, env: &Env) {
        let mut cw = Vec::<f64>::new();
        for col in 0.. {
            cw.push(0.);
            let mut nb_cols: usize = 0;
            for row in data.rows.clone() {
                if let Some(text) = row.get(col) {
                    nb_cols+=1;
                    let layout = TextLayout::<String>::from_text(text.to_owned());
                    layout.rebuild_if_needed(ctx.text(), env);
                    cw.push(layout.layout_metrics().size.width);
                }
                else {
                    continue;
                }
            }
            if nb_cols == 0 {
                break;
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

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Arc<TableData>, data: &Arc<TableData>, _env: &Env) {
        if !old_data.same(data) {
            self.cw_layout_pass = true;
            ctx.request_layout(); 
            self.build_table(data); 
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Arc<TableData>, env: &Env) -> Size {
        if self.cw_layout_pass {
            self.set_columns_width(ctx, data);
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
