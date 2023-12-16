use crate::{playing::PlayingState, util::stats_ui::build_stats_ui, Client};

use common::{
  game::Stats,
  msg::{MessageIoHandlerNoBlocking, ServerMsgRoundStart},
  PlayerSymbol,
};

use eframe::egui;

pub struct WaitingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,

  stats: Stats,
}

impl WaitingState {
  pub fn new(
    msg_handler: MessageIoHandlerNoBlocking,
    this_player: PlayerSymbol,
    stats: Stats,
  ) -> Self {
    Self {
      msg_handler,
      this_player,
      stats,
    }
  }

  pub fn update(mut self, ctx: &egui::Context) -> Client {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.heading("Waiting for other player...");
        ui.add_space(50.0);
        build_stats_ui(ui, &self.stats, self.this_player);
      })
    });

    if let Some(ServerMsgRoundStart(starting_player)) = self.msg_handler.try_read_msg().unwrap() {
      return Client::Playing(PlayingState::new(
        self.msg_handler,
        self.this_player,
        self.stats,
        starting_player,
      ));
    }

    Client::WaitingForGameStart(self)
  }
}
