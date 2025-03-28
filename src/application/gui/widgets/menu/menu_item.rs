use iced::advanced::{layout, Clipboard, Layout, Shell, Widget};
use iced::{Event, Length, Rectangle, Size};
use iced::advanced::graphics::core::Element;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Style;
use iced::advanced::widget::{tree, Tree};
use iced::mouse::Cursor;
use iced::widget::{button, horizontal_rule, text, Button, Row, Rule, Space};
use iced::widget::button::{Status, StyleFn};
use crate::application::gui::widgets::icons::Icon;
use crate::application::gui::widgets::menu::menu::Menu;

enum MenuItemContent<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog + button::Catalog,
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
    Theme: 'a + text::Catalog + button::Catalog, 
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
    Theme: 'a + text::Catalog + button::Catalog,
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
    Theme: 'a + text::Catalog + button::Catalog,
{
    pub fn new() -> Self {
        Self {
            icon: None,
            content: None,
            _state: std::marker::PhantomData,
        }
    }
    
    pub fn separator(self) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
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
        let mut submenu = submenu;
        submenu.set_submenu();

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
    Theme: 'a + text::Catalog + button::Catalog,
{
    pub fn build(self) -> MenuItem<'a, Message, Theme, Renderer>
    where
        <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
        Rule<'a>: Widget<Message, Theme, Renderer>,
    {
        Into::<MenuItemBuilder<'a, Final, Message, Theme, Renderer>>::into(self).build()
    }
    
    pub fn icon(self, icon: Icon) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
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

    pub fn on_press_maybe<F: Into<Message>>(mut self, action: Option<Message>) -> Self {
        if let Some(MenuItemContent::Button(mut button)) = self.content.take() {
            button = button.on_press_maybe(action);
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
        if let Some(MenuItemContent::Button(button)) = self.content.take() {
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

impl<'a, Message, Theme, Renderer> Into<MenuItemBuilder<'a, Final, Message, Theme, Renderer>>
for MenuItemBuilder<'a, MenuButton, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog + button::Catalog,
{
    fn into(self) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: self.icon,
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, Submenu, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog + button::Catalog,
{
    pub fn build(self) -> MenuItem<'a, Message, Theme, Renderer>
    where
        <Renderer as iced::advanced::text::Renderer>::Font: From<iced::Font>,
        Rule<'a>: Widget<Message, Theme, Renderer>,
    {
        Into::<MenuItemBuilder<'a, Final, Message, Theme, Renderer>>::into(self).build()
    }
    
    pub fn icon(self, icon: Icon) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: Some(icon),
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'a, Message, Theme, Renderer> Into<MenuItemBuilder<'a, Final, Message, Theme, Renderer>>
for MenuItemBuilder<'a, Submenu, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog + button::Catalog,
{
    fn into(self) -> MenuItemBuilder<'a, Final, Message, Theme, Renderer> {
        MenuItemBuilder {
            icon: self.icon,
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

impl<'a, Message, Theme, Renderer> MenuItemBuilder<'a, Final, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog + button::Catalog
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

        let content: Element<'a, Message, Theme, Renderer> = self.content.expect("Illegal State! MenuItem build called without content being set!").into();
        
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
    Theme: 'a + text::Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,

    width: Length,
    height: Length,
    padding: f32,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
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
                let child_tree = tree.children.get_mut(0)
                    .expect("Expected content tree");
                self.content.as_widget().layout(child_tree, renderer, limits)
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle
    ) {
        let child_tree = tree.children.get(0)
            .expect("Expected content tree");
        let child_layout = layout.children().next()
            .expect("Expected content layout");

        self.content.as_widget().draw(
            child_tree,
            renderer,
            theme,
            style,
            child_layout,
            cursor,
            viewport
        );
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.is_empty() {
            // If the tree does not have children, initialize them using the content
            tree.children = self.children();
        } else {
            // Otherwise, delegate diffing to the child tree
            if let Some(child_tree) = tree.children.get_mut(0) {
                self.content.as_widget().diff(child_tree);
            }
        }
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle
    ) -> iced::event::Status {
        let child_layout = layout.children().next()
            .expect("Expected content layout");
        let child_state = state.children.get_mut(0)
            .expect("Expected content tree");

        self.content.as_widget_mut().on_event(
            child_state,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }
}

impl<'a, Message, Theme, Renderer> Into<Element<'a, Message, Theme, Renderer>>
for MenuItem<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        Element::new(self)
    }
}