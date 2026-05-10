use crate::Monster;
use crate::Player;
use rand::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

pub(crate) enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub(crate) struct Map {
    /// Map, coord (line, column) with line (0, 0) at top left,
    /// (line_nb - 1, row_nb - 1) at bottom right
    map: Vec<Vec<char>>,
    /// Part of the map already discovered. Used to redraw walls.
    discovered_map: Vec<Vec<char>>,
    /// Part of the map that should be displayed
    displayed_map: Vec<Vec<char>>,
    line_nb: usize,
    row_nb: usize,
    player: Player,
    monsters: Vec<Monster>,
}

impl Widget for Map {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content: Vec<String> = self
            .displayed_map
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect();
        let content = content.join("\n");
        Text::from(content).render(area, buf)
    }
}

impl Default for Map {
    fn default() -> Self {
        let line_nb = 23;
        let row_nb = 80;
        let map = vec![vec![' '; row_nb]; line_nb];
        let discovered_map = map.clone();
        let displayed_map = map.clone();
        let player = Player::default();
        let monsters = Vec::new();
        Self {
            map,
            discovered_map,
            displayed_map,
            line_nb,
            row_nb,
            player,
            monsters,
        }
    }
}

impl Map {
    /// Generate a level and place the player in it
    fn new(level_nb: u8, player: &mut Player) -> Self {
        let mut map = Self::default();
        // TODO: generate level
        map.generate_empty();
        // TODO: generate monsters
        // TODO: generate loot
        // TODO: place player
        map.player.coord = (map.line_nb / 2, map.row_nb / 2);
        map
    }

    fn generate_empty(&mut self) {
        if self.line_nb < 9 {
            self.line_nb = 23
        }
        if self.row_nb < 9 {
            self.row_nb = 80
        }
        self.map = vec![vec![' '; self.row_nb]; self.line_nb];
        self.discovered_map = vec![vec![' '; self.row_nb]; self.line_nb];
    }

    pub(crate) fn move_player(&mut self, direction: MoveDirection) {
        let curr_coords = self.player.coord;
        let new_coords = match direction {
            MoveDirection::Right => (curr_coords.0, curr_coords.1 + 1),
            MoveDirection::Left => (curr_coords.0, curr_coords.1 - 1),
            MoveDirection::Up => (curr_coords.0 - 1, curr_coords.1),
            MoveDirection::Down => (curr_coords.0 + 1, curr_coords.1),
        };
        match self.get(new_coords) {
            Some('|') | Some('-') => {} // wall, do nothing
            Some('+') | Some('#') => self.player.coord = new_coords, // corridor
            Some(' ') | Some('.') => self.player.coord = new_coords, // empty room
            Some(c) => {
                todo!()
            } //monster
            _ => {}
        }
        self.set(curr_coords, self.discovered_map[curr_coords.0][curr_coords.1]);
        self.set(new_coords, '@');
        self.displayed_map = self.discovered_map.clone();
        self.displayed_map[new_coords.0][new_coords.1] = '@';

    }

    fn get(&self, coords: (usize, usize)) -> Option<char> {
        self.displayed_map.get(coords.0)?.get(coords.1).copied()
    }

    fn set(&mut self, coords:(usize, usize), value: char) {
        *self.map.get_mut(coords.0).unwrap().get_mut(coords.1).unwrap() = value;
    }
    pub fn size(&self) -> (usize, usize) {
        (self.line_nb, self.row_nb)
    }
}
