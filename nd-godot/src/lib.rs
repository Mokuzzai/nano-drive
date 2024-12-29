use godot::prelude::*;

use nd_engine as nd;

struct NanoDrive;

#[gdextension]
unsafe impl ExtensionLibrary for NanoDrive {}

#[derive(GodotClass)]
#[class(no_init, base=Node2D)]
struct EngineHandle {
	handle: nd::engine::EngineHandle,

	base: Base<Node2D>,
}

#[godot_api]
impl EngineHandle {
	#[func]
	fn create_new(engine_path: GString) -> Gd<Self> {
		Gd::from_init_fn(|base| {
			let engine_path = engine_path.to_string();

			let engine_config = nd::engine::EngineConfig {
				plugins: vec![],
				actions: {
					let mut actions = nd::action::Actions::new();

					let _ = actions.insert("jump".to_string(), nd::action::Kind::Absolute);
					let _ = actions.insert("move".to_string(), nd::action::Kind::AbsoluteAxis);

					actions
				},
			};

			Self {
				handle: nd::engine::EngineHandle::spawn(engine_path.into(), engine_config),
				base,
			}
		})
	}
}

#[godot_api]
impl INode2D for EngineHandle {}
