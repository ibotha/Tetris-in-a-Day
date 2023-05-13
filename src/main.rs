use bevy::{prelude::*, sprite::Anchor};
use bevy_fps_counter::{FpsCounter, FpsCounterPlugin};
use rand::prelude::*;
use strum::EnumCount;
use strum_macros::EnumCount as EnumCountMacro;

const BOARD_SIZE: Vec2 = Vec2 { x: 15.0, y: 20.0 };
const BOARD_ORIGIN: Vec2 = Vec2 { x: 6.0, y: 1.0 };
const WORLD_SIZE: Vec2 = Vec2 {
    x: BOARD_SIZE.x + BOARD_ORIGIN.x + 2.,
    y: BOARD_SIZE.y + 2.,
};
fn setup_cam(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(WORLD_SIZE.x / 2.0, WORLD_SIZE.y / 2.0, 0.),
            ..Default::default()
        },
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: WORLD_SIZE.x,
                min_height: WORLD_SIZE.y,
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

#[derive(Resource)]
struct Score(usize);

#[derive(Resource)]
struct Level(usize);

#[derive(Resource)]
struct Board {
    width: usize,
    height: usize,
    squares: Vec<Vec<Entity>>,
    board: Vec<Vec<bool>>,
    colors: Vec<Vec<Color>>,
}

#[derive(Component)]
struct BoardTileColor(Color);

#[derive(Component)]
struct BoardTile;

const MAX_LEVEL: usize = 29;

