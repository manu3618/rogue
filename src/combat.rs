use ratatui::{buffer::Buffer, layout::Rect, text::Text, widgets::Widget};

use crate::{Monster, Player};

#[derive(Debug, Clone, Default)]
enum Action {
    Magic, // TODO: define spell
    /// Physical attack with current equipement
    #[default]
    Physical,
    Flee,
    /// Do nothing
    Pass,
}

struct Combat {
    player: Player,
    monster: Monster,
}

impl Combat {
    fn new(player: Player, monster: Monster) -> Self {
        Self { player, monster }
    }

    fn turn(&mut self) {
        todo!()
    }
}

impl Widget for Combat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        todo!()
    }
}
