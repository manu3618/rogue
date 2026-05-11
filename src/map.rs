use crate::Monster;
use crate::Player;
use itertools::iproduct;
use rand::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub(crate) player: Rc<RefCell<Player>>,
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
        let player = Rc::new(RefCell::new(Player::default()));
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
    pub fn new(level_nb: u8, player: Rc<RefCell<Player>>) -> Self {
        let mut map = Self::default();
        map.player = Rc::clone(&player);

        // TODO: generate level
        map.generate_empty();
        let placements = map.generate_rooms();
        {
            let mut p = Rc::clone(&player);
            (&*p).borrow_mut().coord = placements.first().cloned().unwrap();
        }
        map.generate_corridors();
        // TODO: generate monsters
        // TODO: generate loot
        // TODO: place player
        map
    }

    /// Generate rooms and returns possible positions for object placements
    fn generate_rooms(&mut self) -> Vec<(usize, usize)> {
        let mut rng = rand::rng();
        let max_placement_per_room = 5;
        let mut placements = Vec::new();
        let row_borders: Vec<usize> = (0..4).map(|x| x * self.row_nb / 3).collect();
        let line_borders: Vec<usize> = (0..4).map(|x| x * self.line_nb / 3).collect();
        for ((min_row, max_row), (min_line, max_line)) in iproduct!(
            row_borders.iter().zip(row_borders.iter().skip(1)),
            line_borders.iter().zip(line_borders.iter().skip(1))
        ) {
            if rng.random_range(0..4) == 0 {
                continue;
            }
            let rows: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_row + 1..*max_row))
                .collect();
            let (start_row, end_row) = (
                rows.clone().into_iter().min().unwrap(),
                rows.into_iter().max().unwrap(),
            );
            let lines: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_line + 1..*max_line))
                .collect();
            let (start_line, end_line) = (
                lines.clone().into_iter().min().unwrap(),
                lines.into_iter().max().unwrap(),
            );
            if (end_row - start_row < 3) || (end_line - start_line < 3) {
                continue;
            }

            for line in start_line..=end_line {
                self.set((line, start_row), '|');
                self.set((line, end_row), '|');
            }
            for row in start_row..=end_row {
                self.set((start_line, row), '-');
                self.set((end_line, row), '-');
            }
            placements.append(
                &mut (0..max_placement_per_room)
                    .map(|_| {
                        (
                            rng.random_range(start_line + 1..end_line),
                            rng.random_range(start_row + 1..end_row),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }
        placements
    }

    fn generate_corridors(&mut self) {}

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
        let curr_coords = self.player.borrow().coord;
        let new_coords = match direction {
            MoveDirection::Right => (curr_coords.0, curr_coords.1 + 1),
            MoveDirection::Left => (curr_coords.0, curr_coords.1 - 1),
            MoveDirection::Up => (curr_coords.0 - 1, curr_coords.1),
            MoveDirection::Down => (curr_coords.0 + 1, curr_coords.1),
        };
        {
            let mut p = (&*self.player).borrow_mut();
            match self.get(new_coords) {
                Some('|') | Some('-') => {
                    return;
                } // wall, do no move
                Some('+') | Some('#') => p.coord = new_coords, // corridor
                Some(' ') | Some('.') => p.coord = new_coords, // empty room
                Some('@') => {}                                // no move
                Some(c) => {
                    dbg!(c);
                    todo!()
                } //monster
                _ => {}
            }
        }

        for (row, col) in iproduct!(
            curr_coords.0 - 2..=curr_coords.0 + 2,
            curr_coords.1 - 2..=curr_coords.1 + 2
        ) {
            self.discovered_map[row][col] = self.map[row][col].clone();
            self.displayed_map[row][col] = self.discovered_map[row][col];
            // TODO: add monster
        }
        self.displayed_map[new_coords.0][new_coords.1] = '@';
    }

    fn get(&self, coords: (usize, usize)) -> Option<char> {
        self.displayed_map.get(coords.0)?.get(coords.1).copied()
    }

    fn set(&mut self, coords: (usize, usize), value: char) {
        *self
            .map
            .get_mut(coords.0)
            .unwrap()
            .get_mut(coords.1)
            .unwrap() = value;
    }
    pub fn size(&self) -> (usize, usize) {
        (self.line_nb, self.row_nb)
    }
}
