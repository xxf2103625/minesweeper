//! Bevy 2D 扫雷（macOS）
//! 参考 https://bevy.org 与 https://dev.to/qongzi/bevy-minesweeper-introduction-4l7f
//! 玩法：左键翻开，右键插旗/取消旗；结束后点击任意位置重新开始。
//! 依赖 bevy 0.19 / rand 0.10（当前最新稳定 API）。

use std::collections::HashMap;

use bevy::prelude::*;
use rand::seq::SliceRandom;

// ---------- 常量 ----------
const BOARD_COLS: usize = 9;
const BOARD_ROWS: usize = 9;
const MINE_COUNT: usize = 10;
const CELL_SIZE: f32 = 48.0;
const STEP: f32 = CELL_SIZE + 4.0;
const BOARD_OFFSET_Y: f32 = -30.0;

const COLOR_BG: Color = Color::srgb(0.13, 0.14, 0.18);
const COLOR_CELL: Color = Color::srgb(0.36, 0.40, 0.48);
const COLOR_HOVER: Color = Color::srgb(0.50, 0.55, 0.65);
const COLOR_REVEALED: Color = Color::srgb(0.20, 0.22, 0.27);
const COLOR_FLAG: Color = Color::srgb(0.95, 0.72, 0.20);
const COLOR_MINE_HIT: Color = Color::srgb(0.90, 0.25, 0.25);
const COLOR_MINE: Color = Color::srgb(0.60, 0.25, 0.25);


fn cell_to_world(col: usize, row: usize) -> Vec2 {
    let x = (col as f32 - (BOARD_COLS - 1) as f32 / 2.0) * STEP;
    let y = ((BOARD_ROWS - 1) as f32 / 2.0 - row as f32) * STEP + BOARD_OFFSET_Y;
    Vec2::new(x, y)
}

fn number_color(n: u8) -> Color {
    match n {
        1 => Color::srgb(0.40, 0.65, 1.00),
        2 => Color::srgb(0.45, 0.85, 0.45),
        3 => Color::srgb(1.00, 0.40, 0.40),
        4 => Color::srgb(0.70, 0.45, 0.95),
        5 => Color::srgb(1.00, 0.65, 0.25),
        6 => Color::srgb(0.35, 0.85, 0.85),
        7 => Color::srgb(0.95, 0.95, 0.95),
        _ => Color::srgb(0.95, 0.40, 0.80),
    }
}

fn neighbors(col: usize, row: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(8);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = col as i32 + dx;
            let ny = row as i32 + dy;
            if nx >= 0 && nx < BOARD_COLS as i32 && ny >= 0 && ny < BOARD_ROWS as i32 {
                out.push((nx as usize, ny as usize));
            }
        }
    }
    out
}

// ---------- 组件 ----------
#[derive(Component)]
struct Cell {
    col: usize,
    row: usize,
}

/// 格子里显示数字/地雷的文本（格子的子实体）
#[derive(Component)]
struct CellLabel;

/// 顶部信息栏文本
#[derive(Component)]
struct InfoText;

// ---------- 资源 ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameStatus {
    Playing,
    Won,
    Lost,
}

#[derive(Resource)]
struct BoardState {
    mines: Vec<bool>,
    adjacent: Vec<u8>,
    revealed: Vec<bool>,
    flagged: Vec<bool>,
    status: GameStatus,
    /// 每次重开自增，用于让渲染缓存失效
    generation: u32,
}

impl Default for BoardState {
    fn default() -> Self {
        let total = BOARD_COLS * BOARD_ROWS;
        Self {
            mines: vec![false; total],
            adjacent: vec![0; total],
            revealed: vec![false; total],
            flagged: vec![false; total],
            status: GameStatus::Playing,
            generation: 0,
        }
    }
}

impl BoardState {
    fn idx(&self, col: usize, row: usize) -> usize {
        row * BOARD_COLS + col
    }

