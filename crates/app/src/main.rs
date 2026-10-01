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
                    // 只保留 error 及以上：屏蔽 winit 退出时的 INFO 刷屏。
                    //
                    // icu_provider=off 是必需的：parley 排版每条 CJK 文本都会经
                    // icu_segmenter 的 CJK 断行字典取一次数据，取不到就 warn，
                    // debug 构建下约 50 行/秒刷屏（release 不打印，但同样无解）。
                    // 前提是根 Cargo.toml 给 icu_provider 开了 logging feature，
                    // 让它走 log→tracing 管线而不是 debug 构建下的裸 eprintln。
                    // 详见 docs/bevy-engine.md §7。
                    filter: "error,icu_provider=off".into(),
                    ..default()
                }),
        )
        .add_plugins(MinesweeperPlugin)
        .run();
}
