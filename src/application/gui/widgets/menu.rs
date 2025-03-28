use iced::advanced::{layout, Layout, Widget};
use iced::{Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::{tree, Tree};
use iced::widget::text::Text;
use iced::mouse::Cursor;
use iced::widget::{horizontal_rule, Row, Space};
use crate::application::gui::widgets::icons::Icon;

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
    Theme: 'a + iced::widget::text::Catalog,
{
    menus: Vec<Menu<'a, Message, Theme, Renderer>>,
    state: MenuBarState,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
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
    Theme: 'a + iced::widget::text::Catalog,
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
    Theme: 'a + iced::widget::text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}

pub struct Menu<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    menu_items: Vec<MenuItem<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    pub fn new() -> Self {
        Self {
            menu_items: Vec::new(),
        }
    }

    fn add_item(&mut self, item: MenuItem<'a, Message, Theme, Renderer>) {
        self.menu_items.push(item);
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for Menu<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
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
    Theme: 'a + iced::widget::text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}

pub struct MenuItem<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    item_type: MenuItemType<'a, Message, Theme, Renderer>,
    // TODO: investigate if there is any way this can become desynced
    content: Element<'a, Message, Theme, Renderer>,

    width: Length,
    height: Length,
    padding: f32,
}

enum MenuItemType<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    Separator,
    Button {
        label: Element<'a, Message, Theme, Renderer>,
        icon: Option<Icon<'a, Message>>,
    },
    SubMenu {
        menu: Menu<'a, Message, Theme, Renderer>,
        icon: Option<Icon<'a, Message>>,
    },
}

impl<'a, Message, Theme, Renderer> MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog + iced::widget::rule::Catalog,
    // TODO: Is this where clause appropriate?
    Icon<'a, Message>: Into<Element<'a, Message, Theme, Renderer>>
{
    fn new(item_type: MenuItemType<'a, Message, Theme, Renderer>) -> Self {
        let mut menu_item = Self {
            item_type,
            content: Element::new(Row::new()),
            
            width: Length::Fill,
            height: Length::Shrink,
            padding: 5.0,
        };

        menu_item.compute_content()
    }
    
    fn compute_content(mut self) -> Self {
        match self.item_type {
            MenuItemType::Separator => {
                let row = Row::new().push(horizontal_rule(2));
                self.content = row.into();
                self
            }
            MenuItemType::Button { label, icon } => {
                let icon_element: Element<'a, Message, Theme, Renderer> = match icon {
                    // TODO: above where clause added to fix this into() call - investigate whether that is appropriate
                    Some(icon) => icon.clone().into(),
                    None => Space::new(Length::Shrink, Length::Shrink).into(),
                };

                let row = Row::new()
                    .push(icon_element)
                    .push(label);
                
                self.content = row.into();
                self
            }
            MenuItemType::SubMenu { menu, icon } => {
                
            }
        }
    }
    
    pub fn button(label: impl Into<Text<'a, Theme, Renderer>>) -> Self 
    where
        Renderer: 'a + iced::advanced::text::Renderer,
    {
        // TODO: maybe make this more expansive to include any sub-widget?
        Self::new(MenuItemType::Button {
            label: label.into().into(),
            icon: None,
        })
    }
    
    pub fn sub_menu(menu: impl Into<Menu<'a, Message, Theme, Renderer>>) -> Self {
        Self::new(MenuItemType::SubMenu {
            menu: menu.into(),
            icon: None,
        })
    }

    /// Icons can only be added to Buttons and Submenus
    pub fn icon(mut self, icon: Icon<'a, Message>) -> Result<Self, String> {
        match &mut self.item_type {
            MenuItemType::Button { icon: existing_icon, .. } => {
                *existing_icon = Some(icon);
                Ok(self.compute_content())
            }
            MenuItemType::SubMenu { icon: existing_icon, .. } => {
                *existing_icon = Some(icon);
                Ok(self.compute_content())
            }
            MenuItemType::Separator => {
                Err("Cannot add an icon to a separator!".to_string())
            }
        }
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self.compute_content()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        layout::padded(
            limits,
            self.width,
            self.height,
            self.padding,
            |limits| {
                match &self.item_type {
                    MenuItemType::Separator => {}
                    MenuItemType::Button { label, icon } => {
                        
                    }
                    MenuItemType::SubMenu { menu, icon } => {
                        menu.layout(
                            &mut tree.children[0],
                            renderer,
                            limits
                        )
                    }
                }
            },
        )
    }

    fn draw(&self, tree: &Tree, renderer: &mut Renderer, theme: &Theme, style: &Style, layout: Layout<'_>, cursor: Cursor, viewport: &Rectangle) {
        todo!()
    }
}

impl<'a, Message, Theme, Renderer> Into<Element<'a, Message, Theme, Renderer>>
for MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + iced::widget::text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}