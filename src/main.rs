//#![windows_subsystem = "windows"]

use std::error::Error;
use std::io::prelude::*;
use std::time::Duration;
use std::path::{Path, PathBuf};

use clap::Parser;

use tokio::time;
use tokio_modbus::prelude::*;
use tokio_serial::{
	SerialStream,
	StopBits,
	DataBits,
	Parity,
	available_ports,
};

mod gui;
mod project;

/// Relay Modbus Controller
#[derive(clap::Parser, Debug)]
#[clap(author, version, about="(\\|)(;,,,;)(|/)", long_about=None)]
pub struct Args {
	/// Do not display window. Just apply state and leave
	#[clap(short='w', long="nowin", group="WIN", value_parser)]
	no_window: bool,

	/// Project file containing all settings
	#[clap(value_parser)]
	project: Option<PathBuf>,

	/// State of relays. It is a string of N chars (0 and 1). Overrides project settings
	#[clap(short, long, value_parser, required_if_eq("WIN", "true"), default_value="")]
	relays: String,

	/// COM port name. Overrides project settings
	#[clap(short, long, value_parser, default_value="", required_if_eq_all(&[("project", "None"), ("WIN", "true")]))]
	interface: String,

	/// Modbus slave ID
	#[clap(short, long, value_parser, required_if_eq_all(&[("project", "None"), ("WIN", "true")]))]
	slave: Option<u8>,
}

const N_RELAYS: usize = 16;
const BAUDRATE: u32 = 9600;
const DELAY_AFTER_OPERATION: u64 = 5; // millis

// Relay commands
const RELAY_CMD_ON: u16  = 0x0100;
const RELAY_CMD_OFF: u16 = 0x0200;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let args = Args::parse();
	println!("{:#?}", args);

	// Set up project
	let mut project: project::Project;
	// If we have project file
	if let Some(proj_path) = args.project {
		// Open project file
		project = project::Project::from_file(&proj_path)?;
		println!("{:#?}", project);
	} else {
		// If not, create project from scratch
		project = project::Project::default();
	}
	
	if !args.relays.is_empty() && check_state_str(&args.relays) {
		project.relays = args.relays;
	}
	if !args.interface.is_empty() {
		project.interface = args.interface;
	}
	if let Some(slave) = args.slave {
		project.slave = slave;
	}
	
	if args.no_window {
		let butstate = state_str_to_bool(&project.relays).unwrap();
		set_relays(&project.interface, project.slave, &butstate).await?;	
	} else {
		let mut a = gui::Gui::new(project);
		a.run().await?;
	}
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

async fn open_connection(com: &str, slave: u8) -> Result<client::Context, Box<dyn Error>> {
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
async fn set_relays(com: &str, slave: u8, state: &[bool]) -> Result<(), Box<dyn Error>> {
	let mut ctx = open_connection(com, slave).await?;

	// Iteration 1: turn off all needed relays
	// Iteration 2: turn on all needed relays
	for &relay_operation in &[false, true] {
		// For all relays
		for (i, &e) in state.iter().enumerate() {
			// If we need to change this relay now
			if e == relay_operation {
				set_one_relay(
					&mut ctx,
					i,
					if relay_operation {RELAY_CMD_ON} else {RELAY_CMD_OFF}
				).await?;
				// Delay after each operation
				time::sleep(Duration::from_millis(DELAY_AFTER_OPERATION)).await;
			}
		}
	}
	
	Ok(())
}

#[inline(always)]
async fn set_one_relay(ctx: &mut client::Context, relay_number: usize, command: u16) -> Result<(), Box<dyn Error>> {
	time::timeout(
		Duration::from_secs(2),
		ctx.write_single_register(relay_number as u16 + 1, command)
	).await??;
	println!("{} - {}", relay_number, command);
	Ok(())
}

async fn get_relays(com: &str, slave: u8) -> Result<Vec<bool>, Box<dyn Error>> {
	let mut ctx = open_connection(com, slave).await?;
	
	let rsp = tokio::time::timeout(
		Duration::from_secs(2),
		ctx.read_holding_registers(0x01, N_RELAYS as u16)
	).await??;
	let state = rsp.iter().map(|&x| x == 1).collect();
	
	Ok(state)
}

fn state_str_to_bool(state: &str) -> Result<Vec<bool>, Box<dyn Error>> {
	if check_state_str(state) {
		Ok(state.chars().map(|c| c == '1').collect())
	} else {
		Err("Failed to parse string representing relays.".into())
	}
}

fn state_bool_to_str(state: &[bool]) -> String {
	state.iter().map(|&x| if x {'1'} else {'0'}).collect()
}

// Returns true if string is valid
fn check_state_str(state: &str) -> bool {
	state.len() == N_RELAYS ||
	state.chars().all(|x| x == '0' || x == '1')
}
