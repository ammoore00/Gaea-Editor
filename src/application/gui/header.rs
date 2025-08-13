use std::sync::Arc;
use crate::services::translation_service::TranslationKey;
use crate::application::gui::header::menu::Item;
use iced::{Border, Element, Length, Task};
use iced::border::Radius;
use iced::widget::Button;
use iced_aw::{menu, menu_bar, menu_items, Menu};
use iced_aw::style::Status;
use crate::application::app_context::AppContext;
use crate::application::gui::window;

const MENU_WIDTH: f32 = 180.0;
const MENU_OFFSET: f32 = 8.0;
const MENU_SPACING: f32 = 0.0;

#[derive(Debug, Clone)]
pub enum Message {
    TranslationsUpdated(HeaderTranslations),
}

pub struct Header {
    app_context: Arc<AppContext>,
    
    translations: HeaderTranslations,
}

impl Header {
    pub fn with_task(app_context: Arc<AppContext>) -> (Self, Task<window::Message>) {
        let self_ = Self {
            app_context: app_context.clone(),
            
            translations: HeaderTranslations::default(),
        };
        
        (self_, Task::perform(Self::translate(app_context.clone()), |translation| Message::TranslationsUpdated(translation).into()))
    }

    pub fn update(&mut self, message: Message) -> Task<window::Message> {
        match message {
            Message::TranslationsUpdated(translations) => {
                self.translations = translations;
            },
        }
        
        // TODO: implement header menu bar
        Task::none()
    }

    pub fn view(&self) -> Element<window::Message> {
        let menu_bar = menu_bar!(
            (Button::new(self.translations.file_menu.title.as_str()), self.create_file_menu())
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
    
    async fn translate(app_context: Arc<AppContext>) -> HeaderTranslations {
        let file_menu_translations = {
            let title = app_context.translation_service_context().read().await.translate(&FileMenuTranslationKeys::Title);
            let import = app_context.translation_service_context().read().await.translate(&FileMenuTranslationKeys::Import);
            let export = app_context.translation_service_context().read().await.translate(&FileMenuTranslationKeys::Export);
            
            FileMenuTranslations {
                title,
                import,
                export,
            }
        };
        
        HeaderTranslations {
            file_menu: file_menu_translations,
        }
    }
    
    fn create_file_menu(&self) -> Menu<'_, window::Message, iced::Theme, iced::Renderer> {
        let file_translations = &self.translations.file_menu;
        
        Menu::new(menu_items!(
            (Button::new(file_translations.import.as_str()).width(Length::Fill).style(iced::widget::button::secondary))
            (Button::new(file_translations.export.as_str()).width(Length::Fill).style(iced::widget::button::secondary))
        ))
        .max_width(MENU_WIDTH)
        .offset(MENU_OFFSET)
        .spacing(MENU_SPACING)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, translation_macro::TranslationKey)]
pub enum FileMenuTranslationKeys {
    #[translation(en_us = "File")]
    Title,
    #[translation(en_us = "Import Project")]
    Import,
    #[translation(en_us = "Export Project")]
    Export,
}

#[derive(Debug, Clone, Default)]
pub struct HeaderTranslations {
    file_menu: FileMenuTranslations,
}

#[derive(Debug, Clone, Default)]
pub struct FileMenuTranslations {
    pub title: String,
    pub import: String,
    pub export: String,
}