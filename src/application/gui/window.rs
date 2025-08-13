// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;
use iced::{Element, Length, Task, Theme};
use iced::widget::{Column, Container, pane_grid, PaneGrid};
use iced::widget::pane_grid::Axis;
use crate::application::app_context::AppContext;
use crate::application::gui::header::Header;
use crate::application::gui::{header, text_editor};
use crate::application::gui::text_editor::{highlighter, TextEditor};

#[derive(Debug, Clone)]
pub enum Message {
    // Global messages
    ThemeSelected(highlighter::Theme),
    
    // Main window messages
    ResizedPane(pane_grid::ResizeEvent),
    ClickedPane(pane_grid::Pane),
    
    // Element messages
    TextEditorMessage(text_editor::Message),
    HeaderMessage(header::Message),
}

impl From<text_editor::Message> for Message {
    fn from(value: text_editor::Message) -> Self {
        Message::TextEditorMessage(value)
    }
}

impl From<header::Message> for Message {
    fn from(value: header::Message) -> Self {
        Message::HeaderMessage(value)
    }
}

pub struct ApplicationWindow {
    theme: highlighter::Theme,
    
    panes: pane_grid::State<PaneState>,
    focus: Option<pane_grid::Pane>,
    
    header: Header,
    text_editor: TextEditor,
    
    app_context: Arc<AppContext>,
}

impl ApplicationWindow {
    pub fn new(app_context: AppContext) -> (Self, Task<Message>) {
        let app_context = Arc::new(app_context);

        let file_tree_pane = PaneState::new(PaneType::FileTree);
        let main_content_pane = PaneState::new(PaneType::MainContent);
        let preview_pane = PaneState::new(PaneType::Preview);
        
        let panes = pane_grid::State::with_configuration(
            pane_grid::Configuration::Split{
                axis: Axis::Vertical,
                ratio: 0.2,
                a: Box::new(pane_grid::Configuration::Pane(file_tree_pane)),
                b: Box::new(pane_grid::Configuration::Split{
                    axis: Axis::Vertical,
                    ratio: 0.66,
                    a: Box::new(pane_grid::Configuration::Pane(main_content_pane)),
                    b: Box::new(pane_grid::Configuration::Pane(preview_pane)),
                }),
            });

        let theme = highlighter::Theme::SolarizedDark;

        let (header, header_message) = Header::with_task(app_context.clone());
        let (text_editor, editor_message) = TextEditor::with_task(theme.clone());
        
        let window = Self {
            theme,
            
            panes,
            focus: None,
            
            header,
            text_editor,
            
            app_context,
        };
        
        (window, Task::batch([
            header_message,
            editor_message,
        ]))
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::ResizedPane(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
                Task::none()
            }
            Message::ClickedPane(pane) => {
                self.focus = Some(pane);
                Task::none()
            }
            Message::TextEditorMessage(message) => self.text_editor.update(message),
            Message::HeaderMessage(message) => self.header.update(message),
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header_menu = Container::new(self.header.view());
        let action_menu = Container::new(iced::widget::text("Action Menu"));
        
        let main_view = PaneGrid::new(&self.panes, |pane, state, is_maximized| {
            pane_grid::Content::new(
                match state.pane_type {
                    PaneType::FileTree => Container::new(iced::widget::text("File Tree")),
                    PaneType::MainContent => Container::new(self.text_editor.view()),
                    PaneType::Preview => Container::new(iced::widget::text("Preview")),
                })
        })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_click(Message::ClickedPane)
            .on_resize(10, Message::ResizedPane);
        
        let total_window = Column::new()
            .push(header_menu)
            .push(action_menu)
            .push(main_view);
        
        Container::new(total_window)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    pub fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

//------------//

#[derive(Debug, Clone)]
struct PaneState {
    pane_type: PaneType,
}

impl PaneState {
    fn new(pane_type: PaneType) -> Self {
        Self {
            pane_type,
        }
    }
}

//------------//

#[derive(Debug, Clone)]
enum PaneType {
    FileTree,
    MainContent,
    Preview
}