
use std::path::PathBuf;
use std::collections::VecDeque;

use fltk::{
	app,
	prelude::*,
	window::Window,
	frame::Frame,
	button::{Button, CheckButton},
	menu::Choice,
	enums::{Color, Event, Align, Key},
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

	project: project::Project,
	buttons: Vec<Button>,
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
	pub fn new(project: project::Project) -> Self {
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
			let b = Button::new(
				OFFSET + RELAYW + HGAP, OFFSET + RELAYH*(i as i32), RELAYW, RELAYH, bn
			);
			buttons.push(b);
		}
		for i in 0..NR2 {
			let bn: &str = Box::leak((i+1+NR2).to_string().into_boxed_str());
			let b = Button::new(
				OFFSET, OFFSET + RELAYH*(NR2 - 1 - i as i32), RELAYW, RELAYH, bn
			);
			buttons.push(b);
		};

		// Set relay states from project file
		if let Ok(relay_bools) = state_str_to_bool(&project.relays) {
			for (b, &r) in buttons.iter_mut().zip(relay_bools.iter()) {
				b.set_color(if r {COLOR_ON} else {COLOR_OFF});
			}
		}
		
		let mut button_relay_on = Button::new(
			OFFSET,
			OFFSET + RELAYH*NR2 + OFFSET,
			BUTTONW,
			BUTTONH,
			"ON all"
		);
		let mut button_relay_off = Button::new(
			OFFSET + RELAYW + HGAP,
			OFFSET + RELAYH*NR2 + OFFSET,
			BUTTONW,
			BUTTONH,
			"OFF all"
		);

		const SECOND_X: i32 = OFFSET + RELAYW*2 + HGAP*2 + HGAP/4;

		// Project manipulation
		let mut input_preset = Input::default()
			.with_pos(
				SECOND_X,
				OFFSET
			)
			.with_size(
				INPUTW,
				BUTTONH
			)
			.with_align(Align::Left)
			.with_label("Add preset");
		let mut menu_preset = Choice::new(
			SECOND_X,
			OFFSET + (BUTTONH + OFFSET),
			INPUTW,
			BUTTONH,
			"Presets"
		);
		if !project.preset_names.is_empty() {
			for preset in &project.preset_names {
				menu_preset.add_choice(&preset);
			}
			menu_preset.set_value(project.current_preset);
		}

		let mut button_apply = Button::new(
			SECOND_X,
			OFFSET + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"Apply"
		);

		let mut button_remove = Button::new(
			SECOND_X,
			OFFSET + (BUTTONH + OFFSET)*3,
			BUTTONW,
			BUTTONH,
			"Remove preset"
		);
		
		let mut button_save = Button::new(
			SECOND_X + BUTTONW,
			OFFSET + (BUTTONH + OFFSET)*2,
			BUTTONW,
			BUTTONH,
			"Save project @save"
		);

		let mut button_select = Button::new(
			SECOND_X + BUTTONW,
			OFFSET + (BUTTONH + OFFSET)*3,
			BUTTONW,
			BUTTONH,
			"Open project @fileopen"
		);



		

		
		let mut menu_com = Choice::new(
			SECOND_X,
			OFFSET + (BUTTONH + OFFSET)*4,
			INPUTW - BUTTONH,
			BUTTONH,
			"Serial port"
		);
		let mut button_refresh_com = Button::new(
			SECOND_X + INPUTW - BUTTONH,
			OFFSET + (BUTTONH + OFFSET)*4,
			BUTTONH,
			BUTTONH,
			"@refresh"
		);
		let mut input_slave = IntInput::default()
			.with_pos(
				SECOND_X,
				OFFSET + (BUTTONH + OFFSET)*5
			)
			.with_size(
				INPUTW,
				BUTTONH
			)
			.with_align(Align::Left)
			.with_label("Slave id");
		input_slave.set_value(&project.slave.to_string());
		
		
		

		let mut cb_realtime = CheckButton::new(
			SECOND_X,
			OFFSET*2 + RELAYH*NR2 - BUTTONH,
			BUTTONW,
			BUTTONH,
			"Realtime"
		);
		cb_realtime.set_checked(project.realtime);
		
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

		// Read available com ports
		let mut ports = refresh_com(&mut menu_com);
		// Set com port saved in project file
		if !project.interface.is_empty() {
			if ports.contains(&project.interface) {
				// If saved port is in list, select it
				let index = ports
					.iter()
					.position(|x| x.as_str() == project.interface)
					.unwrap();
				menu_com.set_value(index as i32);
			} else {
				// If no port in list, add new one to ports list
				ports.push(project.interface.clone());
				menu_com.add_choice(&project.interface);
				menu_com.set_value(ports.len() as i32 - 1);
			}
		}
		
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
		//button_select.emit(chan_s, Message::SelectPreset);
		button_apply.emit(chan_s, Message::ApplyPreset);
		button_remove.emit(chan_s, Message::RemovePreset);
		button_set.emit(chan_s, Message::Set);
		button_get.emit(chan_s, Message::Get);
		//button_save.emit(chan_s, Message::SavePreset);
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

			project,
			buttons,
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

	pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
		//self.app.run().unwrap();
		
		while self.app.wait() {
			let msg = self.chan_r.recv();
			
			match msg {
				Some(Message::SelectPreset) => {
					// TODO
					let mut dialog = FileDialog::new(FileDialogType::BrowseFile);
					dialog.show();
					/*match fix_pathbuf_parts(&dialog.filenames()) {
						Some(preset_filename) => {
							self.add_preset(&preset_filename);
							self.apply_preset();
						},
						None => {},
					}*/
				}
				Some(Message::AddPreset) => {
					if app::event_key_down(Key::Enter) {
						let new_preset = self.input_preset.value();
						if !new_preset.is_empty() {
							let butstate = self.get_buttons();
							self.add_preset(&new_preset, &butstate);
							self.input_preset.set_value("");
						}
 					}
				},
				Some(Message::ApplyPreset) => {
					self.apply_preset();
				},
				Some(Message::Close) => {
					println!("Close window");
					// TODO save project
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
						// TODO
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
					let result = Some("presetname"); // TODO
					match result {
						Some(preset_name) => {
							let butstate = self.get_buttons();
							self.add_preset(&preset_name, &butstate);
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

	fn add_preset(&mut self, name: &str, value: &[bool]) -> bool {
		let index = self.project.preset_names.iter()
			.position(|e| e == name)
			.unwrap_or(usize::MAX);
		
		if index == usize::MAX {
			self.menu_preset.add_choice(&name);
			self.project.preset_names.push(name.into());
			self.project.preset_values.push(state_bool_to_str(value));
			self.menu_preset.set_value(self.project.preset_names.len() as i32 - 1);
			dbg!(&self.project.preset_names);
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
				self.project.preset_names.remove(i as usize);
				self.project.preset_values.remove(i as usize);
				self.menu_preset.remove(i);
				self.menu_preset.redraw();
				dbg!(&self.project.preset_names);
			},
			_ => {},
		};
	}
	
	fn apply_preset(&mut self) {
		let index = self.menu_preset.value();
		dbg!(&index);
		
		if index >= 0 {
			let p = &self.project.preset_values[index as usize];
			if let Ok(butstate) = state_str_to_bool(p) {
				self.set_buttons(&butstate);
				self.realtime_check_and_set();
			} else {
				self.button_apply.set_color(COLOR_ERROR);
			}
		} else {
			self.button_apply.set_color(COLOR_ERROR);
		}
		self.app.redraw();
	}

	fn realtime_check_and_set(&mut self) {
		// If realtime option is selected
		if self.cb_realtime.is_checked() {
			// Set relays immediately
			// (imitate pressing set button)
			self.chan_s.send(Message::Set);
		}
	}
} // End impl

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
