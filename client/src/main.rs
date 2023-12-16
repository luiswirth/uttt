mod connecting;
pub mod playing;
pub mod waiting;

pub mod util;

use crate::{connecting::ConnectingState, playing::PlayingState, waiting::WaitingState};

use std::mem;

use eframe::egui;

fn main() {
  eframe::run_native(
    "UTTT",
    Default::default(),
    Box::new(|cc| Box::new(Client::new(cc))),
  )
  .unwrap();
}

pub enum Client {
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

impl eframe::App for Client {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint();
    self.update_state(ctx);
  }
}

impl Client {
  fn update_state(&mut self, ctx: &egui::Context) {
    *self = match mem::take(self) {
      Client::Connecting(state) => state.update(ctx),
      Client::WaitingForGameStart(state) => state.update(ctx),
      Client::Playing(state) => state.update(ctx),
    }
  }
}
