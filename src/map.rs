use crate::Monster;
use crate::Player;
use itertools::iproduct;
use rand::prelude::*;
// use ratatui::prelude::Stylize;
use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget};
// use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Default, Clone)]
struct Room {
    id: usize,
    start_line: usize,
    end_line: usize,
    start_col: usize,
    end_col: usize,
    has_light: bool, // entirely dicoverable or not
}

impl Room {
    /// Draw an empty room on the map contained in the grid
    fn draw(&self, grid: &mut Vec<Vec<char>>) {
        assert!(self.end_line < grid.len(), "not enough lines in grid");
        assert!(
            grid.iter()
                .filter(|line| !(self.end_col < line.len()))
                .collect::<Vec<_>>()
                .is_empty(),
            "not enough column in grid"
        );
        for (line, col) in iproduct!(self.start_line..self.end_line, self.start_col..self.end_col) {
            grid[line][col] = ' ';
        }
        for line in self.start_line..=self.end_line {
            grid[line][self.start_col] = '|';
            grid[line][self.end_col] = '|';
        }
        for col in self.start_col..=self.end_col {
            grid[self.start_line][col] = '-';
            grid[self.end_line][col] = '-';
        }
    }

    fn center(&self) -> (usize, usize) {
        (
            (self.end_line + self.start_line) / 2,
            (self.end_col + self.start_col) / 2,
        )
    }

    fn is_inside(&self, coord: (usize, usize)) -> bool {
        [
            coord.0 > self.start_line,
            coord.0 < self.end_line,
            coord.1 > self.start_col,
            coord.1 < self.end_col,
        ]
        .iter()
        .all(|&x| x)
    }

    fn is_border(&self, coord: (usize, usize)) -> bool {
        [
            coord.0 == self.start_line && coord.1 >= self.start_col && coord.1 <= self.end_col,
            coord.0 == self.end_line && coord.1 >= self.start_col && coord.1 <= self.end_col,
            coord.1 == self.start_col && coord.0 >= self.start_line && coord.0 <= self.end_line,
            coord.1 == self.end_col && coord.0 >= self.start_line && coord.0 <= self.end_line,
        ]
        .iter()
        .any(|&x| x)
    }

    fn get_all_coords(&self) -> Vec<(usize, usize)> {
        iproduct!(
            self.start_line..=self.end_line,
            self.start_col..=self.end_col
        )
        .collect()
    }

    fn get_borders(&self) -> Vec<(usize, usize)> {
        (self.start_line..=self.end_line)
            .map(|line| (line, self.start_col))
            .chain((self.start_line..=self.end_line).map(|line| (line, self.end_col)))
            .chain((self.start_col..=self.end_col).map(|col| (self.start_line, col)))
            .chain((self.start_col..=self.end_col).map(|col| (self.end_line, col)))
            .collect()
    }
}

fn get_trajectory(
    direction: MoveDirection,
    start: (usize, usize),
    end: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    match direction {
        MoveDirection::Right => {
            let mid = (start.1 + end.1) / 2;
            path.append(&mut (start.1..=mid).map(|col| (start.0, col)).collect());
            dbg!(&path);
            if start.0 <= end.0 {
                path.append(&mut (start.0..=end.0).map(|line| (line, mid)).collect());
            } else {
                path.append(&mut (end.0..=start.0).rev().map(|line| (line, mid)).collect());
            }
            dbg!(&path);
            path.append(&mut (mid..=end.1).map(|col| (end.0, col)).collect());
        }
        MoveDirection::Down => {
            let mid = (start.0 + end.0) / 2;
            path.append(&mut (start.0..=mid).map(|line| (line, start.1)).collect());
            dbg!(&path);
            if start.1 <= end.1 {
                path.append(&mut (start.1..=end.1).map(|col| (mid, col)).collect());
            } else {
                path.append(&mut (end.1..=start.1).rev().map(|col| (mid, col)).collect());
            }
            dbg!(&path);
            path.append(&mut (mid..=end.0).map(|line| (line, end.1)).collect());
        }
        _ => {
            unimplemented!()
        }
    }
    path.dedup();
    path
}

#[derive(Debug, Clone)]
pub(crate) struct Map {
    /// Map, coord (line, column) with line (0, 0) at top left,
    /// (line_nb - 1, col_nb - 1) at bottom right
    map: Vec<Vec<char>>,
    /// Part of the map already discovered. Used to redraw walls.
    discovered_map: Vec<Vec<char>>,
    /// Part of the map that should be displayed
    displayed_map: Vec<Vec<char>>,
    line_nb: usize,
    col_nb: usize,
    pub(crate) player: Rc<RefCell<Player>>,
    rooms: Vec<Room>,
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
        let col_nb = 80;
        let map = vec![vec![' '; col_nb]; line_nb];
        let discovered_map = map.clone();
        let displayed_map = map.clone();
        let player = Rc::new(RefCell::new(Player::default()));
        let monsters = Vec::new();
        let rooms = Vec::new();
        Self {
            map,
            discovered_map,
            displayed_map,
            line_nb,
            col_nb,
            player,
            monsters,
            rooms,
        }
    }
}

