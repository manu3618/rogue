use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use itertools::iproduct;
use rand::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Span, Text};
use ratatui::widgets::Widget;

use crate::{Monster, Object, Player};

#[derive(Debug, Clone)]
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
                .filter(|line| self.end_col >= line.len())
                .collect::<Vec<_>>()
                .is_empty(),
            "not enough column in grid"
        );
        for (line, col) in iproduct!(self.start_line..self.end_line, self.start_col..self.end_col) {
            grid[line][col] = ' ';
        }
        for line in grid
            .iter_mut()
            .take(self.end_line + 1)
            .skip(self.start_line)
        {
            line[self.start_col] = '|';
            line[self.end_col] = '|';
        }
        let new = ['-'].repeat(self.end_col - self.start_col + 1);
        grid[self.start_line].splice(self.start_col..=self.end_col, new.clone());
        grid[self.end_line].splice(self.start_col..=self.end_col, new);
    }

    fn center(&self) -> (usize, usize) {
        (
            (self.end_line + self.start_line) / 2,
            (self.end_col + self.start_col) / 2,
        )
    }

    fn random_inside(&self, rng: &mut ThreadRng) -> (usize, usize) {
        (
            rng.random_range(self.start_line + 1..self.end_line),
            rng.random_range(self.start_col + 1..self.end_col),
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

    fn is_corner(&self, coord: (usize, usize)) -> bool {
        [
            coord.0 == self.start_line && coord.1 == self.start_col,
            coord.0 == self.start_line && coord.1 == self.end_col,
            coord.0 == self.end_line && coord.1 == self.start_col,
            coord.0 == self.end_line && coord.1 == self.end_col,
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

/// draw a path in form of zigzag
fn get_trajectory_s(
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

/// draw a path in form of L
fn get_trajectory_l(
    direction: MoveDirection,
    start: (usize, usize),
    end: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    match direction {
        MoveDirection::Right => {
            path.append(&mut (start.1..=end.1).map(|col| (start.0, col)).collect());
        }
        MoveDirection::Down => {
            path.append(&mut (start.0..=end.0).map(|line| (line, start.1)).collect());
        }
        MoveDirection::Left => {
            path.append(&mut (end.1..=start.1).rev().map(|col| (start.0, col)).collect());
        }
        MoveDirection::Up => {
            path.append(
                &mut (end.0..=start.0)
                    .rev()
                    .map(|line| (line, start.1))
                    .collect(),
            );
        }
    }
    match direction {
        MoveDirection::Right | MoveDirection::Left => {
            if start.0 <= end.0 {
                path.append(&mut (start.0..=end.0).map(|line| (line, end.1)).collect());
            } else {
                path.append(&mut (end.0..=start.0).rev().map(|line| (line, end.1)).collect());
            }
        }
        MoveDirection::Up | MoveDirection::Down => {
            if start.1 <= end.1 {
                path.append(&mut (start.1..=end.1).map(|col| (end.0, col)).collect());
            } else {
                path.append(&mut (end.1..=start.1).rev().map(|col| (end.0, col)).collect());
            }
        }
    }
    path.dedup();
    path
}

#[derive(Debug, Clone)]
pub(crate) enum LevelDirection {
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub(crate) enum Encounter {
    Loot(Object),
    Monster(Monster),
    Door(LevelDirection),
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
    /// Objects
    loots: Vec<Object>,
    previous_level_door: Option<(usize, usize)>,
    next_level_door: Option<(usize, usize)>,
    /// Am I encoutering something?
    pub(crate) encounter: Option<Encounter>,
}

impl Widget for Map {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1)].repeat(self.line_nb))
            .split(area);
        for line_idx in 0..self.line_nb {
            let mut skip_following = false;
            let mut line = String::new();
            for row_idx in 0..self.col_nb {
                let character = String::from(self.get((line_idx, row_idx)).unwrap());
                let s = Span::raw(&character);
                if !skip_following {
                    line.push_str(character.as_str());
                    if s.width() == 2 {
                        skip_following = true;
                    }
                } else {
                    skip_following = false;
                }
            }
            Text::from(line).render(lines[line_idx], buf);
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        let line_nb = 23;
        let col_nb = 80;
        let map = vec![vec![' '; col_nb]; line_nb];
        let discovered_map = map.clone();
        let displayed_map = map.clone();
        let player = Rc::new(RefCell::new(Player::new()));
        let monsters = Vec::new();
        let rooms = Vec::new();
        let loot = Vec::new();
        Self {
            map,
            discovered_map,
            displayed_map,
            line_nb,
            col_nb,
            player,
            monsters,
            rooms,
            loots: loot,
            encounter: None,
            next_level_door: None,
            previous_level_door: None,
        }
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "  {}",
            (0..self.col_nb / 10)
                .map(|c| format!("{:<10}", c))
                .collect::<Vec<_>>()
                .join("")
        )?;
        writeln!(
            f,
            "  {}",
            (0..self.col_nb)
                .map(|c| format!("{}", c % 10))
                .collect::<Vec<_>>()
                .join("")
        )?;
        write!(
            f,
            "{}",
            self.map
                .iter()
                .enumerate()
                .map(|(idx, line)| format!("{:>2} {}", idx, line.iter().collect::<String>()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl Map {
    /// Generate a level and place the player in it
    pub fn new(level_nb: u8, player: Rc<RefCell<Player>>) -> Self {
        let mut rng = rand::rng();
        let mut map = Map {
            player: Rc::clone(&player),
            ..Default::default()
        };
        let min_room_nb = 3;

        // TODO: generate level

        // generate rooms
        map.generate_empty();
        let mut placements = map.generate_rooms(&mut rng);
        while map.rooms.len() < min_room_nb {
            map.generate_empty();
            placements = map.generate_rooms(&mut rng);
        }
        map.rooms.shuffle(&mut rng);
        for room in &map.rooms {
            room.draw(&mut map.map);
        }
        placements.shuffle(&mut rng);
        {
            let p = Rc::clone(&player);
            (*p).borrow_mut().coord = placements.pop().unwrap();
        }

        // link rooms
        // TODO: check all rooms are connected
        map.generate_corridors(&mut rng);

        // TODO: generate monsters
        // TODO: generate loot
        // TODO: place player
        // TODO: place exits (possible symbol ␧)
        // XXX
        let mut offsets = map.get_room_neighbors(player.borrow().coord.clone());
        offsets.shuffle(&mut rng);
        map.previous_level_door = offsets
            .iter()
            .filter(|c| !placements.contains(c))
            .next()
            .copied();
        map.next_level_door = placements.pop(); // TODO: place doors
        eprintln!("{}", &map);
        map
    }

    pub(crate) fn room_nb(&self) -> usize {
        self.rooms.len()
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
            if rng.random_range(0..8) == 0 {
                continue;
            }
            let cols: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_col + 1..*max_col))
                .collect();
            let (start_col, end_col) = (
                cols.clone().into_iter().min().unwrap(),
                cols.into_iter().max().unwrap(),
            );
            let lines: Vec<_> = (0..3)
                .map(|_| rng.random_range(min_line + 1..*max_line))
                .collect();
            let (start_line, end_line) = (
                lines.clone().into_iter().min().unwrap(),
                lines.into_iter().max().unwrap(),
            );
            if (end_col - start_col < 4) || (end_line - start_line < 4) {
                continue;
            }

            let new_room = Room {
                id,
                start_line,
                end_line,
                start_col,
                end_col,
                has_light: true,
            };

            placements.append(
                &mut (0..max_placement_per_room)
                    .map(|_| new_room.random_inside(rng))
                    .collect::<Vec<_>>(),
            );
            self.rooms.push(new_room);
        }
        placements
    }

    /// Generate corridors between rooms
    ///
    /// generate corridors between rooms until all rooms are connected.
    fn generate_corridors(&mut self, rng: &mut ThreadRng) {
        let mut not_connected: Vec<usize> = self.rooms.iter().map(|r| r.id).collect();
        let mut connected: Vec<usize> = vec![not_connected.choose(rng).copied().unwrap()];
        assert!(!connected.is_empty());
        not_connected = not_connected
            .iter()
            .filter(|elt| !connected.contains(elt))
            .copied()
            .collect();
        assert!(!not_connected.is_empty());
        let mut max_iter = 1000;
        while !not_connected.is_empty() {
            assert!(max_iter > 0);
            max_iter -= 1;
            let anchor = connected.choose(rng).copied().unwrap();
            let anchor = self.rooms.iter().find(|r| r.id == anchor).cloned().unwrap();
            let new_room = not_connected.choose(rng).copied().unwrap();
            let new_room = self
                .rooms
                .iter()
                .find(|r| r.id == new_room)
                .cloned()
                .unwrap();

            if let Ok(id) = self.connect_rooms(&anchor, &new_room, rng)
                && !connected.contains(&id)
            {
                connected.push(id)
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

    /// Connect two rooms togethers
    ///
    /// Draw a single corridor
    /// if the choosen path intersect with either a corridor or a room,
    /// then the path drawing stops at the intersection.
    /// Return the room id connected (should be room2.id)
    fn connect_rooms(
        &mut self,
        room1: &Room,
        room2: &Room,
        rng: &mut ThreadRng,
    ) -> Result<usize, String> {
        let (start_coord, end_coord) = (room1.random_inside(rng), room2.center());
        let mut directions = Vec::new();
        if start_coord.0.abs_diff(end_coord.0) >= start_coord.1.abs_diff(end_coord.1) {
            // vertical direction
            if start_coord.0 > end_coord.0 {
                directions.push(MoveDirection::Up);
            } else {
                directions.push(MoveDirection::Down);
            }
        } else {
            // horizontal direction
            if start_coord.1 > end_coord.1 {
                directions.push(MoveDirection::Left);
            } else {
                directions.push(MoveDirection::Right);
            }
        };

        let Some(direction) = directions.choose(rng).cloned() else {
            return Err("not enough space".into());
        };
        let cells = get_trajectory_l(direction, start_coord, end_coord).into_iter();
        let cells = cells.collect::<Vec<_>>();
        let cells = cells.windows(2);
        for cell in cells.skip_while(|c| room1.is_border(c[1]) || room1.is_inside(c[1])) {
            let (previous_cell, current_cell) = (cell[0], cell[1]);
            if let Some(room) = self
                .rooms
                .iter()
                .find(|r| r.is_corner(previous_cell) || r.is_corner(current_cell))
            {
                return Err(format!("into a the corner of room {}", room.id));
            }
            if room1.is_border(previous_cell) && !room1.is_border(current_cell) {
                // exit door
                self.map[previous_cell.0][previous_cell.1] = '+';
            }
            if let Some(room) = self.rooms.iter().find(|r| r.is_corner(current_cell)) {
                return Err(format!("too close to room corner {}", room.id));
            }
            if let Some(room) = self
                .rooms
                .iter()
                .find(|r| r.is_border(current_cell) && r.is_border(previous_cell))
            {
                return Err(format!("along room {}", room.id));
            }

            if let Some(room) = self
                .rooms
                .iter()
                .find(|r| r.is_border(previous_cell) && r.is_inside(current_cell))
            {
                // end in a room
                self.map[previous_cell.0][previous_cell.1] = '+';
                return Ok(room.id);
            }
            if self.map[current_cell.0][current_cell.1] == '#' {
                return Err("Corridor cross another corridor".into());
            }

            self.map[current_cell.0][current_cell.1] = '#'
        }

        // redraw doors
        for r in &self.rooms {
            for cell in r.get_borders() {
                let c = self.map[cell.0][cell.1];
                let u = self.map[cell.0 - 1][cell.1];
                let d = self
                    .map
                    .get(cell.0 + 1)
                    .cloned()
                    .unwrap_or_default()
                    .get(cell.1)
                    .copied()
                    .unwrap_or('-');
                let l = self.map[cell.0][cell.1 - 1];
                let r = self.map[cell.0].get(cell.1 + 1).copied().unwrap_or('|');
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
        Ok(room2.id)
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

    /// Place monsters on the map
    pub(crate) fn place_monsters(&mut self, monsters: &mut Vec<Monster>) {
        let mut rng = rand::rng();
        let player_coord = self.player.borrow().coord;
        let mut placements: Vec<(usize, usize)> = Vec::new();
        while placements.len() < monsters.len() {
            let mut new_placements: Vec<(usize, usize)> = self
                .rooms
                .iter()
                .filter_map(|r| {
                    if r.is_inside(player_coord) {
                        None
                    } else {
                        Some(r.random_inside(&mut rng))
                    }
                })
                .collect();
            placements.append(&mut new_placements);
        }
        placements.shuffle(&mut rng);
        for monster in &mut *monsters {
            monster.coord = placements.pop().expect("there should be enough placements");
        }
        self.monsters = monsters.to_vec();
    }

    /// Place objects on map
    pub(crate) fn place_loot(&mut self, objects: &mut Vec<Object>, rng: &mut ThreadRng) {
        let mut unusable_placements: Vec<(usize, usize)> =
            self.monsters.iter().map(|m| m.coord).collect();
        unusable_placements.push(self.player.borrow().coord);
        let mut placements: Vec<(usize, usize)> = Vec::new();
        // TODO: adjust number of item to pop
        let level = 1_usize;
        while placements.len() < (20 - level) {
            let mut new_placements: Vec<(usize, usize)> = self
                .rooms
                .iter()
                .map(|r| r.random_inside(rng))
                .filter(|c| !unusable_placements.contains(c))
                .collect();
            placements.append(&mut new_placements);
        }
        placements.shuffle(rng);
        for object in &mut *objects {
            object.coord = placements.pop().expect("there should be enouh placements")
        }
        objects.retain(|o| o.coord != (0, 0));
        self.loots = objects.to_vec();
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
            let mut p = (*self.player).borrow_mut();
            match self.get(new_coords) {
                Some('|') | Some('-') => return,               // wall, do no move
                Some('\u{00A0}') | Some('\u{202F}') => return, // wall outside a room
                Some('+') | Some('#') => p.coord = new_coords, // corridor
                Some(' ') | Some('.') => p.coord = new_coords, // empty room
                Some('@') => {}                                // no move
                Some(c) => {
                    if let Some(m) = self
                        .monsters
                        .extract_if(.., |m| m.coord == new_coords)
                        .next()
                    {
                        self.encounter = Some(Encounter::Monster(m))
                    } else if let Some(l) =
                        self.loots.extract_if(.., |l| l.coord == new_coords).next()
                    {
                        self.encounter = Some(Encounter::Loot(l))
                    } else {
                        unreachable!()
                    }
                    p.coord = new_coords
                }
                _ => {}
            }
        }

        self.discover_map(curr_coords, new_coords);
    }

    pub(crate) fn discover_map(&mut self, curr_coords: (usize, usize), new_coords: (usize, usize)) {
        for room in &self.rooms {
            if room.is_inside(self.player.borrow().coord) && room.has_light {
                // See all room
                for (row, col) in room.get_all_coords() {
                    self.discovered_map[row][col] = self.map[row][col];
                    self.displayed_map[row][col] = self.discovered_map[row][col];
                }

                for monster in &self.monsters {
                    if room.is_inside(monster.coord) {
                        let fl = monster.get_name().chars().next().unwrap();
                        self.displayed_map[monster.coord.0][monster.coord.1] = fl;
                    }
                }
                for loot in &self.loots {
                    if room.is_inside(loot.coord) {
                        let fl = loot.name.chars().next().unwrap();
                        self.displayed_map[loot.coord.0][loot.coord.1] = fl;
                    }
                }
            }
        }

        let discovered = self
            .get_neighbors(curr_coords)
            .into_iter()
            .chain(self.get_neighbors(new_coords));
        for (row, col) in discovered {
            self.discovered_map[row][col] = self.map[row][col].clone();
            self.displayed_map[row][col] = self.discovered_map[row][col];
            // place monster
            if let Some(monster) = self.monsters.iter().find(|m| m.coord == (row, col)) {
                let fl = monster.get_name().chars().next().unwrap();
                self.displayed_map[monster.coord.0][monster.coord.1] = fl;
            }
            if let Some(loot) = self.loots.iter().find(|m| m.coord == (row, col)) {
                let fl = loot.name.chars().next().expect("name should not be empty");
                self.displayed_map[loot.coord.0][loot.coord.1] = fl;
            }
        }
        self.displayed_map[new_coords.0][new_coords.1] = '@';
    }
    fn get(&self, coords: (usize, usize)) -> Option<char> {
        self.displayed_map.get(coords.0)?.get(coords.1).copied()
    }

    fn get_neighbors(&self, coords: (usize, usize)) -> Vec<(usize, usize)> {
        let area = 1;
        let line_min = if coords.0 < area { 0 } else { coords.0 - area };
        let line_max = self.line_nb.min(coords.0 + area);
        let col_min = if coords.1 < area { 0 } else { coords.1 - area };
        let col_max = self.col_nb.min(coords.1 + area);
        iproduct!(line_min..=line_max, col_min..=col_max).collect()
    }

    /// get neighbors inside a room
    fn get_room_neighbors(&self, coords: (usize, usize)) -> Vec<(usize, usize)> {
        self.get_neighbors(coords)
            .iter()
            .filter(|&c| self.rooms.iter().any(|r| r.is_inside(*c)))
            .copied()
            .collect()
    }

    pub(crate) fn size(&self) -> (usize, usize) {
        (self.line_nb, self.col_nb)
    }

    /// Simple render method, useful when no character spanning over multiple
    /// columns exists
    fn render_monospace(self, area: Rect, buf: &mut Buffer) {
        let content: Vec<String> = self
            .displayed_map
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect();
        let content = content.join("\n");
        Text::from(content).render(area, buf)
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
        let traj = get_trajectory_s(MoveDirection::Right, start, end);
        assert!(traj.contains(&(0, 5)));
    }

    #[test]
    fn traj1() {
        let start = (0, 0);
        let end = (7, 10);
        let traj = get_trajectory_s(MoveDirection::Down, start, end);
        assert!(traj.contains(&(3, 7)));
    }

    #[test]
    fn traj3() {
        let start = (7, 0);
        let end = (0, 10);
        let traj = get_trajectory_s(MoveDirection::Right, start, end);
        assert!(traj.contains(&(5, 5)));
    }

    #[test]
    fn traj_l0() {
        let start = (0, 0);
        let end = (5, 10);
        let traj_l = get_trajectory_l(MoveDirection::Right, start, end);
        assert!(traj_l.contains(&(0, 5)));
    }

    #[test]
    fn traj_l1() {
        let start = (0, 0);
        let end = (7, 10);
        let traj_l = get_trajectory_l(MoveDirection::Down, start, end);
        assert!(traj_l.contains(&(0, 0)));
        assert!(traj_l.contains(&(7, 0)));
        assert!(traj_l.contains(&(7, 10)));
    }

    #[test]
    fn traj_l3() {
        let start = (7, 0);
        let end = (0, 10);
        let traj_l = get_trajectory_l(MoveDirection::Right, start, end);
        assert!(traj_l.contains(&(7, 0)));
        assert!(traj_l.contains(&(7, 10)));
        assert!(traj_l.contains(&(0, 10)));
    }
    #[test]
    fn traj_l4() {
        let start = (19, 68);
        let end = (18, 15);
        let traj_l = get_trajectory_l(MoveDirection::Left, start, end);
        dbg!(&traj_l);
        assert!(traj_l.contains(&(19, 60)));
        assert!(traj_l.contains(&(19, 22)));
    }
}
