use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::Widget,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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

#[derive(Default, Debug, Clone)]
pub(crate) struct Combat {
    player: Rc<RefCell<Player>>,
    monster: Monster,
    menu: Rc<RefCell<UIMenu<Action>>>,
}

impl Combat {
    pub(crate) fn new(player: Rc<RefCell<Player>>, monster: Monster) -> Self {
        Self {
            player,
            monster,
            menu: Rc::new(RefCell::new(UIMenu::new())),
        }
    }

    pub(crate) fn select_next(&self) {
        let menu = Rc::clone(&self.menu);
        let mut menu = menu.borrow_mut();
        menu.select_next();
    }

    pub(crate) fn select_previous(&self) {
        let menu = Rc::clone(&self.menu);
        let mut menu = menu.borrow_mut();
        menu.select_previous();
    }

    pub(crate) fn select_item(&self, idx: usize) {
        let menu = Rc::clone(&self.menu);
        let mut menu = menu.borrow_mut();
        menu.select_number(idx);
    }

    fn turn(&mut self) {
        todo!()
    }
}

impl Widget for Combat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let menu = Rc::clone(&self.menu);
        let menu = menu.borrow();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(menu.len() as u16)])
            .split(area);
        // TODO: render monster and player state
        // TODO: display action menu
        Text::from(format!("{}", menu)).render(chunks[1], buf);
    }
}

#[derive(Debug, Default, Clone)]
struct UIMenu<T> {
    items: Vec<T>,
    menu: Vec<String>,
    index: usize,
}

impl<T: EnumCategory + Serialize> UIMenu<T> {
    fn new() -> Self {
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

impl<T> fmt::Display for UIMenu<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        write!(f, "{content}")
    }
}

impl<T> UIMenu<T> {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn select_next(&mut self) {
        self.index = (self.index + 1) % self.len();
    }

    fn select_previous(&mut self) {
        self.index = (self.index + self.len() - 1) % self.items.len();
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
        Text::from(format!("{}\n", &self)).render(area, but)
    }
}
