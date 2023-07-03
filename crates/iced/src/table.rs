use iced::advanced::layout::Limits;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::{layout, Layout};
use iced::advanced::{widget, Widget};
use iced::widget::{container, text, Column, Row};
use iced::Pixels;
use iced::{Element, Length, Rectangle, Size};
use std::iter;

#[derive(Debug, Clone)]
pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct Table {
    data: Vec<Vec<String>>,
    text_size: f32,
}

impl Table {
    pub fn new(data: &TableData) -> Self {
        let last_col = data.header.len() - 1;
        let data = iter::once(&data.header)
            .chain(&data.rows)
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(i, s)| if i < last_col { stretch(s, 1) } else { s.clone() })
                    .collect()
            })
            .collect();

        Self { data, text_size: 12.0 }
    }

    pub fn size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = text_size.into().0;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Table
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container>,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let columns_max_width = get_max_width::<Message, Renderer>(&self.data, renderer);
        let table = create_table::<Message, Renderer>(&self.data, self.text_size, &columns_max_width);
        table.as_widget().layout(renderer, limits)
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let columns_max_width = get_max_width::<Message, Renderer>(&self.data, renderer);
        let table = create_table::<Message, Renderer>(&self.data, self.text_size, &columns_max_width);
        let widget = table.as_widget();
        let state = Tree {
            tag: widget.tag(),
            state: widget.state(),
            children: widget.children(),
        };

        widget.draw(&state, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message, Renderer> From<Table> for Element<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container>,
{
    fn from(table: Table) -> Element<'a, Message, Renderer> {
        Element::new(table)
    }
}

fn get_max_width<Message, Renderer>(data: &[Vec<String>], renderer: &Renderer) -> Vec<f32>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet,
{
    let limits = Limits::new(Size::ZERO, Size::INFINITY);
    data.iter().fold(vec![0.0; data[0].len()], |acc, row| {
        acc.iter()
            .zip(row.iter())
            .map(|(max, s)| {
                let text: Element<Message, Renderer> = text(s.clone()).into();
                let layout = text.as_widget().layout(renderer, &limits);
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

fn create_table<'a, Message, Renderer>(data: &[Vec<String>], text_size: f32, columns_max_width: &[f32]) -> Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container>,
{
    let mut flip = false;
    let infos = data
        .iter()
        .map(|row| {
            let info: Vec<Element<Message, Renderer>> = row
                .iter()
                .zip(columns_max_width)
                .map(|(i, width)| container(text(i).size(text_size)).style(style(flip)).width(*width).into())
                .collect();
            flip = !flip;
            Row::with_children(info).padding([5, 0, 5, 0]).into()
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

fn stretch(s: &str, w: usize) -> String {
    format!("{}{}", s, " ".repeat(w))
}
