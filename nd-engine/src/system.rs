use crate::world::Commands;
use crate::world::RawWorld;

use std::any::Any;

pub trait System: Any + Send + Sync {
	fn run(&mut self, world: &RawWorld, commands: &mut Commands);
}

pub struct FromFn<F>(pub F);

impl<F: Any + Send + Sync + FnMut(&RawWorld, &mut Commands)> System for FromFn<F> {
	fn run(&mut self, world: &RawWorld, commands: &mut Commands) {
		let Self(callback) = self;

		callback(world, commands)
	}
}

pub struct Systems {
	systems: Vec<Box<dyn System>>,
}

impl Systems {
	pub fn new() -> Self {
		Self {
			systems: Vec::new(),
		}
	}
	pub fn add(&mut self, system: impl System) {
		self.systems.push(Box::new(system))
	}
	pub fn iter_mut(&mut self) -> impl Send + Sync + Iterator<Item = &mut dyn System> {
		self.systems.iter_mut().map(|x| &mut **x)
	}
}
