pub mod board_ui;
pub mod util;

use crate::{
  board_ui::build_board_ui,
  util::{choose_random_tile, player_color},
};
use common::{
  game::{RoundOutcome, RoundState, Stats},
  message::{ClientMessage, MessageIoHandlerNoBlocking, ServerMessage},
  PlayerSymbol, DEFAULT_IP, DEFAULT_PORT, PLAYERS,
};

use std::{
  mem,
  net::{Ipv4Addr, SocketAddrV4, TcpStream},
  str::FromStr,
};

use eframe::egui;

enum Client {
  Connecting(ConnectingState),
  WaitingForGameStart(WaitingState),
  Playing(PlayingState),
}
impl Default for Client {
  fn default() -> Self {
    Self::Connecting(Default::default())
  }
}
impl Client {
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    let mut style = egui::Style::default();
    style.text_styles.insert(
      egui::TextStyle::Heading,
      egui::FontId::new(50.0, egui::FontFamily::Proportional),
    );
    cc.egui_ctx.set_style(style);

    Self::default()
  }
}

struct ConnectingState {
  ip_addr: String,
  ip_addr_error: Option<String>,
  port: String,
  port_error: Option<String>,
  connection_error: Option<String>,

  msg_handler: Option<MessageIoHandlerNoBlocking>,
}
impl Default for ConnectingState {
  fn default() -> Self {
    let ip_addr = DEFAULT_IP.to_string();
    let port = DEFAULT_PORT.to_string();
    Self {
      ip_addr,
      ip_addr_error: None,
      port,
      port_error: None,
      connection_error: None,
      msg_handler: None,
    }
  }
}
struct WaitingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,

  stats: Stats,
}
struct PlayingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,

  stats: Stats,
  round: RoundState,

  can_place_symbol: bool,
  outcome: Option<RoundOutcome>,
}

impl eframe::App for Client {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint();
    self.update_state(ctx);
  }
}

impl Client {
  fn update_state(&mut self, ctx: &egui::Context) {
    *self = match mem::take(self) {
      Client::Connecting(state) => Self::update_connecting(state, ctx),
      Client::WaitingForGameStart(state) => Self::update_waiting(state, ctx),
      Client::Playing(state) => Self::update_playing(state, ctx),
    }
  }

  fn update_connecting(mut state: ConnectingState, ctx: &egui::Context) -> Self {
    egui::CentralPanel::default()
      .show(ctx, |ui| {
        ui.add_space(50.0);
        ui.vertical_centered(|ui| {
          ui.heading("Welcome to UTTT!");
          ui.label("Connect to a server.");

          let ip_label = ui.label("IP:");
          ui.text_edit_singleline(&mut state.ip_addr)
            .labelled_by(ip_label.id);
          let port_label = ui.label("Port:");
          ui.text_edit_singleline(&mut state.port)
            .labelled_by(port_label.id);

          if state.msg_handler.is_none()
            && (ui.button("Connect").clicked() || cfg!(feature = "auto_connect"))
          {
            match Ipv4Addr::from_str(state.ip_addr.trim()) {
              Err(e) => state.ip_addr_error = Some(e.to_string()),
              Ok(ip_addr) => {
                state.ip_addr_error = None;
                match state.port.trim().parse::<u16>() {
                  Err(e) => state.port_error = Some(e.to_string()),
                  Ok(port) => {
                    state.port_error = None;
                    let socket_addr = SocketAddrV4::new(ip_addr, port);
                    match TcpStream::connect(socket_addr) {
                      Err(e) => state.connection_error = Some(e.to_string()),
                      Ok(tcp_stream) => {
                        state.connection_error = None;
                        tcp_stream.set_nonblocking(true).unwrap();
                        let new_msg_handler = MessageIoHandlerNoBlocking::new(tcp_stream);
                        state.msg_handler = Some(new_msg_handler);
                      }
                    }
                  }
                }
              }
            }
          }
          if let Some(e) = state.ip_addr_error.as_ref() {
            ui.colored_label(egui::Color32::RED, format!("Invalid IP address: {}", e));
          }
          if let Some(e) = state.port_error.as_ref() {
            ui.colored_label(egui::Color32::RED, format!("Invalid port: {}", e));
          }
          if let Some(e) = state.connection_error.as_ref() {
            ui.colored_label(
              egui::Color32::RED,
              format!("Failed to connect to server: {}", e),
            );
          }
          if let Some(mut msg_handler) = state.msg_handler {
            ui.colored_label(egui::Color32::GREEN, "Successfully connected to server.");
            ui.label("Waiting for other player...");

            if let Some(msg) = msg_handler.try_read_message::<ServerMessage>().unwrap() {
              let symbol = msg.symbol_assignment();
              Client::WaitingForGameStart(WaitingState {
                msg_handler,
                this_player: symbol,
                stats: Stats::default(),
              })
            } else {
              state.msg_handler = Some(msg_handler);
              Client::Connecting(state)
            }
          } else {
            Client::Connecting(state)
          }
        })
        .inner
      })
      .inner
  }

