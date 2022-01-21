
use std::io::prelude::*;
use std::time::Duration;

use fltk::{
	app,
	prelude::*,
	window::Window,
	frame::Frame,
	button::Button,
	menu::Choice,
	enums::{Color, Event, Align},
	input::{Input, IntInput, FileInput},
	dialog::{FileDialog, FileDialogType},
};

use tokio::time;
use tokio_modbus::prelude::*;
use tokio_serial::{SerialStream, StopBits, DataBits, Parity};


#[derive(Copy, Clone, PartialEq)]
enum Message { Select, Apply, Set, Get, Preset, Close, Save }

const BAUDRATE: u32 = 9600;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	const OFFSET:  i32 = 10;
	const BUTTONW: i32 = 80;
	const BUTTONH: i32 = 40;
	const HGAP:    i32 = 80;
	const INPUTW:  i32 = BUTTONW * 2;

	let app = app::App::default();
	//.with_scheme(app::Scheme::Gtk);
	let mut wind = Window::default()
		.with_size(
			OFFSET*2 + BUTTONW*2 + HGAP*2 + HGAP/4 + INPUTW,
			OFFSET*3 + BUTTONH*9
		)
		.with_label("R4D3B16 modbus controller");
	
	let _frame = Frame::new(
		0,
		0,
		OFFSET*2 + BUTTONW*2 + HGAP*2 + HGAP/4 + 150,
		OFFSET*2 + BUTTONH*8,
		""
	);

	let mut buttons = Vec::<Button>::with_capacity(16);

	for i in 0..8 {
		let bn: &str = Box::leak((i+1).to_string().into_boxed_str());
		let mut b = Button::new(
			OFFSET, OFFSET + BUTTONH*(i as i32), BUTTONW, BUTTONH, bn
		);
		b.set_color(Color::Inactive);
		buttons.push(b);
	}
	for i in 0..8 {
		let bn: &str = Box::leak((i+1+8).to_string().into_boxed_str());
		let mut b = Button::new(
			OFFSET + BUTTONW + HGAP, OFFSET + BUTTONH*(7 - i as i32), BUTTONW, BUTTONH, bn
		);
		b.set_color(Color::Inactive);
		buttons.push(b);
	};

	let mut button_save = Button::new(
		OFFSET,
		OFFSET*2 + BUTTONH*8,
		BUTTONW*2 + HGAP,
		BUTTONH,
		"Save preset"
	);

	let input_com = Input::default()
		.with_pos(
			OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
			OFFSET
		)
		.with_size(
			INPUTW,
			BUTTONH
		)
		.with_align(Align::Left)
		.with_label("Serial port");
	let mut input_slave = IntInput::default()
		.with_pos(
			OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
			OFFSET + (BUTTONH + OFFSET)
		)
		.with_size(
			INPUTW,
			BUTTONH
		)
		.with_align(Align::Left)
		.with_label("Slave id");
	input_slave.set_value("1");
	
	let mut input_preset = FileInput::default()
		.with_pos(
			OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
			OFFSET + (BUTTONH + OFFSET)*2 + OFFSET*2
		)
		.with_size(
			INPUTW,
			BUTTONH
		)
		.with_align(Align::Left)
		.with_label("New preset");
	let mut menu_preset = Choice::new(
		OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
		OFFSET + (BUTTONH + OFFSET)*3 + OFFSET*2,
		INPUTW,
		BUTTONH,
		"Presets"
	);

	let mut button_select = Button::new(
		OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
		OFFSET + (BUTTONH + OFFSET)*4 + OFFSET*2,
		BUTTONW,
		BUTTONH,
		"..."
	);
	let mut button_apply = Button::new(
		OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4 + BUTTONW,
		OFFSET + (BUTTONH + OFFSET)*4 + OFFSET*2,
		BUTTONW,
		BUTTONH,
		"Apply"
	);


	let mut button_set = Button::new(
		OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4,
		OFFSET*2 + BUTTONH*8,
		BUTTONW,
		BUTTONH,
		"SET"
	);
	let mut button_get = Button::new(
		OFFSET + BUTTONW*2 + HGAP*2 + HGAP/4 + BUTTONW,
		OFFSET*2 + BUTTONH*8,
		BUTTONW,
		BUTTONH,
		"GET"
	);
	
	wind.end();
	wind.show();

	// Set callbacks
	for i in 0..16 {
		buttons[i].set_callback(|b| {
			if b.color() == Color::Inactive { b.set_color(Color::Green); } else { b.set_color(Color::Inactive); }
		});
	}
	
	let (s, r) = app::channel::<Message>();
	input_preset.emit(s, Message::Preset);
	button_select.emit(s, Message::Select);
	button_apply.emit(s, Message::Apply);
	button_set.emit(s, Message::Set);
	button_get.emit(s, Message::Get);
	button_save.emit(s, Message::Save);

	wind.set_callback(move |_| {
        if app::event() == Event::Close {
            s.send(Message::Close);
        }
    });

	while app.wait() {
		let msg = r.recv();
        match msg {
			Some(Message::Select) => {
				let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
				dialog.show();
				let preset_parts = dialog.filenames();
				if !preset_parts.is_empty() {
					let mut preset_filename = String::new();
					for p in preset_parts {
						preset_filename.push_str(p.to_str().unwrap());
						preset_filename.push_str(" ");
					}
					preset_filename.pop();
					menu_preset.add_choice(&preset_filename);
					let menu_item = menu_preset.find_item(&preset_filename).unwrap();
					menu_preset.set_item(&menu_item);
				}
			}
            Some(Message::Apply) => {
				let filename = menu_preset.choice().ok_or("Choice select error");
				let mut file = std::fs::File::open(filename.unwrap())?;
				let mut contents = String::new();
				file.read_to_string(&mut contents)?;
				for (i, c) in contents.chars().enumerate() {
					buttons[i].set_color(if c == '1' {Color::Green} else {Color::Inactive});
				}
				app.redraw();
			},
			Some(Message::Close) => {
				println!("Close window");
				app.quit();
			},
            Some(Message::Set) | Some(Message::Get) => {
				let butstate: Vec::<bool> = buttons.iter().map(|b| b.color() == Color::Green).collect();
				
				let mut do_apply = true;

				let com = input_com.value();
				if com.is_empty() {
					do_apply = false;
				}
				
				let slave = match input_slave.value().parse() {
					Ok(v) => v,
					Err(_) => {
						do_apply = false;
						0
					},
				};
				
				let msg2 = msg.unwrap();
				if do_apply {
					match msg2 {
						Message::Set => {
							let s = set(com.as_str(), slave, &butstate);
							match s.await {
								Err(_) => button_set.set_color(Color::Red),
								_      => button_set.set_color(Color::Background),
							};
						},
						Message::Get => {
							let g = get(com.as_str(), slave);
							match g.await {
								Err(_) => button_get.set_color(Color::Red),
								Ok(v)  => {
									button_get.set_color(Color::Background);
									set_buttons(&mut buttons, &v);
									app.redraw();
								},
							};
						},
						_ => {},
					}
				} else {
					match msg2 {
						Message::Set => button_set.set_color(Color::Red),
						Message::Get => button_get.set_color(Color::Red),
						_ => {},
					}
				}
			},
			Some(Message::Preset) => {
				let new_preset = input_preset.value();
				if !new_preset.is_empty() {
					menu_preset.add_choice(&new_preset);
					let menu_item = menu_preset.find_item(&new_preset).unwrap();
					menu_preset.set_item(&menu_item);
				}
			},
			Some(Message::Save) => {
				let mut dialog = FileDialog::new(FileDialogType::BrowseSaveFile);
				dialog.show();
				let preset_parts = dialog.filenames();
				if !preset_parts.is_empty() {
					let mut preset_filename = String::new();
					for p in preset_parts {
						preset_filename.push_str(p.to_str().unwrap());
						preset_filename.push_str(" ");
					}
					preset_filename.pop();

					let butstate: Vec::<bool> = buttons.iter().map(|b| b.color() == Color::Green).collect();
					let butstate_str: String = butstate.iter().map(|&x| if x {'1'} else {'0'}).collect();
					match std::fs::write(&preset_filename, butstate_str) {
						Ok(_)  => button_save.set_color(Color::Background),
						Err(_) => button_save.set_color(Color::Red),
					}
					
					menu_preset.add_choice(&preset_filename);
					let menu_item = menu_preset.find_item(preset_filename.as_str()).unwrap();
					menu_preset.set_item(&menu_item);
				}
			}
            None => (),
        }
    }
	
	app.run()?;
		
	Ok(())
}

