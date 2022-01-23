
use std::io::prelude::*;
use std::time::Duration;
use std::path::{Path, PathBuf};

use fltk::{
	app,
	prelude::*,
	window::Window,
	frame::Frame,
	button::Button,
	menu::Choice,
	enums::{Color, Event, Align},
	input::{Input, IntInput},
	dialog::{FileDialog, FileDialogType},
};

use tokio::time;
use tokio_modbus::prelude::*;
use tokio_serial::{SerialStream, StopBits, DataBits, Parity};


#[derive(Copy, Clone, PartialEq)]
enum Message { Select, Apply, Set, Get, AddPreset, Close, Save }

const BAUDRATE: u32 = 9600;

const OFFSET:   i32 = 10;
const BUTTONW:  i32 = 80;
const BUTTONH:  i32 = 30;
const RELAYW:   i32 = 80;
const RELAYH:   i32 = 50;
const HGAP:     i32 = 80;
const INPUTW:   i32 = BUTTONW * 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut a = Main::new();
	a.run().await?;
	Ok(())
}

#[allow(dead_code)]
struct Main {
	app: app::App,
	wind: Window,
	frame: Frame,
	
	buttons: Vec<Button>,
	presets: Vec<PathBuf>,
	
	input_com:    Input,
	input_slave:  IntInput,
	input_preset: Input,
	menu_preset:  Choice,
	
	button_save:   Button,
	button_select: Button,
	button_apply:  Button,
	button_set:    Button,
	button_get:    Button,

	chan_s: app::Sender<Message>,
	chan_r: app::Receiver<Message>,
}

impl Main {
	fn new() -> Self {
		let app = app::App::default();
		//.with_scheme(app::Scheme::Gtk);

		const WINDOW_W: i32 = OFFSET*2 + RELAYW*2 + HGAP*2 + HGAP/4 + INPUTW;
		const WINDOW_H: i32 = OFFSET*3 + RELAYH*8 + BUTTONH;
		
		let mut wind = Window::default()
			.with_size(
				WINDOW_W,
				WINDOW_H,
			)
			.with_label("R4D3B16 modbus controller");
		
		let frame = Frame::new(
			0,
			0,
			WINDOW_W,
			WINDOW_H,
			""
		);

		let mut buttons = Vec::<Button>::with_capacity(16);

		for i in 0..8 {
			let bn: &str = Box::leak((i+1).to_string().into_boxed_str());
			let mut b = Button::new(
				OFFSET, OFFSET + RELAYH*(i as i32), RELAYW, RELAYH, bn
			);
			b.set_color(Color::Inactive);
			buttons.push(b);
		}
		for i in 0..8 {
			let bn: &str = Box::leak((i+1+8).to_string().into_boxed_str());
			let mut b = Button::new(
				OFFSET + RELAYW + HGAP, OFFSET + RELAYH*(7 - i as i32), RELAYW, RELAYH, bn
			);
			b.set_color(Color::Inactive);
			buttons.push(b);
		};

		let mut button_save = Button::new(
			OFFSET,
			OFFSET*2 + RELAYH*8,
			RELAYW*2 + HGAP,
			BUTTONH,
			"Save preset"
		);

		const SECOND_X: i32 = OFFSET + RELAYW*2 + HGAP*2 + HGAP/4;
		
		let input_com = Input::default()
			.with_pos(
				SECOND_X,
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
				SECOND_X,
				OFFSET + (BUTTONH + OFFSET)
			)
			.with_size(
				INPUTW,
				BUTTONH
			)
			.with_align(Align::Left)
			.with_label("Slave id");
		input_slave.set_value("1");
		
		let mut input_preset = Input::default()
			.with_pos(
				SECOND_X,
				OFFSET + RELAYH*3
			)
			.with_size(
				INPUTW,
				BUTTONH
			)
			.with_align(Align::Left)
			.with_label("New preset");
		let menu_preset = Choice::new(
			SECOND_X,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET),
			INPUTW,
			BUTTONH,
			"Presets"
		);
		let presets = Vec::<PathBuf>::new();

		let mut button_select = Button::new(
			SECOND_X,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"..."
		);
		let mut button_apply = Button::new(
			SECOND_X + BUTTONW,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"Apply"
		);


		let mut button_set = Button::new(
			SECOND_X,
			OFFSET*2 + RELAYH*8,
			BUTTONW,
			BUTTONH,
			"SET"
		);
		let mut button_get = Button::new(
			SECOND_X + BUTTONW,
			OFFSET*2 + RELAYH*8,
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
		
		let (chan_s, chan_r) = app::channel::<Message>();
		input_preset.emit(chan_s, Message::AddPreset);
		button_select.emit(chan_s, Message::Select);
		button_apply.emit(chan_s, Message::Apply);
		button_set.emit(chan_s, Message::Set);
		button_get.emit(chan_s, Message::Get);
		button_save.emit(chan_s, Message::Save);

		wind.set_callback(move |_| {
			if app::event() == Event::Close {
				chan_s.send(Message::Close);
			}
		});

		Main {
			app,
			wind,
			frame,
			
			buttons,
			presets,
			
			input_com,
			input_slave,
			input_preset,
			menu_preset,
			
			button_save,
			button_select,
			button_apply,
			button_set,
			button_get,

			chan_s,
			chan_r,
		}
	}

