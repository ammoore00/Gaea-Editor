// SPDX-License-Identifier: MPL-2.0

use iced::{Element, highlighter, Length, Task, Theme};
use iced::widget::{Column, Container, pane_grid, PaneGrid, Row};
use iced::widget::pane_grid::Axis;

#[derive(Debug, Clone)]
pub enum Message {
    ResizedPane(pane_grid::ResizeEvent),
    ClickedPane(pane_grid::Pane),
}

pub struct ApplicationWindow {
    theme: highlighter::Theme,
    
    panes: pane_grid::State<PaneState>,
    focus: Option<pane_grid::Pane>,
}

impl ApplicationWindow {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }
    
    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }
    
    pub fn view(&self) -> Element<Message> {
        let header_menu = Container::new(iced::widget::text("Header Menu"));
        
        let main_view = PaneGrid::new(&self.panes, |pane, state, is_maximized| {
            pane_grid::Content::new(
                match state.pane_type {
                    PaneType::FileTree => Container::new(iced::widget::text("File Tree")),
                    PaneType::MainContent => Container::new(iced::widget::text("Main Content")),
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

impl Default for ApplicationWindow {
    fn default() -> Self {
        let file_tree_pane = PaneState::new(PaneType::FileTree);
        let main_content_pain = PaneState::new(PaneType::MainContent);
        let preview_pane = PaneState::new(PaneType::Preview);
        
        let panes = pane_grid::State::with_configuration(
            pane_grid::Configuration::Split{
                axis: Axis::Vertical,
                ratio: 0.2,
                a: Box::new(pane_grid::Configuration::Pane(file_tree_pane)),
                b: Box::new(pane_grid::Configuration::Split{
                    axis: Axis::Vertical,
                    ratio: 0.66,
                    a: Box::new(pane_grid::Configuration::Pane(main_content_pain)),
                    b: Box::new(pane_grid::Configuration::Pane(preview_pane)),
                }),
            });
        
        Self {
            theme: highlighter::Theme::SolarizedDark,
            
            panes,
            focus: None
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