// use anyhow::Result;
use crate::object::Object;
use rand::prelude::*;
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

    fn use_object(&mut self, object: Object) {
        self.gold += object.gold;
        self.max_hp += object.increase_max_hp;
        self.hp = self
            .max_hp
            .min((self.hp as i32 + object.increase_hp) as usize);
        self.strength = 0.max(object.increase_strength + self.strength as i32) as usize;
        self.arm = self.arm.max(object.armor);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Monster {
    name: String,
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
}
