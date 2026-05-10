use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::prelude::*;
use ratatui::prelude::Stylize;
use ratatui::{
    DefaultTerminal, Frame, Terminal,
    prelude::CrosstermBackend,
    buffer::Buffer,
        layout::{Rect, Layout, Direction, Constraint},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        execute,
    },
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::BufReader;

#[derive(Debug, Default, Clone)]
struct Player {
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

    fn fight(&mut self, monster: &Monster) {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Monster {
    name: String,
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
    fn generate(&self) -> Self {
        let mut rng = rand::rng();
        Self {
            hp: rng.random_range(self.min_hp..=self.max_hp),
            strength: rng.random_range(self.min_strength..=self.max_strength),
            ..self.clone()
        }
    }
}

#[derive(Debug, Default, Clone)]
struct App {
    player: Player,
    map: Map,
    exit: bool,
}

impl Widget for App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default().direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        Line::from("log".red()).render(chunks[0], buf);
        self.map.render(chunks[1], buf);
        Line::from("status".blue()).render(chunks[2], buf);

    }
}

impl App {
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
        todo!()
    }
}

#[derive(Debug, Default, Clone)]
struct Map {
    /// Map, coord (line, column) with line (0, 0) at top left,
    /// (line_nb - 1, row_nb - 1) at bottom right
    map: Vec<Vec<char>>,
    line_nb: usize,
    row_nb: usize,
}

impl Widget for Map {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::from("TODO".red()).render(area, buf)
    }
}

fn main() -> Result<()> {
    let path = "data/monsters.json";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let monster_collection: Vec<Monster> = serde_json::from_reader(reader)?;

    let mut app = App::default();
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
