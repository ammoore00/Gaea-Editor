use iced::advanced::{Layout, Widget};
use iced::{Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::{tree, Tree};
use iced::mouse::Cursor;
use iced::widget::button::Catalog;

#[derive(Debug, Clone)]
struct MenuBarState {
    active_menu: Option<usize>,
}

impl Default for MenuBarState {
    fn default() -> Self {
        todo!()
    }
}

pub struct MenuBar<'a, Message, Theme, Renderer> {
    menus: Vec<Menu<'a, Message, Theme, Renderer>>,
    state: MenuBarState,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer> {
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            state: MenuBarState::default(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for MenuBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        todo!()
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        todo!()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::Some(Box::new(self.state.clone()))
    }
}

pub struct Menu<'a, Message, Theme, Renderer> {
    menu_items: Vec<MenuItem<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Menu<'a, Message, Theme, Renderer> {
    pub fn new() -> Self {
        Self {
            menu_items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: impl Into<MenuItem<'a, Message, Theme, Renderer>>) {
        self.menu_items.push(item.into());
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Theme: Catalog,
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

pub struct MenuItem<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> MenuItem<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        todo!()
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        todo!()
    }
}

impl<'a, Message, Theme, Renderer, T> From<T> for MenuItem<'a, Message, Theme, Renderer>
where
    T: Into<Element<'a, Message, Theme, Renderer>>
{
    fn from(value: T) -> Self {
        MenuItem::new(value)
    }
}