impl Board {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            squares: vec![vec![Entity::from_raw(0); width]; height],
            board: vec![vec![false; width]; height],
            colors: vec![vec![Color::WHITE; width]; height],
        }
    }

    fn update_sprite_colors(mut sprites: Query<(&mut Sprite, &mut BoardTileColor)>) {
        for (mut sprite, color) in sprites.iter_mut() {
            sprite.color = color.0;
        }
    }

    fn update_board_sprites(
        mut commands: Commands,
        current_piece: Res<CurrentPiece>,
        board: Res<Board>,
        display_board: Res<DisplayBoard>,
        current_piece_board: Res<CurrentPieceBoard>,
    ) {
        for row in 0..board.height {
            for col in 0..board.width {
                let entity = board.squares[row][col];
                if board.board[row][col] {
                    commands.entity(entity).insert(Visibility::Visible);
                    commands
                        .entity(entity)
                        .insert(BoardTileColor(board.colors[row][col]));
                } else {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }

        for row in 0..display_board.0.height {
            for col in 0..display_board.0.width {
                let entity = display_board.0.squares[row][col];
                if display_board.0.board[row][col] {
                    commands.entity(entity).insert(Visibility::Visible);
                } else {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }

        for row in 0..current_piece_board.0.height {
            for col in 0..current_piece_board.0.width {
                let entity = current_piece_board.0.squares[row][col];
                if current_piece_board.0.board[row][col] {
                    commands.entity(entity).insert(Visibility::Visible);
                    commands
                        .entity(entity)
                        .insert(BoardTileColor(current_piece.get_color()));
                } else {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn debug(mut board: ResMut<Board>, mut timer: Local<Option<Timer>>, time: Res<Time>) {
        if timer.is_none() {
            *timer = Some(Timer::from_seconds(0.125, TimerMode::Repeating));
        }
        if timer.as_mut().unwrap().tick(time.delta()).just_finished() {
            let mut tries = 20;
            loop {
                let x: usize = rand::thread_rng().gen_range(0..board.width);
                let y: usize = rand::thread_rng().gen_range(0..board.height);
                if board.board[y][x] == false {
                    board.board[y][x] = true;
                    break;
                }
                tries -= 1;
                if tries == 0 {
                    break;
                }
            }
        }
    }

    fn assess_board(
        mut board: ResMut<Board>,
        mut score: ResMut<Score>,
        mut level: ResMut<Level>,
        mut cleared_lines: Local<Option<u32>>,
    ) {
        if cleared_lines.is_none() {
            *cleared_lines = Some(0);
        }
        let mut rows_to_remove = vec![];
        for row in 0..board.height {
            let full = board.board[row].iter().all(|&x| x);
            if full {
                rows_to_remove.push(row);
            }
        }

        for row in rows_to_remove.iter().rev() {
            board.board.remove(*row);
            board.colors.remove(*row);
            let width: usize = board.width;
            board.board.push(vec![false; width]);
            board.colors.push(vec![Color::WHITE; width]);
        }
        let old_level = level.0;
        let get_score = match rows_to_remove.len() {
            0 => ((level.0 + 1) * 0) as usize,
            1 => ((level.0 + 1) * 40) as usize,
            2 => ((level.0 + 1) * 100) as usize,
            3 => ((level.0 + 1) * 300) as usize,
            4 => ((level.0 + 1) * 1200) as usize,
            _ => ((level.0 + 1) * 1200) as usize,
        };
        *cleared_lines = Some(cleared_lines.unwrap() + rows_to_remove.len() as u32);
        score.0 += get_score;
        let line_threshold = match old_level {
            0..=8 => old_level * 10 + 10,
            _ => std::cmp::min(std::cmp::max(100, old_level * 10 - 50), 200),
        };
        if cleared_lines.unwrap() >= line_threshold as u32 {
            level.0 += 1;
            *cleared_lines = Some(cleared_lines.unwrap() - line_threshold as u32);
        }
        level.0 = std::cmp::min(level.0, MAX_LEVEL); // Assign to level later :/
        let new_level = level.0;
        if old_level != new_level {
            level.0 = new_level;
        }
    }
}

#[derive(Component)]
struct LevelModifier(usize);

const MAX_PIECE_IN_BAG: usize = Piece::COUNT; // 7;

#[derive(Resource)]
struct Bag(Vec<Piece>);

#[derive(Debug, Clone, Copy, EnumCountMacro)]
enum Piece {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Bag {
    fn new() -> Self {
        let mut bag = Self(vec![]);
        bag.fill_bag();
        bag
    }

    fn fill_bag(&mut self) {
        let mut pieces = vec![
            Piece::I,
            Piece::O,
            Piece::T,
            Piece::S,
            Piece::Z,
            Piece::J,
            Piece::L,
        ];
        let mut rng = rand::thread_rng();
        for _ in 0..(MAX_PIECE_IN_BAG * 10) {
            let index = rng.gen_range(0..pieces.len());
            let index1 = rng.gen_range(0..pieces.len());
            pieces.swap(index, index1);
        }
        self.0 = pieces;
    }

    fn next_piece(&mut self) -> Piece {
        let ret = self.0.pop().unwrap();
        if self.0.is_empty() {
            self.fill_bag();
        }
        ret
    }

    fn peek(&self) -> Piece {
        self.0.last().unwrap().clone()
    }
}

#[derive(Resource, Clone, Copy)]
struct CurrentPiece {
    piece: Piece,
    position: IVec2,
    rotation: i32,
    placed: bool,
}

impl CurrentPiece {
    fn new(piece: Piece) -> Self {
        Self {
            piece,
            position: IVec2 {
                x: BOARD_SIZE.x as i32 / 2,
                y: BOARD_SIZE.y as i32 - 3,
            },
            rotation: 0,
            placed: false,
        }
    }

    fn width(&self) -> i32 {
        piece_width(self.piece, self.rotation)
    }

    fn get_color(self) -> Color {
        match self.piece {
            Piece::I => Color::rgb(0.5, 1.0, 1.0),
            Piece::O => Color::rgb(1.0, 1.0, 0.5),
            Piece::T => Color::rgb(1.0, 0.5, 1.0),
            Piece::S => Color::rgb(0.5, 1.0, 0.5),
            Piece::Z => Color::rgb(1.0, 0.5, 0.5),
            Piece::J => Color::rgb(0.5, 0.5, 1.0),
            Piece::L => Color::rgb(1.0, 0.7, 0.5),
        }
    }
}

fn piece_width(piece: Piece, rotation: i32) -> i32 {
    get_piece_meat_positions(piece, IVec2::ZERO, rotation)
        .iter()
        .map(|x| x.x)
        .max()
        .unwrap()
        + 1
}

fn place_piece(
    mut current_piece: ResMut<CurrentPiece>,
    mut board: ResMut<Board>,
    mut display_board: ResMut<DisplayBoard>,
    mut current_piece_board: ResMut<CurrentPieceBoard>,
    mut bag: ResMut<Bag>,
) {
    if current_piece.placed {
        place_piece_in_array(
            current_piece.piece,
            current_piece.position,
            current_piece.rotation,
            &mut board,
            Some(current_piece.get_color()),
        );
        *current_piece = CurrentPiece::new(bag.next_piece());
        current_piece_board.0.board =
            vec![vec![false; current_piece_board.0.width]; current_piece_board.0.height];
        place_piece_in_array(
            current_piece.piece,
            IVec2::ZERO,
            current_piece.rotation,
            &mut current_piece_board.0,
            None,
        );
        display_board.0.board = vec![vec![false; display_board.0.width]; display_board.0.height];
        place_piece_in_array(bag.peek(), IVec2::ZERO, 0, &mut display_board.0, None);
    }
}
fn check_piece_obstructed(
    piece: Piece,
    offset: IVec2,
    rotation: i32,
    array: &mut [&mut [bool]],
) -> bool {
    let meat = get_piece_meat_positions(piece, offset, rotation);
    for pos in meat.iter() {
        if pos.x < 0 || pos.x >= BOARD_SIZE.x as i32 || pos.y < 0 {
            return true;
        }
        if pos.y >= BOARD_SIZE.y as i32 {
            continue;
        }
        if array[pos.y as usize][pos.x as usize] {
            return true;
        }
    }
    false
}

fn place_piece_in_array(
    piece: Piece,
    offset: IVec2,
    rotation: i32,
    board: &mut Board,
    color: Option<Color>,
) {
    let color = match color {
        Some(c) => c,
        None => Color::WHITE,
    };
    let meat = get_piece_meat_positions(piece, offset, rotation);
    for pos in meat.iter() {
        board.board[pos.y as usize][pos.x as usize] = true;
    }

    for pos in meat.iter() {
        board.colors[pos.y as usize][pos.x as usize] = color;
    }
}

fn get_piece_meat_positions(piece: Piece, offset: IVec2, rotation: i32) -> [IVec2; 4] {
    let x = offset.x as i32;
    let y = offset.y as i32;
    match piece {
        Piece::I => match rotation {
            0 | 2 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 2, y: y + 0 },
                IVec2 { x: x + 3, y: y + 0 },
            ],
            1 | 3 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 0, y: y + 2 },
                IVec2 { x: x + 0, y: y + 3 },
            ],
            _ => unreachable!(),
        },
        Piece::O => [
            IVec2 { x: x + 0, y: y + 0 },
            IVec2 { x: x + 0, y: y + 1 },
            IVec2 { x: x + 1, y: y + 0 },
            IVec2 { x: x + 1, y: y + 1 },
        ],
        Piece::T => match rotation {
            0 => [
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 1, y: y + 2 },
            ],
            1 => [
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 2, y: y + 1 },
            ],
            2 => [
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 2 },
            ],
            3 => [
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 2, y: y + 0 },
            ],
            _ => unreachable!(),
        },
        Piece::S => match rotation {
            0 | 2 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 2 },
            ],
            1 | 3 => [
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 2, y: y + 0 },
            ],
            _ => unreachable!(),
        },
        Piece::Z => match rotation {
            0 | 2 => [
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 0, y: y + 2 },
            ],
            1 | 3 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 2, y: y + 1 },
            ],
            _ => unreachable!(),
        },
        Piece::J => match rotation {
            0 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 2, y: y + 1 },
            ],
            1 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 0, y: y + 2 },
                IVec2 { x: x + 1, y: y + 0 },
            ],
            2 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 2, y: y + 0 },
                IVec2 { x: x + 2, y: y + 1 },
            ],
            3 => [
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 2 },
                IVec2 { x: x + 0, y: y + 2 },
            ],
            _ => unreachable!(),
        },
        Piece::L => match rotation {
            0 => [
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 2, y: y + 0 },
            ],
            1 => [
                IVec2 { x: x + 1, y: y + 0 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 1, y: y + 2 },
                IVec2 { x: x + 0, y: y + 0 },
            ],
            2 => [
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 1, y: y + 1 },
                IVec2 { x: x + 2, y: y + 1 },
                IVec2 { x: x + 2, y: y + 0 },
            ],
            3 => [
                IVec2 { x: x + 0, y: y + 0 },
                IVec2 { x: x + 0, y: y + 1 },
                IVec2 { x: x + 0, y: y + 2 },
                IVec2 { x: x + 1, y: y + 2 },
            ],
            _ => unreachable!(),
        },
    }
}

