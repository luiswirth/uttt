pub mod board_ui;
pub mod util;

use crate::{board_ui::build_board_ui, util::choose_random_tile};
use common::{
  specific::{
    game::GameState,
    message::{ClientMessage, MessageIoHandlerNoBlocking, ServerMessage},
  },
  PlayerSymbol, DEFAULT_IP, DEFAULT_PORT,
};
use util::player_color;

use std::{
  mem,
  net::{Ipv4Addr, SocketAddrV4, TcpStream},
  str::FromStr,
};

use eframe::egui;

const RANDOM_MOVES: bool = false;

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
}
struct PlayingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: PlayerSymbol,
  game_state: GameState,
  can_place_symbol: bool,
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
    egui::CentralPanel::default()
      .show(ctx, |ui| {
        ui.add_space(50.0);
        ui.colored_label(egui::Color32::GREEN, "Successfully connected to server.");
        ui.label("Waiting for other player...");

        if let Some(msg) = state
          .msg_handler
          .try_read_message::<ServerMessage>()
          .unwrap()
        {
          let starting_player = msg.game_start();
          let game_state = GameState::new(starting_player);
          Client::Playing(PlayingState {
            msg_handler: state.msg_handler,
            this_player: state.this_player,
            game_state,
            can_place_symbol: true,
          })
        } else {
          Client::WaitingForGameStart(state)
        }
      })
      .inner
  }

  fn update_playing(mut state: PlayingState, ctx: &egui::Context) -> Self {
    state.msg_handler.try_write_message::<()>(None).unwrap();

    egui::SidePanel::left("left-panel").show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(10.0);

        ui.label(egui::RichText::new("You are").size(30.0));
        let (response, painter) =
          ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
        let rect = response.rect;
        board_ui::draw_symbol(&painter, rect, state.this_player);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        ui.label(egui::RichText::new("Turn of").size(30.0));
        ui.label(
          egui::RichText::new(if state.this_player == state.game_state.current_player() {
            "YOU"
          } else {
            "THEM"
          })
          .color(player_color(state.game_state.current_player()))
          .size(50.0),
        );

        let (response, painter) =
          ui.allocate_painter(egui::vec2(100.0, 100.0), egui::Sense::hover());
        let rect = response.rect;
        board_ui::draw_symbol(&painter, rect, state.game_state.current_player());
      });
    });

    egui::CentralPanel::default()
      .show(ctx, |ui| {
        let mut chosen_tile = build_board_ui(ui, &state.game_state, state.this_player);

        if RANDOM_MOVES {
          chosen_tile = Some(choose_random_tile(&state.game_state));
        }

        if state.this_player == state.game_state.current_player() && state.can_place_symbol {
          if let Some(chosen_tile) = chosen_tile {
            if state.game_state.could_play_move(chosen_tile) {
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
              assert!(state.game_state.try_play_move(global_pos));
              state.can_place_symbol = true;

              if let Some(outcome) = state.game_state.game_outcome() {
                // TODO: handle outcome
                println!("{:?}", outcome);
                Client::WaitingForGameStart(WaitingState {
                  msg_handler: state.msg_handler,
                  this_player: state.this_player,
                })
              } else {
                Client::Playing(state)
              }
            }
            _ => panic!("unexpected message: {:?}", msg),
          }
        } else {
          Client::Playing(state)
        }
      })
      .inner
  }
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
