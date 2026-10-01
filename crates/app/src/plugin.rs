//! 插件装配：把资源初始化、输入、渲染同步挂到 bevy App 上。

use bevy::prelude::*;
use game_core::{levels, Theme};

use crate::assets::{load_font, to_color, RenderParams};
use crate::board::{despawn_board, spawn_board, BoardEntity, InfoText};
use crate::input::{hover_highlight, mouse_click};
use crate::render::{sync_board, update_info};
use crate::state::{AppFont, AppTheme, Game, Hover, LevelProgress, Revision};

pub struct MinesweeperPlugin;

impl Plugin for MinesweeperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hover>()
            .init_resource::<Revision>()
            .insert_resource(AppTheme(Theme::default()))
            .insert_resource(LevelProgress(levels::FIRST_LEVEL))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    // 输入在前，渲染同步在后，保证同一帧内“状态→外观”一致
                    (mouse_click, hover_highlight).chain(),
                    (on_level_change, sync_board, update_info).chain(),
                )
                    .chain(),
            );
    }
}

/// 启动：相机 + 字体 + 首关棋盘 + 信息栏。
fn setup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<bevy::text::Font>>,
    theme: Res<AppTheme>,
) {
    commands.spawn(Camera2d);

    let font = load_font(&mut fonts);
    commands.insert_resource(AppFont(font.clone()));

    let game = Game::new_game(levels::FIRST_LEVEL);
    commands.insert_resource(game.clone());
    spawn_board(&mut commands, &game, &theme, &font);

    let params = RenderParams::from(&theme.0);
    let info_y = (game.0.rows as f32 * params.step) / 2.0 + theme.0.board_offset_y.abs() + 40.0;
    commands.spawn((
        InfoText,
        Text2d::new(String::new()),
        TextFont {
            font,
            font_size: FontSize::Px(params.font_size),
            ..default()
        },
        TextColor(to_color(theme.0.text)),
        TextLayout {
            justify: Justify::Center,
            // 中文无空格分词，按字符断行，避免整词被折行
            linebreak: LineBreak::AnyCharacter,
        },
        Transform::from_xyz(0.0, info_y, 10.0),
    ));
}

/// 关卡尺寸变化（进入不同规格的新关卡）时重建整块棋盘实体。
/// 同规格重开由 revision bump 触发 sync_board 重画，不在此处理。
fn on_level_change(
    mut commands: Commands,
    game: Res<Game>,
    theme: Res<AppTheme>,
    font: Res<AppFont>,
    board_entities: Query<Entity, With<BoardEntity>>,
    mut last_size: Local<Option<(usize, usize)>>,
) {
    let cur = (game.0.cols, game.0.rows);
    if *last_size == Some(cur) {
        return;
    }
    // 首帧：setup 已生成实体，仅记录尺寸；之后尺寸变化才重建
    if last_size.is_some() {
        despawn_board(&mut commands, &board_entities);
        spawn_board(&mut commands, &game, &theme, &font.0);
    }
    *last_size = Some(cur);
}
