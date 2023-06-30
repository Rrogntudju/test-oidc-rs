use iced::advanced::layout::Limits;
use iced::advanced::mouse;
use iced::advanced::renderer;
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
    pub fn new(data: &TableData, text_size: impl Into<Pixels>) -> Self {
        let last_col = data.header.len() - 1;
        let data = iter::once(&data.header)
            .chain(&data.rows)
            .map(|row| {
                row.into_iter()
                    .enumerate()
                    .map(|(i, s)| if i < last_col { stretch(&s, 1) } else { s.clone() })
                    .collect()
            })
            .collect();

        Self {
            data,
            text_size: text_size.into().0,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Table
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container>,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let limits = Limits::new(Size::ZERO, layout.bounds().size());
        let columns_max_width = self.data.iter().fold(vec![0.0; self.data[0].len()], |acc, row| {
            acc.iter()
                .zip(row.iter())
                .map(|(max, s)| {
                    let text: Element<'_, Message, Renderer> = text(s.clone()).into();
                    let layout = text.as_widget().layout(renderer, &limits);
                    let width = layout.bounds().width;
                    if width > *max {
                        width
                    } else {
                        *max
                    }
                })
                .collect()
        });
        let table = create_table::<Message, Renderer>(&self.data, self.text_size, columns_max_width);
        let node = table.as_widget().layout(renderer, &limits);
        table.as_widget().draw(state, renderer, theme, style, Layout::new(&node), cursor, viewport);
    }
}

impl<'a, Message, Renderer> From<Table>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container>,
{
    fn from(
        content: Table,
    ) -> Element<'a, Message, Renderer> {
        Element::new(content)
    }
}

fn create_table<'a, Message, Renderer>(
    data: &Vec<Vec<String>>,
    text_size: f32,
    columns_max_width: Vec<f32>,
) -> Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
    Renderer::Theme: text::StyleSheet + container::StyleSheet<Style = iced::theme::Container> + 'a,
{
    let mut flip = true;
    let infos = data
        .iter()
        .map(|row| {
            let info: Vec<Element<'_, Message, Renderer>> = row
                .iter()
                .zip(&columns_max_width)
                .map(|(i, width)| {
                    flip = !flip;
                    container(text(i).size(text_size)).style(style(flip)).width(*width).into()
                })
                .collect::<Vec<_>>();
            Row::with_children(info).padding([5, 0, 5, 0]).into()
        })
        .collect::<Vec<_>>();

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
