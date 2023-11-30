use common::{
  generic::board::{TileBoardState, TrivialTile},
  specific::{
    board::OuterBoard,
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
  curr_player: Player,
  outer_board: OuterBoard,
  curr_outer_pos_opt: Option<OuterPos>,
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
                    ClientState::Playing(PlayingState {
                      msg_handler,
                      this_player,
                      curr_player: starting_player,
                      outer_board: OuterBoard::default(),
                      curr_outer_pos_opt: None,
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
          if pstate.this_player == pstate.curr_player {
            ui.label("Your turn.");
          } else {
            ui.label("Opponent's turn.");
          }
          let clicked_tile = draw_board(ui, &pstate.outer_board);
          if let Some(clicked_tile) = clicked_tile {
            let mut is_allowed_to_place = pstate.curr_player == pstate.this_player
              && pstate
                .outer_board
                .tile(OuterPos::from(clicked_tile))
                .is_free()
              && pstate.outer_board.trivial_tile(clicked_tile).is_free();
            if let Some(curr_outer_pos) = pstate.curr_outer_pos_opt {
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
                  .outer_board
                  .place_symbol(global_pos, pstate.curr_player);
                pstate.curr_outer_pos_opt = Some(InnerPos::from(global_pos).as_outer());
                pstate.curr_player.switch();
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

fn draw_board(ui: &mut egui::Ui, outer_board: &OuterBoard) -> Option<GlobalPos> {
  use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Sense, Stroke, Vec2};

  let size = Vec2::splat(ui.available_size().min_elem()); // Concise way to determine the size
  let (response, painter) = ui.allocate_painter(size, Sense::click());

  // Helper function to draw a cross
  let draw_cross = |painter: &Painter, rect: Rect, stroke: Stroke| {
    let offset = rect.width() / 8.0;
    let a = rect.left_top() + Vec2::splat(offset);
    let b = rect.right_bottom() - Vec2::splat(offset);
    painter.line_segment([a, b], stroke);
    let a = rect.right_top() + vec2(-offset, offset);
    let b = rect.left_bottom() + vec2(offset, -offset);
    painter.line_segment([a, b], stroke);
  };

  // Helper function to draw a circle
  let draw_circle = |painter: &Painter, center: Pos2, radius: f32, stroke: Stroke| {
    painter.circle_stroke(center, radius, stroke);
  };

  let draw_tile = |painter: &Painter, rect: Rect, tile: TrivialTile| match tile {
    TrivialTile::Free => {}
    TrivialTile::Won(symbol) => {
      let color = match symbol {
        Player::Cross => Color32::RED,
        Player::Circle => Color32::BLUE,
      };
      let stroke = Stroke::new(5.0, color);
      match symbol {
        Player::Cross => draw_cross(painter, rect, stroke),
        Player::Circle => {
          let center = rect.center();
          let radius = rect.width() / 2.0 / 1.5;
          draw_circle(painter, center, radius, stroke);
        }
      }
    }
  };

  let draw_grid = |painter: &Painter, rect: Rect, cell_size: Vec2, stroke: Stroke| {
    for i in 1..=2 {
      let y = rect.top() + i as f32 * cell_size.y;
      let x = rect.left() + i as f32 * cell_size.x;
      painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
      painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
    }
  };

  // Draw outer grid
  let outer_rect = response.rect;
  let w_third = outer_rect.width() / 3.0;
  let cell_size = vec2(w_third, w_third);
  let stroke = Stroke::new(10.0, Color32::BLACK);
  draw_grid(&painter, outer_rect, cell_size, stroke);

  // Iterate over the outer and inner boards
  for youter in 0..3 {
    for xouter in 0..3 {
      let inner_rect = Rect::from_min_size(
        pos2(
          outer_rect.left() + xouter as f32 * w_third,
          outer_rect.top() + youter as f32 * w_third,
        ),
        cell_size,
      );

      // Draw inner grid
      let stroke = Stroke::new(5.0, Color32::BLACK);
      draw_grid(
        &painter,
        inner_rect,
        vec2(w_third / 3.0, w_third / 3.0),
        stroke,
      );

      // Draw the tiles
      let inner_board = outer_board.tile(OuterPos::new(xouter, youter));
      if let TileBoardState::Won(p) = inner_board.board_state() {
        draw_tile(&painter, inner_rect, TrivialTile::Won(p))
      }

      for yinner in 0..3 {
        for xinner in 0..3 {
          let tile_rect = Rect::from_min_size(
            pos2(
              inner_rect.left() + xinner as f32 * w_third / 3.0,
              inner_rect.top() + yinner as f32 * w_third / 3.0,
            ),
            vec2(w_third / 3.0, w_third / 3.0),
          );
          let tile = inner_board.tile(InnerPos::new(xinner, yinner));
          draw_tile(&painter, tile_rect, *tile);

          // check if mouse clicked inside tile_rect
          if response.clicked()
            && tile_rect.contains(ui.ctx().input(|r| r.pointer.hover_pos().unwrap()))
          {
            return Some(dbg!(GlobalPos::from((
              OuterPos::new(xouter, youter),
              InnerPos::new(xinner, yinner),
            ))));
          }
        }
      }
    }
  }
  None
}