    fn regenerate(&mut self) {
        let total = BOARD_COLS * BOARD_ROWS;
        let mut indices: Vec<usize> = (0..total).collect();
        indices.shuffle(&mut rand::rng());

        self.mines = vec![false; total];
        for &i in &indices[..MINE_COUNT] {
            self.mines[i] = true;
        }
        let mut counts = vec![0u8; total];
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                counts[self.idx(col, row)] = neighbors(col, row)
                    .iter()
                    .filter(|&&(c, r)| self.mines[r * BOARD_COLS + c])
                    .count() as u8;
            }
        }
        self.adjacent = counts;
        self.revealed = vec![false; total];
        self.flagged = vec![false; total];
        self.status = GameStatus::Playing;
        self.generation += 1;
    }
}

/// 当前悬停的格子
#[derive(Resource, Default)]
struct Hover(Option<Entity>);

// ---------- 初始化 ----------
/// 加载 macOS 自带简体中文黑体（Hiragino Sans GB，非日文字体）
fn load_cjk_font(fonts: &mut ResMut<Assets<bevy::text::Font>>) -> Option<Handle<bevy::text::Font>> {
    let path = "/System/Library/Fonts/Hiragino Sans GB.ttc";
    let data = std::fs::read(path).ok()?;
    Some(fonts.add(bevy::text::Font::from_bytes(data)))
}

fn setup(
    mut commands: Commands,
    mut board: ResMut<BoardState>,
    mut fonts: ResMut<Assets<bevy::text::Font>>,
) {
    commands.spawn(Camera2d);

    let board_w = BOARD_COLS as f32 * STEP;
    let board_h = BOARD_ROWS as f32 * STEP;
    commands.spawn(Sprite::from_color(
        COLOR_BG,
        Vec2::new(board_w + 24.0, board_h + 130.0),
    ));

    board.regenerate();

    // 加载一次中文字体，InfoText 与所有 CellLabel 复用同一 Handle
    let cjk = load_cjk_font(&mut fonts);
    let cjk_src = match cjk {
        Some(h) => FontSource::Handle(h),
        None => FontSource::default(),
    };

    commands.spawn((
        InfoText,
        Text2d::new(""),
        TextFont {
            font: cjk_src.clone(),
            font_size: FontSize::Px(26.0),
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.92, 0.95)),
        TextLayout {
            justify: Justify::Center,
            // 中文无空格分词，按字符断行可跳过 ICU4X 词切分（消除那条警告）
            linebreak: LineBreak::AnyCharacter,
            ..default()
        },
        Transform::from_xyz(0.0, board_h / 2.0 + BOARD_OFFSET_Y + 40.0, 10.0),
    ));

    for row in 0..BOARD_ROWS {
        for col in 0..BOARD_COLS {
            let pos = cell_to_world(col, row);
            let cell = commands
                .spawn((
                    Cell { col, row },
                    Sprite::from_color(COLOR_CELL, Vec2::splat(CELL_SIZE)),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                ))
                .id();
            commands.spawn((
                CellLabel,
                Text2d::new(""),
                TextFont {
                    font: cjk_src.clone(),
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Visibility::Hidden,
                TextLayout {
                    justify: Justify::Center,
                    linebreak: LineBreak::AnyCharacter,
                    ..default()
                },
                // 子标签用相对格子的局部坐标，GlobalTransform 由父格子传播
                Transform::from_xyz(0.0, 0.0, 1.0),
                ChildOf(cell),
            ));
        }
    }
}

// ---------- 逻辑 ----------
fn reveal(board: &mut BoardState, col: usize, row: usize) {
    let i = board.idx(col, row);
    if board.revealed[i] || board.flagged[i] {
        return;
    }
    board.revealed[i] = true;

    if board.mines[i] {
        board.status = GameStatus::Lost;
        return;
    }

    // 空白格：泛洪展开
    if board.adjacent[i] == 0 {
        for (nc, nr) in neighbors(col, row) {
            let ni = board.idx(nc, nr);
            if !board.revealed[ni] && !board.flagged[ni] && !board.mines[ni] {
                reveal(board, nc, nr);
            }
        }
    }

    let safe_total = BOARD_COLS * BOARD_ROWS - MINE_COUNT;
    if board.revealed.iter().filter(|&&r| r).count() >= safe_total {
        board.status = GameStatus::Won;
    }
}

