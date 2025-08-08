// SPDX-License-Identifier: MPL-2.0
#![feature(associated_type_defaults)]

use iced::{Font, Task};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::application::gui::window::ApplicationWindow;
use crate::application::app_context::AppContextBuilder;
use crate::application::gui::window;

mod application;
mod services;
mod data;
mod repositories;
mod database;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().unwrap()
});

pub fn main() -> iced::Result {
    setup_logging();
    
    iced::application("Gaea - Minecraft Resource and Datapack Editor", ApplicationWindow::update, ApplicationWindow::view)
        .theme(ApplicationWindow::theme)
        .font(include_bytes!("../resources/assets/fonts/icons.ttf").as_slice())
        .default_font(Font::DEFAULT)
        .run_with(create_application)
}

fn create_application() -> (ApplicationWindow, Task<window::Message>) {
    let app_context = AppContextBuilder::default().build();
    ApplicationWindow::new(app_context)
}

fn setup_logging() {
    let filter_directives = if cfg!(debug_assertions) {
        "gaea=debug,iced=info,warn"
    } else {
        "gaea=info,error"
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(filter_directives));

    let fmt_layer = layer()
        .with_target(true)
        .compact();
    
    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}