use common::{
  generic::board::{TileBoardState, TrivialTile},
  specific::{
    game::GameState,
    message::{ClientMessage, MessageIoHandlerNoBlocking, ServerMessage},
    pos::{GlobalPos, InnerPos, OuterPos},
  },
  Player, DEFAULT_IP, DEFAULT_PORT,
};

use std::{
  mem,
  net::{Ipv4Addr, SocketAddrV4, TcpStream},
  str::FromStr,
};

use eframe::egui;

#[derive(Default)]
pub struct Client {
  client_state: ClientState,
}

impl Client {
  pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
  this_player: Option<Player>,
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
      this_player: None,
    }
  }
}
struct PlayingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: Player,
  game_state: GameState,
}

enum ClientState {
  Connecting(ConnectingState),
  Playing(PlayingState),
}
impl Default for ClientState {
  fn default() -> Self {
    Self::Connecting(Default::default())
  }
}

impl eframe::App for Client {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint();

    egui::CentralPanel::default().show(ctx, |ui| {
      type Ts = egui::TextStyle;
      let mut style = (*ctx.style()).clone();
      style.text_styles.insert(
        Ts::Heading,
        egui::FontId::new(50.0, egui::FontFamily::Proportional),
      );
      ctx.set_style(style);

      self.client_state = match mem::take(&mut self.client_state) {
        ClientState::Connecting(mut cstate) => {
          if let Some(ref mut msg_handler) = cstate.msg_handler {
            msg_handler.try_write_message::<()>(None).unwrap();
          }

          ui.add_space(50.0);
          ui.vertical_centered(|ui| {
            ui.heading("Welcome to UTTT!");
            ui.label("Connect to a server.");
            let ip_label = ui.label("IP:");
            ui.text_edit_singleline(&mut cstate.ip_addr)
              .labelled_by(ip_label.id);
            let port_label = ui.label("Port:");
            ui.text_edit_singleline(&mut cstate.port)
              .labelled_by(port_label.id);
            if cstate.msg_handler.is_none() && ui.button("Connect").clicked() {
              match Ipv4Addr::from_str(cstate.ip_addr.trim()) {
                Ok(ip_addr) => {
                  cstate.ip_addr_error = None;
                  match cstate.port.trim().parse::<u16>() {
                    Ok(port) => {
                      cstate.port_error = None;
                      let socket_addr = SocketAddrV4::new(ip_addr, port);
                      match TcpStream::connect(socket_addr) {
                        Ok(tcp_stream) => {
                          cstate.connection_error = None;
                          tcp_stream.set_nonblocking(true).unwrap();
                          let new_msg_handler = MessageIoHandlerNoBlocking::new(tcp_stream);
                          cstate.msg_handler = Some(new_msg_handler);
                        }
                        Err(e) => {
                          cstate.connection_error = Some(e.to_string());
                        }
                      }
                    }
                    Err(e) => {
                      cstate.port_error = Some(e.to_string());
                    }
                  }
                }
                Err(e) => {
                  cstate.ip_addr_error = Some(e.to_string());
                }
              }
            }
            if let Some(e) = cstate.ip_addr_error.as_ref() {
              ui.colored_label(egui::Color32::RED, format!("Invalid IP address: {}", e));
            }
            if let Some(e) = cstate.port_error.as_ref() {
              ui.colored_label(egui::Color32::RED, format!("Invalid port: {}", e));
            }
            if let Some(e) = cstate.connection_error.as_ref() {
              ui.colored_label(
                egui::Color32::RED,
                format!("Failed to connect to server: {}", e),
              );
            }
            if let Some(mut msg_handler) = cstate.msg_handler {
              ui.colored_label(egui::Color32::GREEN, "Successfully connected to server.");
              ui.label("Waiting for other player...");

              match cstate.this_player {
                None => {
                  if let Some(msg) = msg_handler.try_read_message::<ServerMessage>().unwrap() {
                    let symbol = msg.symbol_assignment();
                    cstate.this_player = Some(symbol);
                  }
                  cstate.msg_handler = Some(msg_handler);
                  ClientState::Connecting(cstate)
                }
                Some(this_player) => {
                  if let Some(msg) = msg_handler.try_read_message::<ServerMessage>().unwrap() {
                    let starting_player = msg.game_start();
                    let game_state = GameState::new(starting_player);
                    ClientState::Playing(PlayingState {
                      msg_handler,
                      this_player,
                      game_state,
                    })
                  } else {
                    cstate.msg_handler = Some(msg_handler);
                    ClientState::Connecting(cstate)
                  }
                }
              }
            } else {
              ClientState::Connecting(cstate)
            }
          })
          .inner
        }
        ClientState::Playing(mut pstate) => {
          pstate.msg_handler.try_write_message::<()>(None).unwrap();
          if pstate.this_player == pstate.game_state.curr_player {
            ui.label("Your turn.");
          } else {
            ui.label("Opponent's turn.");
          }
          let clicked_tile = draw_board(ui, &pstate.game_state);
          if let Some(clicked_tile) = clicked_tile {
            let mut is_allowed_to_place = pstate.game_state.curr_player == pstate.this_player
              && pstate
                .game_state
                .outer_board
                .tile(OuterPos::from(clicked_tile))
                .is_free()
              && pstate
                .game_state
                .outer_board
                .trivial_tile(clicked_tile)
                .is_free();
            if let Some(curr_outer_pos) = pstate.game_state.curr_outer_pos_opt {
              is_allowed_to_place &= curr_outer_pos == OuterPos::from(clicked_tile);
            }
            if is_allowed_to_place {
              let msg = ClientMessage::PlaceSymbolProposal(clicked_tile);
              pstate.msg_handler.try_write_message(Some(msg)).unwrap();
            }
          }
          if let Some(msg) = pstate
            .msg_handler
            .try_read_message::<ServerMessage>()
            .unwrap()
          {
            match msg {
              ServerMessage::PlaceSymbolAccepted(global_pos) => {
                pstate
                  .game_state
                  .outer_board
                  .place_symbol(global_pos, pstate.game_state.curr_player);
                let next_outer_pos = InnerPos::from(global_pos).as_outer();
                if pstate.game_state.outer_board.tile(next_outer_pos).is_free() {
                  pstate.game_state.curr_outer_pos_opt =
                    Some(InnerPos::from(global_pos).as_outer());
                } else {
                  pstate.game_state.curr_outer_pos_opt = None;
                }
                pstate.game_state.curr_player.switch();
              }
              _ => panic!(),
            }
          }
          ClientState::Playing(pstate)
        }
      }
    });
  }
}

