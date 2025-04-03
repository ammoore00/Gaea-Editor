use iced::advanced::{Layout, Widget};
use iced::{Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::Tree;
use iced::mouse::Cursor;
use iced::widget::text;
use crate::application::gui::widgets::menu::menu_item::MenuItem;

pub struct Menu<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    menu_items: Vec<MenuItem<'a, Message, Theme, Renderer>>,
    is_submenu: bool,
}

impl<'a, Message, Theme, Renderer> Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    pub fn new() -> Self {
        Self {
            menu_items: Vec::new(),
            is_submenu: false,
        }
    }

    pub fn push(&mut self, item: MenuItem<'a, Message, Theme, Renderer>) {
        self.menu_items.push(item);
    }
    
    pub(super) fn set_submenu(&mut self) {
        self.is_submenu = true;
    }
    
    pub fn is_submenu(&self) -> bool {
        self.is_submenu
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    fn size(&self) -> Size<Length> {
        // Adds all menu item heights together, and find the maximum width of all items
        // TODO: verify AI code
        let mut total_height = Length::Shrink;
        let mut max_width = Length::Shrink;

        for item in &self.menu_items {
            let item_size = item.size();

            if let Length::Fixed(height) = item_size.height {
                total_height = match total_height {
                    Length::Fixed(current) => Length::Fixed(current + height),
                    _ => Length::Fixed(height),
                };
            }

            if let Length::Fixed(width) = item_size.width {
                max_width = match max_width {
                    Length::Fixed(current_max) => Length::Fixed(current_max.max(width)),
                    _ => Length::Fixed(width),
                };
            }
        }

        Size::new(max_width, total_height)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        todo!()
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        todo!()
    }
}

impl<'a, Message, Theme, Renderer> Into<Element<'a, Message, Theme, Renderer>>
for Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}