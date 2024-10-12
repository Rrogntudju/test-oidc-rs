use iced::advanced::layout::Limits;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::{self, Widget};
use iced::widget::{container, text, Column, Row};
use iced::{advanced, mouse, Color, Element, Length, Padding, Pixels, Rectangle, Size};
use std::cell::OnceCell;

pub struct Table<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + advanced::text::Renderer,
{
    data: &'a [Vec<String>],
    font_size: Option<f32>,
    header_color: Option<Color>,
    col_spacing: Pixels,
    inner: OnceCell<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Table<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + advanced::text::Renderer,
{
    pub fn new(data: &'a [Vec<String>]) -> Self {
        Self {
            data,
            font_size: None,
            header_color: None,
            col_spacing: Pixels(0.),
            inner: OnceCell::new(),
        }
    }

    pub fn font_size(mut self, font_size: impl Into<Pixels>) -> Self {
        self.font_size = Some(font_size.into().0);
        self
    }

    pub fn header_color(mut self, color: impl Into<Color>) -> Self {
        self.header_color = Some(color.into());
        self
    }

    pub fn col_spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.col_spacing = amount.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, iced::Theme, Renderer> for Table<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
{
    fn size(&self) -> Size<iced_core::Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let table = self
            .inner
            .get_or_init(|| {
                let widths = get_max_width::<Message, Renderer>(&self.data, self.font_size, self.col_spacing, renderer);
                create_table::<Message, Renderer>(&self.data, self.font_size, self.header_color, &widths)
            })
            .as_widget();
        *tree = Tree::new(table);
        table.layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let table = self.inner.get().unwrap().as_widget();
        table.draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, Message, Renderer> From<Table<'a, Message, iced::Theme, Renderer>> for Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
{
    fn from(table: Table<'a, Message, iced::Theme, Renderer>) -> Element<'a, Message, iced::Theme, Renderer> {
        Element::new(table)
    }
}

// Obtenir la largeur maximale de chaque colonne de la table
fn get_max_width<Message, Renderer>(data: &[Vec<String>], font_size: Option<f32>, col_spacing: Pixels, renderer: &Renderer) -> Vec<f32>
where
    Renderer: advanced::Renderer + advanced::text::Renderer,
{
    let limits = Limits::new(Size::ZERO, Size::INFINITY);
    let nb_cols = data[0].len();
    let last_col = nb_cols - 1;
    data.iter().fold(vec![0.0; nb_cols], |acc, row| {
        acc.iter()
            .zip(row.iter().enumerate())
            .map(|(max, (i, s))| {
                let text = match font_size {
                    Some(size) => text(s).size(size),
                    None => text(s),
                };

                let p = if i < last_col {
                    Padding::new(0.).right(col_spacing)
                } else {
                    Padding::new(0.).right(col_spacing / 3.)
                };

                let text: Element<Message, iced::Theme, Renderer> = container(text).padding(p).into();
                let mut tree = Tree::new(text.as_widget());
                let layout = text.as_widget().layout(&mut tree, renderer, &limits);
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

fn create_table<'a, Message, Renderer>(
    data: &'a [Vec<String>],
    font_size: Option<f32>,
    header_color: Option<Color>,
    columns_max_width: &[f32],
) -> Element<'a, Message, iced::Theme, Renderer>
where
    Message: 'a,
    Renderer: advanced::Renderer + advanced::text::Renderer + 'a,
{
    let mut flip = false;
    let infos: Vec<Element<Message, iced::Theme, Renderer>> = data
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let info: Vec<Element<Message, iced::Theme, Renderer>> = row
                .iter()
                .zip(columns_max_width)
                .map(|(s, width)| {
                    let text = match font_size {
                        Some(size) => text(s).size(size),
                        None => text(s),
                    };

                    let text = if i == 0 { text.color_maybe(header_color) } else { text };
                    container(text).width(*width).padding([5, 0]).style(style(flip)).into()
                })
                .collect();
            flip = !flip;
            Row::with_children(info).into()
        })
        .collect();

    Column::with_children(infos).into()
}

fn style(flip: bool) -> Box<dyn Fn(&iced::Theme) -> container::Style> {
    if flip {
        Box::new(container::rounded_box)
    } else {
        Box::new(container::transparent)
    }
}
