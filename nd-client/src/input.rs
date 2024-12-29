use fxhash::FxHashMap;
use glium::winit::event::ButtonId;
use glium::winit::event::DeviceEvent;
use glium::winit::event::ElementState;
use glium::winit::event::MouseScrollDelta;
use glium::winit::event::RawKeyEvent;
use glium::winit::keyboard::KeyCode;
use glium::winit::keyboard::PhysicalKey;

use std::collections::hash_map::Entry;
use std::time::Instant;

struct Digital;
struct Motion {
	x: f64,
}
struct Motion2 {
	x: f64,
	y: f64,
}

trait Apply {
	fn apply(&mut self, other: Self);
}

impl Apply for Digital {
	fn apply(&mut self, _other: Self) {}
}

impl Apply for Motion {
	fn apply(&mut self, other: Self) {
		self.x += other.x
	}
}

impl Apply for Motion2 {
	fn apply(&mut self, other: Self) {
		self.x += other.x;
		self.y += other.y;
	}
}

enum State<T> {
	Started { start: Instant, state: T },
	Ended { start: Instant },
}

impl<T> State<T> {
	fn ended() -> Self {
		Self::Ended {
			start: Instant::now(),
		}
	}
	fn started(state: T) -> Self {
		Self::Started {
			start: Instant::now(),
			state,
		}
	}
	fn new(state: Option<T>) -> Self {
		match state {
			None => Self::ended(),
			Some(state) => Self::started(state),
		}
	}
	fn apply(&mut self, value: Option<T>)
	where
		T: Apply,
	{
		if let Some(value) = value {
			match self {
				Self::Started { state, .. } => state.apply(value),
				Self::Ended { .. } => *self = State::started(value),
			}
		} else {
			*self = State::ended()
		}
	}
}

struct Input {
	key: FxHashMap<PhysicalKey, State<Digital>>,
	button: FxHashMap<ButtonId, State<Digital>>,
	motion: FxHashMap<u32, State<Motion>>,
	mouse_wheel: State<Motion2>,
	mouse_motion: State<Motion2>,
}

impl Input {
	pub fn new() -> Self {
		Self {
			key: FxHashMap::default(),
			button: FxHashMap::default(),
			motion: FxHashMap::default(),
			mouse_wheel: State::ended(),
			mouse_motion: State::ended(),
		}
	}
	pub fn handle_device_event(&mut self, event: DeviceEvent) {
		match event {
			DeviceEvent::MouseMotion { delta: (x, y) } => self.handle_mouse_motion(x, y),
			DeviceEvent::Button { button, state } => self.handle_button_press(button, state),
			DeviceEvent::Key(RawKeyEvent {
				physical_key,
				state,
			}) => self.handle_key_press(physical_key, state),
			DeviceEvent::Motion { axis, value } => self.handle_axis(axis, value),
			DeviceEvent::MouseWheel { delta } => self.handle_mouse_wheel(delta),
			DeviceEvent::Added => (),
			DeviceEvent::Removed => (),
		}
	}
	fn handle_mouse_motion(&mut self, x: f64, y: f64) {
		let state = if x == 0.0 && y == 0.0 {
			None
		} else {
			Some(Motion2 { x, y })
		};

		self.mouse_motion.apply(state);
	}
	fn handle_button_press(&mut self, button: u32, state: ElementState) {
		let state = match state {
			ElementState::Pressed => Some(Digital),
			ElementState::Released => None,
		};

		match self.button.entry(button) {
			Entry::Vacant(vacant) => {
				vacant.insert(State::new(state));
			}
			Entry::Occupied(occupied) => occupied.into_mut().apply(state),
		}
	}
	fn handle_key_press(&mut self, key: PhysicalKey, state: ElementState) {
		let state = match state {
			ElementState::Pressed => Some(Digital),
			ElementState::Released => None,
		};

		match self.key.entry(key) {
			Entry::Vacant(vacant) => {
				vacant.insert(State::new(state));
			}
			Entry::Occupied(occupied) => occupied.into_mut().apply(state),
		}
	}
	fn handle_axis(&mut self, motion: u32, x: f64) {
		let state = if x == 0.0 { None } else { Some(Motion { x }) };

		match self.motion.entry(motion) {
			Entry::Vacant(vacant) => {
				vacant.insert(State::new(state));
			}
			Entry::Occupied(occupied) => occupied.into_mut().apply(state),
		}
	}
	fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
		let (x, y) = match delta {
			MouseScrollDelta::LineDelta(x, y) => (x, y),
			MouseScrollDelta::PixelDelta(_) => (0.0, 0.0),
		};

		let state = if x == 0.0 && y == 0.0 {
			None
		} else {
			let x = x as f64;
			let y = y as f64;

			Some(Motion2 { x, y })
		};

		self.mouse_wheel.apply(state);
	}
}
