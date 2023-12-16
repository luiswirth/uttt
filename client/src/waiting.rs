use crate::{playing::PlayingState, util::stats_ui::build_stats_ui, Client};

use common::{
  game::{RoundState, Stats},
  message::{MessageIoHandlerNoBlocking, ServerMessage},
  PlayerSymbol,
};

use eframe::egui;

pub struct WaitingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,

  stats: Stats,
}

impl WaitingState {
  pub fn new(msg_handler: MessageIoHandlerNoBlocking, this_player: PlayerSymbol) -> Self {
    Self {
      msg_handler,
      this_player,
      stats: Stats::default(),
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

    if let Some(msg) = self
      .msg_handler
      .try_read_message::<ServerMessage>()
      .unwrap()
    {
      let starting_player = msg.round_start();
      let round_state = RoundState::new(starting_player);
      return Client::Playing(PlayingState::new(
        self.msg_handler,
        self.this_player,
        round_state,
      ));
    }

    Client::WaitingForGameStart(self)
  }
}
