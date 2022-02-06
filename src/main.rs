#![windows_subsystem = "windows"]

use std::io::prelude::*;
use std::time::Duration;
use std::path::{Path, PathBuf};

use tokio::time;
use tokio_modbus::prelude::*;
use tokio_serial::{SerialStream, StopBits, DataBits, Parity};

mod gui;

const N_RELAYS: usize = 16;
const BAUDRATE: u32 = 9600;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut a = gui::Gui::new();
	a.run().await?;
	Ok(())
}

// If path contains spaces, dialog returns it as vector of pathbufs
// This function concatenates them back into one PathBuf with spaces
fn fix_pathbuf_parts(parts: &[PathBuf]) -> Option<PathBuf> {
	if parts.is_empty() {
		return None;
	}
	let mut filename = String::new();
	for p in parts {
		filename.push_str(p.to_str().unwrap());
		filename.push_str(" ");
	}
	filename.pop();

	Some(PathBuf::from(filename))
}

async fn open_connection(com: &str, slave: u8) -> Result<client::Context, Box<dyn std::error::Error>> {
	let s = Slave(slave);
	let builder = tokio_serial::new(com, BAUDRATE)
		.data_bits(DataBits::Eight)
		.parity(Parity::None)
		.stop_bits(StopBits::One)
		.timeout(Duration::from_secs(1));
	let port = SerialStream::open(&builder)?;
	
	let ctx = rtu::connect_slave(port, s).await?;
	Ok(ctx)
}

// For this board we turn relay ON with code 0100
// and turn it OFF with code 0200
// No one knows why, but it works :)
async fn set_relays(com: &str, slave: u8, state: &[bool]) -> Result<(), Box<dyn std::error::Error>> {
	let mut ctx = open_connection(com, slave).await?;

	// Iteration 1: turn off all needed relays
	// Iteration 2: torn on all needed relays
	for relay_operation in [false, true] {
		// For all relays
		for (i, &e) in state.iter().enumerate() {
			// If we need to change this relay now
			if e == relay_operation {
				time::timeout(
					Duration::from_secs(2),
					ctx.write_single_register(
						(i as u16)+1,
						if relay_operation {0x0100} else {0x0200}
					)
				).await??;
				// Delay after each operation
				time::sleep(Duration::from_millis(5)).await;
			}
		}
	}
	
	Ok(())
}

async fn get_relays(com: &str, slave: u8) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
	let mut ctx = open_connection(com, slave).await?;
	
	let rsp = tokio::time::timeout(
		Duration::from_secs(2),
		ctx.read_holding_registers(0x01, N_RELAYS as u16)
	).await??;
	let state = rsp.iter().map(|&x| x == 1).collect();
	
	Ok(state)
}

fn state_str_to_bool(state: &str) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
	if state.len() != N_RELAYS || state.chars().any(|x| x != '0' && x != '1') {
		Err("Invalid preset format".into())
	} else {
		Ok(state.chars().map(|c| c == '1').collect())
	}
}

fn state_bool_to_str(state: &[bool]) -> String {
	state.iter().map(|&x| if x {'1'} else {'0'}).collect()
}
