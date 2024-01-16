use bevy::{prelude::*, window::WindowResolution};
use cfg::*;


mod cfg {
	pub mod board {
		pub const SIZE: u32 = 16;
		pub const MID: u32 = SIZE / 2;
		pub const AREA: u32 = SIZE * SIZE;
		pub const TILEPX: f32 = 32.;
	}

	pub mod snake {
		pub const SPEED: f32 = 5.;
		pub const LENGTH: u32 = 5;
	}
}


fn main() {
	App::new()
	.insert_resource(ClearColor(Color::rgb(0., 0., 0.2)))
	.add_plugins(DefaultPlugins
		.set(WindowPlugin {
			primary_window: Some(Window {
				title: "$NAKE >:)".into(),
				resolution: WindowResolution::new(1080., 720.),
				resizable: false,
				..default()
			}),
			..default()
		})
	)
	.add_state::<GameState>()
	.init_resource::<Score>()
	.init_resource::<HeadIndex>()
	.init_resource::<StepTimer>()
	.init_resource::<Direction>()
	.add_systems(Startup, (
		setup_res,
		setup_camera,
		setup_board
	))
	.add_systems(Update, 
		(
			set_direction,
			set_colors,
			step
		)
		.run_if(in_state(GameState::InGame)))
	.run();
}


#[derive(Component)]
struct Tile { kind: Kind, pos: Position }
enum Kind { Empty, Obstacle, Food, Snake(u32) }
#[derive(PartialEq)]
struct Position { x: i32, y: i32 }

impl Tile {
	fn is_tail(&self) -> bool {
		if let Kind::Snake(i) = self.kind { i == 0 }
		else { false }
	}
	fn is_head(&self, index: u32) -> bool {
		if let Kind::Snake(i) = self.kind { i == index }
		else { false }
	}
	fn into_head(&mut self, index: u32) {
		self.kind = Kind::Snake(index + 1)
	}
}


#[derive(Resource, Default)]
struct Score(u32);


#[derive(Resource, Default)]
struct HeadIndex(u32);


#[derive(Resource, Default)]
struct StepTimer(Timer);

impl StepTimer {
	fn new(speed: f32) -> Self {
		Self(Timer::new(
			std::time::Duration::from_secs_f32(1./speed),
			TimerMode::Repeating
		))
	}
}


#[derive(Resource, Default)]
enum Direction {
	#[default]
	Right,
	Left, Up, Down
}

impl Direction {
	fn to_xy(&self) -> (i32, i32) {
		match self {
			Direction::Right => (1, 0),
			Direction::Left => (-1, 0),
			Direction::Up => (0, 1),
			Direction::Down => (0, -1),
		}
	}
}


#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum GameState {
	#[default]
	InGame,
	GameOver
}


fn setup_res (
	mut head_index: ResMut<HeadIndex>,
	mut timer: ResMut<StepTimer>
) {
	head_index.0 = snake::LENGTH - 1;
	*timer = StepTimer::new(snake::SPEED);
}


fn setup_camera
(
	mut commands: Commands
) {
	const CAM_OFFSET: f32 = (board::MID as f32) * board::TILEPX;
	commands.spawn(Camera2dBundle {
		transform: Transform::from_translation(Vec3::new(CAM_OFFSET, CAM_OFFSET, 0.)),
		..default()
	});
}


fn setup_board
(
	mut commands: Commands
) {
	const MAX: u32 = board::SIZE - 1;
	const MID: u32 = board::SIZE / 2;
	let (tail, head) = (MID-2, MID+2);
	let range = tail..=head;

	let mut board: Vec<Tile> = Vec::with_capacity(board::AREA as usize);

	for i in 0..board::AREA {
		let (x, y) = (i%board::SIZE, i/board::SIZE);
		let kind = match (x, y) {
			(0, _) | (MAX, _) | (_, 0) | (_, MAX) => Kind::Obstacle,
			(x, MID) if range.contains(&x) => Kind::Snake(x - tail),
			_ => Kind::Empty
		};
		board.push(Tile { kind, pos: Position { x: x as i32, y: y as i32 } });
	}

	for tile in board {
		let (x, y) = ((tile.pos.x as f32) * board::TILEPX, (tile.pos.y as f32) * board::TILEPX);

		commands.spawn(SpriteBundle {
			sprite: Sprite {
				custom_size: Some(Vec2::new(board::TILEPX, board::TILEPX)),
				..default()
			},
			transform: Transform::from_translation(Vec3::new(x, y, 0.)),
			..default()
		})
		.insert(tile);
	}
}


fn set_direction
(
	mut direction: ResMut<Direction>,
	kb: Res<Input<KeyCode>>
)
{
	kb.get_pressed().next()
		.and_then(|key_code| {
			match key_code {
				KeyCode::W => Some(Direction::Up),
				KeyCode::A => Some(Direction::Left),
				KeyCode::S => Some(Direction::Down),
				KeyCode::D => Some(Direction::Right),
				_ => None
			}
		})
		.map(|next_direction| {
			*direction = next_direction;
		})
	;
}


fn set_colors
(
	mut sprites: Query<(&mut Sprite, &Tile)>
) {
	for (mut sprite, tile) in &mut sprites {
		sprite.color = match tile.kind {
			Kind::Empty => Color::DARK_GRAY,
			Kind::Obstacle => Color::GRAY,
			Kind::Food => Color::MIDNIGHT_BLUE,
			Kind::Snake(_) => Color::hsl(0., 0., 0.8)
		};
	}
}


fn step
(
	mut tiles: Query<&mut Tile>,
	direction: Res<Direction>,
	mut head_index: ResMut<HeadIndex>,
	mut score: ResMut<Score>,
	mut timer: ResMut<StepTimer>,
	time: Res<Time>,
	mut next_state: ResMut<NextState<GameState>>
) {
	timer.0.tick(time.delta());

	if !timer.0.just_finished() { return; }

	let head_tile = tiles.iter()
		.find(|x| x.is_head(head_index.0))
		.expect("snake head not found")
	;

	let Position {x, y} = head_tile.pos;
	let (dx, dy) = direction.to_xy();
	let next_pos = Position { x: x + dx, y: y + dy };
	let mut next_tile = tiles.iter_mut().find(|x| x.pos == next_pos).unwrap();

	match next_tile.kind {
		Kind::Obstacle | Kind::Snake(_) => { 
			next_state.set(GameState::GameOver);
			return;
		}
		Kind::Food => {
			next_tile.into_head(head_index.0);
			head_index.0 += 1;
			score.0 += 1;
			return;
		}
		Kind::Empty => {
			next_tile.into_head(head_index.0);
		}
	}

	tiles.iter_mut()
		.find(|x| x.is_tail())
		.expect("snake tail not found")
		.kind = Kind::Empty
	;

	tiles.iter_mut().for_each(|mut x| {
		if let Kind::Snake(i) = x.kind {
			x.kind = Kind::Snake(i-1)
		}})
	;
}
