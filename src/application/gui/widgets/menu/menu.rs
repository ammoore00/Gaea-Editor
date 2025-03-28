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

    pub fn add_item(&mut self, item: MenuItem<'a, Message, Theme, Renderer>) {
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
        // Should add all menu item heights together, and find the maximum width of all items
        todo!()
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