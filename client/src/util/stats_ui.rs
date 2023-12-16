use common::{game::Stats, PlayerSymbol, PLAYERS};
use eframe::egui;

pub fn build_stats_ui(ui: &mut egui::Ui, stats: &Stats, this_player: PlayerSymbol) {
  ui.label(egui::RichText::new("Stats").size(30.0));
  ui.label(format!("Game #{}", stats.ngames));
  ui.label(format!(
    "Wins {}/{}: {}",
    PLAYERS[0].as_char(),
    if this_player == PLAYERS[0] {
      "YOU"
    } else {
      "THEM"
    },
    stats.scores[0]
  ));
  ui.label(format!(
    "Wins {}/{}: {}",
    PLAYERS[1].as_char(),
    if this_player == PLAYERS[1] {
      "YOU"
    } else {
      "THEM"
    },
    stats.scores[1]
  ));
}