fn main() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  let native_options = eframe::NativeOptions {
    initial_window_size: Some(egui::vec2(320.0, 240.0)),
    ..Default::default()
  };
  eframe::run_native(
    "UTTT",
    native_options,
    Box::new(|cc| Box::new(Client::new(cc))),
  )
  .unwrap();
}

fn draw_board(ui: &mut egui::Ui, game_state: &GameState) -> Option<GlobalPos> {
  use egui::{pos2, vec2, Color32, Painter, Rect, Sense, Stroke, Vec2};

  let available_size = Vec2::splat(ui.available_size().min_elem());
  let (response, painter) = ui.allocate_painter(available_size, Sense::click());

  let draw_cross = |painter: &Painter, rect: Rect, stroke: Stroke| {
    let offset = rect.width() / 8.0;
    let a = rect.left_top() + Vec2::splat(offset);
    let b = rect.right_bottom() - Vec2::splat(offset);
    painter.line_segment([a, b], stroke);
    let a = rect.right_top() + vec2(-offset, offset);
    let b = rect.left_bottom() + vec2(offset, -offset);
    painter.line_segment([a, b], stroke);
  };

  let draw_circle = |painter: &Painter, rect: Rect, stroke: Stroke| {
    let center = rect.center();
    let radius = rect.width() / 2.0 / 1.5;
    painter.circle_stroke(center, radius, stroke);
  };

  let draw_symbol = |painter: &Painter, rect: Rect, symbol: Player| {
    let stroke = Stroke::new(5.0, player_color(symbol));
    match symbol {
      Player::Cross => draw_cross(painter, rect, stroke),
      Player::Circle => draw_circle(painter, rect, stroke),
    }
  };

  let outer_rect = response.rect;

  let inner_board_padding = 10.0;
  let inner_board_size = (outer_rect.width() - 2.0 * inner_board_padding) / 3.0;
  let tile_padding = 5.0;
  let tile_size = (inner_board_size - 2.0 * tile_padding) / 3.0;

  let mut clicked_tile = None;
  for youter in 0..3 {
    for xouter in 0..3 {
      let inner_rect = Rect::from_min_size(
        pos2(
          outer_rect.left() + xouter as f32 * (inner_board_size + inner_board_padding),
          outer_rect.top() + youter as f32 * (inner_board_size + inner_board_padding),
        ),
        Vec2::splat(inner_board_size),
      );

      let inner_board = game_state.outer_board.tile(OuterPos::new(xouter, youter));
      if let TileBoardState::Won(p) = inner_board.board_state() {
        draw_symbol(&painter, inner_rect, p);
      }

      for yinner in 0..3 {
        for xinner in 0..3 {
          let tile_rect = Rect::from_min_size(
            pos2(
              inner_rect.left() + xinner as f32 * (tile_size + tile_padding),
              inner_rect.top() + yinner as f32 * (tile_size + tile_padding),
            ),
            Vec2::splat(tile_size),
          );
          let tile = inner_board.tile(InnerPos::new(xinner, yinner));

          let unhovered_color = match game_state.curr_outer_pos_opt {
            Some(curr_outer_pos) if curr_outer_pos == OuterPos::new(xouter, youter) => {
              player_color(game_state.curr_player)
            }
            _ => Color32::DARK_GRAY,
          };

          let tile_color = ui.ctx().input(|r| {
            r.pointer
              .hover_pos()
              .map(|pos| match tile_rect.contains(pos) {
                true => Color32::WHITE,
                false => unhovered_color,
              })
              .unwrap_or(unhovered_color)
          });
          painter.rect_filled(tile_rect, 0.0, tile_color);

          if let TrivialTile::Won(p) = tile {
            draw_symbol(&painter, tile_rect, *p);
          }

          if response.clicked()
            && tile_rect.contains(ui.ctx().input(|r| r.pointer.hover_pos().unwrap()))
          {
            clicked_tile = Some(dbg!(GlobalPos::from((
              OuterPos::new(xouter, youter),
              InnerPos::new(xinner, yinner),
            ))));
          }
        }
      }
    }
  }
  clicked_tile
}

fn player_color(player: Player) -> egui::Color32 {
  use egui::Color32;
  match player {
    Player::Cross => Color32::RED,
    Player::Circle => Color32::BLUE,
  }
}
