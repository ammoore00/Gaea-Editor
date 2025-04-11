// SPDX-License-Identifier: MPL-2.0
#![feature(associated_type_defaults)]

use iced::Font;

use application::gui::window::ApplicationWindow;

mod application;
pub mod services;
mod data;
mod persistence;

pub fn main() -> iced::Result {
    iced::application("Gaea - Minecraft Resource and Datapack Editor", ApplicationWindow::update, ApplicationWindow::view)
        .theme(ApplicationWindow::theme)
        .font(include_bytes!("../resources/assets/fonts/icons.ttf").as_slice())
        .default_font(Font::DEFAULT)
        .run_with(ApplicationWindow::new)
}