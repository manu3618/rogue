use rand::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use tracing::{info, instrument};

use crate::{Monster, Player};
use rogue_macro::Category;
use rogue_trait::EnumCategory;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Category)]
enum Action {
    // Magic, // TODO: add magic (or not)
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
    player_advantage: bool,
    monster_advantage: bool,
    round: usize,
    pub(crate) log_messages: Vec<String>,
}

impl Combat {
    pub(crate) fn new(player: Rc<RefCell<Player>>, monster: Monster) -> Self {
        Self {
            player,
            monster,
            menu: Rc::new(RefCell::new(UIMenu::<Action>::new())),
            player_advantage: false,
            monster_advantage: false,
            round: 0,
            log_messages: Vec::new(),
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

    pub(crate) fn validate(&mut self) {
        let menu = Rc::clone(&self.menu);
        let menu = menu.borrow();
        match menu.get_selected() {
            // Action::Magic => todo!(),
            Action::Flee => todo!(),
            Action::Physical => self.strike(),
            Action::Pass => {}
        }
    }

    pub(crate) fn previous(&self) {
        // TODO: implement navigation through menus
    }

    pub(crate) fn is_over(&self) -> bool {
        self.player.borrow().is_dead() || self.monster.is_dead()
    }

    #[instrument]
    fn strike(&mut self) {
        info!("beginning round {}", self.round);
        self.round += 1;
        let player = Rc::clone(&self.player);
        let mut player = player.borrow_mut();

        // Player stike monster
        info!("player attack");
        let proba = dice(20, self.player_advantage, self.monster_advantage);
        let attack_success = match proba {
            1 => false, // critical failure
            20 => true, // critical success
            d => player.get_strength() + d > self.monster.get_strength(),
        };
        if attack_success {
            info!("player attack succeed");
            let damage = dice(20, false, false) + player.get_strength();
            info!("monster get {damage} damage");
            self.monster.get_damage(damage);
        } else {
            info!("player attack failed");
        }

        if self.monster.is_dead() {
            return;
        }

        // Monster strike player
        info!("monster attack");
        let proba = dice(20, self.monster_advantage, self.player_advantage);
        let attack_success = match proba {
            1 => false, // critical failure
            20 => true, // critical success
            d => self.monster.get_strength() + d > player.get_strength(),
        };
        if attack_success {
            info!("monster attack succeed");
            let damage = dice(20, false, false) + self.monster.get_strength();
            let damage = if player.get_armor() > damage {
                0
            } else {
                damage - player.get_armor()
            };
            info!("player get {damage} damage");
            player.get_damage(damage);
        } else {
            info!("monster attack failed");
        }
    }
}

impl Widget for Combat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let menu = Rc::clone(&self.menu);
        let menu = menu.borrow();
        let lines_blocks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length((menu.len() as u16).max(self.log_messages.len() as u16 + 4)),
            ])
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20),
                Constraint::Length(30),
                Constraint::Length(30),
            ])
            .split(lines_blocks[1]);

        // TODO: render monster
        let monster = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(10)])
            .split(chunks[1]);
        Text::from(format!("{}", self.monster).red()).render(monster[0], buf);
        Paragraph::new(Text::from(self.monster.get_description()))
            .wrap(Wrap { trim: false })
            .render(monster[1], buf);

        // TODO: display action menu
        Text::from(format!("{}", menu)).render(chunks[0], buf);

        // TODO: display log messages
        Paragraph::new(Text::from(
            self.log_messages
                .iter()
                .map(|s| String::from(s))
                .collect::<Vec<_>>()
                .join("\n"),
        ))
        .block(Block::bordered().title("last events"))
        .render(chunks[2], buf);
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

    fn get_selected(&self) -> &T {
        self.items
            .get(self.index)
            .expect("index should be less than item length")
    }
}

impl<T> Widget for UIMenu<T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Text::from(format!("{}\n", &self)).render(area, buf)
    }
}

#[instrument]
fn dice(d: usize, advantage: bool, disavantage: bool) -> usize {
    let mut rng = rand::rng();
    let res = match (advantage, disavantage) {
        (true, false) => rng.random_range(1..=d).max(rng.random_range(1..=d)),
        (false, true) => rng.random_range(1..=d).min(rng.random_range(1..=d)),
        _ => rng.random_range(1..=d),
    };
    info!("🎲: {}", &res);
    if res == 1 {
        info!("critical failure");
    }
    if res == d {
        info!("critical sucess");
    }
    d
}
