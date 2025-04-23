// SPDX-License-Identifier: MPL-2.0
#![feature(associated_type_defaults)]

use iced::{Font, Task};

use crate::application::gui::window::ApplicationWindow;
use crate::application::app_context::{AppContextBuilder, DefaultAdapterProvider};
use crate::application::gui::window;

mod application;
mod services;
mod data;
mod repositories;
mod database;

pub fn main() -> iced::Result {
    iced::application("Gaea - Minecraft Resource and Datapack Editor", ApplicationWindow::update, ApplicationWindow::view)
        .theme(ApplicationWindow::theme)
        .font(include_bytes!("../resources/assets/fonts/icons.ttf").as_slice())
        .default_font(Font::DEFAULT)
        .run_with(create_application)
}

fn create_application() -> (ApplicationWindow<DefaultAdapterProvider>, Task<window::Message>) {
    let app_context = AppContextBuilder::default().build();
    ApplicationWindow::new(app_context)
}