	async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		//self.app.run().unwrap();
		
		while self.app.wait() {
			let msg = self.chan_r.recv();
			
			match msg {
				Some(Message::Select) => {
					let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
					dialog.show();
					match fix_pathbuf_parts(&dialog.filenames()) {
						Some(preset_filename) => {
							if self.add_preset(&preset_filename.to_str().unwrap()) {
								self.presets.push(preset_filename.into());
								dbg!(&self.presets);
							}
						},
						None => {},
					}
				}
				Some(Message::Apply) => {
					match self.menu_preset.value() {
						-1 => self.button_apply.set_color(Color::Red),
						i if i >= 0 => {
							let filename: &Path = &self.presets[i as usize];
							match self.read_preset(filename) {
								Ok(p) => {
									self.set_buttons(&p);
									self.button_apply.set_color(Color::Background);
									self.app.redraw();
								},
								Err(_) => { self.button_apply.set_color(Color::Red); },
							}
						},
						_ => {},
					};
				},
				Some(Message::Close) => {
					println!("Close window");
					self.app.quit();
				},
				Some(Message::Set) | Some(Message::Get) => {
					let butstate = self.get_buttons();
					
					let mut do_apply = true;

					let com = self.input_com.value();
					if com.is_empty() { do_apply = false; }
					
					let slave = match self.input_slave.value().parse() {
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
								let s = self.set_relays(com.as_str(), slave, &butstate);
								match s.await {
									Err(_) => self.button_set.set_color(Color::Red),
									_      => self.button_set.set_color(Color::Background),
								};
							},
							Message::Get => {
								let g = self.get_relays(com.as_str(), slave);
								match g.await {
									Err(_) => self.button_get.set_color(Color::Red),
									Ok(v)  => {
										self.button_get.set_color(Color::Background);
										self.set_buttons(&v);
										self.app.redraw();
									},
								};
							},
							_ => {},
						}
					} else {
						match msg2 {
							Message::Set => self.button_set.set_color(Color::Red),
							Message::Get => self.button_get.set_color(Color::Red),
							_ => {},
						}
					}
				},
				Some(Message::AddPreset) => {
					let new_preset = self.input_preset.value();
					if !new_preset.is_empty() {
						if self.add_preset(&new_preset) {
							self.presets.push(new_preset.into());
							dbg!(&self.presets);
						}
					}
				},
				Some(Message::Save) => {
					let mut dialog = FileDialog::new(FileDialogType::BrowseSaveFile);
					dialog.show();
					//let preset_parts = dialog.filenames();
					match fix_pathbuf_parts(&dialog.filenames()) {
						Some(preset_filename) => {
							let butstate = self.get_buttons();
							match self.save_preset(&preset_filename, &butstate) {
								Ok(_)  => self.button_save.set_color(Color::Background),
								Err(_) => self.button_save.set_color(Color::Red),
							}

							if self.add_preset(&preset_filename.to_str().unwrap()) {
								self.presets.push(preset_filename.into());
								dbg!(&self.presets);
							}
						},
						None => {},
					}
				}
				None => (),
			}; // End match
		} // End while
		
		Ok(())
	}

	async fn open_connection(&self, com: &str, slave: u8) ->  Result<client::Context, Box<dyn std::error::Error>> {
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
	
	async fn set_relays(&self, com: &str, slave: u8, state: &Vec<bool>) -> Result<(), Box<dyn std::error::Error>> {
		let mut ctx = self.open_connection(com, slave).await?;

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

	async fn get_relays(&self, com: &str, slave: u8) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
		let mut ctx = self.open_connection(com, slave).await?;
		
		let rsp = tokio::time::timeout(Duration::from_secs(2), ctx.read_holding_registers(0x01, 16)).await??;
		let state = rsp.iter().map(|&x| x == 1).collect();
		
		Ok(state)
	}

	fn set_buttons(&mut self, state: &Vec<bool>) {
		for (i, &e) in state.iter().enumerate() {
			self.buttons[i].set_color(if e {Color::Green} else {Color::Inactive});
		}
	}

	fn get_buttons(&mut self) -> Vec<bool> {
		return self.buttons.iter().map(|b| b.color() == Color::Green).collect();
	}

	fn save_preset(&self, filename: &Path, state: &Vec<bool>) -> Result<(), Box<dyn std::error::Error>> {
		let state_str: String = state.iter().map(|&x| if x {'1'} else {'0'}).collect();
		std::fs::write(filename, state_str)?;
		Ok(())
	}

	fn read_preset(&self, filename: &Path) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
		let mut contents = String::new();
		let mut f = std::fs::File::open(filename)?;
		f.read_to_string(&mut contents)?;
		let state = contents.chars().map(|c| c == '1').collect();
		Ok(state)
	}

	fn add_preset(&mut self, value: &str) -> bool {
		let a = match self.menu_preset.choice() {
			Some(_) => 0,
			None    => 1,
		};

		let index = self.menu_preset.find_index(value);
		if index == -1 {
			self.menu_preset.add_choice(value);
			self.menu_preset.set_value(self.menu_preset.size() - 1 - a);
			return true;
		}
		else {
			self.menu_preset.set_value(index);
			return false;
		}
	}
}

fn fix_pathbuf_parts(parts: &Vec<PathBuf>) -> Option<PathBuf> {
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
