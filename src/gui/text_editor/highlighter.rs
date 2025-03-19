// Copyright 2019 Héctor Ramón, Iced contributors
// This code is licensed under MIT license (see third-party-licenses/LICENSE-MIT or https://opensource.org/licenses/MIT)

/**
Substantial portions of code duplicated from iced_highlighter::lib.rs
Changes made to allow for non-built-in syntax highlighting
 */

use std::ops::{Deref, Range};
use dashmap::DashMap;
use iced::{Color, Font, font};
use iced::advanced::text::highlighter::Format;
use iced::highlighter::Highlighter;
use once_cell::sync::Lazy;
use syntect::{highlighting, parsing};
use crate::version::MinecraftVersion;

static SYNTAXES: Lazy<DashMap<MinecraftVersion, parsing::SyntaxSet>> =
    Lazy::new(|| {
        let map = DashMap::new();
        let default_version = MinecraftVersion::default();
        map.insert(default_version, load_syntax_set_for_version(default_version));
        map
    });

static THEMES: Lazy<highlighting::ThemeSet> =
    Lazy::new(highlighting::ThemeSet::load_defaults);

const LINES_PER_SNAPSHOT: usize = 50;

fn load_syntax_set_for_version(version: MinecraftVersion) -> parsing::SyntaxSet {
    let mut builder = parsing::SyntaxSet::load_defaults_nonewlines().into_builder();

    // Load the base mcfunction syntax
    builder.add_from_folder("./resources/assets/syntaxes/mcfunction/base.sublime-syntax", true).expect("Failed to load base syntax");

    // Version-specific command syntaxes
    let version_str = version.as_str();
    let commands_pattern = format!("./resources/assets/syntaxes/mcfunction/commands/*/{}.sublime-syntax", version_str);

    // Use glob to find all command syntaxes for the specified version
    for entry in glob::glob(&commands_pattern).expect("Failed to read command syntax pattern") {
        if let Ok(path) = entry {
            if let Err(err) = builder.add_from_folder(path, false) {
                eprintln!("Failed to load syntax file: {}", err);
            }
        }
    }

    builder.build()
}

fn get_syntax_set_for_version(version: MinecraftVersion) -> &'static parsing::SyntaxSet {
    SYNTAXES.entry(version)
        .or_insert_with(|| load_syntax_set_for_version(version));

    let reference = SYNTAXES.get(&version).unwrap();

    // This transmute is safe because SYNTAXES is 'static
    unsafe {
        std::mem::transmute::<&parsing::SyntaxSet, &'static parsing::SyntaxSet>(reference.deref())
    }
}

#[derive(Debug)]
pub(crate) struct MinecraftHighlighter {
    version: MinecraftVersion,
    syntax: &'static parsing::SyntaxReference,
    highlighter: highlighting::Highlighter<'static>,
    caches: Vec<(parsing::ParseState, parsing::ScopeStack)>,
    current_line: usize,
}

impl MinecraftHighlighter {
    pub fn set_minecraft_version(&mut self, version: MinecraftVersion) {
        self.version = version;

        // Pre-load the syntax set if it's not already loaded
        if !SYNTAXES.contains_key(&version) {
            SYNTAXES.insert(version, load_syntax_set_for_version(version));
        }
    }

    pub fn get_minecraft_version(&self) -> MinecraftVersion {
        self.version.clone()
    }
}

impl iced::advanced::text::highlighter::Highlighter for MinecraftHighlighter {
    type Settings = Settings;
    type Highlight = Highlight;
    
    type Iterator<'a> =
    Box<dyn Iterator<Item = (Range<usize>, Self::Highlight)> + 'a>;
    
    fn new(settings: &Self::Settings) -> Self {
        let version = settings.version.clone();

        let syntax_set = get_syntax_set_for_version(version);
        let syntax = syntax_set.find_syntax_by_token(&settings.token).expect(format!("Failed to find syntax for token {}", settings.token).as_str());
        
        let highlighter = highlighting::Highlighter::new(
            &THEMES.themes[settings.theme.key()],
        );
        
        let parser = parsing::ParseState::new(syntax);
        let stack = parsing::ScopeStack::new();
        
        MinecraftHighlighter {
            version,
            syntax,
            highlighter,
            caches: vec![(parser, stack)],
            current_line: 0,
        }
    }
    
    fn update(&mut self, new_settings: &Self::Settings) {
        let version = new_settings.version.clone();

        let syntax_set = get_syntax_set_for_version(version);
        self.syntax = syntax_set.find_syntax_by_token(&new_settings.token).expect(format!("Failed to find syntax for token {}", new_settings.token).as_str());
        
        self.highlighter = highlighting::Highlighter::new(
            &THEMES.themes[new_settings.theme.key()],
        );
        
        // Restart the highlighter
        self.change_line(0);
    }
    
