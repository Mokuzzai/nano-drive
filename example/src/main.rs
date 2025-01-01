use std::time::Duration;
use std::time::Instant;

use clap::Parser;

use nd_engine::engine::EngineBuilder;
use nd_engine::system::FromFn;
use nd_engine::system::System;
use nd_engine::world::Commands;
use nd_engine::world::RawWorld;

use serde_json as json;

#[derive(Parser)]
struct Config {
	config_json: String,
}

fn hello_system(world: &RawWorld, commands: &mut Commands) {
	println!("Hello world!");
}

fn hello_on_repeate(world: &RawWorld, commands: &mut Commands) {
	// println!("tick tock!")
}

struct Tps {
	ticks: u32,
	start: Instant,
	last_second: Instant,
}

impl Tps {
	fn new() -> Self {
		Self {
			ticks: 0,
			start: Instant::now(),
			last_second: Instant::now(),
		}
	}
}

impl System for Tps {
	fn run(&mut self, world: &RawWorld, commands: &mut Commands) {
		self.ticks += 1;

		if self.last_second.elapsed() > Duration::from_secs(1) {
			self.last_second = Instant::now();

			let ticks_per_second = (self.ticks as f64) / self.start.elapsed().as_secs_f64();

			println!("tps: {}", ticks_per_second);
		}
	}
}

fn main() {
	let config = Config::parse();

	let engine_builder: EngineBuilder = json::from_str(&config.config_json).unwrap();

	println!("{:#?}", engine_builder);

	engine_builder
		.build()
		.add_startup_system(FromFn(hello_system))
		.add_system(FromFn(hello_on_repeate))
		.add_system(Tps::new())
		.run();
}
