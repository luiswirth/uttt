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
  game::{PlayerAction, RoundOutcome, RoundState, Stats},
  msg::{
    ClientMsgAction, ClientReqRoundStart, MessageIoHandlerNoBlocking, ServerMsgOpponentAction,
  },
  PlayerSymbol,
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

    let mut action = None;
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
            action = Some(PlayerAction::GiveUp);
          }
        }
      })
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      let chosen_tile = build_board_ui(ui, &self.round, self.this_player);
      action = action.or(chosen_tile.map(PlayerAction::MakeMove));
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

    if self.outcome.is_none() {
      self.update_round(action);
    }

    Client::Playing(self)
  }

  fn update_round(&mut self, mut action: Option<PlayerAction>) {
    let my_turn = self.round.current_player() == self.this_player;

    if my_turn {
      if cfg!(feature = "auto_play") {
        action = Some(PlayerAction::MakeMove(choose_random_tile(&self.round)));
      }
    } else {
      debug_assert!(
        action.is_none(),
        "UI should not allow actions, when it's not your turn."
      );
      let msg = self.msg_handler.try_read_msg().unwrap();
      action = msg.map(|ServerMsgOpponentAction(action)| action);
    }

    if let Some(action) = action {
      match action {
        PlayerAction::MakeMove(chosen_tile) => self
          .round
          .try_play_move(self.round.current_player(), chosen_tile)
          .unwrap(),
        PlayerAction::GiveUp => {
          self.outcome = Some(RoundOutcome::Win(self.round.current_player().other()));
        }
      };

      if my_turn {
        self
          .msg_handler
          .try_write_msg(Some(ClientMsgAction(action)))
          .unwrap();
      }
    }

    self.outcome = self.outcome.or(self.round.outcome());
    if let Some(outcome) = self.outcome {
      self.stats.update(outcome);
    };
  }
}
