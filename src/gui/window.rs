// SPDX-License-Identifier: MPL-2.0

use iced::{Element, Length, Task, Theme};
use iced::widget::{Column, Container, pane_grid, PaneGrid, Row};
use iced::widget::pane_grid::Axis;
use crate::gui::text_editor;
use crate::gui::text_editor::{highlighter, TextEditor};

#[derive(Debug, Clone)]
pub enum Message {
    ResizedPane(pane_grid::ResizeEvent),
    ClickedPane(pane_grid::Pane),
    
    ThemeSelected(highlighter::Theme),
    
    TextEditorMessage(i32, text_editor::Message),
}

pub struct ApplicationWindow {
    theme: highlighter::Theme,
    
    panes: pane_grid::State<PaneState>,
    focus: Option<pane_grid::Pane>,
    
    text_editors: Vec<TextEditor>
}

impl ApplicationWindow {
    pub fn new() -> (Self, Task<Message>) {
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
        
        let mut window = Self {
            theme: highlighter::Theme::SolarizedDark,
            
            panes,
            focus: None,
            
            text_editors: Vec::new(),
        };
        
        let editor = TextEditor::new(window.theme.clone());
        window.text_editors.push(editor.0);
        
        (window, editor.1)
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
            // TODO: Fix indexes not being used
            Message::TextEditorMessage(index, message) => self.text_editors[0].update(message),
        }
    }
    
    pub fn view(&self) -> Element<Message> {
        let header_menu = Container::new(iced::widget::text("Header Menu"));
        
        let main_view = PaneGrid::new(&self.panes, |pane, state, is_maximized| {
            pane_grid::Content::new(
                match state.pane_type {
                    PaneType::FileTree => Container::new(iced::widget::text("File Tree")),
                    PaneType::MainContent => Container::new(self.text_editors[0].view()),
                    PaneType::Preview => Container::new(iced::widget::text("Preview")),
                })
        })
            .width(Length::Fill)
            .height(Length::Fill)
            .on_click(Message::ClickedPane)
            .on_resize(10, Message::ResizedPane);
        
        let total_window = Column::new()
            .push(header_menu)
            .push(main_view);
        
        Container::new(total_window)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    pub(crate) fn theme(&self) -> Theme {
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