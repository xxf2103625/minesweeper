//! 扫雷核心棋盘逻辑：布雷、相邻计数、翻开/泛洪、插旗、判胜负。
//! 纯 Rust（仅依赖 rand），不依赖 bevy，可跨平台复用与单元测试。
//! 尺寸/雷数在运行时由 [`LevelSpec`] 决定，支持多关卡。

use rand::seq::SliceRandom;

/// 一局游戏的状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
}

/// 一关的规格（关卡设计的最小单元）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelSpec {
    pub name: &'static str,
    pub cols: usize,
    pub rows: usize,
    pub mines: usize,
}

/// 棋盘运行时状态。所有格子用一维 Vec 存储，索引 = row * cols + col。
#[derive(Clone)]
pub struct Board {
    pub cols: usize,
    pub rows: usize,
    pub mines_total: usize,
    pub mines: Vec<bool>,
    pub adjacent: Vec<u8>,
    pub revealed: Vec<bool>,
    pub flagged: Vec<bool>,
    pub status: GameStatus,
    /// 每次重开自增，供渲染层做缓存失效判断。
    pub generation: u32,
}

impl Board {
    /// 按关卡规格建一个空棋盘（尚未布雷）。
    pub fn new(spec: &LevelSpec) -> Self {
        let total = spec.cols * spec.rows;
        Self {
            cols: spec.cols,
            rows: spec.rows,
            mines_total: spec.mines,
            mines: vec![false; total],
            adjacent: vec![0; total],
            revealed: vec![false; total],
            flagged: vec![false; total],
            status: GameStatus::Playing,
            generation: 0,
        }
    }

    #[inline]
    pub fn idx(&self, col: usize, row: usize) -> usize {
        row * self.cols + col
    }

    #[inline]
    pub fn total(&self) -> usize {
        self.cols * self.rows
    }

    #[inline]
    pub fn safe_total(&self) -> usize {
        self.total().saturating_sub(self.mines_total)
    }

    #[inline]
    pub fn flags(&self) -> usize {
        self.flagged.iter().filter(|&&f| f).count()
    }

    /// 周边 8 邻域（裁剪到棋盘范围内）。
    pub fn neighbors(&self, col: usize, row: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(8);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = col as i32 + dx;
                let ny = row as i32 + dy;
                if nx >= 0 && nx < self.cols as i32 && ny >= 0 && ny < self.rows as i32 {
                    out.push((nx as usize, ny as usize));
                }
            }
        }
        out
    }

    /// 重新随机布雷并复位所有状态。
    pub fn regenerate(&mut self) {
        let total = self.total();
        let mut indices: Vec<usize> = (0..total).collect();
        indices.shuffle(&mut rand::rng());

        self.mines = vec![false; total];
        for &i in &indices[..self.mines_total.min(total)] {
            self.mines[i] = true;
        }

        let mut counts = vec![0u8; total];
        for row in 0..self.rows {
            for col in 0..self.cols {
                let n = self
                    .neighbors(col, row)
                    .iter()
                    .filter(|&&(c, r)| self.mines[r * self.cols + c])
                    .count() as u8;
                counts[row * self.cols + col] = n;
            }
        }
        self.adjacent = counts;

        self.revealed = vec![false; total];
        self.flagged = vec![false; total];
        self.status = GameStatus::Playing;
        self.generation += 1;
    }

    /// 翻开一格：命中雷判负；空白格递归泛洪；翻完所有安全格判胜。
    pub fn reveal(&mut self, col: usize, row: usize) {
        let i = self.idx(col, row);
        if self.revealed[i] || self.flagged[i] {
            return;
        }
        self.revealed[i] = true;

        if self.mines[i] {
            self.status = GameStatus::Lost;
            return;
        }

        if self.adjacent[i] == 0 {
            for (nc, nr) in self.neighbors(col, row) {
                let ni = self.idx(nc, nr);
                if !self.revealed[ni] && !self.flagged[ni] && !self.mines[ni] {
                    self.reveal(nc, nr);
                }
            }
        }

        if self.revealed.iter().filter(|&&r| r).count() >= self.safe_total() {
            self.status = GameStatus::Won;
        }
    }

    /// 右键插旗/取消旗（已翻开的格子不可插旗）。
    pub fn toggle_flag(&mut self, col: usize, row: usize) {
        let i = self.idx(col, row);
        if self.revealed[i] {
            return;
        }
        self.flagged[i] = !self.flagged[i];
    }
}
