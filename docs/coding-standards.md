# 代码规范（本项目）

面向 Rust + Bevy 0.19 的三 crate 工作区。规则来自本仓库既有写法与 Bevy 官方建议
（<https://bevy.org/learn/book/modular-architecture/project-organization/>、<https://bevy.org/learn/book/development-practices/testing/>、<https://bevy.org/learn/quick-start/plugin-development/>）；
引擎 API 相关约束见 `docs/bevy-engine.md`。

## 1. 分层与依赖方向（最硬的规则）

```
game-core  ←  platform  ←  app(bin)
   纯逻辑        平台适配      bevy 装配/输入/渲染
```

- **`crates/game-core` 禁止 `use bevy::*`**，只允许 `rand`。布雷、邻接计数、翻开/泛洪、插旗、判胜负、关卡表、主题数据全在这里。理由：能在无 GPU 的机器上秒级单测，且渲染后端可替换。
- **`crates/platform` 禁止第三方依赖**，只允许 `cfg(target_os)` 分支和 `std`。窗口尺寸、输入方式、系统字体候选路径属于这里。
- **只有 `crates/app` 可以碰 bevy**：`Plugin`、`Component`、`Resource`、`System`、`Sprite`、`Text2d` 都在这里，且 bevy 类型不得从核心层暴露出来。
- 单位换算：核心层用 `Rgb` / `[f32; 2]` / `f32` 这类纯数据描述视觉与布局；`assets::to_color` 是 `Rgb → Color` 的唯一出口，新增视觉属性请走 `Theme`，不要在系统里写字面量颜色。
- 依赖版本统一在根 `Cargo.toml` 的 `[workspace.dependencies]`，子 crate 用 `xxx = { workspace = true }` 继承；不要在子 crate 写死版本号。

## 2. 模块组织

- **一个模块一件事**，按“领域”拆（`state` / `assets` / `board` / `input` / `render` / `plugin`），**不按 ECS 角色拆**（不要出现 `components.rs`、`systems.rs` 这种大杂烩）。
- **一个模块一个插件**；`Plugin` 实现放在模块文件顶部，充当“目录”。`crates/app/src/plugin.rs` 是唯一装配点，`main.rs` 里只留 `App::new()` + `DefaultPlugins` 配置 + `add_plugins(MinesweeperPlugin)`。
- 可见性默认 `pub(crate)`；只有刻意共享的接口（`Board`、`LevelSpec`、`Theme`、`cell_center`、`LEVELS`、插件本身）才 `pub`。结构体字段可见性与所属结构体一致，不做无意义的 getter/setter。
- 单文件超过约 1000 行就该拆；本仓库模块都在 100–200 行量级，保持这个体量。
- 跨 crate 复制粘贴前先想清楚该逻辑属于哪一层：**能进 `game-core` 的绝不留在 `app`**。

## 3. 状态先行：逻辑与表现分离

- **系统只改核心状态，渲染系统只读状态写外观**。任何“游戏事实”（是否翻开、旗子数、胜负）的唯一真源是 `Board`，不得从 `Sprite.color` 反推。
- 同一 schedule 内的固定顺序：`(输入).chain() → (渲染同步).chain()`。写新系统时想清楚它属于哪一段，跨段用 `.chain()` 显式声明，**不要依赖 Bevy 的默认并行顺序**。
- 渲染同步必须带脏检查：本仓库用 `render::Signature`（`generation` + hover + status + flags + revealed/flagged + level），签名一致提前 `return`。新增渲染维度就把它加进 `Signature`，否则会出现“状态变了画面没变”的诡异 bug。
- `generation` 由 `Board::regenerate()` 自增，是“棋盘已重开”的信号；不要用“长度变化”“下标复位”之类启发式猜重开。
- 状态转换（`Playing → Won/Lost`）只在 `Board::reveal` 里判定；输入层负责“结束后点击 → 下一关/重开”，不要在多处写 `status = ...`。

## 4. Bevy 用法约定

