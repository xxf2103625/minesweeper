//! 素材适配：把核心层的 [`Theme`]/[`Rgb`] 与字体路径映射成 bevy 资源。
//! 这样核心层保持无 bevy 依赖，只有这里知道 bevy 的 `Color` / `Font` 长什么样。

use bevy::prelude::*;
use bevy::text::Font as BevyFont;
use game_core::{Rgb, Theme};

/// 线性 sRGB 颜色转换。
#[inline]
pub fn to_color(c: Rgb) -> Color {
    Color::srgb(c.0, c.1, c.2)
}

/// 加载字体：按 [`platform::available_font`] 给的候选路径逐个尝试，
/// 第一个能读进内存的注册进 `Assets<Font>` 并返回其 `FontSource`。
/// 都失败则返回默认（引擎内置字体，仅够渲染 ASCII）。
pub fn load_font(fonts: &mut ResMut<Assets<BevyFont>>) -> FontSource {
    if let Some(path) = platform::available_font() {
        if let Ok(data) = std::fs::read(path) {
            return FontSource::Handle(fonts.add(BevyFont::from_bytes(data)));
        }
    }
    FontSource::default()
}

/// 从主题派生一份可复用的渲染参数（字号等），避免每帧读字段。
#[derive(Clone, Copy)]
pub struct RenderParams {
    pub font_size: f32,
    pub cell_size: f32,
    pub step: f32,
}

impl From<&Theme> for RenderParams {
    fn from(t: &Theme) -> Self {
        Self {
            font_size: t.font_size,
            cell_size: t.cell_size,
            step: t.step(),
        }
    }
}