  fn update_waiting(mut state: WaitingState, ctx: &egui::Context) -> Self {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.heading("Waiting for other player...");
        ui.add_space(50.0);
        build_stats_ui(ui, &state.stats, state.this_player);
      })
    });

    if let Some(msg) = state
      .msg_handler
      .try_read_message::<ServerMessage>()
      .unwrap()
    {
      let starting_player = msg.round_start();
      let round_state = RoundState::new(starting_player);
      Client::Playing(PlayingState {
        msg_handler: state.msg_handler,
        this_player: state.this_player,
        stats: state.stats,
        round: round_state,
        can_place_symbol: true,
        outcome: None,
      })
    } else {
      Client::WaitingForGameStart(state)
    }
  }

  fn update_playing(mut state: PlayingState, ctx: &egui::Context) -> Self {
    state.msg_handler.try_write_message::<()>(None).unwrap();

    state = match egui::SidePanel::left("left-panel")
      .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
          ui.add_space(10.0);

          build_stats_ui(ui, &state.stats, state.this_player);

          ui.add_space(20.0);
          ui.separator();
          ui.add_space(20.0);

          ui.label(egui::RichText::new("You are").size(30.0));
          let (response, painter) =
            ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
          let rect = response.rect;
          board_ui::draw_symbol(&painter, rect, state.this_player);

          ui.add_space(20.0);
          ui.separator();
          ui.add_space(20.0);

          if let Some(outcome) = state.outcome {
            ui.heading(match outcome {
              RoundOutcome::Win(p) => match p == state.this_player {
                true => "You won!".to_string(),
                false => "You lost!".to_string(),
              },
              RoundOutcome::Draw => "Draw!".to_string(),
            });

            if ui.button("Play again").clicked() || cfg!(feature = "auto_next_round") {
              let msg = ClientMessage::StartRoundRequest;
              state.msg_handler.try_write_message(Some(msg)).unwrap();
              return Ok(Client::WaitingForGameStart(WaitingState {
                msg_handler: state.msg_handler,
                this_player: state.this_player,
                stats: state.stats,
              }));
            }
          } else {
            ui.label(egui::RichText::new("Turn of").size(30.0));
            ui.label(
              egui::RichText::new(if state.this_player == state.round.current_player() {
                "YOU"
              } else {
                "THEM"
              })
              .color(player_color(state.round.current_player()))
              .size(50.0),
            );
            let (response, painter) =
              ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
            let rect = response.rect;
            board_ui::draw_symbol(&painter, rect, state.round.current_player());

            if state.round.current_player() == state.this_player && ui.button("Give up").clicked() {
              let msg = ClientMessage::GiveUp;
              state.msg_handler.try_write_message(Some(msg)).unwrap();
              let outcome = state
                .outcome
                .insert(RoundOutcome::Win(state.this_player.other()));
              state.stats.update(*outcome);
            }
          }

          Err(state)
        })
        .inner
      })
      .inner
    {
      Ok(new_state) => return new_state,
      Err(old_state) => old_state,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
      let mut chosen_tile = build_board_ui(ui, &state.round, state.this_player);

      if state.outcome.is_some() {
        return;
      }

      if cfg!(feature = "auto_play") {
        chosen_tile = Some(choose_random_tile(&state.round));
      }

      if state.this_player == state.round.current_player() && state.can_place_symbol {
        if let Some(chosen_tile) = chosen_tile {
          if state.round.could_play_move(chosen_tile) {
            let msg = ClientMessage::PlaceSymbolProposal(chosen_tile);
            state.msg_handler.try_write_message(Some(msg)).unwrap();
            state.can_place_symbol = false;
          }
        }
      }

      if let Some(msg) = state
        .msg_handler
        .try_read_message::<ServerMessage>()
        .unwrap()
      {
        match msg {
          ServerMessage::PlaceSymbolAccepted(global_pos) => {
            state.round.try_play_move(global_pos).unwrap();
            state.can_place_symbol = true;
          }
          ServerMessage::OtherGiveUp => {
            state.outcome = Some(RoundOutcome::Win(state.this_player));
          }
          _ => panic!(
            "expected `PlaceSymbolAccepted` or `OtherGiveUp`, got `{:?}`",
            msg
          ),
        }
      }

      state.outcome = state.outcome.or(state.round.outcome());
      if let Some(outcome) = state.outcome {
        state.stats.update(outcome);
      };
    });
    Client::Playing(state)
  }
}

fn build_stats_ui(ui: &mut egui::Ui, stats: &Stats, this_player: PlayerSymbol) {
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

fn main() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  eframe::run_native(
    "UTTT",
    Default::default(),
    Box::new(|cc| Box::new(Client::new(cc))),
  )
  .unwrap();
}