fn mouse_click(
    mut board: ResMut<BoardState>,
    cells: Query<&Cell, With<Cell>>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let left = buttons.just_pressed(MouseButton::Left);
    let right = buttons.just_pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    if board.status != GameStatus::Playing {
        board.regenerate();
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok((camera, cam_tf)) = cameras.single() else { return };
    let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) else { return };

    for cell in cells.iter() {
        let pos = cell_to_world(cell.col, cell.row);
        if (world.x - pos.x).abs() > CELL_SIZE / 2.0
            || (world.y - pos.y).abs() > CELL_SIZE / 2.0
        {
            continue;
        }
        let i = board.idx(cell.col, cell.row);
        if board.revealed[i] {
            break;
        }
        if right {
            board.flagged[i] = !board.flagged[i];
        } else {
            reveal(&mut board, cell.col, cell.row);
        }
        break;
    }
}

fn hover_highlight(
    mut hover: ResMut<Hover>,
    cells: Query<(Entity, &Cell), With<Cell>>,
    board: Res<BoardState>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let mut target: Option<Entity> = None;
    if board.status == GameStatus::Playing {
        if let (Ok(window), Ok((camera, cam_tf))) = (windows.single(), cameras.single()) {
            if let Some(cursor) = window.cursor_position() {
                if let Ok(world) = camera.viewport_to_world_2d(cam_tf, cursor) {
                    for (entity, cell) in cells.iter() {
                        let pos = cell_to_world(cell.col, cell.row);
                        if (world.x - pos.x).abs() <= CELL_SIZE / 2.0
                            && (world.y - pos.y).abs() <= CELL_SIZE / 2.0
                        {
                            let i = board.idx(cell.col, cell.row);
                            if !board.revealed[i] && !board.flagged[i] {
                                target = Some(entity);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
    hover.0 = target;
}

// ---------- 渲染同步 ----------
/// 格子外观只在棋盘状态变化时同步一次，避免每帧写组件。
fn render_board(
    board: Res<BoardState>,
    hover: Res<Hover>,
    cells: Query<(Entity, &Cell), With<Cell>>,
    mut sprites: Query<&mut Sprite, With<Cell>>,
    mut labels: Query<
        (
            &ChildOf,
            &mut Text2d,
            &mut TextColor,
            &mut Visibility,
        ),
        (With<CellLabel>, Without<InfoText>),
    >,
    mut info: Query<&mut Text2d, (With<InfoText>, Without<CellLabel>)>,
    mut signature: Local<Option<(u32, GameStatus, usize, Vec<bool>, Vec<bool>, Option<Entity>)>>,
) {
    let flags = board.flagged.iter().filter(|&&f| f).count();
    let sig = (
        board.generation,
        board.status,
        flags,
        board.revealed.clone(),
        board.flagged.clone(),
        hover.0,
    );
    if signature.as_ref() == Some(&sig) {
        return;
    }
    *signature = Some(sig);

    // 格子底色
    let mut cell_of: HashMap<Entity, (usize, usize)> =
        HashMap::with_capacity(BOARD_COLS * BOARD_ROWS);
    for (entity, cell) in cells.iter() {
        cell_of.insert(entity, (cell.col, cell.row));
        let i = board.idx(cell.col, cell.row);
        let color = if board.revealed[i] && board.mines[i] {
            COLOR_MINE_HIT
        } else if board.revealed[i] {
            COLOR_REVEALED
        } else if board.flagged[i] {
            COLOR_FLAG
        } else if board.status != GameStatus::Playing && board.mines[i] {
            COLOR_MINE
        } else if hover.0 == Some(entity) {
            COLOR_HOVER
        } else {
            COLOR_CELL
        };
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            sprite.color = color;
        }
    }

    // 数字标签
    for (parent, mut text, mut color, mut vis) in labels.iter_mut() {
        let Some(&(col, row)) = cell_of.get(&parent.parent()) else {
            continue;
        };
        let i = board.idx(col, row);
        if board.revealed[i] {
            if board.mines[i] {
                *text = Text2d::new("*");
                *color = TextColor(Color::srgb(1.0, 1.0, 1.0));
                *vis = Visibility::Visible;
            } else if board.adjacent[i] > 0 {
                let n = board.adjacent[i];
                *text = Text2d::new(n.to_string());
                *color = TextColor(number_color(n));
                *vis = Visibility::Visible;
            } else {
                *text = Text2d::new(String::new());
                *vis = Visibility::Hidden;
            }
        } else if board.status != GameStatus::Playing && board.mines[i] {
            *text = Text2d::new("*");
            *color = TextColor(Color::srgb(1.0, 1.0, 1.0));
            *vis = Visibility::Visible;
        } else {
            *text = Text2d::new(String::new());
            *vis = Visibility::Hidden;
        }
    }

    // 信息栏
    let msg = match board.status {
        GameStatus::Playing => format!("剩余地雷: {}", MINE_COUNT.saturating_sub(flags)),
        GameStatus::Won => "你赢了！点击任意位置重新开始".to_string(),
        GameStatus::Lost => "踩雷了！点击任意位置重新开始".to_string(),
    };
    for mut text in info.iter_mut() {
        *text = Text2d::new(msg.clone());
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy 扫雷".into(),
                        resolution: (520u32, 620u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    // 只保留 error 及以上：屏蔽 ICU4X 对 CJK 分词的无害警告
                    // （文字为单行、用系统字体渲染，不受分词影响）和 winit 退出刷屏
                    filter: "warn".into(),
                    ..default()
                }),
        )
        .init_resource::<BoardState>()
        .init_resource::<Hover>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (mouse_click, hover_highlight, render_board).chain(),
        )
        .run();
}

// ---------- 测试 ----------
#[cfg(test)]
mod tests {
    use super::*;

    /// 手工构造棋盘：给定雷位列表
    fn board_with(mines: &[(usize, usize)]) -> BoardState {
        let total = BOARD_COLS * BOARD_ROWS;
        let mut b = BoardState {
            mines: vec![false; total],
            adjacent: vec![0; total],
            revealed: vec![false; total],
            flagged: vec![false; total],
            status: GameStatus::Playing,
            generation: 0,
        };
        for &(c, r) in mines {
            let i = r * BOARD_COLS + c;
            b.mines[i] = true;
        }
        let mut counts = vec![0u8; total];
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                let n = neighbors(c, r)
                    .iter()
                    .filter(|&&(c2, r2)| b.mines[r2 * BOARD_COLS + c2])
                    .count() as u8;
                counts[r * BOARD_COLS + c] = n;
            }
        }
        b.adjacent = counts;
        b
    }

    #[test]
    fn regenerate_places_exact_mine_count() {
        let mut b = BoardState::default();
        for _ in 0..50 {
            b.regenerate();
            assert_eq!(b.mines.iter().filter(|&&m| m).count(), MINE_COUNT);
            assert_eq!(b.status, GameStatus::Playing);
            assert!(b.revealed.iter().all(|&r| !r));
            assert!(b.flagged.iter().all(|&f| !f));
        }
    }

    #[test]
    fn corner_has_three_neighbors() {
        assert_eq!(neighbors(0, 0).len(), 3);
        assert_eq!(neighbors(8, 8).len(), 3);
        assert_eq!(neighbors(4, 4).len(), 8);
        assert_eq!(neighbors(0, 4).len(), 5);
        // 全部在界内
        for (c, r) in neighbors(0, 0) {
            assert!(c < BOARD_COLS && r < BOARD_ROWS);
        }
    }

    #[test]
    fn adjacent_counts_are_correct() {
        // 3x3 区域中心一个雷：周围 8 格都应为 1
        let b = board_with(&[(4, 4)]);
        for (c, r) in neighbors(4, 4) {
            assert_eq!(b.adjacent[b.idx(c, r)], 1, "cell ({c},{r})");
        }
        assert_eq!(b.adjacent[b.idx(0, 0)], 0);
    }

    #[test]
    fn clicking_mine_loses() {
        let mut b = board_with(&[(2, 3)]);
        reveal(&mut b, 2, 3);
        assert_eq!(b.status, GameStatus::Lost);
        assert!(b.revealed[b.idx(2, 3)]);
    }

    #[test]
    fn empty_cell_flood_fills_until_numbers() {
        // 雷全在最右列（加 (7,8)），左侧全是空白格，(0,4) 泛洪应展开大片区域
        let mines = [
            (8, 0), (8, 1), (8, 2), (8, 3), (8, 4), (8, 5), (8, 6), (8, 7), (8, 8), (7, 8),
        ];
        let mut b = board_with(&mines);
        assert_eq!(b.adjacent[b.idx(0, 4)], 0, "precondition: (0,4) is empty");
        reveal(&mut b, 0, 4);
        assert!(b.revealed[b.idx(0, 4)]);
        // 泛洪应展开到同侧空白区的远处格子
        assert!(b.revealed[b.idx(5, 1)], "flood should reach (5,1)");
        // 展开数量应相当大（左侧空白区约 60 格）
        let count = b.revealed.iter().filter(|&&r| r).count();
        assert!(count > 30, "flood only revealed {count} cells");
        // 泛洪不应翻开任何雷
        for &(c, r) in &mines {
            assert!(!b.revealed[b.idx(c, r)], "mine ({c},{r}) got revealed");
        }
        // 此布局下泛洪恰好翻开全部 71 个安全格，直接获胜
        assert_eq!(count, BOARD_COLS * BOARD_ROWS - MINE_COUNT);
        assert_eq!(b.status, GameStatus::Won);
    }

    #[test]
    fn flagged_cell_is_protected() {
        let mut b = board_with(&[(1, 1)]);
        let i = b.idx(1, 1);
        b.flagged[i] = true;
        reveal(&mut b, 1, 1);
        assert!(!b.revealed[i]);
        assert_eq!(b.status, GameStatus::Playing);
    }

    #[test]
    fn revealing_all_safe_cells_wins() {
        let mut b = board_with(&[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), (8, 8), (0, 8)]);
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if !b.mines[b.idx(c, r)] {
                    reveal(&mut b, c, r);
                }
                if b.status != GameStatus::Playing {
                    break;
                }
            }
            if b.status != GameStatus::Playing {
                break;
            }
        }
        assert_eq!(b.status, GameStatus::Won);
    }

    #[test]
    fn revealed_mine_does_not_flood() {
        // 数字格（非空白）只翻开自己
        let mut b = board_with(&[(1, 0)]);
        reveal(&mut b, 0, 0); // adjacent=1，不应泛洪
        assert!(b.revealed[b.idx(0, 0)]);
        assert!(!b.revealed[b.idx(5, 5)]);
        assert_eq!(b.status, GameStatus::Playing);
    }

    #[test]
    fn cell_to_world_centers_board() {
        let center = cell_to_world(BOARD_COLS / 2, BOARD_ROWS / 2);
        assert!(center.x.abs() < STEP);
        let topleft = cell_to_world(0, 0);
        let bottomright = cell_to_world(BOARD_COLS - 1, BOARD_ROWS - 1);
        assert!(topleft.x < bottomright.x);
        assert!(topleft.y > bottomright.y); // y 轴向下
    }
}
