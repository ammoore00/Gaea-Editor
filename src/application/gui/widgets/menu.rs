use iced::advanced::{layout, Layout, Widget};
use iced::{widget, Alignment, Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::{tree, Tree};
use iced::mouse::Cursor;
use iced::widget::{button, horizontal_rule, Button, Row, Rule, Space};
use iced::widget::button::{Status, StyleFn};
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
    is_submenu: bool,
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
            is_submenu: false,
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
    Theme: 'a + widget::text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}

enum MenuItemContent<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog,
{
    Separator(Rule<'a>),
    Button(Button<'a, Message, Theme, Renderer>),
    SubMenu(Menu<'a, Message, Theme, Renderer>),
}


impl<'a, Message, Theme, Renderer> From<MenuItemContent<'a, Message, Theme, Renderer>>
for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog, 
    Rule<'a>: Widget<Message, Theme, Renderer>
{
    fn from(content: MenuItemContent<'a, Message, Theme, Renderer>) -> Self {
        match content {
            MenuItemContent::Separator(rule) => Element::new(rule),
            MenuItemContent::Button(button) => button.into(),
            MenuItemContent::SubMenu(menu) => menu.into(),
        }
    }
}

pub struct MenuItemBuilder<'a, State, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog,
{
    icon: Option<Icon>,
    content: Option<MenuItemContent<'a, Message, Theme, Renderer>>,
    _state: std::marker::PhantomData<&'a (State)>,
}

struct Initial;
struct MenuButton;
struct Submenu;
struct Final;

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, Initial, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog,
{
    pub fn new() -> Self {
        Self {
            icon: None,
            content: None,
            _state: std::marker::PhantomData,
        }
    }
    
    pub fn separator(mut self) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: self.icon,
            content: Some(MenuItemContent::Separator(horizontal_rule(2))),
            _state: std::marker::PhantomData,
        }
    }

    pub fn button(
        self,
        button_text: &'a str,
    ) -> MenuItemBuilder<'a, MenuButton, Message, Theme, Renderer> {
        let button = Button::new(button_text);
        
        MenuItemBuilder {
            icon: self.icon,
            content: Some(MenuItemContent::Button(button)),
            _state: std::marker::PhantomData,
        }
    }

    pub fn submenu(
        self,
        submenu: Menu<'a, Message, Theme, Renderer>,
    ) -> MenuItemBuilder<'a, Submenu, Message, Theme, Renderer> {
        let submenu = Menu {
            menu_items: submenu.menu_items,
            is_submenu: true,
        };
        
        MenuItemBuilder {
            icon: self.icon,
            content: Some(MenuItemContent::SubMenu(submenu)),
            _state: std::marker::PhantomData,
        }
    }
}

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, MenuButton, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + widget::button::Catalog,
{
    pub fn build(self) -> MenuItem<'a, Message, Theme, Renderer>
    where
        <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
        Rule<'a>: Widget<Message, Theme, Renderer>,
    {
        MenuItemBuilder::<'a, Final, Message, Theme, Renderer> {
            icon: self.icon,
            content: self.content,
            _state: std::marker::PhantomData,
        }.build()
    }
    
    pub fn icon(mut self, icon: Icon) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: Some(icon),
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }

    pub fn on_press<F: Into<Message>>(mut self, action: F) -> Self {
        if let Some(MenuItemContent::Button(mut button)) = self.content.take() {
            button = button.on_press(action.into());
            self.content = Some(MenuItemContent::Button(button));
        }
        else {
            panic!("Invalid state! Tried to call button action with no button.")
        }
        self
    }

    pub fn on_press_with<F: Into<Message>>(mut self, action: impl Fn() -> Message + 'a) -> Self {
        if let Some(MenuItemContent::Button(mut button)) = self.content.take() {
            button = button.on_press_with(action);
            self.content = Some(MenuItemContent::Button(button));
        }
        else {
            panic!("Invalid state! Tried to call button action with no button.")
        }
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme, Status) -> button::Style + 'a) -> Self
    where
        <Theme as button::Catalog>::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        if let Some(MenuItemContent::Button(mut button)) = self.content.take() {
            let button = button.style(style);
            self.content = Some(MenuItemContent::Button(button));
        }
        else {
            panic!("Invalid state! Tried to call button action with no button.")
        }
        self
    }

    pub fn tooltip(mut self, tooltip: &'a str) -> Self {
        todo!()
    }
}

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, Submenu, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + widget::button::Catalog,
{
    pub fn build(self) -> MenuItem<'a, Message, Theme, Renderer>
    where
        <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
        Rule<'a>: Widget<Message, Theme, Renderer>,
    {
        MenuItemBuilder::<'a, Final, Message, Theme, Renderer> {
            icon: self.icon,
            content: self.content,
            _state: std::marker::PhantomData,
        }.build()
    }
    
    pub fn icon(mut self, icon: Icon) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: Some(icon),
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, Final, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog
{
    pub fn build(self) -> MenuItem<'a, Message, Theme, Renderer>
    where
        <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
        Rule<'a>: Widget<Message, Theme, Renderer>,
    {
        let icon_or_space: Element<'a, Message, Theme, Renderer> = self.icon.map_or_else(
            || Space::with_width(Length::Shrink).into(),
            |icon| icon.into(),
        );

        let content: Element<'a, Message, Theme, Renderer> = self.content.expect("MenuItem content must be set before building.").into();
        
        let content = Row::new()
            .push(icon_or_space)
            .push(content);

        MenuItem {
            content: content.into(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: 0.0,
        }
    }
}

pub struct MenuItem<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,

    width: Length,
    height: Length,
    padding: f32,
}

impl<'a, Message, Theme, Renderer> MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + widget::text::Catalog + button::Catalog,
    // TODO: Is this where clause appropriate?
    Icon: Into<Element<'a, Message, Theme, Renderer>>
{
    
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
                todo!()
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