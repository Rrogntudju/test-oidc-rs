use iced::advanced::layout::Limits;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::{self, Widget};
use iced::widget::{container, text, Column, Row};
use iced::{mouse, Pixels};
use iced::{Element, Rectangle, Size};
use std::cell::OnceCell;
use std::iter;

#[derive(Debug, Clone, PartialEq)]
pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct Table<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    data: Vec<Vec<String>>,
    font_size: Option<f32>,
    inner: OnceCell<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    pub fn new(data: &TableData) -> Self {
        let last_col = data.header.len() - 1;
        let data = iter::once(&data.header)
            .chain(&data.rows)
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(i, s)| if i < last_col { format!("{s}  ") } else { format!("{s} ") })
                    .collect()
            })
            .collect();

        Self {
            data,
            font_size: None,
            inner: OnceCell::new(),
        }
    }

    pub fn size(mut self, font_size: impl Into<Pixels>) -> Self {
        self.font_size = Some(font_size.into().0);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Table<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::container::StyleSheet + iced::widget::text::StyleSheet,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    <Theme as iced::widget::container::StyleSheet>::Style: From<iced::theme::Container>,
{
/*     fn size(&self) -> Size<iced_core::Length> {
        Size {
            width: iced_core::Length::Shrink,
            height: iced_core::Length::Shrink,
        }
    } */

    fn layout(&self, state: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let table = self.inner.get_or_init(|| {
            let widths = get_max_width::<Message, Theme, Renderer>(state, &self.data, self.font_size, renderer);
            create_table::<Message, Theme, Renderer>(&self.data, self.font_size, &widths)
        });
        table.as_widget().layout(state, renderer, limits)
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let table = self.inner.get().unwrap();
        let widget = table.as_widget();
        let state = Tree::new(widget);
        widget.draw(&state, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message, Theme, Renderer> From<Table<'a, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Theme: iced::widget::container::StyleSheet + iced::widget::text::StyleSheet,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    <Theme as iced::widget::container::StyleSheet>::Style: From<iced::theme::Container>,
{
    fn from(table: Table<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(table)
    }
}

fn get_max_width<Message, Theme, Renderer>(state: &mut widget::Tree, data: &[Vec<String>], font_size: Option<f32>, renderer: &Renderer) -> Vec<f32>
where
    Theme: iced::widget::container::StyleSheet + iced::widget::text::StyleSheet,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
{
    let limits = Limits::new(Size::ZERO, Size::INFINITY);
    data.iter().fold(vec![0.0; data[0].len()], |acc, row| {
        acc.iter()
            .zip(row.iter())
            .map(|(max, s)| {
                let text: Element<Message, Theme, Renderer> = match font_size {
                    Some(size) => text(s.clone()).size(size).into(),
                    None => text(s.clone()).into(),
                };
                let layout = text.as_widget().layout(state, renderer, &limits);
                let width = layout.bounds().width;
                if width > *max {
                    width
                } else {
                    *max
                }
            })
            .collect()
    })
}

fn create_table<'a, Message, Theme, Renderer>(
    data: &[Vec<String>],
    font_size: Option<f32>,
    columns_max_width: &[f32],
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet + iced::widget::text::StyleSheet,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    <Theme as iced::widget::container::StyleSheet>::Style: From<iced::theme::Container>,
{
    let mut flip = false;
    let infos: Vec<Element<Message, Theme, Renderer>> = data
        .iter()
        .map(|row| {
            let info: Vec<Element<Message, Theme, Renderer>> = row
                .iter()
                .zip(columns_max_width)
                .map(|(i, width)| {
                    container(match font_size {
                        Some(size) => text(i).size(size),
                        None => text(i),
                    })
                    .style(style(flip))
                    .width(*width)
                    .padding([5, 0, 5, 0])
                    .into()
                })
                .collect();
            flip = !flip;
            Row::with_children(info).into()
        })
        .collect();

    Column::with_children(infos).into()
}

fn style(flip: bool) -> iced::theme::Container {
    if flip {
        iced::theme::Container::Box
    } else {
        iced::theme::Container::default()
    }
}
