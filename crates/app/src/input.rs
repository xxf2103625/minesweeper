//! 输入系统：桌面鼠标（左键翻开 / 右键插旗）。
//! 结束后点击：胜利→进入下一关，失败→重开本关。
//! 触摸端的长按插旗等可在后续按 platform::InputMode 扩展。

use bevy::prelude::*;
use game_core::{levels, GameStatus};

use crate::board::Cell;
use crate::state::{AppTheme, Game, Hover, LevelProgress, Revision};

/// 把光标世界坐标换算成命中的格子 (col,row)。
fn pick_cell(world: Vec2, cols: usize, rows: usize, theme: &game_core::Theme) -> Option<(usize, usize)> {
    let half = theme.cell_size / 2.0;
    for row in 0..rows {
        for col in 0..cols {
            let c = game_core::cell_center(theme, cols, rows, col, row);
            if (world.x - c[0]).abs() <= half && (world.y - c[1]).abs() <= half {
                return Some((col, row));
            }
        }
    }
    None
}

fn cursor_world(
    windows: &Query<&Window>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.single().ok()?;
    let cursor = window.cursor_position()?;
    let (camera, cam_tf) = cameras.single().ok()?;
    camera.viewport_to_world_2d(cam_tf, cursor).ok()
}

pub fn mouse_click(
    mut game: ResMut<Game>,
    mut progress: ResMut<LevelProgress>,
    mut revision: ResMut<Revision>,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    theme: Res<AppTheme>,
) {
    let left = buttons.just_pressed(MouseButton::Left);
    let right = buttons.just_pressed(MouseButton::Right);
    if !left && !right {
        return;
    }

    if game.0.status != GameStatus::Playing {
        // 结束：胜利进下一关，失败重开本关。重开/换关都 bump revision 强制重画。
        if game.0.status == GameStatus::Won {
            progress.0 = levels::next_level(progress.0);
        }
        game.0 = Game::new_game(progress.0).0;
        revision.0 += 1;
        return;
    }

    let Some(world) = cursor_world(&windows, &cameras) else {
        return;
    };
    let Some((col, row)) = pick_cell(world, game.0.cols, game.0.rows, &theme.0) else {
        return;
    };
    if game.0.revealed[game.0.idx(col, row)] {
        return;
    }
    if right {
        game.0.toggle_flag(col, row);
    } else {
        game.0.reveal(col, row);
    }
}

pub fn hover_highlight(
    mut hover: ResMut<Hover>,
    cells: Query<(Entity, &Cell)>,
    game: Res<Game>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    theme: Res<AppTheme>,
) {
    let mut target: Option<Entity> = None;
    if game.0.status == GameStatus::Playing {
        if let Some(world) = cursor_world(&windows, &cameras) {
            if let Some((col, row)) = pick_cell(world, game.0.cols, game.0.rows, &theme.0) {
                let i = game.0.idx(col, row);
                if !game.0.revealed[i] && !game.0.flagged[i] {
                    for (entity, cell) in cells.iter() {
                        if cell.col == col && cell.row == row {
                            target = Some(entity);
                            break;
                        }
                    }
                }
            }
        }
    }
    hover.0 = target;
}