async fn set(com: &str, slave: u8, state: &Vec<bool>) -> Result<(), Box<dyn std::error::Error>> {
	let s = Slave(slave);
    let builder = tokio_serial::new(com, BAUDRATE)
		.data_bits(DataBits::Eight)
		.parity(Parity::None)
		.stop_bits(StopBits::One);
	let port = SerialStream::open(&builder).unwrap();

	let mut ctx = rtu::connect_slave(port, s).await?;

	// Off
	for (i, &e) in state.iter().enumerate() {
		if !e {
			time::timeout(
				Duration::from_secs(2),
				ctx.write_single_register((i as u16)+1, 0x0200)
			).await??;
			time::sleep(Duration::from_millis(5)).await;
		}
	}
	// On
	for (i, &e) in state.iter().enumerate() {
		if e {
			time::timeout(
				Duration::from_secs(2),
				ctx.write_single_register((i as u16)+1, 0x0100)
			).await??;
			time::sleep(Duration::from_millis(5)).await;
		}
	}
	
	Ok(())
}

async fn get(com: &str, slave: u8) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    let s = Slave(slave);
    let builder = tokio_serial::new(com, BAUDRATE)
		.data_bits(DataBits::Eight)
		.parity(Parity::None)
		.stop_bits(StopBits::One);
    let port = SerialStream::open(&builder).unwrap();

	let mut ctx = rtu::connect_slave(port, s).await?;
	
	let rsp = tokio::time::timeout(Duration::from_secs(2), ctx.read_holding_registers(0x01, 16)).await??;
	
	let state = rsp.iter().map(|&x| x == 1).collect();
	Ok(state)
}

fn set_buttons(buttons: &mut Vec::<Button>, state: &Vec<bool>) {
	for (i, &e) in state.iter().enumerate() {
		buttons[i].set_color(if e {Color::Green} else {Color::Inactive});
	}
}

/*fn save_preset(filename: &str, butstate: &Vec<bool>) -> Result<(), Box<dyn std::error::Error>>  {
	let butstate_str: String = butstate.iter().map(|x| if *x {'1'} else {'0'}).collect();
	std::fs::write(filename, butstate_str)?;
	Ok(())
}*/