impl Map {
    /// Generate a level and place the player in it
    pub fn new(level_nb: u8, player: Rc<RefCell<Player>>) -> Self {
        let mut rng = rand::rng();
        let mut map = Self::default();
        map.player = Rc::clone(&player);
        let min_room_nb = 3;

        // TODO: generate level

        // generate rooms
        map.generate_empty();
        let mut placements = map.generate_rooms(&mut rng);
        while map.rooms.len() < min_room_nb {
            map.generate_empty();
            placements = map.generate_rooms(&mut rng);
        };
        map.rooms.shuffle(&mut rng);
        for room in &map.rooms {
            room.draw(&mut map.map);
        }
        placements.shuffle(&mut rng);
        {
            let p = Rc::clone(&player);
            (&*p).borrow_mut().coord = placements.pop().unwrap();
        }

        // link rooms
        // TODO: check all rooms are connected
        map.generate_corridors(&mut rng);

        // TODO: generate monsters
        // TODO: generate loot
        // TODO: place player
        // TODO: place exits
        map
    }

    /// Generate rooms and returns possible positions for object placements
    fn generate_rooms(&mut self, rng: &mut ThreadRng) -> Vec<(usize, usize)> {
        let max_placement_per_room = 5;
        let mut placements = Vec::new();
        let col_nb = 3;
        let line_nb = 3;
        let col_borders: Vec<usize> = (0..=col_nb).map(|x| x * self.col_nb / col_nb).collect();
        let line_borders: Vec<usize> = (0..=line_nb).map(|x| x * self.line_nb / line_nb).collect();
        for (id, ((min_col, max_col), (min_line, max_line))) in iproduct!(
            col_borders.iter().zip(col_borders.iter().skip(1)),
            line_borders.iter().zip(line_borders.iter().skip(1))
        )
        .enumerate()
        {
            if rng.random_range(0..4) == 0 {
                continue;
            }
            let cols: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_col + 1..*max_col))
                .collect();
            let (start_col, end_col) = (
                cols.clone().into_iter().min().unwrap(),
                cols.into_iter().max().unwrap() - 1,
            );
            let lines: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_line + 1..*max_line))
                .collect();
            let (start_line, end_line) = (
                lines.clone().into_iter().min().unwrap(),
                lines.into_iter().max().unwrap() - 1,
            );
            if (end_col - start_col < 4) || (end_line - start_line < 4) {
                continue;
            }

            self.rooms.push(Room {
                id,
                start_line,
                end_line,
                start_col,
                end_col,
                has_light: true,
            });

            placements.append(
                &mut (0..max_placement_per_room)
                    .map(|_| {
                        (
                            rng.random_range(start_line + 1..end_line),
                            rng.random_range(start_col + 1..end_col),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }
        placements
    }

    /// Generate corridors between rooms
    ///
    /// generate a single corridor between consecutive rooms.
    /// if the choosen path intersect with either a corridor or a room,
    /// then the path drawing stops at the intersection.
    fn generate_corridors(&mut self, rng: &mut ThreadRng) {
        let mut not_connected: Vec<usize> = self.rooms.iter().map(|r| r.id).collect();
        let mut connected: Vec<usize> = vec![not_connected.choose(rng).copied().unwrap()];
        not_connected = not_connected
            .iter()
            .filter(|elt| !connected.contains(elt))
            .copied()
            .collect();
        while !not_connected.is_empty() {
            let anchor = connected.choose(rng).copied().unwrap();
            let anchor = self
                .rooms
                .iter()
                .filter(|r| r.id == anchor)
                .cloned()
                .next()
                .unwrap();
            let new_room = not_connected.choose(rng).copied().unwrap();
            let new_room = self
                .rooms
                .iter()
                .filter(|r| r.id == new_room)
                .cloned()
                .next()
                .unwrap();
            match self.connect_rooms(&anchor, &new_room, rng) {
                Ok(_) => connected.push(new_room.id),
                Err(_) => {} // failed to connect this time
            }
            not_connected = self
                .rooms
                .iter()
                .filter_map(|r| {
                    if connected.contains(&r.id) {
                        None
                    } else {
                        Some(r.id)
                    }
                })
                .collect();
        }
    }

    fn connect_rooms(
        &mut self,
        room1: &Room,
        room2: &Room,
        rng: &mut ThreadRng,
    ) -> Result<(), String> {
        let (direction, start_coord, end_coord) =
            if (room2.start_col > 3) && (room1.end_col < room2.start_col - 3) {
                // enough space for horizontal corridor room1 -> room2
                (
                    MoveDirection::Right,
                    (
                        rng.random_range(room1.start_line + 1..room1.end_line),
                        room1.end_col,
                    ),
                    (room2.center()),
                )
            } else if (room2.start_line > 3) && (room1.end_line < room2.start_line - 3) {
                // enough space for vertical corridor room1 -> room2
                (
                    MoveDirection::Down,
                    (
                        room1.end_line,
                        rng.random_range(room1.start_col + 1..room1.end_col),
                    ),
                    (room2.center()),
                )
            } else if (room1.start_col > 3) && (room2.end_col < room1.start_col - 3) {
                // enough space for horizontal corridor room2 -> room1
                (
                    MoveDirection::Right,
                    (
                        rng.random_range(room2.start_line + 1..room2.end_line),
                        room2.end_col,
                    ),
                    (room1.center()),
                )
            } else if (room1.start_line > 3) && (room2.end_line < room1.start_line - 3) {
                // enough space for vertical corridor room2 -> room1
                (
                    MoveDirection::Down,
                    (
                        room2.end_line,
                        rng.random_range(room2.start_col + 1..room2.end_col),
                    ),
                    (room1.center()),
                )
            } else {
                // No corridor to draw
                unreachable!();
            };

        let mut cells = get_trajectory(direction, start_coord, end_coord).into_iter();

        // find first cell outside first room
        let mut exit_room = (0, 0);
        for cell in &mut cells {
            let is_inside = [
                room1.is_border(cell),
                room1.is_inside(cell),
                room1.is_border(cell),
                room1.is_inside(cell),
            ]
            .iter()
            .any(|&x| x);
            if is_inside {
                exit_room = cell;
            } else {
                self.map[cell.0][cell.1] = '#';
                break;
            }
        }
        self.map[exit_room.0][exit_room.1] = '+';

        for cell in &mut cells {
            if room1.is_border(cell) || room2.is_border(cell) {
                self.map[cell.0][cell.1] = '+';
                break;
            }
            if room1.is_inside(cell) || room2.is_inside(cell) {
                break;
            }
            match self.map[cell.0][cell.1] {
                '|' | '-' => return Err("end in an unexepected room".into()),
                '#' => return Err("Corridor cross another corridor".into()),
                _ => self.map[cell.0][cell.1] = '#',
            }
        }

        // redraw doors
        for r in &self.rooms {
            for cell in r.get_borders() {
                let c = self.map[cell.0][cell.1];
                let u = self.map[cell.0 - 1][cell.1];
                let d = self.map[cell.0 + 1][cell.1];
                let l = self.map[cell.0][cell.1 - 1];
                let r = self.map[cell.0][cell.1 + 1];
                match (c, u, d, l, r) {
                    (_, '|', _, '-', _)
                    | (_, '|', _, _, '-')
                    | (_, _, '|', '-', _)
                    | (_, _, '|', _, '-') => self.map[cell.0][cell.1] = '-', // corner
                    ('|', _, _, '#', _) | ('|', _, _, _, '#') => self.map[cell.0][cell.1] = '+',
                    ('-', '#', _, _, _) | ('-', _, '#', _, _) => self.map[cell.0][cell.1] = '+',
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn generate_empty(&mut self) {
        self.rooms.truncate(0);
        if self.line_nb < 9 {
            self.line_nb = 23
        }
        if self.col_nb < 9 {
            self.col_nb = 80
        }
        self.map = vec![vec!['\u{00A0}'; self.col_nb]; self.line_nb];
        self.discovered_map = vec![vec![' '; self.col_nb]; self.line_nb];
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
                Some('|') | Some('-') => return,               // wall, do no move
                Some('\u{00A0}') | Some('\u{202F}') => return, // wall outside a room
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

        for room in &self.rooms {
            if room.is_inside(self.player.borrow().coord) && room.has_light {
                // See all room
                for (row, col) in room.get_all_coords() {
                    self.discovered_map[row][col] = self.map[row][col].clone();
                    self.displayed_map[row][col] = self.discovered_map[row][col];
                }
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
        (self.line_nb, self.col_nb)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn traj0() {
        let start = (0, 0);
        let end = (5, 10);
        let traj = get_trajectory(MoveDirection::Right, start, end);
        assert!(traj.contains(&(0, 5)));
    }

    #[test]
    fn traj1() {
        let start = (0, 0);
        let end = (7, 10);
        let traj = get_trajectory(MoveDirection::Down, start, end);
        assert!(traj.contains(&(3, 7)));
    }

    #[test]
    fn traj3() {
        let start = (7, 0);
        let end = (0, 10);
        let traj = get_trajectory(MoveDirection::Right, start, end);
        assert!(traj.contains(&(5, 5)));
    }
}
