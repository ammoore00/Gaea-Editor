// SPDX-License-Identifier: MPL-2.0
#![feature(associated_type_defaults)]

use iced::Font;

use application::gui::window::ApplicationWindow;

mod domain;
mod application;
pub mod repositories;
pub mod services;
mod database;

pub fn main() -> iced::Result {
    iced::application("Gaea - Minecraft Resource and Datapack Editor", ApplicationWindow::update, ApplicationWindow::view)
        .theme(ApplicationWindow::theme)
        .font(include_bytes!("../resources/assets/fonts/icons.ttf").as_slice())
        .default_font(Font::DEFAULT)
        .run_with(ApplicationWindow::new)
}