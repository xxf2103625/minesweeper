//! 多关卡设计：一关 = 棋盘尺寸 + 雷数。关卡只读、跨平台复用。
//!
//! 难度递增：格子更多、雷密度更高。新增关卡只需往 [`LEVELS`] 追加一项。

use crate::board::LevelSpec;

/// 全部关卡（按难度排序）。索引即关卡号。
pub const LEVELS: &[LevelSpec] = &[
    LevelSpec {
        name: "初级 9×9",
        cols: 9,
        rows: 9,
        mines: 10,
    },
    LevelSpec {
        name: "中级 12×12",
        cols: 12,
        rows: 12,
        mines: 22,
    },
    LevelSpec {
        name: "高级 16×16",
        cols: 16,
        rows: 16,
        mines: 40,
    },
    LevelSpec {
        name: "专家 20×16",
        cols: 20,
        rows: 16,
        mines: 55,
    },
];

pub const FIRST_LEVEL: usize = 0;

/// 胜利后进入下一关（到顶则回到第一关循环）。
#[inline]
pub fn next_level(current: usize) -> usize {
    (current + 1) % LEVELS.len()
}

#[inline]
pub fn level(current: usize) -> LevelSpec {
    LEVELS[current.min(LEVELS.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_are_well_formed() {
        for l in LEVELS {
            assert!(l.cols >= 5 && l.rows >= 5);
            assert!(l.mines > 0);
            // 雷数必须小于总格数，否则永远无法胜利
            assert!(l.mines < l.cols * l.rows, "{}", l.name);
        }
    }

    #[test]
    fn difficulty_is_monotonic() {
        for w in LEVELS.windows(2) {
            let d0 = w[0].mines as f64 / (w[0].cols * w[0].rows) as f64;
            let d1 = w[1].mines as f64 / (w[1].cols * w[1].rows) as f64;
            assert!(d1 >= d0 - 1e-9, "density should not drop: {}", w[1].name);
        }
    }

    #[test]
    fn next_level_wraps_around() {
        assert_eq!(next_level(0), 1);
        assert_eq!(next_level(LEVELS.len() - 1), FIRST_LEVEL);
    }
}
