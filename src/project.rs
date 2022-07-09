
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use serde::{Serialize, Deserialize};

use super::*; // variables and functions from main.rs

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name:           String,
    pub relays:         String,
    pub interface:      String,
	pub slave:          u8,
	pub preset_names:   Vec<String>,
	pub preset_values:  Vec<String>,
	pub current_preset: i32,
	pub realtime:       bool,
}

impl Project {
	pub fn from_file(path: &Path) -> std::result::Result<Project, Box<dyn Error>> {
		let file = File::open(path)?;
		let reader = BufReader::new(file);
		let project: Project = serde_json::from_reader(reader)?;
		Ok(project)
	}
}

impl Default for Project {
	fn default() -> Self {
		Project {
			name: String::new(),
			relays: String::from_iter(['0'; N_RELAYS]),
			interface: String::new(),
			slave: 1,
			preset_names: Vec::<String>::new(),
			preset_values: Vec::<String>::new(),
			current_preset: -1,
			realtime: false,
		}
	}
}

/*impl Project {
	pub fn new(path: PathBuf) -> Self {
		Project {
		}
	}
}*/
