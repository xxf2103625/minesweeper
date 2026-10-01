//! 棋盘实体：组件定义 + 按关卡规格生成/重建整块棋盘。

use bevy::prelude::*;
use game_core::Board;

use crate::assets::{to_color, RenderParams};
use crate::state::{AppTheme, Game};

/// 棋盘上所有实体的公共标记（换关卡时整体清理）。
#[derive(Component)]
pub struct BoardEntity;

/// 格子本体。
#[derive(Component)]
pub struct Cell {
    pub col: usize,
    pub row: usize,
}

/// 格子内显示数字/地雷的文本（格子的子实体）。
#[derive(Component)]
pub struct CellLabel;

/// 顶部信息栏文本。
#[derive(Component)]
pub struct InfoText;

/// 生成一整个关卡的棋盘实体。初始外观即正确（未翻开 = 格子底色），
/// 后续视觉同步交给 render::sync_board。
pub fn spawn_board(
    commands: &mut Commands,
    game: &Game,
    theme: &AppTheme,
    font: &FontSource,
) {
    let board: &Board = &game.0;
    let params = RenderParams::from(&theme.0);

    let board_w = board.cols as f32 * params.step;
    let board_h = board.rows as f32 * params.step;
    commands.spawn((
        BoardEntity,
        Sprite::from_color(
            to_color(theme.0.bg),
            Vec2::new(board_w + 24.0, board_h + 130.0),
        ),
    ));

    for row in 0..board.rows {
        for col in 0..board.cols {
            let pos = game_core::cell_center(&theme.0, board.cols, board.rows, col, row);
            let cell = commands
                .spawn((
                    BoardEntity,
                    Cell { col, row },
                    Sprite::from_color(to_color(theme.0.cell), Vec2::splat(params.cell_size)),
                    Transform::from_xyz(pos[0], pos[1], 1.0),
                ))
                .id();
            commands.spawn((
                BoardEntity,
                CellLabel,
                Text2d::new(String::new()),
                TextFont {
                    font: font.clone(),
                    font_size: FontSize::Px(params.font_size),
                    ..default()
                },
                TextColor(to_color(theme.0.text)),
                Visibility::Hidden,
                TextLayout {
                    justify: Justify::Center,
                    linebreak: LineBreak::AnyCharacter,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1.0), // 相对父格子的局部坐标
                ChildOf(cell),
            ));
        }
    }
}

/// 删除旧棋盘（含背景、格子）。bevy 0.19 里 despawn 父实体会自动级联删除子实体（标签）。
pub fn despawn_board(commands: &mut Commands, entities: &Query<Entity, With<BoardEntity>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
