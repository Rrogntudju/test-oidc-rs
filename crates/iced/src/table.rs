use iced::advanced::layout::Limits;
use iced::advanced::mouse;
use iced::advanced::renderer;
use iced::advanced::{layout, Layout};
use iced::advanced::{widget, Widget};
use iced::widget::{column, container, Row, text};
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
        Self { data, text_size: text_size.into().0 }
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
    let entêtes = data
        .header
        .iter()
        .zip(columns_max_width)
        .map(|(h, width)| Element::from(container(text(h).size(text_size)).width(width)))
        .collect::<Vec<_>>();

    let entêtes = Row::with_children(entêtes).padding([5, 0, 5, 0]);

    let infos = data
    .rows
    .iter()
    .map(|row| {
        row.iter().zip(columns_max_width)
        .map(|(r, width)| Element::from(container(text(r).size(text_size)).width(width)))
        .collect::<Vec<_>>()
    }
    )  .collect::<Vec<_>>();



let infos = infos.iter().map(Row::with_children(infos).padding([5, 0, 5, 0]);

    /*            let mut infos = column![];
    let mut flip = false;

    for row in &data.rows {
        let info = row!
            container(text(stretch(&row[0], count[0] + 1)).style(style(flip)),
            container(text(stretch(&row[1], count[1])).size(12)).style(style(flip)),

        .padding([5, 0, 0, 0]);
        infos = infos.push(info);
        flip = !flip;
    }
    column![entêtes, infos].into() */
}

fn style(flip: bool) -> iced::theme::Container {
    if flip {
        iced::theme::Container::Box
    } else {
        iced::theme::Container::default()
    }
}
