use crate::client::ClientEvent;

pub struct World {
	pub app_events: EventWriter<ClientEvent>,
}

impl World {
	pub fn new() -> Self {
		Self {
			app_events: EventWriter::new(),
		}
	}
}

pub struct EventWriter<T> {
	write: Vec<T>,
}

impl<T> EventWriter<T> {
	pub fn new() -> Self {
		Self { write: Vec::new() }
	}
	pub fn push(&mut self, event: T) {
		self.write.push(event)
	}
}

pub struct EventBuffer<T> {
	read: Vec<T>,
}

impl<T> EventBuffer<T> {
	pub fn new() -> Self {
		Self { read: Vec::new() }
	}
	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.read.iter()
	}
}
