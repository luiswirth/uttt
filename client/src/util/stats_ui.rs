use common::{game::Stats, PlayerSymbol};
use eframe::egui;

pub fn build_stats_ui(ui: &mut egui::Ui, stats: &Stats, this_player: PlayerSymbol) {
  ui.label(egui::RichText::new("Stats").size(30.0));
  let text = |s| egui::RichText::new(s).size(20.0);
  ui.label(text(format!("Game #{}", stats.ngames)));
  ui.label(text(format!(
    "YOUR WINS: {}",
    stats.scores[this_player.idx()]
  )));
  ui.label(text(format!(
    "THEIR WINS: {}",
    stats.scores[this_player.other().idx()],
  )));
}
