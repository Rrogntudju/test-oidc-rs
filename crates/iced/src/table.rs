use iced::advanced::layout::Limits;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::{layout, Layout};
use iced::advanced::{widget, Widget};
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Rectangle, Size};
use std::iter;

pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct Table {
    data: TableData,
}

impl Table {
    pub fn new(data: TableData) -> Self {
        Self { data }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Table
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
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
        let limits = Limits::new(Size::ZERO, layout.bounds().size());
        let columns_max_width = self
            .data
            .rows
            .iter()
            .chain(iter::once(&self.data.header))
            .fold(vec![0.0; self.data.header.len()], |acc, row| {
                acc.iter()
                    .zip(row.iter())
                    .map(|(max, s)| {
                        let text = Into::<Element<'a, Message, Renderer>>::into(text(s));
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
    }
}

fn create_table<'a, Message, Renderer>(data: TableData, columns_max_width: Option<Vec<Length>>) -> Element<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet,
{
    row![column![container(text(""))]].into()
}
