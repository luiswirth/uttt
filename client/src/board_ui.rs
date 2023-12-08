use common::{
  generic::board::{TileBoardState, TrivialTile},
  specific::{
    game::GameState,
    pos::{GlobalPos, InnerPos, OuterPos},
  },
  PlayerSymbol,
};
use eframe::egui;
use egui::{pos2, vec2, Color32, Painter, Rect, Sense, Stroke, Vec2};

use crate::util::{lightened_color, player_color};

/// returns `Some(global_pos)` if a tile was clicked
pub fn build_board_ui(ui: &mut egui::Ui, game_state: &GameState) -> Option<GlobalPos> {
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

  let draw_symbol = |painter: &Painter, rect: Rect, symbol: PlayerSymbol| {
    let stroke = Stroke::new(5.0, player_color(symbol));
    match symbol {
      PlayerSymbol::X => draw_cross(painter, rect, stroke),
      PlayerSymbol::O => draw_circle(painter, rect, stroke),
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
      let inner_board = game_state.board().tile(OuterPos::new(xouter, youter));

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

          let mut tile_color = match game_state.current_outer_pos() {
            Some(curr_outer_pos) if curr_outer_pos == OuterPos::new(xouter, youter) => {
              lightened_color(player_color(game_state.current_player()), 100)
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
