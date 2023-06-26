use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::{layout, Layout};
use iced::advanced::{renderer, Renderer};
use iced::advanced::{widget, Widget};
use iced::{Color, Element, Event, Length, Point, Rectangle, Size};

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
    Renderer: iced::advanced::Renderer,
{
    fn width(&self) -> Length {
        self.base.as_widget().width()
    }

    fn height(&self) -> Length {
        self.base.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.base.as_widget().layout(renderer, limits)
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
        self.base
            .as_widget()
            .draw(&state.children[0], renderer, theme, style, layout, cursor, viewport);
    }
}

fn layout(data: &TableData, columns_max_width: Option<Vec<Length>>) -> Node {
    Node::new(Size::ZERO)
}
