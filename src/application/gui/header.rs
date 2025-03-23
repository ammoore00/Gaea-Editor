use crate::application::gui::header::menu::Item;
use iced::{Border, Element, Task};
use iced::border::Radius;
use iced_aw::{menu, menu_bar, menu_items, Menu};
use iced_aw::menu::primary;
use iced_aw::style::Status;
use crate::application::gui::window;

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
        let menu = |items| Menu::new(items).max_width(180.0).offset(15.0).spacing(5.0);
        
        let menu_bar = menu_bar!(
            (iced::widget::button::Button::new("File"), menu(menu_items!(
                (iced::widget::button::Button::new("Test"))
            )))
        )
        .draw_path(menu::DrawPath::Backdrop)
        .style(|theme:&iced::Theme, status: Status| menu::Style {
            path_border: Border{
                radius: Radius::new(6.0),
                ..Default::default()
            },
            ..primary(theme, status)
        });
        
        menu_bar.into()
    }
}

pub enum Message {
    
}