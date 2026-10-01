//! 渲染同步：把核心 [`Board`] 状态映射成 bevy 组件外观。
//! 只在棋盘状态真正变化时写一次组件（用签名缓存去重），不每帧写。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use game_core::GameStatus;

use crate::assets::to_color;
use crate::board::{Cell, CellLabel, InfoText};
use crate::state::{AppTheme, Game, Hover, LevelProgress, Revision};

/// 一次渲染的输入签名：任一项变了才重画。
#[derive(PartialEq, Eq)]
pub struct Signature {
    /// 渲染版本号：重开/换关时 bump，保证同规格重开也能重画。
    revision: u32,
    generation: u32,
    hover: Option<(usize, usize)>,
    status: GameStatus,
    flags: usize,
    revealed: Vec<bool>,
    flagged: Vec<bool>,
    level: usize,
}

/// 格子标签查询：子实体（`ChildOf`）+ 文本三件套，排除信息栏。
/// 单独起别名，避免系统签名里塞四元组的四元组。
type Labels<'w, 's> = Query<
    'w,
    's,
    (
        &'static ChildOf,
        &'static mut Text2d,
        &'static mut TextColor,
        &'static mut Visibility,
    ),
    (With<CellLabel>, Without<InfoText>),
>;

/// `sync_board` 的全部输入：只读状态 + 外观查询 + 上一帧签名。
/// 合成一个 [`SystemParam`]，让系统签名保持在一行内。
#[derive(SystemParam)]
pub struct BoardView<'w, 's> {
    game: Res<'w, Game>,
    hover: Res<'w, Hover>,
    progress: Res<'w, LevelProgress>,
    revision: Res<'w, Revision>,
    theme: Res<'w, AppTheme>,
    cells: Query<'w, 's, (Entity, &'static Cell)>,
    sprites: Query<'w, 's, &'static mut Sprite, With<Cell>>,
    labels: Labels<'w, 's>,
    /// 上一帧的渲染签名（系统私有状态，随 `Local` 跨帧保留）。
    signature: Local<'s, Option<Signature>>,
}

/// 格子外观同步（底色 / 翻开色 / 悬停高亮 / 数字 / 地雷 / 旗）。
pub fn sync_board(mut view: BoardView<'_, '_>) {
    let board = &view.game.0;

    // entity -> (col,row)，供子标签反查所属格子
    let cell_of: std::collections::HashMap<Entity, (usize, usize)> = view
        .cells
        .iter()
        .map(|(e, c)| (e, (c.col, c.row)))
        .collect();
    let hover_cell: Option<(usize, usize)> = view.hover.0.and_then(|e| cell_of.get(&e).copied());

    let sig = Signature {
        revision: view.revision.0,
        generation: board.generation,
        hover: hover_cell,
        status: board.status,
        flags: board.flags(),
        revealed: board.revealed.clone(),
        flagged: board.flagged.clone(),
        level: view.progress.0,
    };
    if view.signature.as_ref() == Some(&sig) {
        return;
    }
    *view.signature = Some(sig);

    // 底色
    for (entity, cell) in view.cells.iter() {
        let i = board.idx(cell.col, cell.row);
        let is_hover = hover_cell == Some((cell.col, cell.row));
        if let Ok(mut sprite) = view.sprites.get_mut(entity) {
            let color = if board.status == GameStatus::Lost && board.mines[i] && board.revealed[i] {
                view.theme.0.mine_hit
            } else if board.revealed[i] {
                view.theme.0.revealed
            } else if is_hover {
                view.theme.0.hover
            } else if board.flagged[i] {
                view.theme.0.flag
            } else {
                view.theme.0.cell
            };
            sprite.color = to_color(color);
        }
    }

    // 子标签（数字 / 地雷 / 旗）
    for (parent, mut text, mut tcolor, mut vis) in view.labels.iter_mut() {
        let Some(&(col, row)) = cell_of.get(&parent.parent()) else {
            continue;
        };
        let i = board.idx(col, row);
        if !board.revealed[i] {
            if board.flagged[i] {
                *text = Text2d::new("旗".to_string());
                *tcolor = TextColor(to_color(view.theme.0.flag));
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
            continue;
        }
        if board.mines[i] {
            *text = Text2d::new("*".to_string());
            *tcolor = TextColor(to_color(view.theme.0.mine));
            *vis = Visibility::Visible;
        } else if board.adjacent[i] > 0 {
            let n = board.adjacent[i];
            *text = Text2d::new(n.to_string());
            *tcolor = TextColor(to_color(view.theme.0.number_color(n)));
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// 顶部信息栏文本。
pub fn update_info(
    game: Res<Game>,
    progress: Res<LevelProgress>,
    mut info: Query<&mut Text2d, (With<InfoText>, Without<CellLabel>)>,
) {
    let board = &game.0;
    let flags = board.flags();
    let level_no = progress.0 + 1;
    let total = game_core::levels::LEVELS.len();
    let msg = match board.status {
        GameStatus::Playing => format!(
            "剩余地雷: {}   第 {level_no}/{total} 关",
            board.mines_total.saturating_sub(flags),
        ),
        GameStatus::Won => {
            let next = game_core::levels::next_level(progress.0);
            format!(
                "过关！点击进入下一关：{}",
                game_core::levels::LEVELS[next].name
            )
        }
        GameStatus::Lost => "踩雷了！点击重新开始本关".to_string(),
    };
    if let Ok(mut text) = info.single_mut() {
        *text = Text2d::new(msg);
    }
}
