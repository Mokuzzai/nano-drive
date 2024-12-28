use nd_core::plugin::Plugin;
use nd_core::ApplicationEvent;

use nd_core::system::Commands;
use nd_core::world::World;

fn enter(world: &World, commands: &Commands) {
	println!("Hello main!");
}

fn build(plugin: &mut Plugin) {
	plugin.add_startup_system(enter as for<'a, 'b> fn(&'a _, &'b _));
}
