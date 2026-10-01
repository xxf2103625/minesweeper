//! 素材抽象：把配色、字体、尺寸等“皮肤”数据从渲染逻辑里剥离出来。
//!
//! 核心层用纯数据（[`Rgb`] / f32）描述素材，不依赖 bevy 的任何类型；
//! 渲染层负责把 [`Theme`] 映射成 `Color`、`FontSize` 等。换肤 = 换一个 [`Theme`]。

/// 线性 sRGB 颜色（无 bevy 依赖）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub f32, pub f32, pub f32);

impl Rgb {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self(r, g, b)
    }
}

/// 一整套视觉素材。
#[derive(Clone, Debug)]
pub struct Theme {
    pub cell_size: f32,
    pub spacing: f32,
    /// 棋盘整体在屏幕上的纵向偏移（给顶部信息栏留位置）。
    pub board_offset_y: f32,
    pub font_size: f32,

    pub bg: Rgb,
    pub cell: Rgb,
    pub hover: Rgb,
    pub revealed: Rgb,
    pub flag: Rgb,
    pub mine_hit: Rgb,
    pub mine: Rgb,
    pub text: Rgb,
    /// 数字 1..=8 的配色（下标 0 不用）。
    pub numbers: [Rgb; 9],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            cell_size: 48.0,
            spacing: 4.0,
            board_offset_y: -30.0,
            font_size: 26.0,
            bg: Rgb::new(0.13, 0.14, 0.18),
            cell: Rgb::new(0.36, 0.40, 0.48),
            hover: Rgb::new(0.50, 0.55, 0.65),
            revealed: Rgb::new(0.20, 0.22, 0.27),
            flag: Rgb::new(0.95, 0.72, 0.20),
            mine_hit: Rgb::new(0.90, 0.25, 0.25),
            mine: Rgb::new(0.60, 0.25, 0.25),
            text: Rgb::new(0.92, 0.92, 0.95),
            numbers: [
                Rgb::new(0.9, 0.9, 0.9), // 0 占位
                Rgb::new(0.40, 0.65, 1.00),
                Rgb::new(0.45, 0.85, 0.45),
                Rgb::new(1.00, 0.40, 0.40),
                Rgb::new(0.70, 0.45, 0.95),
                Rgb::new(1.00, 0.65, 0.25),
                Rgb::new(0.35, 0.85, 0.85),
                Rgb::new(0.95, 0.95, 0.95),
                Rgb::new(0.95, 0.40, 0.80),
            ],
        }
    }
}

impl Theme {
    #[inline]
    pub fn step(&self) -> f32 {
        self.cell_size + self.spacing
    }

    #[inline]
    pub fn number_color(&self, n: u8) -> Rgb {
        self.numbers[(n as usize).min(8)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_is_cell_plus_spacing() {
        let t = Theme::default();
        assert_eq!(t.step(), t.cell_size + t.spacing);
    }

    #[test]
    fn number_color_clamps() {
        let t = Theme::default();
        assert_eq!(t.number_color(1), t.numbers[1]);
        assert_eq!(t.number_color(99), t.numbers[8]);
    }
}
