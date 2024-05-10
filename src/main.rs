mod menu;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};

use bevy::{math::vec2, prelude::*};
use menu::MenuPlugin;

const STEP_TIME_MS: f32 = 200.;
const TILE_SIZE: f32 = 64.;

#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

#[derive(Component)]
struct Player {
    movement_timer: Timer,
    move_dir: Direction,
    next_move_dir: Direction,
}

fn spawn_player_system(mut commands: Commands, ass_serv: Res<AssetServer>) {
    commands.spawn((
        Player {
            movement_timer: Timer::from_seconds(STEP_TIME_MS / 1000., TimerMode::Once),
            move_dir: Direction::None,
            next_move_dir: Direction::None,
        },
        SpriteBundle {
            texture: ass_serv.load("player.png"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct Immovable;

#[derive(Component)]
pub struct Movable;

fn player_movement_system(
    time: Res<Time>,
    mut player_query: Query<(&mut Player, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let (mut player, mut transform) = player_query.single_mut();

    if match player.move_dir {
        Direction::Up => input.pressed(KeyCode::KeyW),
        Direction::Down => input.pressed(KeyCode::KeyS),
        Direction::Left => input.pressed(KeyCode::KeyA),
        Direction::Right => input.pressed(KeyCode::KeyD),
        Direction::None => false,
    } {
        player.next_move_dir = player.move_dir
    } else {
        let mut changed = false;
        if input.pressed(KeyCode::KeyW) {
            player.next_move_dir = Direction::Up;
            changed = true;
        }
        if input.pressed(KeyCode::KeyS) {
            player.next_move_dir = Direction::Down;
            changed = true;
        }
        if input.pressed(KeyCode::KeyD) {
            player.next_move_dir = Direction::Right;
            changed = true;
        }
        if input.pressed(KeyCode::KeyA) {
            player.next_move_dir = Direction::Left;
            changed = true;
        }
        if !changed {
            player.next_move_dir = Direction::None;
        }
    }

    player.movement_timer.tick(time.delta());

    if player.movement_timer.finished() {
        player.movement_timer.reset();
        player.move_dir = player.next_move_dir;
        match player.move_dir {
            Direction::Up => transform.translation.y += TILE_SIZE,
            Direction::Down => transform.translation.y -= TILE_SIZE,
            Direction::Left => transform.translation.x -= TILE_SIZE,
            Direction::Right => transform.translation.x += TILE_SIZE,
            Direction::None => player
                .movement_timer
                .set_elapsed(Duration::from_millis(STEP_TIME_MS as u64)),
        }

        player.next_move_dir = Direction::None;
    }
}

fn player_collision_system(
    mut player_query: Query<(&Player, &mut Transform), (Without<Movable>, Without<Immovable>)>,
    mut movable_query: Query<(&mut Movable, &mut Transform), (Without<Player>, Without<Immovable>)>,
    immovable_query: Query<(&Immovable, &Transform), (Without<Movable>, Without<Player>)>,
) {
    let (player, mut ptransform) = player_query.single_mut();

    for (_, transform) in &immovable_query {
        if ptransform.translation.distance(transform.translation) < TILE_SIZE {
            match player.move_dir {
                Direction::Up => ptransform.translation.y -= TILE_SIZE,
                Direction::Down => ptransform.translation.y += TILE_SIZE,
                Direction::Left => ptransform.translation.x += TILE_SIZE,
                Direction::Right => ptransform.translation.x -= TILE_SIZE,
                Direction::None => (),
            }
        }
    }
    for (_, mut transform) in &mut movable_query {
        if ptransform.translation.distance(transform.translation) < TILE_SIZE {
            match player.move_dir {
                Direction::Up => transform.translation.y += TILE_SIZE,
                Direction::Down => transform.translation.y -= TILE_SIZE,
                Direction::Left => transform.translation.x -= TILE_SIZE,
                Direction::Right => transform.translation.x += TILE_SIZE,
                Direction::None => (),
            }
        }
    }
}

#[derive(Component)]
struct Lifetile {
    curstate: LifeType,
    nextstate: LifeType,
}

#[derive(Component)]
struct Tile;

#[derive(PartialEq, Eq, Clone, Copy)]
enum LifeType {
    Living,
    Dead,
}

fn load_tutoral_system(mut commands: Commands, ass_serv: Res<AssetServer>) {
    load_level(&mut commands, "tutorial.map", ass_serv);
}

fn load_level(commands: &mut Commands, levelname: &str, ass_serv: Res<AssetServer>) {
    let file = File::open(format!("assets/maps/{}", levelname)).unwrap();

    let mut offset = vec2(0., 0.);

    for (r, line) in BufReader::new(file).lines().enumerate() {
        if let Ok(line) = line {
            for (c, character) in line.chars().enumerate() {
                if c % 2 == 1 {
                    match character {
                        '#' => {
                            commands.spawn((
                                Tile,
                                Movable,
                                SpriteBundle {
                                    texture: ass_serv.load("cell.png"),
                                    transform: Transform::from_xyz(
                                        (c as f32 / 2.).floor() * TILE_SIZE,
                                        r as f32 * TILE_SIZE,
                                        0.,
                                    ),
                                    ..default()
                                },
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Playing,
    Living,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum PausedState {
    #[default]
    Paused,
    Unpaused,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::Rgba {
                red: 0.,
                green: 0.,
                blue: 0.,
                alpha: 1.,
            }),
            ..default()
        },
        ..default()
    });
}

fn toggle_paused(
    key_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<PausedState>>,
    mut next_state: ResMut<NextState<PausedState>>,
) {
    if key_input.just_pressed(KeyCode::KeyP) {
        match state.get() {
            PausedState::Paused => next_state.set(PausedState::Unpaused),
            PausedState::Unpaused => next_state.set(PausedState::Paused),
        }
    }
}

fn enable_sim(
    key_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if key_input.just_pressed(KeyCode::KeyL) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Living),
            GameState::Living => next_state.set(GameState::Playing),
        }
    }
}

#[derive(Component)]
struct Sim {
    tiles: Vec<Option<Entity>>,
    width: usize,
    height: usize,
    timer: Timer,
}

fn load_sim(
    mut commands: Commands,
    t_query: Query<&Transform, With<Tile>>,
    ass_serv: Res<AssetServer>,
) {
    let mut sim = Sim {
        tiles: vec![None; 100 * 100],
        width: 100,
        height: 100,
        timer: Timer::from_seconds(2., TimerMode::Repeating),
    };
    for transform in &t_query {
        let x = transform.translation.x / TILE_SIZE;
        let y = transform.translation.y / TILE_SIZE;
        println!("{}: {}", x, y);
        if x < 100. && y < 100. {
            let tile = commands.spawn((
                Lifetile {
                    curstate: LifeType::Living,
                    nextstate: LifeType::Living,
                },
                SpriteBundle {
                    texture: ass_serv.load("cell.png"),
                    transform: Transform::from_xyz(
                        (x as f32).floor() * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        0.,
                    ),
                    ..default()
                },
            ));
            sim.tiles[(y * 100. + x) as usize] = Some(tile.id());
        } else {
            println!("Failed");
        }
    }
    for (i, cell) in sim.tiles.clone().iter().enumerate() {
        // println!("{}, {}", i % 100, i / 100);
        if (*cell).is_none() {
            let tile = commands.spawn((
                Lifetile {
                    curstate: LifeType::Dead,
                    nextstate: LifeType::Dead,
                },
                SpriteBundle {
                    texture: ass_serv.load("dead_cell.png"),
                    transform: Transform::from_xyz(
                        ((i % 100) as f32 / 2.).floor() * TILE_SIZE,
                        (i / 100) as f32 * TILE_SIZE,
                        0.,
                    ),
                    ..default()
                },
            ));
            sim.tiles[i] = Some(tile.id());
        }
    }
    commands.spawn(sim);
}

fn despawn_world(
    mut commands: Commands,
    p_query: Query<Entity, With<Player>>,
    t_query: Query<Entity, With<Tile>>,
) {
    let player = p_query.single();
    commands.get_entity(player).unwrap().despawn();
    for tile in &t_query {
        commands.get_entity(tile).unwrap().despawn_recursive();
    }
}

fn despawn_sim(
    mut commands: Commands,
    l_query: Query<Entity, With<Lifetile>>,
    sim_q: Query<Entity, With<Sim>>,
) {
    for sim in &sim_q {
        commands.get_entity(sim).unwrap().despawn_recursive();
    }
    for life in &l_query {
        commands.get_entity(life).unwrap().despawn_recursive();
    }
}

fn update_sim(
    mut commands: Commands,
    mut sim_q: Query<&mut Sim>,
    mut tile_q: Query<(&mut Lifetile, &mut Handle<Image>)>,
    ass_serv: Res<AssetServer>,
    time: Res<Time>,
) {
    let mut sim = sim_q.single_mut();

    sim.timer.tick(time.delta());

    if sim.timer.finished() {
        for (i, tile) in sim.tiles.iter().enumerate() {
            let mut neighbors = 0;
            for offsetx in -1..2 {
                for offsety in -1..2 {
                    if i as i32 + offsetx + offsety * 100 > 0
                        && i as i32 + offsetx + offsety * 100 < (sim.width * sim.height) as i32
                    {
                        let (tile, _) = tile_q
                            .get(sim.tiles[(i as i32 + offsetx + offsety * 100) as usize].unwrap())
                            .unwrap();
                        if tile.curstate == LifeType::Living {
                            neighbors += 1;
                        }
                    }
                }
            }
            let (mut tile, mut handle) = tile_q.get_mut(sim.tiles[i].unwrap()).unwrap();
            if tile.curstate == LifeType::Living {
                neighbors -= 1;
                if neighbors < 2 || neighbors > 3 {
                    tile.nextstate = LifeType::Dead;
                    handle.set_if_neq(ass_serv.load("dead_cell.png"));
                }
            } else {
                if neighbors == 3 {
                    tile.nextstate = LifeType::Living;
                    handle.set_if_neq(ass_serv.load("cell.png"));
                }
            }
        }
    }
    for (mut tile, _) in &mut tile_q {
        tile.curstate = tile.nextstate;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .init_state::<PausedState>()
        .add_plugins(MenuPlugin {
            state: PausedState::Paused,
        })
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(GameState::Playing),
            (spawn_player_system, load_tutoral_system, despawn_sim),
        )
        .add_systems(
            OnEnter(GameState::Living),
            (
                load_sim.before(despawn_world),
                despawn_world.after(load_sim),
            ),
        )
        .add_systems(Update, (toggle_paused, enable_sim))
        .add_systems(
            Update,
            (
                player_movement_system,
                player_collision_system.after(player_movement_system),
            )
                .run_if(in_state(PausedState::Unpaused).and_then(in_state(GameState::Playing))),
        )
        .add_systems(
            Update,
            (update_sim)
                .run_if(in_state(PausedState::Unpaused).and_then(in_state(GameState::Living))),
        )
        .run();
}
