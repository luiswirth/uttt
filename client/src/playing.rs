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
  msg::{ClientReqRoundStart, MessageIoHandlerNoBlocking, MsgPlayerAction},
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
    stats: Stats,
    starting_player: PlayerSymbol,
  ) -> Self {
    let round = RoundState::new(starting_player);
    Self {
      msg_handler,
      this_player,
      stats,
      round,
      outcome: None,
    }
  }

  pub fn update(mut self, ctx: &egui::Context) -> Client {
    self.msg_handler.try_write_msg::<()>(None).unwrap();

    let mut should_restart_game = false;
    let mut give_up = false;
    let mut chosen_tile = None;

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
            give_up = true;
          }
        }
      })
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      chosen_tile = build_board_ui(ui, &self.round, self.this_player);
    });

    if should_restart_game {
      self
        .msg_handler
        .try_write_msg(Some(ClientReqRoundStart))
        .unwrap();
      return Client::WaitingForGameStart(WaitingState::new(
        self.msg_handler,
        self.this_player,
        self.stats,
      ));
    }

    Self::update_round(&mut self, chosen_tile, give_up);

    Client::Playing(self)
  }

  fn update_round(&mut self, mut clicked_tile: Option<GlobalPos>, give_up: bool) {
    if self.outcome.is_some() {
      return;
    }

    if give_up {
      let outcome = self
        .outcome
        .insert(RoundOutcome::Win(self.this_player.other()));
      self.stats.update(*outcome);

      self
        .msg_handler
        .try_write_msg(Some(MsgPlayerAction::GiveUp))
        .unwrap();

      return;
    }

    if cfg!(feature = "auto_play") {
      clicked_tile = Some(choose_random_tile(&self.round));
    }

    if self.round.current_player() == self.this_player {
      if let Some(chosen_tile) = clicked_tile {
        if self.round.try_play_move(chosen_tile).is_ok() {
          self
            .msg_handler
            .try_write_msg(Some(MsgPlayerAction::MakeMove(chosen_tile)))
            .unwrap();
        }
      }
    } else {
      #[allow(clippy::collapsible_else_if)]
      if let Some(action) = self.msg_handler.try_read_msg().unwrap() {
        match action {
          MsgPlayerAction::MakeMove(global_pos) => self.round.try_play_move(global_pos).unwrap(),
          MsgPlayerAction::GiveUp => self.outcome = Some(RoundOutcome::Win(self.this_player)),
        }
      }
    }

    self.outcome = self.outcome.or(self.round.outcome());
    if let Some(outcome) = self.outcome {
      self.stats.update(outcome);
    };
  }
}
