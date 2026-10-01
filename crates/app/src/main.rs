//! Bevy 2D 扫雷（多平台）。
//! 参考 https://bevy.org 与 https://dev.to/qongzi/bevy-minesweeper-introduction-4l7f
//!
//! 工程分层：
//! - `game-core`：纯逻辑（棋盘 / 多关卡 / 素材主题），无 bevy 依赖，可跨平台复用与单测。
//! - `platform` ：各平台（macOS / Windows / Android）适配（窗口、输入、字体路径）。
//! - `app`      ：bevy 应用层（本 crate），把上面两者接到一起渲染。

mod assets;
mod board;
mod input;
mod plugin;
mod render;
mod state;

use bevy::prelude::*;
use plugin::MinesweeperPlugin;

fn main() {
    let platform = platform::current();
    let (w, h) = platform.window_size;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy 扫雷".into(),
                        resolution: (w, h).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    // 只保留 error 及以上：屏蔽 parley/icu_segmenter 关于中日文
                    // 分词模型的无害提示（LineBreak::AnyCharacter 已绕开词切分），
                    // 以及 winit 退出时的 INFO 刷屏。
                    filter: "error".into(),
                    ..default()
                }),
        )
        .add_plugins(MinesweeperPlugin)
        .run();
}
