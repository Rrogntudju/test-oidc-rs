use std::boxed;

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
pub struct Table<T> {
    label_width: Vec<f64>,
    inner: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> Table<T> {
    pub fn new() -> Self {
        Table {
            label_width: Vec::new(),
            inner: WidgetPod::new(Label::new("LOL")).boxed(),
        }
    }
}

// If this widget has any child widgets it should call its event, update and layout
// (and lifecycle) methods as well to make sure it works. Some things can be filtered,
// but a general rule is to just pass it through unless you really know you don't want it.
impl<T: Data> Widget<T> for Table<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.inner.layout(ctx, bc, data, env)
    }

    // The paint method gets called last, after an event flow.
    // It goes event -> update -> layout -> paint, and each method can influence the next.
    // Basically, anything that changes the appearance of a widget causes a paint.
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env);

        
    }
}

