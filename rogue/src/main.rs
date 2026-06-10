use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
// use rand::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::{
    // DefaultTerminal, Frame,
    Terminal,
    buffer::Buffer,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    text::Line,
    widgets::{
        //Block, Paragraph,
        Widget,
    },
};
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::rc::Rc;

mod combat;
mod map;
mod monster;
mod object;

use combat::Combat;
use monster::{Monster, Player};
use object::Object;

/// What to display on screen
#[derive(Debug, Default, Clone)]
enum Display {
    #[default]
    /// Map of current level
    Map,
    /// Combat with an enemy
    Combat,
    /// Inventory, last log messages, small help
    Inventory,
}

#[derive(Debug, Default, Clone)]
struct App {
    player: Rc<RefCell<Player>>,
    // monsters: Vec<Monster>,
    /// Current combat, if any
    combat: Combat,
    /// current map
    map: map::Map,
    /// curretnly generated maps
    maps: Vec<map::Map>,
    display: Display,
    exit: bool,
    log: Vec<String>,
}

impl Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(self.map.size().0.try_into().unwrap()),
                Constraint::Length(1),
            ])
            .split(area);
        Line::from(self.log.last().unwrap_or(&"LOG".into()).clone()).render(chunks[0], buf);
        match self.display {
            Display::Map => self.map.render(chunks[1], buf),
            Display::Combat => self.combat.render(chunks[1], buf),
            Display::Inventory => todo!(),
        }
        Line::from("status".blue()).render(chunks[2], buf);
    }
}

impl App {
    fn new() -> Self {
        let player = Rc::new(RefCell::new(Default::default()));
        let mut map = map::Map::new(0, Rc::clone(&player));
        let mut monsters: Vec<Monster> = (0..3).map(|_| Monster::generate()).collect();
        map.place_monsters(&mut monsters);
        Self {
            map,
            player: Rc::clone(&player),
            ..Default::default()
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match self.display {
            Display::Map => match key_event.code {
                KeyCode::Left => self.map.move_player(map::MoveDirection::Left),
                KeyCode::Right => self.map.move_player(map::MoveDirection::Right),
                KeyCode::Up => self.map.move_player(map::MoveDirection::Up),
                KeyCode::Down => self.map.move_player(map::MoveDirection::Down),
                KeyCode::Char('q') => todo!("quit?"),
                KeyCode::Char('?') => self.display = Display::Inventory,
                KeyCode::Char(u) => self.log.push(format!("Key pressed: {u}")),
                _ => {
                    dbg!(key_event);
                    todo!()
                }
            },
            Display::Combat => match key_event.code {
                KeyCode::Up => self.combat.select_previous(),
                KeyCode::Down => self.combat.select_next(),
                KeyCode::Left => self.combat.previous(),
                KeyCode::Right | KeyCode::Enter => self.combat.validate(),
                KeyCode::Char(c) if ('0'..='9').contains(&c) => {
                    self.combat.select_item(c.to_digit(10).unwrap() as usize)
                }
                KeyCode::Char('q') => todo!("quit?"),
                KeyCode::Char('?') => self.display = Display::Inventory,
                KeyCode::Char(u) => self.log.push(format!("Key pressed: {u}")),
                _ => {
                    dbg!(key_event);
                    todo!()
                }
            },
            Display::Inventory => todo!(),
        }

        match &self.map.encounter {
            None => {}
            Some(map::Encounter::Monster(m)) => {
                self.combat = Combat::new(Rc::clone(&self.player), m.clone());
                self.display = Display::Combat
            }
            Some(map::Encounter::Loot(l)) => todo!(),
        }
        self.map.encounter = None;

        Ok(())
    }

    /// Change level, regenrating monster if needed
    fn change_level(&mut self, new_level: u8) {
        todo!()
    }
}

fn main() -> Result<()> {
    let path = "data/monsters.json";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let monster_collection: Vec<Monster> = serde_json::from_reader(reader)?;
    let object_collection = Object::collection_from_file("data/objects.json")?;
    // let mut app = App::default();
    let mut app = App::new();
    app.log.push(format!(
        "Player position: {:?}",
        app.map.player.borrow().coord
    ));
    app.log.push(format!("room number: {}", app.map.room_nb()));
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    while !app.exit {
        terminal.draw(|frame| frame.render_widget(app.clone(), frame.area()))?;
        &app.handle_events();
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
