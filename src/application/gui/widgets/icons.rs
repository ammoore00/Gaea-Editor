// SPDX-License-Identifier: MPL-2.0

use iced::{Element, Font};
use iced::widget::{button, center, container, text, tooltip};

pub fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(center(content).width(30)).height(30);
    
    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
            .style(container::rounded_box)
            .into()
    } else {
        action.style(button::secondary).into()
    }
}

pub const NEW_ICON: char = '\u{0e800}';
pub const SAVE_ICON: char = '\u{0e801}';
pub const OPEN_ICON: char = '\u{0f115}';

#[derive(Clone, Debug)]
pub struct Icon<'a, Message> {
    _marker: std::marker::PhantomData<&'a Message>,
    font: Font,
    codepoint: char,
}

impl<'a, Message> Icon<'a, Message> {
    pub fn new(codepoint: char) -> Self {
        Self {
            _marker: Default::default(),
            font: Font::with_name("editor-icons"),
            codepoint,
        }
    }
}

impl<'a, Message, Theme, Renderer> Into<Element<'a, Message, Theme, Renderer>> for Icon<'a, Message>
where
    Message: 'a + Clone,
    Renderer: 'a + iced::advanced::text::Renderer,
    Theme: 'a + text::Catalog,
    <Renderer as iced::advanced::text::Renderer>::Font: From<Font>,
{
    fn into(self) -> Element<'a, Message, Theme, Renderer> {
        let renderer_font: <Renderer as iced::advanced::text::Renderer>::Font = self.font.into();
        text(self.codepoint).font(renderer_font).into()
    }
}