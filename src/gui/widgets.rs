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

pub fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

pub fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

pub fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

pub fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("text_editor-icons");
    
    text(codepoint).font(ICON_FONT).into()
}