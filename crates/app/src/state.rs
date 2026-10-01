//! 应用层状态：把核心 [`Board`]、关卡进度、悬停、主题、渲染版本号包进 bevy 资源。

use bevy::prelude::*;
use game_core::{levels, Board, Theme};

/// 当前关卡索引（跨局保留，胜利后前进）。
#[derive(Resource)]
pub struct LevelProgress(pub usize);

/// 棋盘资源：包一层核心 Board，方便系统里 ResMut 访问。
#[derive(Resource, Clone)]
pub struct Game(pub Board);

impl Game {
    /// 依据关卡规格新建一局（重新布雷）。
    pub fn new_game(index: usize) -> Self {
        let spec = levels::level(index);
        let mut board = Board::new(&spec);
        board.regenerate();
        Self(board)
    }
}

/// 渲染版本号：每次重开/换关自增，供 sync_board 强制重画
/// （同规格重开时 generation 会复位到相同值，单靠它无法区分）。
#[derive(Resource, Default)]
pub struct Revision(pub u32);

/// 当前悬停的格子。
#[derive(Resource, Default)]
pub struct Hover(pub Option<Entity>);

/// 视觉主题（素材抽象的实例）。渲染层从这里取颜色/尺寸，不写死。
#[derive(Resource, Clone)]
pub struct AppTheme(pub Theme);

/// 加载好的字体源（中文字体），InfoText 与所有 CellLabel 共用。
#[derive(Resource, Clone)]
pub struct AppFont(pub bevy::text::FontSource);
