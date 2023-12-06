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

const RANDOM_MOVES: bool = true;

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
  this_player: Player,
}
struct PlayingState {
  msg_handler: MessageIoHandlerNoBlocking,
  this_player: Player,
  game_state: GameState,
  can_place_symbol: bool,
}

enum ClientState {
  Connecting(ConnectingState),
  WaitingForGameStart(WaitingState),
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

              if let Some(msg) = msg_handler.try_read_message::<ServerMessage>().unwrap() {
                let symbol = msg.symbol_assignment();
                ClientState::WaitingForGameStart(WaitingState {
                  msg_handler,
                  this_player: symbol,
                })
              } else {
                cstate.msg_handler = Some(msg_handler);
                ClientState::Connecting(cstate)
              }
            } else {
              ClientState::Connecting(cstate)
            }
          })
          .inner
        }
        ClientState::WaitingForGameStart(mut wstate) => {
          ui.add_space(50.0);
          ui.colored_label(egui::Color32::GREEN, "Successfully connected to server.");
          ui.label("Waiting for other player...");

          if let Some(msg) = wstate
            .msg_handler
            .try_read_message::<ServerMessage>()
            .unwrap()
          {
            let starting_player = msg.game_start();
            let game_state = GameState::new(starting_player);
            ClientState::Playing(PlayingState {
              msg_handler: wstate.msg_handler,
              this_player: wstate.this_player,
              game_state,
              can_place_symbol: true,
            })
          } else {
            ClientState::WaitingForGameStart(wstate)
          }
        }
        ClientState::Playing(mut pstate) => {
          pstate.msg_handler.try_write_message::<()>(None).unwrap();

          if pstate.this_player == pstate.game_state.curr_player {
            ui.label("Your turn.");
          } else {
            ui.label("Opponent's turn.");
          }

          let mut chosen_tile = draw_board(ui, &pstate.game_state);

          if RANDOM_MOVES {
            chosen_tile = Some(random_tile(&pstate.game_state));
          }

          if pstate.this_player == pstate.game_state.curr_player && pstate.can_place_symbol {
            if let Some(chosen_tile) = chosen_tile {
              if pstate.game_state.could_place_symbol(chosen_tile) {
                let msg = ClientMessage::PlaceSymbolProposal(chosen_tile);
                pstate.msg_handler.try_write_message(Some(msg)).unwrap();
                pstate.can_place_symbol = false;
              }
            }
          }

          if let Some(msg) = pstate
            .msg_handler
            .try_read_message::<ServerMessage>()
            .unwrap()
          {
            match msg {
              ServerMessage::PlaceSymbolAccepted(global_pos) => {
                assert!(pstate
                  .game_state
                  .outer_board
                  .try_place_symbol(global_pos, pstate.game_state.curr_player));

                pstate.game_state.update_outer_pos(global_pos);
                pstate.game_state.curr_player.switch();
                pstate.can_place_symbol = true;

                if !pstate.game_state.outer_board.board_state().is_placeable() {
                  ClientState::WaitingForGameStart(WaitingState {
                    msg_handler: pstate.msg_handler,
                    this_player: pstate.this_player,
                  })
                } else {
                  ClientState::Playing(pstate)
                }
              }
              _ => panic!("unexpected message: {:?}", msg),
            }
          } else {
            ClientState::Playing(pstate)
          }
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

          let mut tile_color = match game_state.curr_outer_pos_opt {
            Some(curr_outer_pos) if curr_outer_pos == OuterPos::new(xouter, youter) => {
              lightened_color(player_color(game_state.curr_player), 100)
            }
            _ => Color32::DARK_GRAY,
          };

          ui.ctx().input(|r| {
            if let Some(hover_pos) = r.pointer.hover_pos() {
              if tile_rect.contains(hover_pos) {
                tile_color = lightened_color(tile_color, 100);
              }
            }
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

      if let TileBoardState::Won(p) = inner_board.board_state() {
        draw_symbol(&painter, inner_rect, p);
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

fn lightened_color(color: egui::Color32, amount: u8) -> egui::Color32 {
  egui::Color32::from_rgb(
    color.r().saturating_add(amount),
    color.g().saturating_add(amount),
    color.b().saturating_add(amount),
  )
}

fn random_tile(game_state: &GameState) -> GlobalPos {
  use rand::Rng;
  let mut rng = rand::thread_rng();
  let outer_pos = game_state.curr_outer_pos_opt.unwrap_or_else(|| loop {
    let outer_pos = OuterPos::new(rng.gen_range(0..3), rng.gen_range(0..3));
    if game_state
      .outer_board
      .tile(outer_pos)
      .board_state()
      .is_placeable()
    {
      break outer_pos;
    }
  });
  let inner_pos = loop {
    let inner_pos = InnerPos::new(rng.gen_range(0..3), rng.gen_range(0..3));
    if game_state
      .outer_board
      .tile(outer_pos)
      .tile(inner_pos)
      .is_free()
    {
      break inner_pos;
    }
  };
  GlobalPos::from((outer_pos, inner_pos))
}
