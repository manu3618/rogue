use std::fmt;
use std::fs::File;
use std::io::BufReader;

use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Widget, Wrap};
use serde::{Deserialize, Serialize};

// TODO add objects "❤" "⚒" "⚕" "⚗" "⚛" "⛀" "⛁" "▢" "⮅" "⮸"

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Object {
    pub(crate) name: String,
    pub(crate) description: String,

    #[serde(default)]
    /// Placement on the map (if relevant)
    pub(crate) coord: (usize, usize),

    #[serde(default)]
    pub(crate) gold: usize,

    /// increase the current value of HP by this amount
    #[serde(default)]
    pub(crate) increase_hp: i32,

    /// increase the maximal value of HP by this amount
    #[serde(default)]
    pub(crate) increase_max_hp: usize,

    /// increase the current value of strength by this amount
    #[serde(default)]
    pub(crate) increase_strength: i32,

    /// increase the maximal value of strength by this amount
    #[serde(default)]
    pub(crate) increase_max_strength: usize,

    /// set armor the this value (if greater than current one)
    #[serde(default)]
    pub(crate) armor: usize,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Object {
    pub(crate) fn collection_from_file(filepath: &str) -> Result<Vec<Self>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    /// Whether the object should be put in inventory or not
    /// Any object that can be consumed should be consummed and not kept
    pub(crate) fn should_keep(&self) -> bool {
        let keepable = [self.increase_hp > 0, self.increase_strength > 0];
        keepable.iter().any(|&x| x)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Inventory {
    objects: Vec<Object>,
    index: usize,
}

impl FromIterator<Object> for Inventory {
    fn from_iter<T: IntoIterator<Item = Object>>(iter: T) -> Self {
        Self {
            objects: iter.into_iter().collect(),
            ..Default::default()
        }
    }
}

impl<'a> FromIterator<&'a Object> for Inventory {
    fn from_iter<T: IntoIterator<Item = &'a Object>>(iter: T) -> Self {
        Self {
            objects: iter.into_iter().cloned().collect(),
            ..Default::default()
        }
    }
}

impl Inventory {
    fn len(&self) -> usize {
        self.objects.len()
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub(crate) fn select_next(&mut self) {
        if self.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.len();
    }

    pub(crate) fn select_previous(&mut self) {
        if self.is_empty() {
            return;
        }
        self.index = (self.index + self.len() - 1) % self.objects.len();
    }

    pub(crate) fn select_number(&mut self, idx: usize) {
        if idx < self.len() {
            self.index = idx
        }
    }

    pub(crate) fn pop_selected(&mut self) -> Option<Object> {
        if self.objects.is_empty() {
            return None;
        }
        Some(self.objects.swap_remove(self.index))
    }

    fn get_selected(&self) -> Option<&Object> {
        self.objects.get(self.index)
    }

    pub(crate) fn add_item(&mut self, obj: Object) {
        self.objects.push(obj);
    }

    fn format_menu(&self) -> String {
        if self.is_empty() {
            return "(empty)".into();
        }
        let content: Vec<String> = self
            .objects
            .iter()
            .enumerate()
            .map(|(idx, obj)| {
                format!(
                    "{} {:>2}. {}",
                    if idx == self.index { ">" } else { " " },
                    idx,
                    obj.name,
                )
            })
            .collect();
        content.join("\n")
    }
}

impl Widget for Inventory {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(35), Constraint::Length(35)])
            .margin(1)
            .split(area);
        Text::from(self.format_menu()).render(chunks[0], buf);
        Paragraph::new(Text::from(
            self.get_selected().cloned().unwrap_or_default().description,
        ))
        .wrap(Wrap { trim: false })
        .render(chunks[1], buf);
    }
}
