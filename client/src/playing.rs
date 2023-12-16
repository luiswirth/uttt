use crate::{
  util::{
    board_ui::{self, build_board_ui},
    choose_random_tile, player_color,
    stats_ui::build_stats_ui,
  },
  waiting::WaitingState,
  Client,
};

use common::{
  game::{RoundOutcome, RoundState, Stats},
  message::{ClientMessage, MessageIoHandlerNoBlocking, ServerMessage},
  GlobalPos, PlayerSymbol,
};

use eframe::egui;

pub struct PlayingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,

  stats: Stats,
  round: RoundState,

  outcome: Option<RoundOutcome>,
}

impl PlayingState {
  pub fn new(
    msg_handler: MessageIoHandlerNoBlocking,
    this_player: PlayerSymbol,
    round: RoundState,
  ) -> Self {
    Self {
      msg_handler,
      this_player,
      stats: Stats::default(),
      round,
      outcome: None,
    }
  }

  pub fn update(mut self, ctx: &egui::Context) -> Client {
    self.msg_handler.try_write_message::<()>(None).unwrap();

    let mut should_restart_game = false;
    egui::SidePanel::left("left-panel").show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(10.0);

        build_stats_ui(ui, &self.stats, self.this_player);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        ui.label(egui::RichText::new("You are").size(30.0));
        let (response, painter) =
          ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
        let rect = response.rect;
        board_ui::draw_symbol(&painter, rect, self.this_player);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        if let Some(outcome) = self.outcome {
          ui.heading(match outcome {
            RoundOutcome::Win(p) => match p == self.this_player {
              true => "You won!".to_string(),
              false => "You lost!".to_string(),
            },
            RoundOutcome::Draw => "Draw!".to_string(),
          });

          if ui.button("Play again").clicked() || cfg!(feature = "auto_next_round") {
            should_restart_game = true;
          }
        } else {
          ui.label(egui::RichText::new("Turn of").size(30.0));
          ui.label(
            egui::RichText::new(if self.this_player == self.round.current_player() {
              "YOU"
            } else {
              "THEM"
            })
            .color(player_color(self.round.current_player()))
            .size(50.0),
          );
          let (response, painter) =
            ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
          let rect = response.rect;
          board_ui::draw_symbol(&painter, rect, self.round.current_player());

          if self.round.current_player() == self.this_player && ui.button("Give up").clicked() {
            let msg = ClientMessage::GiveUp;
            self.msg_handler.try_write_message(Some(msg)).unwrap();
            let outcome = self
              .outcome
              .insert(RoundOutcome::Win(self.this_player.other()));
            self.stats.update(*outcome);
          }
        }
      })
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      let chosen_tile = build_board_ui(ui, &self.round, self.this_player);
      Self::update_round(&mut self, chosen_tile);
    });

    if should_restart_game {
      let msg = ClientMessage::StartRoundRequest;
      self.msg_handler.try_write_message(Some(msg)).unwrap();
      return Client::WaitingForGameStart(WaitingState::new(self.msg_handler, self.this_player));
    }

    Client::Playing(self)
  }

  fn update_round(state: &mut Self, mut clicked_tile: Option<GlobalPos>) {
    if state.outcome.is_some() {
      return;
    }

    if cfg!(feature = "auto_play") {
      clicked_tile = Some(choose_random_tile(&state.round));
    }

    if state.round.current_player() == state.this_player {
      if let Some(chosen_tile) = clicked_tile {
        if state.round.try_play_move(chosen_tile).is_ok() {
          let msg = ClientMessage::PlaceSymbol(chosen_tile);
          state.msg_handler.try_write_message(Some(msg)).unwrap();
        }
      }
    } else {
      #[allow(clippy::collapsible_else_if)]
      if let Some(msg) = state
        .msg_handler
        .try_read_message::<ServerMessage>()
        .unwrap()
      {
        match msg {
          ServerMessage::OpponentPlaceSymbol(global_pos) => {
            state.round.try_play_move(global_pos).unwrap();
          }
          ServerMessage::OpponentGiveUp => {
            state.outcome = Some(RoundOutcome::Win(state.this_player));
          }
          _ => panic!("unexpected message: `{:?}`", msg),
        }
      }
    }

    state.outcome = state.outcome.or(state.round.outcome());
    if let Some(outcome) = state.outcome {
      state.stats.update(outcome);
    };
  }
}
