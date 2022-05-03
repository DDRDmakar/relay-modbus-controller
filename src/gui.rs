
use std::path::PathBuf;
use std::collections::VecDeque;

use fltk::{
	app,
	prelude::*,
	window::Window,
	frame::Frame,
	button::{Button, CheckButton},
	menu::Choice,
	enums::{Color, Event, Align},
	input::{Input, IntInput},
	dialog::{FileDialog, FileDialogType},
};

use super::*; // variables and functions from main.rs


const TITLE: &str = "Relay modbus controller";

// Window parameters
const OFFSET:   i32 = 10;
const BUTTONW:  i32 = 120;
const BUTTONH:  i32 = 30;
const RELAYW:   i32 = 80;
const RELAYH:   i32 = 50;
const HGAP:     i32 = 80;
const INPUTW:   i32 = 80 * 3;

// Window colors
const COLOR_NORMAL: Color = Color::Background;
const COLOR_ERROR:  Color = Color::Red;
const COLOR_ON:     Color = Color::Green;
const COLOR_OFF:    Color = Color::Inactive;

#[derive(Copy, Clone, PartialEq)]
enum Message {
	RefreshCom,
	AddPreset,
	SelectPreset,
	SavePreset,
	ApplyPreset,
	RemovePreset,
	Set,
	SetRelay(usize, bool),
	Get,
	Close,
	AllRelayOn,
	AllRelayOff,
	RealtimeToggle,
}

#[allow(dead_code)]
pub struct Gui {
	app: app::App,
	wind: Window,
	frame: Frame,
	
	buttons: Vec<Button>,
	presets: VecDeque<PathBuf>,
	ports:   Vec<String>,
	
	menu_com:     Choice,
	menu_preset:  Choice,
	input_slave:  IntInput,
	input_preset: Input,
	cb_realtime:  CheckButton,
	
	button_save:   Button,
	button_apply:  Button,
	button_set:    Button,
	button_get:    Button,

	chan_s: app::Sender<Message>,
	chan_r: app::Receiver<Message>,
}

impl Gui {
	pub fn new() -> Self {
		let app = app::App::default();
		//.with_scheme(app::Scheme::Gtk);

		const NR2:      i32 = (N_RELAYS as i32) / 2;
		const WINDOW_W: i32 = OFFSET*2 + RELAYW*2 + HGAP*2 + HGAP/4 + INPUTW;
		const WINDOW_H: i32 = OFFSET*3 + RELAYH*NR2 + BUTTONH;
		
		let mut wind = Window::default()
			.with_size(
				WINDOW_W,
				WINDOW_H,
			)
			.with_label(TITLE);
		
		let frame = Frame::new(
			0,
			0,
			WINDOW_W,
			WINDOW_H,
			""
		);

		let mut buttons = Vec::<Button>::with_capacity(N_RELAYS);

		for i in 0..NR2 {
			let bn: &str = Box::leak((i+1).to_string().into_boxed_str());
			let mut b = Button::new(
				OFFSET + RELAYW + HGAP, OFFSET + RELAYH*(i as i32), RELAYW, RELAYH, bn
			);
			b.set_color(COLOR_OFF);
			buttons.push(b);
		}
		for i in 0..NR2 {
			let bn: &str = Box::leak((i+1+NR2).to_string().into_boxed_str());
			let mut b = Button::new(
				OFFSET, OFFSET + RELAYH*(NR2 - 1 - i as i32), RELAYW, RELAYH, bn
			);
			b.set_color(COLOR_OFF);
			buttons.push(b);
		};
		
		let mut button_save = Button::new(
			OFFSET,
			OFFSET*2 + RELAYH*NR2,
			RELAYW*2 + HGAP,
			BUTTONH,
			"Save preset"
		);

		const SECOND_X: i32 = OFFSET + RELAYW*2 + HGAP*2 + HGAP/4;
		
		let mut menu_com = Choice::new(
			SECOND_X,
			OFFSET,
			INPUTW - BUTTONH,
			BUTTONH,
			"Serial port"
		);
		let mut button_refresh_com = Button::new(
			SECOND_X + INPUTW - BUTTONH,
			OFFSET,
			BUTTONH,
			BUTTONH,
			"@refresh"
		);
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

		let mut button_relay_on = Button::new(
			SECOND_X,
			OFFSET + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"ON all"
		);
		let mut button_relay_off = Button::new(
			SECOND_X + BUTTONW,
			OFFSET + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"OFF all"
		);
		
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
		let mut menu_preset = Choice::new(
			SECOND_X,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET),
			INPUTW - BUTTONH,
			BUTTONH,
			"Presets"
		);
		let presets = VecDeque::<PathBuf>::new();

