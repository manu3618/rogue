use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget};
use serde::{Deserialize, Serialize};

use crate::{Monster, Player};
use rogue_macro::Category;
use rogue_trait::EnumCategory;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Category)]
enum Action {
    Magic, // TODO: define spell
    /// Physical attack with current equipement
    #[default]
    Physical,
    Flee,
    /// Do nothing
    Pass,
}

struct Combat {
    player: Player,
    monster: Monster,
}

impl Combat {
    fn new(player: Player, monster: Monster) -> Self {
        Self { player, monster }
    }

    fn turn(&mut self) {
        todo!()
    }
}

impl Widget for Combat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        todo!()
    }
}

#[derive(Debug)]
struct UIMenu<T> {
    items: Vec<T>,
    menu: Vec<String>,
    index: usize,
}

impl<T: EnumCategory + Serialize> UIMenu<T> {
    fn new(menu_type: T) -> Self {
        let items: Vec<T> = T::categories();
        let menu: Vec<String> = items
            .iter()
            .map(|elt| {
                let s = serde_json::to_string(&elt).unwrap();
                String::from(&s[1..s.len() - 1])
            })
            .collect();
        Self {
            items,
            menu,
            index: 0,
        }
    }
}

impl<T> UIMenu<T> {
    fn select_next(&mut self) {
        self.index = (self.index + 1) % self.items.len();
    }

    fn select_previous(&mut self) {
        self.index = (self.index + self.items.len() - 1) % self.items.len();
    }

    fn select_number(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.index = idx
        }
    }

    fn get_select(&self) -> &T {
        self.items
            .get(self.index)
            .expect("index should be less than item length")
    }
}

impl<T> Widget for UIMenu<T> {
    fn render(self, area: Rect, but: &mut Buffer) {
        let content: Vec<String> = self
            .menu
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                format!(
                    "{} {:>2}. {}",
                    if idx == self.index { ">" } else { " " },
                    idx,
                    line
                )
            })
            .collect();
        let content = content.join("\n");
        Text::from(content).render(area, but)
    }
}
