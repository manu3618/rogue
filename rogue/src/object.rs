use anyhow::Result;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Paragraph, Widget, Wrap},
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Object {
    pub(crate) name: String,
    pub(crate) description: String,

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
    pub(crate) increase_max_strength: i32,

    /// set armor the this value (if greater than current one)
    #[serde(default)]
    pub(crate) armor: usize,
}

impl Object {
    pub(crate) fn collection_from_file(filepath: &str) -> Result<Vec<Self>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }
}

#[derive(Debug, Default, Clone)]
struct ObjectMenu {
    objects: Vec<Object>,
    index: usize,
}

impl ObjectMenu {
    fn len(&self) -> usize {
        self.objects.len()
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    fn select_next(&mut self) {
        if self.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.len();
    }

    fn select_previous(&mut self) {
        if self.is_empty() {
            return;
        }
        self.index = (self.index + self.len() - 1) % self.objects.len();
    }

    fn select_number(&mut self, idx: usize) {
        if idx < self.len() {
            self.index = idx
        }
    }

    fn pop_selected(&mut self) -> Option<Object> {
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

impl Widget for ObjectMenu {
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
