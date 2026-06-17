use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::prelude::*;
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
use std::sync::{Arc, Mutex};
use tracing::field::Visit;
use tracing::{Subscriber, info};
use tracing_subscriber::{
    Layer, fmt,
    layer::{Context, SubscriberExt},
    registry::{LookupSpan, Registry},
    util::SubscriberInitExt,
};

mod combat;
mod map;
mod monster;
mod object;

use combat::Combat;
use monster::{Monster, Player};
use object::{Inventory, Object};

// tracing stuff

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

#[derive(Default, Debug, Clone)]
struct MessageCapture {
    messages: Arc<Mutex<Vec<String>>>,
}

impl MessageCapture {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    fn get_last_message(&self) -> Option<String> {
        self.messages.lock().unwrap().last().cloned()
    }
    fn push_message(&self, msg: String) {
        self.messages.lock().unwrap().push(msg);
    }
}

impl<S> Layer<S> for MessageCapture
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor::default();
        let metadata = event.metadata();
        event.record(&mut visitor);
        let msg = visitor.message;
        if !msg.is_empty() {
            self.messages.lock().unwrap().push(msg);
        }
    }
}

/// What to display on screen
#[derive(Debug, Default, Clone)]
enum Display {
    #[default]
    /// Map of current level
    Map,
    /// Combat with an enemy
    Combat,
    /// Inventory, last log messages, small help?
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
    /// currently generated maps
    maps: Vec<map::Map>,
    /// player inventory
    inventory: Inventory,
    /// select which screen to display
    display: Display,
    exit: bool,
    log: Vec<String>,
    /// messages captured from tracing
    captured_messages: Arc<Mutex<Vec<String>>>,
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
            Display::Inventory => self.inventory.render(chunks[1], buf),
        }
        Line::from(vec![
            "status: ".blue(),
            self.player.borrow().status().into(),
        ])
        .render(chunks[2], buf);
    }
}

impl App {
    fn new() -> Self {
        let mut rng = rand::rng();
        let player = Rc::new(RefCell::new(Player::new()));
        let capture = Arc::new(Mutex::new(Vec::new()));
        let capture_layer = MessageCapture {
            messages: Arc::clone(&capture),
        };
        let subscriber = tracing_subscriber::registry().with(capture_layer);
        tracing::subscriber::set_global_default(subscriber);
        let mut map = map::Map::new(0, Rc::clone(&player));
        let mut monsters: Vec<Monster> = (0..3).map(|_| Monster::generate()).collect();
        let object_collection = Object::collection_from_file("data/objects.json").unwrap();
        let mut objects: Vec<Object> = (0..3)
            .map(|_| object_collection.choose(&mut rng).cloned().unwrap())
            .collect();
        map.place_monsters(&mut monsters);
        map.place_loot(&mut objects, &mut rng);
        Self {
            map,
            player: Rc::clone(&player),
            inventory: objects.iter().collect(),
            captured_messages: Arc::clone(&capture),
            ..Default::default()
        }
    }

    fn update_last_message(&mut self) {
        let last_message = self.captured_messages.lock().unwrap().last().cloned();
        if last_message.is_some() && last_message != self.log.last().cloned() {
            self.log.push(last_message.unwrap())
        }
        let last_messages = self.captured_messages.lock().unwrap();
        let last_messages: Vec<String> = last_messages
            .iter()
            .rev()
            .take(4)
            .cloned()
            .into_iter()
            .rev()
            .collect();
        self.combat.log_messages = last_messages;
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
        self.update_last_message();
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        match (&self.display, key_event.code) {
            (Display::Map, KeyCode::Left) => self.map.move_player(map::MoveDirection::Left),
            (Display::Map, KeyCode::Right) => self.map.move_player(map::MoveDirection::Right),
            (Display::Map, KeyCode::Up) => self.map.move_player(map::MoveDirection::Up),
            (Display::Map, KeyCode::Down) => self.map.move_player(map::MoveDirection::Down),
            (Display::Combat, KeyCode::Up) => self.combat.select_previous(),
            (Display::Combat, KeyCode::Down) => self.combat.select_next(),
            (Display::Combat, KeyCode::Left) => self.combat.previous(),
            (Display::Combat, KeyCode::Right | KeyCode::Enter) => self.combat.validate(),
            (Display::Combat, KeyCode::Char(c)) if c.is_ascii_digit() => {
                self.combat.select_item(c.to_digit(10).unwrap() as usize)
            }
            (Display::Inventory, KeyCode::Up) => todo!(),
            (Display::Inventory, KeyCode::Down) => todo!(),
            (Display::Inventory, KeyCode::Left) => todo!(),
            (Display::Inventory, KeyCode::Right | KeyCode::Enter) => todo!(),
            (Display::Inventory, KeyCode::Char(c)) if c.is_ascii_digit() => todo!(),
            (_, KeyCode::Char('?')) => self.display = Display::Inventory,
            (_, KeyCode::Char('q')) => todo!("quit?"),
            (_, KeyCode::Char(u)) => self.log.push(format!("Key pressed: {u}")),
            _ => unreachable!(),
        }

        match &self.map.encounter {
            None => {}
            Some(map::Encounter::Monster(m)) => {
                info!("encountring monster {}", m);
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
