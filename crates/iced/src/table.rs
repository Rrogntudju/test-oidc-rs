use iced::advanced::layout::Limits;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::{layout, Layout};
use iced::advanced::{widget, Widget};
use iced::widget::{column, container, text, Row};
use iced::Pixels;
use iced::{Element, Length, Rectangle, Size};
use std::iter;

pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct Table {
    data: TableData,
    text_size: f32,
}

impl Table {
    pub fn new(data: TableData, text_size: impl Into<Pixels>) -> Self {
        Self {
            data,
            text_size: text_size.into().0,
        }
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
        state: &widget::Tree,
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
                        let text: Element<'a, Message, Renderer> = Element::from(text(s));
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
        let table = create_table::<'a, Message, Renderer>(self.data, self.text_size, columns_max_width);
        let layout = Layout::new(&table.as_widget().layout(renderer, &limits));
        table.as_widget().draw(state, renderer, theme, style, layout, cursor, viewport);
    }
}

fn create_table<'a, Message, Renderer>(
    data: TableData,
    text_size: impl Into<Pixels>,
    columns_max_width: Vec<impl Into<Length>>,
) -> Element<'a, Message, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: text::StyleSheet + container::StyleSheet,
{
    let entête = data
        .header
        .iter()
        .zip(columns_max_width)
        .map(|(h, width)| Element::from(container(text(h).size(text_size)).width(width)))
        .collect::<Vec<_>>();

    let entête = Row::with_children(entête).padding([5, 0, 5, 0]);

    let mut flip = false;
    let infos = data
        .rows
        .iter()
        .map(|row| {
            let info: Vec<Element<'_, Message, Renderer>> = row.iter()
                .zip(columns_max_width)
                .map(|(i, width)| {
                    flip = !flip;
                    Element::from(container(text(i).size(text_size)).style(style(flip)).width(width))
                })
                .collect::<Vec<_>>();
            Row::with_children(info).padding([5, 0, 5, 0])
        })
        .collect::<Vec<_>>();

   column![entête, infos].into()
}

fn style(flip: bool) -> iced::theme::Container {
    if flip {
        iced::theme::Container::Box
    } else {
        iced::theme::Container::default()
    }
}