    fn change_line(&mut self, line: usize) {
        let snapshot = line / LINES_PER_SNAPSHOT;
        
        if snapshot <= self.caches.len() {
            self.caches.truncate(snapshot);
            self.current_line = snapshot * LINES_PER_SNAPSHOT;
        } else {
            self.caches.truncate(1);
            self.current_line = 0;
        }
        
        let (parser, stack) =
            self.caches.last().cloned().unwrap_or_else(|| {
                (
                    parsing::ParseState::new(self.syntax),
                    parsing::ScopeStack::new(),
                )
            });
        
        self.caches.push((parser, stack));
    }
    
    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_> {
        if self.current_line / LINES_PER_SNAPSHOT >= self.caches.len() {
            let (parser, stack) =
                self.caches.last().expect("Caches must not be empty");
            
            self.caches.push((parser.clone(), stack.clone()));
        }
        
        self.current_line += 1;
        
        let (parser, stack) =
            self.caches.last_mut().expect("Caches must not be empty");
        
        let ops = parser.parse_line(line, get_syntax_set_for_version(self.version)).unwrap_or_default();
        
        let highlighter = &self.highlighter;
        
        Box::new(
            ScopeRangeIterator {
                ops,
                line_length: line.len(),
                index: 0,
                last_str_index: 0,
            }
                .filter_map(move |(range, scope)| {
                    let _ = stack.apply(&scope);
                    
                    if range.is_empty() {
                        None
                    } else {
                        Some((
                            range,
                            Highlight(
                                highlighter.style_mod_for_stack(&stack.scopes),
                            ),
                        ))
                    }
                }),
        )
    }
    
    fn current_line(&self) -> usize {
        self.current_line
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub version: MinecraftVersion,
    pub theme: Theme,
    pub token: String,
}

#[derive(Debug)]
pub struct Highlight(highlighting::StyleModifier);

impl Highlight {
    /// Returns the color of this [`iced::highlighter::Highlight`].
    ///
    /// If `None`, the original text color should be unchanged.
    pub fn color(&self) -> Option<Color> {
        self.0.foreground.map(|color| {
            Color::from_rgba8(color.r, color.g, color.b, color.a as f32 / 255.0)
        })
    }
    
    /// Returns the font of this [`Highlight`].
    ///
    /// If `None`, the original font should be unchanged.
    pub fn font(&self) -> Option<Font> {
        self.0.font_style.and_then(|style| {
            let bold = style.contains(highlighting::FontStyle::BOLD);
            let italic = style.contains(highlighting::FontStyle::ITALIC);
            
            if bold || italic {
                Some(Font {
                    weight: if bold {
                        font::Weight::Bold
                    } else {
                        font::Weight::Normal
                    },
                    style: if italic {
                        font::Style::Italic
                    } else {
                        font::Style::Normal
                    },
                    ..Font::MONOSPACE
                })
            } else {
                None
            }
        })
    }
    
    /// Returns the [`Format`] of the [`Highlight`].
    ///
    /// It contains both the [`color`] and the [`font`].
    ///
    /// [`color`]: Self::color
    /// [`font`]: Self::font
    pub fn to_format(&self) -> Format<Font> {
        Format {
            color: self.color(),
            font: self.font(),
        }
    }
}

/// A highlighting theme.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    SolarizedDark,
    Base16Mocha,
    Base16Ocean,
    Base16Eighties,
    InspiredGitHub,
}

impl Theme {
    /// A static slice containing all the available themes.
    pub const ALL: &'static [Self] = &[
        Self::SolarizedDark,
        Self::Base16Mocha,
        Self::Base16Ocean,
        Self::Base16Eighties,
        Self::InspiredGitHub,
    ];
    
    /// Returns `true` if the [`Theme`] is dark, and false otherwise.
    pub fn is_dark(self) -> bool {
        match self {
            Self::SolarizedDark
            | Self::Base16Mocha
            | Self::Base16Ocean
            | Self::Base16Eighties => true,
            Self::InspiredGitHub => false,
        }
    }
    
    fn key(self) -> &'static str {
        match self {
            Theme::SolarizedDark => "Solarized (dark)",
            Theme::Base16Mocha => "base16-mocha.dark",
            Theme::Base16Ocean => "base16-ocean.dark",
            Theme::Base16Eighties => "base16-eighties.dark",
            Theme::InspiredGitHub => "InspiredGitHub",
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::SolarizedDark => write!(f, "Solarized Dark"),
            Theme::Base16Mocha => write!(f, "Mocha"),
            Theme::Base16Ocean => write!(f, "Ocean"),
            Theme::Base16Eighties => write!(f, "Eighties"),
            Theme::InspiredGitHub => write!(f, "Inspired GitHub"),
        }
    }
}

struct ScopeRangeIterator {
    ops: Vec<(usize, parsing::ScopeStackOp)>,
    line_length: usize,
    index: usize,
    last_str_index: usize,
}

impl Iterator for ScopeRangeIterator {
    type Item = (Range<usize>, parsing::ScopeStackOp);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.ops.len() {
            return None;
        }
        
        let next_str_i = if self.index == self.ops.len() {
            self.line_length
        } else {
            self.ops[self.index].0
        };
        
        let range = self.last_str_index..next_str_i;
        self.last_str_index = next_str_i;
        
        let op = if self.index == 0 {
            parsing::ScopeStackOp::Noop
        } else {
            self.ops[self.index - 1].1.clone()
        };
        
        self.index += 1;
        Some((range, op))
    }
}