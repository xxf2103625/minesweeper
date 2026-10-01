//! # game-core
//!
//! 扫雷的**纯逻辑核心**：棋盘状态、多关卡、素材抽象（主题）。
//! 只依赖 `rand`，**不依赖 bevy**，因此可在 macOS / Windows / Android / 无头测试中复用。
//!
//! 渲染层（`crates/app`）负责把这里的 [`Board`]、[`Theme`]、[`levels`] 映射成 bevy 组件。

pub mod board;
pub mod levels;
pub mod theme;

pub use board::{Board, GameStatus, LevelSpec};
pub use theme::{Rgb, Theme};

/// 布局换算：把格子 (col,row) 换算成世界坐标中心。
/// 放在核心层是为了可测试、且各渲染后端保持一致。
pub fn cell_center(theme: &Theme, cols: usize, rows: usize, col: usize, row: usize) -> [f32; 2] {
    let step = theme.step();
    let x = (col as f32 - (cols - 1) as f32 / 2.0) * step;
    let y = ((rows - 1) as f32 / 2.0 - row as f32) * step + theme.board_offset_y;
    [x, y]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 手工构造棋盘：给定雷位列表（col,row），绕过随机布雷。
    fn board_with(cols: usize, rows: usize, mines: &[(usize, usize)]) -> Board {
        let mut b = Board::new(&LevelSpec {
            name: "test",
            cols,
            rows,
            mines: mines.len(),
        });
        for &(c, r) in mines {
            let i = b.idx(c, r);
            b.mines[i] = true;
        }
        let mut counts = vec![0u8; b.total()];
        for r in 0..rows {
            for c in 0..cols {
                let n = b
                    .neighbors(c, r)
                    .iter()
                    .filter(|&&(c2, r2)| b.mines[r2 * cols + c2])
                    .count() as u8;
                counts[r * cols + c] = n;
            }
        }
        b.adjacent = counts;
        b
    }

    #[test]
    fn regenerate_places_exact_mine_count() {
        let spec = LevelSpec {
            name: "t",
            cols: 9,
            rows: 9,
            mines: 10,
        };
        let mut b = Board::new(&spec);
        for _ in 0..50 {
            b.regenerate();
            assert_eq!(b.mines.iter().filter(|&&m| m).count(), 10);
            assert_eq!(b.status, GameStatus::Playing);
            assert!(b.revealed.iter().all(|&r| !r));
            assert!(b.flagged.iter().all(|&f| !f));
        }
    }

    #[test]
    fn neighbor_counts_by_position() {
        let b = Board::new(&LevelSpec {
            name: "t",
            cols: 9,
            rows: 9,
            mines: 1,
        });
        assert_eq!(b.neighbors(0, 0).len(), 3);
        assert_eq!(b.neighbors(8, 8).len(), 3);
        assert_eq!(b.neighbors(4, 4).len(), 8);
        assert_eq!(b.neighbors(0, 4).len(), 5);
    }

    #[test]
    fn adjacent_counts_are_correct() {
        let b = board_with(9, 9, &[(4, 4)]);
        for (c, r) in b.neighbors(4, 4) {
            assert_eq!(b.adjacent[b.idx(c, r)], 1, "cell ({c},{r})");
        }
        assert_eq!(b.adjacent[b.idx(0, 0)], 0);
    }

    #[test]
    fn clicking_mine_loses() {
        let mut b = board_with(9, 9, &[(2, 3)]);
        b.reveal(2, 3);
        assert_eq!(b.status, GameStatus::Lost);
        assert!(b.revealed[b.idx(2, 3)]);
    }

    #[test]
    fn number_cell_does_not_flood() {
        let mut b = board_with(9, 9, &[(1, 0)]);
        b.reveal(0, 0); // adjacent=1，只翻开自己
        assert!(b.revealed[b.idx(0, 0)]);
        assert!(!b.revealed[b.idx(5, 5)]);
        assert_eq!(b.status, GameStatus::Playing);
    }

    #[test]
    fn empty_cell_flood_fills_until_numbers() {
        // 雷全在最右列（加 (7,8)），左侧全是空白格，(0,4) 泛洪应展开大片区域
        let mines = [
            (8, 0),
            (8, 1),
            (8, 2),
            (8, 3),
            (8, 4),
            (8, 5),
            (8, 6),
            (8, 7),
            (8, 8),
            (7, 8),
        ];
        let mut b = board_with(9, 9, &mines);
        assert_eq!(b.adjacent[b.idx(0, 4)], 0, "precondition: (0,4) is empty");
        b.reveal(0, 4);
        assert!(b.revealed[b.idx(0, 4)]);
        assert!(b.revealed[b.idx(5, 1)], "flood should reach (5,1)");
        let count = b.revealed.iter().filter(|&&r| r).count();
        assert!(count > 30, "flood only revealed {count} cells");
        for &(c, r) in &mines {
            assert!(!b.revealed[b.idx(c, r)], "mine ({c},{r}) got revealed");
        }
        // 此布局下泛洪恰好翻开全部 71 个安全格，直接获胜
        assert_eq!(count, b.safe_total());
        assert_eq!(b.status, GameStatus::Won);
    }

    #[test]
    fn flagged_cell_is_protected() {
        let mut b = board_with(9, 9, &[(1, 1)]);
        let i = b.idx(1, 1);
        b.flagged[i] = true;
        b.reveal(1, 1);
        assert!(!b.revealed[i]);
        assert_eq!(b.status, GameStatus::Playing);
    }

    #[test]
    fn toggle_flag_skips_revealed() {
        // 雷放在 (1,1)：(0,0) adjacent=1，翻开后不会泛洪，(8,8) 仍是未翻开格
        let mut b = board_with(9, 9, &[(1, 1)]);
        b.reveal(0, 0);
        b.toggle_flag(0, 0);
        assert!(!b.flagged[b.idx(0, 0)]);
        b.toggle_flag(8, 8);
        assert!(b.flagged[b.idx(8, 8)]);
        b.toggle_flag(8, 8);
        assert!(!b.flagged[b.idx(8, 8)]);
    }

    #[test]
    fn revealing_all_safe_cells_wins() {
        let mines = [
            (0, 0),
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 4),
            (5, 5),
            (6, 6),
            (7, 7),
            (8, 8),
            (0, 8),
        ];
        let mut b = board_with(9, 9, &mines);
        'outer: for r in 0..9 {
            for c in 0..9 {
                if !b.mines[b.idx(c, r)] {
                    b.reveal(c, r);
                }
                if b.status != GameStatus::Playing {
                    break 'outer;
                }
            }
        }
        assert_eq!(b.status, GameStatus::Won);
    }

    #[test]
    fn cell_center_centers_board() {
        let t = Theme::default();
        let [cx, _] = cell_center(&t, 9, 9, 4, 4);
        assert!(cx.abs() < t.step());
        let [lx, ly] = cell_center(&t, 9, 9, 0, 0);
        let [rx, ry] = cell_center(&t, 9, 9, 8, 8);
        assert!(lx < rx);
        assert!(ly > ry); // y 轴向下
    }
}
