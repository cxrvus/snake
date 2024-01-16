use std::default;
use bevy::{prelude::*, window::WindowResolution, transform::commands, render::color};
use cfg::*;

mod cfg {
	pub const SIZE: u32 = 16;
	pub const AREA: u32 = SIZE * SIZE;
	pub const TILEPX: f32 = 32.;
}


fn main() {
	App::new()
	.insert_resource(ClearColor(Color::rgb(0., 0., 0.2)))
	.add_plugins(DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				title: "SNAKE :)".into(),
				resolution: WindowResolution::new(1080., 720.),
				resizable: false,
				..default()
			}),
			..default()
		})
		.set(ImagePlugin::default_nearest())
	)
	.add_systems(Startup, (
		setup_camera,
		setup_board
	))
	.run();
}


#[derive(Component)]
struct Tile {
	kind: Kind,
	x: u32,
	y: u32
}

enum Kind {
	Empty,
	Obstacle,
	Food,
	Snake(u32)
}

fn setup_camera
(
	mut commands: Commands
) {
	// commads.spawn()
}

fn setup_board
(
	mut commands: Commands
) {
	const MAX: u32 = SIZE - 1;
	const MID: u32 = SIZE / 2;
	let (start, end) = (MID-2, MID+2);
	let range = start..=end;

	let mut board: Vec<Tile> = Vec::with_capacity(AREA as usize);

	for i in 0..AREA {
		let (x, y) = (i%SIZE, i/SIZE);
		let kind = match (x, y) {
			(0, _) | (MAX, _) | (_, 0) | (_, MAX) => Kind::Obstacle,
			(x, MID) if range.contains(&x) => Kind::Snake(x-start),
			_ => Kind::Empty
		};
		board.push(Tile { kind , x , y });
	}

	for tile in board {
		commands.spawn(SpriteBundle {
			sprite: Sprite {
				color: Color::GRAY,
				custom_size: Some(Vec2::new(TILEPX, TILEPX)),
				..default()
			},
			..default()
		})
		.insert(tile);
	}
}
