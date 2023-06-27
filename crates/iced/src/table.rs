use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::{layout, Layout};
use iced::advanced::{renderer};
use iced::advanced::{widget, Widget};
use iced::widget::{column, container};
use iced::{Element, Length, Rectangle, Size, Theme};

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
    Renderer: iced::advanced::Renderer
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
     }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        Node::new(Size::ZERO)
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base
            .as_widget()
            .draw(&state.children[0], renderer, theme, style, layout, cursor, viewport);
    }
}

fn create_table<'a, Message, Renderer>(data: &TableData, columns_max_width: Option<Vec<Length>>) -> Element<'a, Message, Renderer<Theme>>
where
    Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer,
    Renderer::Theme: iced::widget::text::StyleSheet

{
    column![container("")].into()
}
