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
use std::io;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tracing::field::Visit;
use tracing::{Subscriber, debug, info};
use tracing_subscriber::{
    Layer,
    layer::{Context, SubscriberExt},
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
#[derive(Debug, Default, Clone, Eq, PartialEq)]
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
    /// display used before
    previous_display: Display,
    exit: bool,
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
        Line::from(
            self.captured_messages
                .lock()
                .unwrap()
                .last()
                .unwrap_or(&"LOG".into())
                .clone(),
        )
        .render(chunks[0], buf);
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
        let _ = tracing::subscriber::set_global_default(subscriber);
        let mut map = map::Map::new(0, Rc::clone(&player));
        let mut monsters: Vec<Monster> = (0..3).map(|_| Monster::generate()).collect();
        let object_collection = Object::collection_from_file("data/objects.json").unwrap();
        let mut objects: Vec<Object> = (0..10)
            .map(|_| object_collection.choose(&mut rng).cloned().unwrap())
            .collect();
        map.place_monsters(&mut monsters);
        map.place_loot(&mut objects, &mut rng);
        let player_coords = player.borrow().coord;
        map.discover_map(player_coords, player_coords);
        Self {
            map,
            player: Rc::clone(&player),
            captured_messages: Arc::clone(&capture),
            ..Default::default()
        }
    }

    fn update_last_message(&mut self) {
        let last_messages = self.captured_messages.lock().unwrap();
        let last_messages: Vec<String> =
            last_messages.iter().rev().take(15).cloned().rev().collect();
        self.combat.log_messages = last_messages;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                let _ = self.handle_key_event(key_event);
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
            (Display::Map, KeyCode::Backspace) => self.exit = true,
            (Display::Combat, KeyCode::Up) => self.combat.select_previous(),
            (Display::Combat, KeyCode::Down) => self.combat.select_next(),
            (Display::Combat, KeyCode::Left | KeyCode::Backspace) => self.combat.previous(),
            (Display::Combat, KeyCode::Right | KeyCode::Enter) => self.combat.validate(),
            (Display::Combat, KeyCode::Char(c)) if c.is_ascii_digit() => {
                self.combat.select_item(c.to_digit(10).unwrap() as usize)
            }
            (Display::Inventory, KeyCode::Up) => self.inventory.select_previous(),
            (Display::Inventory, KeyCode::Down) => self.inventory.select_next(),
            (Display::Inventory, KeyCode::Left | KeyCode::Backspace) => {
                self.display = self.previous_display.clone()
            }
            (Display::Inventory, KeyCode::Right | KeyCode::Enter) => {
                if let Some(obj) = self.inventory.pop_selected() {
                    self.player.borrow_mut().use_object(obj);
                }
            }
            (Display::Inventory, KeyCode::Char(c)) if c.is_ascii_digit() => self
                .inventory
                .select_number(String::from(c).parse().unwrap()),
            (_, KeyCode::Char('?')) => {
                self.previous_display = self.display.clone();
                self.display = Display::Inventory
            }
            (_, KeyCode::Char('q')) => {
                info!("quit...");
                self.exit = true
            }
            (_, KeyCode::Char(u)) => debug!("Key pressed: {u}"),
            (_, k) => {
                debug!("Key pressed: {k}");
                self.exit = true;
            } // hould be unreachable
              // _ => unreachable!(),
        }

        match &self.map.encounter {
            None => {}
            Some(map::Encounter::Monster(m)) => {
                info!("----------------------");
                info!("encountring monster {}", m);
                self.combat = Combat::new(Rc::clone(&self.player), m.clone());
                self.update_last_message();
                self.previous_display = self.display.clone();
                self.display = Display::Combat
            }
            Some(map::Encounter::Loot(l)) => {
                info!("found {}", l);
                if l.should_keep() {
                    self.inventory.add_item(l.clone());
                } else {
                    self.player.borrow_mut().use_object(l.clone());
                }
            }
        }
        self.map.encounter = None;

        if self.display == Display::Combat && self.combat.is_over() {
            info!("combat ended");
            if self.player.borrow_mut().is_dead() {
                info!("you're dead");
                self.exit = true;
            } else {
                self.player.borrow_mut().increase_exp(1); // TODO: modulate exp gain
                self.previous_display = self.display.clone();
                self.display = Display::Map
            }
        }

        Ok(())
    }

    /// Change level, regenrating monster if needed
    fn change_level(&mut self, new_level: u8) {
        todo!()
    }
}

fn main() -> Result<()> {
    // let mut app = App::default();
    let mut app = App::new();
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    while !app.exit {
        terminal.draw(|frame| frame.render_widget(app.clone(), frame.area()))?;
        &app.handle_events();
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    println!("party summmary:");
    println!("{}", app.captured_messages.lock().unwrap().join("\n"));

    Ok(())
}
