use iced::advanced::{Layout, Widget};
use iced::{Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::{tree, Tree};
use iced::mouse::Cursor;
use iced::widget::text;
use crate::application::gui::widgets::menu::menu::Menu;

#[derive(Debug, Clone)]
struct MenuBarState {
    active_menu: Option<usize>,
}

impl Default for MenuBarState {
    fn default() -> Self {
        Self {
            active_menu: None,
        }
    }
}

pub struct MenuBar<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    menus: Vec<Menu<'a, Message, Theme, Renderer>>,
    state: MenuBarState,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
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
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
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

impl<'a, Message, Theme, Renderer> Into<Element<'a, Message, Theme, Renderer>>
for MenuBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}