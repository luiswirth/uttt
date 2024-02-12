use common::{
  game::{RoundOutcome, RoundState, Stats},
  PlayerSymbol,
};

pub struct GameState {
  this_player: PlayerSymbol,
  stats: Stats,

  round: Option<RoundState>,
}

impl GameState {
  pub fn new(this_player: PlayerSymbol) -> Self {
    Self {
      this_player,
      stats: Stats::default(),
      round: None,
    }
  }
}

pub fn play_game(this_player: PlayerSymbol) {
  let game_state = GameState::new(this_player);

  loop {
    let starting_player = game_start();
    let outcome = play_round(starting_player);
    game_state.stats.update(outcome);
  }
}

pub fn play_round(starting_player: PlayerSymbol) -> RoundOutcome {
  let round_state = RoundState::new(starting_player);

  loop {
    let chosen_tile = get_move(&round_state);
    round_state.try_play_move(chosen_tile).unwrap();

    if let Some(outcome) = round_state.outcome() {
      return outcome;
    }
  }
}
