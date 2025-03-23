use crate::application::gui::header::menu::Item;
use iced::{Border, Element, Length, Task};
use iced::border::Radius;
use iced::widget::Button;
use iced_aw::{menu, menu_bar, menu_items, Menu};
use iced_aw::style::Status;
use crate::application::gui::window;

const MENU_WIDTH: f32 = 180.0;
const MENU_OFFSET: f32 = 8.0;
const MENU_SPACING: f32 = 0.0;

pub enum Message {

}

pub struct Header {

}

impl Header {
    pub fn new() -> (Self, Task<window::Message>) {
        (Self {}, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<window::Message> {
        // TODO: implement header menu bar
        Task::none()
    }

    pub fn view(&self) -> Element<window::Message> {
        let menu_bar = menu_bar!(
            (Button::new("File"), self.create_file_menu())
        )
        .draw_path(menu::DrawPath::Backdrop)
        .style(|theme:&iced::Theme, status: Status| menu::Style {
            path_border: Border{
                radius: Radius::new(0.0),
                ..Default::default()
            },
            ..menu::primary(theme, status)
        });
        
        menu_bar.into()
    }
    
    fn create_file_menu<'a>(&self) -> Menu<'a, window::Message, iced::Theme, iced::Renderer> {
        Menu::new(menu_items!(
            (Button::new("Test").width(Length::Fill).style(iced::widget::button::secondary))
            (Button::new("Test2").width(Length::Fill).style(iced::widget::button::secondary))
        ))
        .max_width(MENU_WIDTH)
        .offset(MENU_OFFSET)
        .spacing(MENU_SPACING)
    }
}