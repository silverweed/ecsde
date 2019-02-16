use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let config = ecs::demo::Config::new(env::args());
	//ecs::demo::gfx_test(&config);
	ecs::demo::console_test(&config);
	Ok(())
}
