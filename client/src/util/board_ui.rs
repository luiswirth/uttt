use common::{
  board::{tile::TrivialTileState, TileBoardState},
  game::RoundState,
  GlobalPos, InnerPos, OuterPos, PlayerSymbol,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Painter, Rect, Sense, Stroke, Vec2};

use crate::util::{lightened_color, player_color};

/// returns `Some(global_pos)` if a tile was clicked
pub fn build_board_ui(
  ui: &mut egui::Ui,
  game_state: &RoundState,
  this_player: PlayerSymbol,
) -> Option<GlobalPos> {
  let available_size = Vec2::splat(ui.available_size().min_elem());
  let (response, painter) = ui.allocate_painter(available_size, Sense::click());

  let full_rect = response.rect;

  let outer_board_padding = full_rect.width() / 30.0;
  let outer_board_size = full_rect.width() - outer_board_padding;
  let inner_board_padding = outer_board_size / 50.0;
  let inner_board_size = (outer_board_size - 2.0 * inner_board_padding) / 3.0;
  let tile_padding = outer_board_padding / 3.0;
  let tile_size = (inner_board_size - 2.0 * tile_padding) / 3.0;

  let outer_rect = Rect::from_center_size(full_rect.center(), Vec2::splat(outer_board_size));

  let current_player_color = player_color(game_state.current_player());

  if game_state.current_player() == this_player {
    painter.rect_stroke(
      full_rect,
      0.0,
      egui::Stroke::new(0.5 * outer_board_padding, current_player_color),
    );
  }

  let mut chosen_tile = None;
  for youter in 0..3 {
    for xouter in 0..3 {
      let inner_rect = Rect::from_min_size(
        pos2(
          outer_rect.left() + xouter as f32 * (inner_board_size + inner_board_padding),
          outer_rect.top() + youter as f32 * (inner_board_size + inner_board_padding),
        ),
        Vec2::splat(inner_board_size),
      );
      let outer_pos = OuterPos::new(xouter, youter);
      let inner_board = game_state.board().tile_state(outer_pos);

      for yinner in 0..3 {
        for xinner in 0..3 {
          let tile_rect = Rect::from_min_size(
            pos2(
              inner_rect.left() + xinner as f32 * (tile_size + tile_padding),
              inner_rect.top() + yinner as f32 * (tile_size + tile_padding),
            ),
            Vec2::splat(tile_size),
          );
          let inner_pos = InnerPos::new(xinner, yinner);
          let global_pos = GlobalPos::from((outer_pos, inner_pos));
          let tile = inner_board.tile_state(inner_pos);

          let is_hovered = ui.ctx().input(|r| {
            r.pointer
              .hover_pos()
              .map(|hover_pos| tile_rect.contains(hover_pos))
              .unwrap_or(false)
          });
          let is_hovered_playable =
            is_hovered && game_state.could_play_move(this_player, global_pos);

          if response.clicked() && is_hovered_playable {
            chosen_tile = Some(GlobalPos::from((
              OuterPos::new(xouter, youter),
              InnerPos::new(xinner, yinner),
            )));
          }

          let default_tile_color = Color32::DARK_GRAY;
          let marked_tile_color = lightened_color(current_player_color, 100);
          let mut tile_color = match game_state.current_outer_pos() {
            Some(curr_outer_pos) if curr_outer_pos == OuterPos::new(xouter, youter) => {
              marked_tile_color
            }
            None => marked_tile_color,
            _ => default_tile_color,
          };
          if is_hovered_playable {
            tile_color = lightened_color(tile_color, 100);
          }
          painter.rect_filled(tile_rect, 0.0, tile_color);

          if let TrivialTileState::Won(p) = tile {
            draw_symbol(&painter, tile_rect, *p);
          }
        }
      }

      if let TileBoardState::Won(p) = inner_board.board_state() {
        draw_symbol(&painter, inner_rect, p);
      }
    }
  }
  chosen_tile
}

pub fn draw_symbol(painter: &Painter, rect: Rect, symbol: PlayerSymbol) {
  let stroke = Stroke::new(rect.width() / 10.0, player_color(symbol));
  match symbol {
    PlayerSymbol::X => draw_cross(painter, rect, stroke),
    PlayerSymbol::O => draw_circle(painter, rect, stroke),
  }
}

pub fn draw_cross(painter: &Painter, rect: Rect, stroke: Stroke) {
  let offset = rect.width() / 8.0;
  let a = rect.left_top() + Vec2::splat(offset);
  let b = rect.right_bottom() - Vec2::splat(offset);
  painter.line_segment([a, b], stroke);
  let a = rect.right_top() + vec2(-offset, offset);
  let b = rect.left_bottom() + vec2(offset, -offset);
  painter.line_segment([a, b], stroke);
}

pub fn draw_circle(painter: &Painter, rect: Rect, stroke: Stroke) {
  let center = rect.center();
  let radius = rect.width() / 2.0 / 1.5;
  painter.circle_stroke(center, radius, stroke);
}
