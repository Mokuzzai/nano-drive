use clap::Parser;

use nd_engine::engine::EngineBuilder;

use serde_json as json;

#[derive(Parser)]
struct Config {
	config_json: String,
}

fn main() {
	let config = Config::parse();

	let engine_builder: EngineBuilder = json::from_str(&config.config_json).unwrap();

	println!("{:#?}", engine_builder);

	let engine = engine_builder.build();

	engine.run();
}
