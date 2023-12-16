use crate::{Client, WaitingState};

use common::{
  message::{MessageIoHandlerNoBlocking, ServerMessage},
  DEFAULT_IP, DEFAULT_PORT,
};

use std::{
  net::{Ipv4Addr, SocketAddrV4, TcpStream},
  str::FromStr,
};

use eframe::egui;

pub struct ConnectingState {
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

impl ConnectingState {
  pub fn update(mut self, ctx: &egui::Context) -> Client {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.add_space(50.0);
      ui.vertical_centered(|ui| {
        ui.heading("Welcome to UTTT!");
        ui.label("Connect to a server.");

        let ip_label = ui.label("IP:");
        ui.text_edit_singleline(&mut self.ip_addr)
          .labelled_by(ip_label.id);
        let port_label = ui.label("Port:");
        ui.text_edit_singleline(&mut self.port)
          .labelled_by(port_label.id);

        if self.msg_handler.is_none()
          && (ui.button("Connect").clicked() || cfg!(feature = "auto_connect"))
        {
          self.on_connect_clicked()
        }

        if let Some(e) = self.ip_addr_error.as_ref() {
          ui.colored_label(egui::Color32::RED, format!("Invalid IP address: {}", e));
        }
        if let Some(e) = self.port_error.as_ref() {
          ui.colored_label(egui::Color32::RED, format!("Invalid port: {}", e));
        }
        if let Some(e) = self.connection_error.as_ref() {
          ui.colored_label(
            egui::Color32::RED,
            format!("Failed to connect to server: {}", e),
          );
        }

        if self.msg_handler.is_some() {
          ui.colored_label(egui::Color32::GREEN, "Successfully connected to server.");
          ui.label("Waiting for other player...");
        }
      })
    });

    if let Some(mut msg_handler) = self.msg_handler {
      if let Some(msg) = msg_handler.try_read_message::<ServerMessage>().unwrap() {
        let symbol = msg.symbol_assignment();
        return Client::WaitingForGameStart(WaitingState::new(msg_handler, symbol));
      } else {
        self.msg_handler = Some(msg_handler);
      }
    }

    Client::Connecting(self)
  }

  fn on_connect_clicked(&mut self) {
    match Ipv4Addr::from_str(self.ip_addr.trim()) {
      Err(e) => self.ip_addr_error = Some(e.to_string()),
      Ok(ip_addr) => {
        self.ip_addr_error = None;
        match self.port.trim().parse::<u16>() {
          Err(e) => self.port_error = Some(e.to_string()),
          Ok(port) => {
            self.port_error = None;
            let socket_addr = SocketAddrV4::new(ip_addr, port);
            match TcpStream::connect(socket_addr) {
              Err(e) => self.connection_error = Some(e.to_string()),
              Ok(tcp_stream) => {
                self.connection_error = None;
                tcp_stream.set_nonblocking(true).unwrap();
                let new_msg_handler = MessageIoHandlerNoBlocking::new(tcp_stream);
                self.msg_handler = Some(new_msg_handler);
              }
            }
          }
        }
      }
    }
  }
}
