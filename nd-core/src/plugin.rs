use dlopen2::raw::Library;
use dlopen2::wrapper::WrapperApi;

use std::path::PathBuf;

use crate::system::IntoSystemDescriptor;
use crate::system::SystemDescriptor;
use crate::system::Systems;

pub struct Plugin {
	startup_systems: Systems,
	systems: Systems,
}

impl Plugin {
	pub fn new() -> Self {
		Self {
			systems: Systems::new(),
			startup_systems: Systems::new(),
		}
	}
	pub fn add_system(&mut self, system: impl IntoSystemDescriptor) -> &mut Self {
		self.systems.add_system(system);
		self
	}
	pub fn add_startup_system(&mut self, system: impl IntoSystemDescriptor) -> &mut Self {
		self.startup_systems.add_system(system);
		self
	}
}

struct PluginEntry {
	path: PathBuf,
	lib: Library,
	plugin: Plugin,
}

#[derive(WrapperApi)]
struct PluginLib<'a> {
	build: fn(plugin: &'a mut Plugin),
}

pub struct Plugins {
	plugins: Vec<PluginEntry>,
}

impl Plugins {
	pub fn new() -> Self {
		Self {
			plugins: Vec::new(),
		}
	}
	pub fn load(&mut self, path: PathBuf) -> usize {
		let lib = Library::open(&path).unwrap();

		let wrapper = unsafe { PluginLib::load(&lib).unwrap() };

		let mut plugin = Plugin::new();

		wrapper.build(&mut plugin);

		drop(wrapper);

		let entry = PluginEntry { path, lib, plugin };

		let index = self.plugins.len();

		self.plugins.push(entry);

		index
	}
}