		let mut button_select = Button::new(
			SECOND_X + INPUTW - BUTTONH,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET),
			BUTTONH,
			BUTTONH,
			"@fileopen"
		);
		let mut button_apply = Button::new(
			SECOND_X,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"Apply"
		);
		let mut button_remove = Button::new(
			SECOND_X,
			OFFSET + RELAYH*3 + (BUTTONH + OFFSET)*3,
			BUTTONW,
			BUTTONH,
			"Remove"
		);

		let mut cb_realtime = CheckButton::new(
			SECOND_X,
			OFFSET*2 + RELAYH*NR2 - BUTTONH,
			BUTTONW,
			BUTTONH,
			"Realtime"
		);
		
		let mut button_set = Button::new(
			SECOND_X,
			OFFSET*2 + RELAYH*NR2,
			BUTTONW,
			BUTTONH,
			"SET"
		);
		let mut button_get = Button::new(
			SECOND_X + BUTTONW,
			OFFSET*2 + RELAYH*NR2,
			BUTTONW,
			BUTTONH,
			"GET"
		);

		let ports = refresh_com(&mut menu_com);
		
		wind.end();
		wind.show();

		// Open channel to send signals from GUI
		let (chan_s, chan_r) = app::channel::<Message>();
		
		// Set callbacks
		for (i, e) in buttons.iter_mut().enumerate() {
			let newchan = chan_s.clone(); // Clone channel to receive messages from each button
			e.set_callback(move |b| {
				let is_on = b.color() == COLOR_ON;
				b.set_color(if is_on {COLOR_OFF} else {COLOR_ON});
				b.redraw();
				newchan.send(Message::SetRelay(i, !is_on));
			});
		}
		
		button_refresh_com.emit(chan_s, Message::RefreshCom);
		input_preset.emit(chan_s, Message::AddPreset);
		button_select.emit(chan_s, Message::SelectPreset);
		button_apply.emit(chan_s, Message::ApplyPreset);
		button_remove.emit(chan_s, Message::RemovePreset);
		button_set.emit(chan_s, Message::Set);
		button_get.emit(chan_s, Message::Get);
		button_save.emit(chan_s, Message::SavePreset);
		menu_preset.emit(chan_s, Message::ApplyPreset);
		button_relay_on.emit(chan_s, Message::AllRelayOn);
		button_relay_off.emit(chan_s, Message::AllRelayOff);
		cb_realtime.emit(chan_s, Message::RealtimeToggle);

		wind.set_callback(move |_| {
			if app::event() == Event::Close {
				chan_s.send(Message::Close);
			}
		});

		Self {
			app,
			wind,
			frame,
			
			buttons,
			presets,
			ports,
			
			menu_com,
			menu_preset,
			input_slave,
			input_preset,
			cb_realtime,
			
			button_save,
			button_apply,
			button_set,
			button_get,

			chan_s,
			chan_r,
		}
	}

	pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		//self.app.run().unwrap();
		
		while self.app.wait() {
			let msg = self.chan_r.recv();
			
			match msg {
				Some(Message::SelectPreset) => {
					let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
					dialog.show();
					match fix_pathbuf_parts(&dialog.filenames()) {
						Some(preset_filename) => {
							self.add_preset(&preset_filename);
							self.apply_preset();
						},
						None => {},
					}
				}
				Some(Message::AddPreset) => {
					let new_preset = self.input_preset.value();
					if !new_preset.is_empty() {
						self.add_preset(&Path::new(&new_preset));
						self.apply_preset();
						self.input_preset.set_value("");
					}
				},
				Some(Message::ApplyPreset) => {
					self.apply_preset();
				},
				Some(Message::Close) => {
					println!("Close window");
					self.app.quit();
				},
				Some(Message::Set) |
				Some(Message::Get) |
				Some(Message::SetRelay(_,_)) => {
					let butstate = self.get_buttons();
					
					let mut do_apply = true;

					let com: &str;
					let value_in_preset_input = self.input_preset.value();
					if value_in_preset_input.is_empty() {
						// Normal behavior
						let com_index = self.menu_com.value();
						com = if com_index >= 0 { &self.ports[com_index as usize] } else { "" };
						if com.is_empty() { do_apply = false; }
					} else {
						// Behavior if input_preset is not empty
						// This tweak is needed for situation if com port is not listed
						com = &value_in_preset_input;
					}
					
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
								let s = set_relays(&com, slave, &butstate);
								match s.await {
									Err(_) => self.button_set.set_color(COLOR_ERROR),
									Ok(_)  => self.button_set.set_color(COLOR_NORMAL),
								};
							},
							Message::Get => {
								let g = get_relays(&com, slave);
								match g.await {
									Err(_) => self.button_get.set_color(COLOR_ERROR),
									Ok(v)  => {
										self.button_get.set_color(COLOR_NORMAL);
										self.set_buttons(&v);
									},
								};
							},
							Message::SetRelay(i, relay_operation) => {
								if self.cb_realtime.is_checked() {
									let mut ctx = open_connection(&com, slave).await?;
									let s = set_one_relay(
										&mut ctx,
										i,
										if relay_operation {RELAY_CMD_ON} else {RELAY_CMD_OFF}
									);
									match s.await {
										Err(_) => self.button_set.set_color(COLOR_ERROR),
										Ok(_)  => self.button_set.set_color(COLOR_NORMAL),
									}
								}
							},
							_ => {},
						}
					} else {
						match msg2 {
							Message::Set           => self.button_set.set_color(COLOR_ERROR),
							Message::Get           => self.button_get.set_color(COLOR_ERROR),
							Message::SetRelay(_,_) => {
								if self.cb_realtime.is_checked() {
									self.button_set.set_color(COLOR_ERROR);
								}
							},
							_ => {},
						}
					}
					self.app.redraw();
				},
				Some(Message::SavePreset) => {
					let mut dialog = FileDialog::new(FileDialogType::BrowseSaveFile);
					dialog.show();
					match fix_pathbuf_parts(&dialog.filenames()) {
						Some(preset_filename) => {
							let butstate = self.get_buttons();
							match self.save_preset(&preset_filename, &butstate) {
								Ok(_)  => self.button_save.set_color(COLOR_NORMAL),
								Err(_) => self.button_save.set_color(COLOR_ERROR),
							}
							self.button_save.redraw();
							self.add_preset(&preset_filename);
						},
						None => {},
					}
				}
				Some(Message::RemovePreset) => {
					self.remove_preset();
				},
				Some(Message::RefreshCom) => {
					self.ports = refresh_com(&mut self.menu_com);
				},
				Some(Message::AllRelayOn) | Some(Message::AllRelayOff) => {
					let color = match msg {
						Some(Message::AllRelayOn)  => COLOR_ON,
						Some(Message::AllRelayOff) => COLOR_OFF,
						_ => COLOR_OFF,
					};
					for b in self.buttons.iter_mut() {
						b.set_color(color);
					}
					self.app.redraw();
					self.realtime_check_and_set();
				},
				Some(Message::RealtimeToggle) => {
					self.realtime_check_and_set();
				}
				None => (),
			}; // End match
		} // End while
		
		Ok(())
	}

	fn set_buttons(&mut self, state: &[bool]) {
		for (b, &s) in self.buttons.iter_mut().zip(state.iter()) {
			b.set_color(if s {COLOR_ON} else {COLOR_OFF});
		}
	}

	fn get_buttons(&mut self) -> Vec<bool> {
		return self.buttons.iter().map(|b| b.color() == COLOR_ON).collect();
	}

	fn save_preset(&self, filename: &Path, state: &[bool]) -> Result<(), Box<dyn std::error::Error>> {
		let state_str: String = state_bool_to_str(state);
		std::fs::write(filename, state_str)?;
		Ok(())
	}

	fn read_preset(&self, filename: &Path) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
		let mut contents = String::new();
		let mut f = std::fs::File::open(filename)?;
		f.read_to_string(&mut contents)?;

		let state = state_str_to_bool(&contents)?;
		Ok(state)
	}

	fn add_preset(&mut self, filename: &Path) -> bool {
		let filename_str = filename.to_str().unwrap();
		let value: String = filename_str.replace("\\", "\\\\");
		let index = self.presets.iter()
			.position(|e| e == filename)
			.unwrap_or(usize::MAX);
		
		if index == usize::MAX {
			self.menu_preset.add_choice(&value);
			self.presets.push_back(filename.into());
			self.menu_preset.set_value(self.presets.len() as i32 - 1);
			dbg!(&self.presets);
			return true;
		}
		else {
			self.menu_preset.set_value(index as i32);
			return false;
		}
	}

	fn remove_preset(&mut self) {
		match self.menu_preset.value() {
			i if i >= 0 => {
				self.presets.remove(i as usize);
				self.menu_preset.remove(i);
				self.menu_preset.redraw();
				dbg!(&self.presets);
			},
			_ => {},
		};
	}
	
	fn apply_preset(&mut self) {
		let index = self.menu_preset.value();
		dbg!(&index);
		
		match index {
			i if i >= 0 => {
				let filename: &Path = &self.presets[i as usize];
				match self.read_preset(filename) {
					Ok(p) => {
						self.set_buttons(&p);
						self.button_apply.set_color(COLOR_NORMAL);
						self.app.redraw();
						self.realtime_check_and_set();
					},
					Err(_) => {
						self.button_apply.set_color(COLOR_ERROR);
						self.button_apply.redraw();
					},
				}
			},
			_ => {
				self.button_apply.set_color(COLOR_ERROR);
				self.button_apply.redraw();
			},
		};
	}

	fn realtime_check_and_set(&mut self) {
		if self.cb_realtime.is_checked() {
			self.chan_s.send(Message::Set);
		}
	}
}

fn refresh_com(menu: &mut Choice) -> Vec<String> {
	let index = menu.value(); // -1 if no item selected
	let ports: Vec<String> = available_ports().unwrap().iter().map(|p| p.port_name.clone()).collect();

	menu.clear();
	for port in &ports {
		menu.add_choice(&port);
	}

	if !ports.is_empty() && index != -1 {
		menu.set_value(index);
	}

	ports
}
