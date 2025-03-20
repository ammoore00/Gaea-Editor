// Copyright 2019 Héctor Ramón, Iced contributors
// This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

use std::ffi;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{Center, Element, Fill, keyboard, Task, widget, Font};
use iced::widget::{Column, horizontal_space, row, Row, text, text_editor, toggler};

use crate::gui::widgets::{action, new_icon, open_icon, save_icon};
use crate::gui::window;
use crate::data::version::MinecraftVersion;

pub mod highlighter;

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    WordWrapToggled(bool),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
    ThemeChanged(highlighter::Theme),
}

fn make_message(message: Message) -> window::Message {
    window::Message::TextEditorMessage(0, message)
}

pub struct TextEditor {
    theme: highlighter::Theme,
    file: Option<PathBuf>,
    content: text_editor::Content,
    word_wrap: bool,
    is_loading: bool,
    is_dirty: bool,
}

impl<'a> TextEditor {
    pub(crate) fn new(theme: highlighter::Theme) -> (Self, Task<window::Message>) {
        (
            Self {
                theme,
                file: None,
                content: text_editor::Content::new(),
                word_wrap: true,
                is_loading: true,
                is_dirty: false,
            },
            Task::batch([
                Task::perform(
                    load_file(format!(
                        "{}/tests/mcfunction/everything.mcfunction",
                        env!("CARGO_MANIFEST_DIR")
                    )),
                    |result| make_message(Message::FileOpened(result)),
                ),
                widget::focus_next(),
            ]),
        )
    }
    
    pub(crate) fn update(&mut self, message: Message) -> Task<window::Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                
                self.content.perform(action);
                
                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;
                
                Task::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }
                
                Task::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;
                    
                    Task::perform(open_file(), |result| make_message(Message::FileOpened(result)))
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;
                
                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }
                
                Task::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Task::none()
                } else {
                    self.is_loading = true;
                    
                    let mut text = self.content.text();
                    /*
                    if let Some(ending) = self.content.line_ending() {
                        if !text.ends_with(ending.as_str()) {
                            text.push_str(ending.as_str());
                        }
                    }
                    */
                    Task::perform(
                        save_file(self.file.clone(), text),
                        |result| make_message(Message::FileSaved(result)),
                    )
                }
            }
            Message::FileSaved(result) => {
                self.is_loading = false;
                
                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }
                
                Task::none()
            }
            Message::ThemeChanged(_) => todo!(),
        }
    }
    
    pub(crate) fn view(&self) -> Element<window::Message> {
        // Row macro didn't like external function calls
        let controls = Row::new()
            .push(action(new_icon(), "New file", Some(make_message(Message::NewFile))))
            .push(action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(make_message(Message::OpenFile))
            ))
            .push(action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(make_message(Message::SaveFile))
            ))
            .push(horizontal_space())
            .push(toggler(self.word_wrap)
                .label("Word Wrap")
                .on_toggle(|toggled| make_message(Message::WordWrapToggled(toggled)))
                .text_size(14)
            )
            .padding([5, 10])
            .spacing(10)
            .align_y(Center);
        
        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
            .spacing(10);
        
        let text_editor = text_editor(&self.content)
            .height(Fill)
            .on_action(|action| make_message(Message::ActionPerformed(action)))
            .wrapping(if self.word_wrap {
                text::Wrapping::Word
            } else {
                text::Wrapping::None
            })
            .key_binding(|key_press| {
                match key_press.key.as_ref() {
                    keyboard::Key::Character("s")
                    if key_press.modifiers.command() =>
                        {
                            Some(text_editor::Binding::Custom(
                                make_message(Message::SaveFile),
                            ))
                        }
                    _ => text_editor::Binding::from_key_press(key_press),
                }
            })
            .font(Font::MONOSPACE);
        
        let extension = self.file
            .as_deref()
            .and_then(Path::extension)
            .and_then(ffi::OsStr::to_str)
            .unwrap_or("json")
            .to_owned();
        
        Column::new()
                .push(controls)
                .push(text_editor.highlight_with::<highlighter::MinecraftHighlighter>(
                highlighter::Settings {
                        version: MinecraftVersion::default(),
                        theme: self.theme,
                        token: extension
                    },
                    |highlight, _theme| highlight.to_format()
                ))
                .push(status)
                .spacing(10)
                .padding(10)
                .into()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

async fn open_file() -> Result<(PathBuf, Arc<String>), Error> {
    let picked_file = rfd::AsyncFileDialog::new()
        .set_title("Open a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;
    
    load_file(picked_file).await
}

async fn load_file(
    path: impl Into<PathBuf>,
) -> Result<(PathBuf, Arc<String>), Error> {
    let path = path.into();
    
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;
    
    Ok((path, contents))
}

async fn save_file(
    path: Option<PathBuf>,
    contents: String,
) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .save_file()
            .await
            .as_ref()
            .map(rfd::FileHandle::path)
            .map(Path::to_owned)
            .ok_or(Error::DialogClosed)?
    };
    
    tokio::fs::write(&path, contents)
        .await
        .map_err(|error| Error::IoError(error.kind()))?;
    
    Ok(path)
}