- 组件：数据类 `struct`/元组结构体 + `#[derive(Component)]`，不带行为方法；标记组件用单元结构体（`BoardEntity`、`CellLabel`、`InfoText`）。
- 资源：`#[derive(Resource)]` 元组结构体包核心类型（`Game(pub Board)`、`AppTheme(pub Theme)`）。0.19 起 `Resource` 是 `Component` 子 trait，**不要再写 `#[derive(Component, Resource)]`**。
- 实体清理：整块棋盘打 `BoardEntity` 标记，换关时按标记 `despawn`（0.19 会级联子实体）。不要用“记 id 列表”的方式手工清理。
- 子实体用 `ChildOf(parent)` + 局部 `Transform::from_xyz(0.0, 0.0, 1.0)`，位置由父格子决定。
- 查询收窄：`With<Cell>` / `With<CellLabel>` / `Without<InfoText>` 明确分开，避免同一系统里多个 `&mut` 查询冲突；也别用 `Query<Entity>`/`Query<()>` 这类“宽查询”（0.19 会和 `Res<T>` 冲突）。
- 容错取用：查询结果可能已失效，用 `if let Ok(mut x) = sprites.get_mut(entity)` / `info.get_single_mut()`，**不要 `unwrap()` 实体或资源**。
- 字体、主题等重资源只在 `Startup` 加载一次，存成 `Resource` 复用；渲染里用 `RenderParams::from(&Theme)` 这种一次性派生值，别每帧重算。
- 输入统一走 `input.rs`：`pick_cell` 负责“坐标 → 格子”，系统只表达意图（翻开/插旗/重开）。触摸端按 `platform::current().input == InputMode::Touch` 扩展新系统，**不要在现有系统里塞 `cfg` 分叉**。

## 5. Rust 风格

- `cargo fmt` 为准，`cargo clippy -D warnings` 必须干净；两者都是提交前的最低门槛。
- 命名：类型 `UpperCamelCase`，函数/变量 `snake_case`，常量 `SCREAMING_SNAKE_CASE`；模块/文件名 `snake_case`。测试函数用 `描述性_条件_期望`（如 `toggle_flag_skips_revealed`）。
- 索引换算集中在 `Board::idx(col, row)`，不要散落 `row * cols + col` 手算（`neighbors` 内部是刻意例外，因为它直接读 `mines`）。
- 边界与溢出：优先 `saturating_sub()`（剩余雷数、`safe_total`）与 `.min()` 夹取（`number_color`、`level(index)`），不要用 `unwrap()` 兜边界。
- 错误处理：`std::fs::read` 之类可能失败的调用用 `if let Ok(..)`/`match`，禁止在系统里 `unwrap()`/`expect()` 崩溃窗口。
- 禁止 `clone()` 大数组进热循环；确需缓存（如 `Signature` 里的 `Vec<bool>`）时接受一次拷贝换取整帧提前返回，并在注释里写明动机。
- 允许 `Vec<(usize, usize)>` 这类直观返回，不要为省一次分配上 `impl Iterator`；本仓库规模下可读性优先。
- 注释：写“为什么/坑”，不写“这是什么”。公开 API 用 `//!` / `///`（核心层已全量中文文档注释，保持一致），行内注释只在解释反直觉决定时出现。
- 文档语言：本仓库注释与用户可见文案统一**中文**；标识符保持英文。

## 6. 测试规范

- 单元测试与被测代码同文件 `#[cfg(test)] mod tests`，命名 `tests`（本仓库现状）。
- **确定性优先**：核心逻辑测试不要依赖随机结果。需要固定雷位时用 `game-core/src/lib.rs` 里的 `board_with(cols, rows, &[(col, row)])` 绕过随机布雷；随机相关断言写成“跑 50 次每次雷数恰好为 10”这种**不变量**（见 `regenerate_places_exact_mine_count`）。
- 每个关卡规格都要满足可玩性不变量（`mines < cols * rows`、密度单调不减），新增关卡必须让 `levels.rs` 的两个测试继续通过。
- 修 bug 先加复现测试，测试名描述被保护的行为。
- 验证顺序：`cargo test -p game-core -p platform`（秒级）→ `cargo clippy --workspace -D warnings` → `cargo run -p app` 手动过一遍交互清单（见 `docs/bevy-engine.md` §9）。

## 7. 提交与改动纪律

- 改动保持小而聚焦：一次提交只做一件事（核心逻辑 / 平台适配 / 渲染表现选一层）。
- 不顺手修无关 bug、不重命名无关标识符、不引入与任务无关的依赖。
- 新增依赖前先确认能否用现有分层解决；新依赖必须进 `[workspace.dependencies]`，并优先考虑 feature 精简（`default-features = false`）。
- `Cargo.lock` 一起提交；`target/`、`.DS_Store`、`*.bak` 已被 `.gitignore` 排除，不要 `git add -f` 进去。
- 文档同步：改了 API、关卡、操作方式或 Bevy 版本，同一批改动里更新 `README.md` / `docs/bevy-engine.md` / `docs/coding-standards.md`。
