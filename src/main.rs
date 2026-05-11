use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::rc::Rc;

mod map;
mod player;
use player::{Monster, Player};

/// What to display on screen
#[derive(Debug, Default, Clone)]
enum Display {
    #[default]
    Map,
    Inventory,
}

#[derive(Debug, Default, Clone)]
struct App {
    player: Rc<RefCell<Player>>,
    // monsters: Vec<Monster>,
    map: map::Map,
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
            Display::Inventory => todo!(),
        }
        Line::from("status".blue()).render(chunks[2], buf);
    }
}

impl App {
    fn new() -> Self {
        let player = Rc::new(RefCell::new(Default::default()));
        Self {
            map: map::Map::new(0, Rc::clone(&player)),
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
        match key_event.code {
            KeyCode::Char(u) => self.log.push(format!("Key pressed: {u}")),
            KeyCode::Left => self.map.move_player(map::MoveDirection::Left),
            KeyCode::Right => self.map.move_player(map::MoveDirection::Right),
            KeyCode::Up => self.map.move_player(map::MoveDirection::Up),
            KeyCode::Down => self.map.move_player(map::MoveDirection::Down),
            _ => {
                dbg!(key_event);
                todo!()
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let path = "data/monsters.json";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let monster_collection: Vec<Monster> = serde_json::from_reader(reader)?;
    // let mut app = App::default();
    let mut app = App::new();
    app.log.push(format!(
        "Player position: {:?}",
        app.map.player.borrow().coord
    ));
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
