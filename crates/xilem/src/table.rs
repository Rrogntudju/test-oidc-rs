use crate::Scene;
use accesskit::Role;
use masonry::text2::TextLayout;
use masonry::widget::{prelude::*, CrossAxisAlignment, Flex, Label, SizedBox, WidgetRef};
use masonry::{theme, AccessCtx, AccessEvent, Color, WidgetPod};
use smallvec::{smallvec, SmallVec};

use std::iter;

const SPACING: f64 = 12.0;
const LAST_SPACING: f64 = SPACING / 2.0;
const SHADING: f64 = 0.1;

#[derive(Clone)]
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

fn rgba_f64(Color { r, g, b, a }: Color) -> (f64, f64, f64, f64) {
    (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0, a as f64 / 255.0)
}

// Find out the maximum layout width of each column
fn layout_columns_width(ctx: &mut LayoutCtx, data: &TableData) -> Option<Vec<f64>> {
    let mut columns_width = Vec::new();
    for j in 0_usize.. {
        let mut end_of_cols = true;
        let mut max_width = 0.0;

        data.rows.iter().chain(iter::once(&data.header)).for_each(|row| {
            if let Some(text) = row.get(j) {
                end_of_cols = false;
                if !text.is_empty() {
                    let mut layout = TextLayout::<String>::new(text.clone(), theme::TEXT_SIZE_NORMAL as f32);
                    layout.rebuild(ctx.font_ctx());
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
    header_text_brush: Option<Color>,
    data: TableData,
    inner: WidgetPod<Flex>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            header_text_brush: None,
            data: TableData::default(),
            inner: WidgetPod::new(Flex::row()),
        }
    }

    fn build(&mut self, ctx: &mut LayoutCtx) {
        let mut table = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);

        if let Some(widths) = layout_columns_width(ctx, &self.data) {
            let last_col = widths.len() - 1;
            let (r, g, b, a) = rgba_f64(theme::WINDOW_BACKGROUND_COLOR);
            let (r, g, b, a) = if r + g + b < 1.5 {
                (
                    (r + SHADING).clamp(0.0, 1.0),
                    (g + SHADING).clamp(0.0, 1.0),
                    (b + SHADING).clamp(0.0, 1.0),
                    a,
                )
            } else {
                (
                    (r - SHADING).clamp(0.0, 1.0),
                    (g - SHADING).clamp(0.0, 1.0),
                    (b - SHADING).clamp(0.0, 1.0),
                    a,
                )
            };
            let shade = Color::rgba(r, g, b, a);

            let mut header = Flex::row();
            for (j, col_name) in self.data.header.iter().enumerate() {
                let mut label = Label::new(col_name.clone());
                if let Some(color) = &self.header_text_brush {
                    label = label.with_text_brush(color.clone());
                }
                header = header.with_child(SizedBox::new(label).width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
            }
            table = table.with_child(header);

            for (i, row) in self.data.rows.iter().enumerate() {
                let mut table_row = Flex::row();
                for (j, text) in row.iter().enumerate() {
                    table_row = table_row
                        .with_child(SizedBox::new(Label::new(text.clone())).width(widths[j] + if j == last_col { LAST_SPACING } else { SPACING }));
                }
                if i % 2 == 0 {
                    table = table.with_child(SizedBox::new(table_row).background(shade))
                } else {
                    table = table.with_child(table_row)
                };
            }
        }

        self.inner = WidgetPod::new(table);
    }

    pub fn set_header_text_brush(&mut self, color: Color) {
        self.header_text_brush = Some(color.into());
    }

    pub fn with_header_text_brush(mut self, color: Color) -> Self {
        self.set_header_text_brush(color);
        self
    }
}

impl Widget for Table {
    fn on_pointer_event(&mut self, _ctx: &mut EventCtx<'_>, _event: &PointerEvent) {}

    fn on_text_event(&mut self, _ctx: &mut EventCtx<'_>, _event: &TextEvent) {}

    fn on_access_event(&mut self, _ctx: &mut EventCtx<'_>, _event: &AccessEvent) {}

    fn on_status_change(&mut self, _ctx: &mut LifeCycleCtx<'_>, _event: &StatusChange) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx<'_>, _event: &LifeCycle) {}

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, bc: &BoxConstraints) -> Size {
        self.build(ctx);
        self.inner.layout(ctx, bc)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
        self.inner.paint(ctx, scene);
    }

    fn accessibility_role(&self) -> Role {
        Role::Window
    }

    fn accessibility(&mut self, _ctx: &mut AccessCtx<'_>) {}

    fn children(&self) -> SmallVec<[WidgetRef<'_, dyn Widget>; 16]> {
        smallvec![self.inner.as_dyn()]
    }
}
