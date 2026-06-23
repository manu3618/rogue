// use anyhow::Result;
use crate::object::Object;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

// TODO: add monsters "⌁" "❀" "☃" "☠" "🌢" "♘"

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
    /// points needed to increase experience level
    exp_points: usize,
}

impl Player {
    pub fn new() -> Self {
        Self {
            hp: 10,
            max_hp: 10,
            strength: 10,
            max_str: 10,
            ..Default::default()
        }
    }

    pub(crate) fn status(&self) -> String {
        [
            format!("Gold: {}", self.gold),
            format!("Hp: {}({})", self.hp, self.max_hp),
            format!("Str: {}({})", self.strength, self.max_str),
            format!("Arm: {}", self.arm),
            format!("Lvl: {}/{}", self.exp_lvl, self.exp_points),
        ]
        .join(" \t")
    }

    fn move_to(&mut self, coord: (usize, usize)) {
        self.coord = coord;
    }

    pub(crate) fn use_object(&mut self, object: Object) {
        self.gold += object.gold;
        self.max_hp += object.increase_max_hp;
        self.hp = self
            .max_hp
            .min((self.hp as i32 + object.increase_hp) as usize);
        self.max_str += object.increase_max_strength;
        self.strength =
            (0.max(object.increase_strength + self.strength as i32) as usize).min(self.max_str);
        self.arm = self.arm.max(object.armor);
    }

    pub(crate) fn get_strength(&self) -> usize {
        self.strength
    }

    pub(crate) fn get_armor(&self) -> usize {
        self.arm
    }

    pub(crate) fn get_damage(&mut self, damage: usize) {
        self.hp = if self.hp > damage {
            self.hp - damage
        } else {
            0
        }
    }

    pub(crate) fn increase_exp(&mut self, amount: usize) {
        if self.exp_points <= amount {
            self.exp_points = 2_usize.pow(self.exp_lvl as u32);
            self.exp_lvl += 1;
            self.max_hp += self.exp_lvl;
            self.max_str += self.exp_lvl;
        } else {
            self.exp_points -= amount
        }
    }

    pub(crate) fn refill_health(&mut self) {
        self.strength = self.max_str;
        self.hp = self.max_hp
    }

    pub(crate) fn is_dead(&self) -> bool {
        self.hp == 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Monster {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    pub(crate) coord: (usize, usize),
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

impl fmt::Display for Monster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Monster {
    /// Generate a new monster
    pub(crate) fn generate() -> Self {
        let mut rng = rand::rng();
        let models: Vec<Monster> =
            serde_json::from_str(include_str!("../data/monsters.json")).unwrap();
        let model = models.choose(&mut rng).unwrap();
        model.from_model(&mut rng)
    }
    fn from_model(&self, rng: &mut ThreadRng) -> Self {
        Self {
            hp: rng.random_range(self.min_hp..=self.max_hp),
            strength: rng.random_range(self.min_strength..=self.max_strength),
            ..self.clone()
        }
    }
    pub(crate) fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub(crate) fn get_strength(&self) -> usize {
        self.strength
    }

    pub(crate) fn get_damage(&mut self, damage: usize) {
        self.hp = if damage > self.hp {
            0
        } else {
            self.hp - damage
        };
    }

    pub(crate) fn get_description(&self) -> &str {
        self.description.as_str()
    }

    pub(crate) fn is_dead(&self) -> bool {
        self.hp == 0
    }
}
