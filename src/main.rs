// SPDX-License-Identifier: MPL-2.0

use iced::Font;

use crate::gui::window::ApplicationWindow;

mod gui;

pub fn main() -> iced::Result {
    iced::application("Gaea - Minecraft Resource and Datapack Editor", ApplicationWindow::update, ApplicationWindow::view)
        .theme(ApplicationWindow::theme)
        .font(include_bytes!("../resources/assets/fonts/icons.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run_with(ApplicationWindow::new)
}