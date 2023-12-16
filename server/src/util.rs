use std::{io, net::Ipv4Addr};

use common::{DEFAULT_IP, DEFAULT_PORT};

pub fn read_ip() -> Ipv4Addr {
  loop {
    println!(
      "Enter IP address (press enter for default = {}.):",
      DEFAULT_IP
    );
    let mut ip_addr = String::new();
    if io::stdin().read_line(&mut ip_addr).is_err() {
      println!("Reading ip address failed.");
      continue;
    }
    let ip_addr = ip_addr.trim();

    match ip_addr.is_empty() {
      true => {
        println!("Using default ip address {}.", DEFAULT_IP);
        break DEFAULT_IP;
      }
      false => match ip_addr.parse::<Ipv4Addr>() {
        Ok(ip_addr) => break ip_addr,
        Err(e) => {
          println!("Parsing ip address failed: {}.", e);
          continue;
        }
      },
    }
  }
}

pub fn read_port() -> u16 {
  loop {
    println!("Enter port (press enter for default = {}):", DEFAULT_PORT);
    let mut port = String::new();
    if io::stdin().read_line(&mut port).is_err() {
      println!("Reading ip address failed.");
      continue;
    }
    let port = port.trim();

    match port.is_empty() {
      true => {
        println!("Using default port {}", DEFAULT_PORT);
        break DEFAULT_PORT;
      }
      false => match port.parse::<u16>() {
        Ok(port) => break port,
        Err(e) => {
          println!("Parsing port failed: {}", e);
          continue;
        }
      },
    }
  }
}