#[derive(Component)]
struct Collider;

fn render_score(mut score_node: Query<&mut Text, With<ScoreDisplay>>, score: Res<Score>) {
    for mut text in score_node.iter_mut() {
        text.sections[0].value = format!("{}", score.0);
    }
}

fn render_level(mut level_node: Query<&mut Text, With<LevelDisplay>>, level: Res<Level>) {
    for mut text in level_node.iter_mut() {
        text.sections[0].value = format!("Level: {}", level.0);
    }
}

#[derive(Component)]
struct ScoreDisplay;

#[derive(Component)]
struct LevelDisplay;

fn setup_board(
    mut commands: Commands,
    mut bag: ResMut<Bag>,
    score: ResMut<Score>,
    level: ResMut<Level>,
    mut board: ResMut<Board>,
    mut display_board: ResMut<DisplayBoard>,
    mut current_piece_board: ResMut<CurrentPieceBoard>,
    asset_server: Res<AssetServer>,
) {
    let piece = bag.next_piece();
    let current_piece = CurrentPiece::new(piece);
    commands.insert_resource(current_piece);

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween, // TODO: May want to change later
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(20.)),
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|left_column| {
                    left_column.spawn((
                        TextBundle::from_section(
                            format!("{}", score.0),
                            TextStyle {
                                font_size: 125.0, // TODO: Notify bevy of this kaka
                                color: Color::WHITE,
                                font: asset_server.load("fonts/UbuntuMonoNerdFontCompleteMono.ttf"), // It is path-ing agnostic
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..Default::default()
                        }),
                        Label,
                        ScoreDisplay,
                    ));

                    left_column.spawn((
                        TextBundle::from_section(
                            format!("Level: {}", level.0),
                            TextStyle {
                                font_size: 80.0, // TODO: Notify bevy of this kaka
                                color: Color::BEIGE,
                                font: asset_server.load("fonts/UbuntuMonoNerdFontCompleteMono.ttf"), // It is path-ing agnostic
                                ..Default::default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(10.0)),
                            ..Default::default()
                        }),
                        Label,
                        LevelDisplay,
                    ));
                });
            parent.spawn(NodeBundle {
                style: Style {
                    size: Size::width(Val::Percent(80.)),
                    ..Default::default()
                },
                ..Default::default()
            });
        });
    let block_image = asset_server.load("textures/block.png");

    for col_index in 0..board.width {
        for row_index in 0..board.height {
            board.squares[row_index][col_index] = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE),
                            anchor: bevy::sprite::Anchor::BottomLeft,
                            ..Default::default()
                        },
                        texture: block_image.clone().into(),
                        transform: Transform {
                            translation: Vec3::new(
                                (col_index as f32) + (BOARD_ORIGIN.x),
                                (row_index as f32) + (BOARD_ORIGIN.y),
                                0.0,
                            ),
                            scale: Vec3 {
                                x: 0.95,
                                y: 0.95,
                                z: 0.95,
                            },
                            ..default()
                        },
                        ..Default::default()
                    },
                    BoardTile,
                ))
                .id();
        }
    }

    for col_index in 0..display_board.0.width {
        for row_index in 0..display_board.0.height {
            display_board.0.squares[row_index][col_index] = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE),
                            color: Color::ORANGE_RED,
                            anchor: bevy::sprite::Anchor::BottomLeft,
                            ..Default::default()
                        },
                        texture: block_image.clone().into(),
                        transform: Transform {
                            translation: Vec3::new(col_index as f32, row_index as f32 + 5., 0.0),
                            scale: Vec3 {
                                x: 0.95,
                                y: 0.95,
                                z: 0.95,
                            },
                            ..default()
                        },
                        ..Default::default()
                    },
                    BoardTile,
                ))
                .id();
        }
    }

    commands
        .spawn((
            TransformBundle {
                local: Transform {
                    translation: Vec3::new(BOARD_ORIGIN.x, BOARD_ORIGIN.y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            VisibilityBundle {
                ..Default::default()
            },
        ))
        .with_children(|board_parent| {
            board_parent
                .spawn((
                    TransformBundle {
                        local: Transform {
                            translation: Vec3::ZERO,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    CurrentPieceTransform,
                    VisibilityBundle {
                        ..Default::default()
                    },
                ))
                .with_children(|current_piece_parent| {
                    for col_index in 0..current_piece_board.0.width {
                        for row_index in 0..current_piece_board.0.height {
                            current_piece_board.0.squares[row_index][col_index] =
                                current_piece_parent
                                    .spawn((
                                        SpriteBundle {
                                            sprite: Sprite {
                                                custom_size: Some(Vec2::ONE),
                                                anchor: bevy::sprite::Anchor::BottomLeft,
                                                ..Default::default()
                                            },
                                            texture: block_image.clone().into(),
                                            transform: Transform {
                                                translation: Vec3::new(
                                                    col_index as f32,
                                                    row_index as f32,
                                                    0.0,
                                                ),
                                                scale: Vec3 {
                                                    x: 0.95,
                                                    y: 0.95,
                                                    z: 0.95,
                                                },
                                                ..default()
                                            },
                                            ..Default::default()
                                        },
                                        BoardTile,
                                    ))
                                    .id();
                        }
                    }
                });
        });

    current_piece_board.0.board = vec![vec![false; display_board.0.width]; display_board.0.height];
    place_piece_in_array(piece, IVec2::ZERO, 0, &mut current_piece_board.0, None);
    display_board.0.board = vec![vec![false; display_board.0.width]; display_board.0.height];
    place_piece_in_array(bag.peek(), IVec2::ZERO, 0, &mut display_board.0, None);
    for row in 0..board.height + 1 {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::ONE),
                color: Color::rgb(0.3, 0.4, 0.5),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..Default::default()
            },
            texture: block_image.clone().into(),
            transform: Transform {
                translation: Vec3::new(-1. + BOARD_ORIGIN.x, row as f32, 0.0),
                scale: Vec3 {
                    x: 0.95,
                    y: 0.95,
                    z: 0.95,
                },
                ..default()
            },
            ..Default::default()
        });
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::ONE),
                color: Color::rgb(0.3, 0.4, 0.5),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..Default::default()
            },
            texture: block_image.clone().into(),
            transform: Transform {
                translation: Vec3::new(BOARD_ORIGIN.x + BOARD_SIZE.x, row as f32, 0.0),
                scale: Vec3 {
                    x: 0.95,
                    y: 0.95,
                    z: 0.95,
                },
                ..default()
            },
            ..Default::default()
        });
    }

    for col_pos in 0..BOARD_SIZE.x as i32 {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::ONE),
                color: Color::rgb(0.3, 0.4, 0.5),
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..Default::default()
            },
            texture: block_image.clone().into(),
            transform: Transform {
                translation: Vec3::new(col_pos as f32 + BOARD_ORIGIN.x, -1. + BOARD_ORIGIN.y, 0.0),
                scale: Vec3 {
                    x: 0.95,
                    y: 0.95,
                    z: 0.95,
                },
                ..default()
            },
            ..Default::default()
        });
    }
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut bag: ResMut<Bag>,
    mut level: ResMut<Level>,
    mut score: ResMut<Score>,
    mut current_piece: ResMut<CurrentPiece>,
    mut current_piece_board: ResMut<CurrentPieceBoard>,
    mut board: ResMut<Board>,
) {
    let mut redraw_piece_board = false;
    let current_piece_snapshot = current_piece.clone();
    if keys.just_pressed(KeyCode::Q) {
        let new_rotation = (current_piece.rotation + 1) % 4;
        let current_width = piece_width(current_piece.piece, current_piece.rotation);
        let new_width = piece_width(current_piece.piece, new_rotation);
        let offset = (current_width - new_width) / 2;
        current_piece.position.x = current_piece.position.x + offset;
        current_piece.rotation = new_rotation;
        redraw_piece_board = true;
    }
    if keys.just_pressed(KeyCode::E) {
        let new_rotation = (current_piece.rotation + 3) % 4;
        let current_width = piece_width(current_piece.piece, current_piece.rotation);
        let new_width = piece_width(current_piece.piece, new_rotation);
        let offset = (current_width - new_width) / 2;
        current_piece.position.x = current_piece.position.x + offset;
        current_piece.rotation = new_rotation;
        redraw_piece_board = true;
    }
    if keys.any_just_pressed([KeyCode::Left, KeyCode::A]) {
        current_piece.position.x = current_piece.position.x - 1;
    }
    if keys.any_just_pressed([KeyCode::Right, KeyCode::D]) {
        current_piece.position.x = current_piece.position.x + 1;
    }
    if keys.just_pressed(KeyCode::Down) {}
    if keys.any_just_pressed([KeyCode::Up, KeyCode::W]) {
        //keys.just_pressed(KeyCode::Up) {
        current_piece.position = IVec2::ZERO;
        current_piece.placed = true;
    }
    if keys.just_pressed(KeyCode::Space) {
        while !check_piece_obstructed(
            current_piece.piece,
            current_piece.position + IVec2::NEG_Y,
            current_piece.rotation,
            board
                .board
                .iter_mut()
                .map(|x| x.as_mut_slice())
                .collect::<Vec<_>>()
                .as_mut_slice(),
        ) {
            current_piece.position.y -= 1;
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        for row in 0..board.height {
            for col in 0..board.width {
                board.board[row][col] = false;
            }
        }
        level.0 = 0;
        score.0 = 0;
        *current_piece = CurrentPiece::new(bag.next_piece());
    }
    let piece_width = current_piece.width();
    if current_piece.position.x < 0 {
        current_piece.position.x = 0;
    }
    if (current_piece.position.x + piece_width) >= BOARD_SIZE.x as i32 {
        current_piece.position.x = BOARD_SIZE.x as i32 - piece_width;
    }

    if check_piece_obstructed(
        current_piece.piece,
        current_piece.position,
        current_piece.rotation,
        board
            .board
            .iter_mut()
            .map(|x| x.as_mut_slice())
            .collect::<Vec<_>>()
            .as_mut_slice(),
    ) {
        *current_piece = current_piece_snapshot;
        current_piece.placed = true;
    }

    if redraw_piece_board {
        current_piece_board.0.board =
            vec![vec![false; current_piece_board.0.width]; current_piece_board.0.height];
        place_piece_in_array(
            current_piece.piece,
            IVec2::ZERO,
            current_piece.rotation,
            &mut current_piece_board.0,
            Some(current_piece.get_color()),
        );
    }
}

fn update_piece_display_position(
    current_piece: Res<CurrentPiece>,
    mut current_piece_transform: Query<&mut Transform, With<CurrentPieceTransform>>,
) {
    let mut current_piece_transform = current_piece_transform.single_mut();

    current_piece_transform.translation.y = current_piece.position.y as f32;
    current_piece_transform.translation.x = current_piece.position.x as f32;
}

fn update(
    keys: Res<Input<KeyCode>>,
    mut current_piece: ResMut<CurrentPiece>,
    mut board: ResMut<Board>,
    mut timer: Local<Option<Timer>>,
    mut last_level: Local<usize>,
    level: Res<Level>,
    time: Res<Time>,
) {
    const BASE_TIME: f32 = 0.8;
    const MIN_TIME: f32 = 0.1;
    if timer.is_none() {
        *last_level = 0;
        *timer = Some(Timer::from_seconds(BASE_TIME, TimerMode::Repeating));
    } else if *last_level != level.0 || keys.just_released(KeyCode::Down) {
        let range = BASE_TIME - MIN_TIME;
        let level_percentage = level.0 as f32 / MAX_LEVEL as f32;
        let exponential_range = level_percentage * level_percentage;
        let interval = BASE_TIME - (range * exponential_range);
        let elapsed = timer.as_mut().unwrap().elapsed();
        *timer = Some(Timer::from_seconds(interval, TimerMode::Repeating));
        timer.as_mut().unwrap().set_elapsed(elapsed);
        *last_level = level.0;
    }
    if keys.just_pressed(KeyCode::Down) {
        let elapsed = timer.as_mut().unwrap().elapsed();
        *timer = Some(Timer::from_seconds(MIN_TIME, TimerMode::Repeating));
        timer.as_mut().unwrap().set_elapsed(elapsed);
    }
    if timer.as_mut().unwrap().tick(time.delta()).just_finished() {
        if !check_piece_obstructed(
            current_piece.piece,
            current_piece.position + IVec2::NEG_Y,
            current_piece.rotation,
            board
                .board
                .iter_mut()
                .map(|x| x.as_mut_slice())
                .collect::<Vec<_>>()
                .as_mut_slice(),
        ) {
            current_piece.position.y -= 1;
        } else {
            current_piece.placed = true;
        }
    }
}

fn sound_engine(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_with_settings(
        asset_server.load("sounds/Tetris.ogg"),
        PlaybackSettings {
            repeat: true,
            volume: 0.1,
            speed: 1.0,
        },
    );
}

#[derive(Resource)]
struct BackgroundImageHandle(Handle<Image>);

fn background(asset_server: Res<AssetServer>, mut handle: ResMut<BackgroundImageHandle>) {
    let texture: Handle<Image> = asset_server.load("textures/background.png");
    handle.0 = texture.clone();
}

fn background_sprite_creator(
    mut commands: Commands,
    assets: Res<Assets<Image>>,
    bg_handle: Res<BackgroundImageHandle>,
    mut ev_asset: EventReader<AssetEvent<Image>>,
) {
    for ev in ev_asset.iter() {
        match ev {
            AssetEvent::Created { handle } => {
                if *handle == bg_handle.0 {
                    let texture = assets.get(handle).unwrap();
                    let ar = texture.size().x as f32 / texture.size().y as f32;
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(ar * WORLD_SIZE.y, WORLD_SIZE.y)),
                            anchor: Anchor::BottomCenter,
                            ..Default::default()
                        },
                        texture: bg_handle.0.clone(),
                        transform: Transform {
                            translation: Vec3::new(WORLD_SIZE.x / 2., 0.0, 0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }
    }
}

fn diagnostic_system(keys: Res<Input<KeyCode>>, mut diag_state: ResMut<FpsCounter>) {
    if keys.just_pressed(KeyCode::F3) {
        match diag_state.is_enabled() {
            true => diag_state.disable(),
            false => diag_state.enable(),
        }
    }
}

#[derive(Resource)]
struct DisplayBoard(Board);

#[derive(Resource)]
struct CurrentPieceBoard(Board);

#[derive(Component)]
struct CurrentPieceTransform;

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::new(BOARD_SIZE.x as usize, BOARD_SIZE.y as usize))
            .insert_resource(DisplayBoard(Board::new(4 as usize, 4 as usize)))
            .insert_resource(CurrentPieceBoard(Board::new(4 as usize, 4 as usize)))
            .insert_resource(Bag::new())
            .insert_resource(Score(0))
            .insert_resource(Level(0))
            .insert_resource(BackgroundImageHandle(Handle::default()))
            .add_startup_system(setup_cam)
            .add_startup_system(setup_board)
            .add_startup_system(sound_engine)
            .add_startup_system(background)
            .add_system(background_sprite_creator)
            .add_system(render_score)
            .add_system(render_level)
            // .add_system(Board::update_sprite_visibility)
            .add_system(Board::update_board_sprites)
            // .add_system(Board::debug)
            .add_system(Board::assess_board)
            .add_system(place_piece)
            .add_system(keyboard_input)
            .add_system(update)
            .add_system(update_piece_display_position)
            .add_system(Board::update_sprite_colors)
            .add_system(diagnostic_system);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FpsCounterPlugin)
        .add_plugin(HelloPlugin)
        .run();
}
