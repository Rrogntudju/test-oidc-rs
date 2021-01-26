use std::boxed;
use std::sync::Arc;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::{prelude::*, Label, LabelText};
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect,
    WindowDesc, WidgetPod, theme,
};

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

    fn build_table(&mut self, data:  &Arc<TableData>) {

    }

    fn set_columns_width(&mut self, ctx: &mut LayoutCtx, data: &Arc<TableData>) {
        let table_data = data.headers
    }
}

// If this widget has any child widgets it should call its event, update and layout
// (and lifecycle) methods as well to make sure it works. Some things can be filtered,
// but a general rule is to just pass it through unless you really know you don't want it.
impl  Widget<Arc<TableData>> for Table {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Arc<TableData>, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Arc<TableData>, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Arc<TableData>, data: &Arc<TableData>, _env: &Env) {
        if !old_data.same(data) {
            self.cw_layout_pass = true;
            ctx.request_layout();   // to set the background of each row, we need the width of each column
            self.build_table(data); // build the table widget using the width of each column
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

