use anyhow::Result;
use rand::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::{
    DefaultTerminal, Frame, Terminal,
    buffer::Buffer,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub(crate) struct Player {
    pub(crate) coord: (usize, usize),
    gold: usize,
    hp: usize,
    max_hp: usize,
    /// strength should be between 3 and 32
    strength: usize,
    /// maximal strength achieved so far
    max_str: usize,
    /// Armor protection
    arm: usize,
    /// experience level
    exp_lvl: usize,
    /// poinst needed to increase experience level
    exp_points: usize,
}

impl Player {
    fn status(&self) -> String {
        [
            format!("Gold:{}", self.gold),
            format!("Hp: {}({})", self.hp, self.max_hp),
            format!("Str: {}({})", self.strength, self.max_str),
            format!("Arm: {}", self.arm),
            format!("Exp: {}/{}", self.exp_lvl, self.exp_points),
        ]
        .join("\t")
    }

    fn move_to(&mut self, coord: (usize, usize)) {
        self.coord = coord;
    }

    fn fight(&mut self, monster: &Monster) {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Monster {
    name: String,
    #[serde(default)]
    coord: (usize, usize),
    #[serde(default)]
    hp: usize,
    min_hp: usize,
    max_hp: usize,
    #[serde(default)]
    strength: usize,
    min_strength: usize,
    max_strength: usize,
    /// can the monster move? will it try to go towards you?
    mobile: bool,
}

impl Monster {
    /// Generate a new monster
    fn generate(&self) -> Self {
        let mut rng = rand::rng();
        Self {
            hp: rng.random_range(self.min_hp..=self.max_hp),
            strength: rng.random_range(self.min_strength..=self.max_strength),
            ..self.clone()
        }
    }
}
