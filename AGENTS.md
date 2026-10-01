# AGENTS.md — Bevy 扫雷

本文件是 AI/自动化改动的**入口约束**。细节在两份独立文档里，动手前先读对应那一份：

| 文档 | 什么时候必读 |
|------|--------------|
| `docs/bevy-engine.md` | 写/改任何 bevy API、Cargo feature、窗口/字体/文本/输入代码；升级 Bevy 版本 |
| `docs/coding-standards.md` | 决定代码放哪个 crate/模块、命名与可见性、写测试、提交前自检 |
| `README.md` | 面向人的项目介绍、操作说明、平台状态 |

## 项目速览

- Rust workspace（`Cargo.toml` members：`crates/game-core`、`crates/platform`、`crates/app`），二进制名 `minesweeper`。
- 依赖版本只在根 `[workspace.dependencies]` 声明：`bevy 0.19`（`default-features = false, features = ["2d"]`）、`rand 0.10`；子 crate 一律 `workspace = true`。
- 工具链：Rust stable（本仓库验证于 1.99.0），Bevy 0.19.1、rand 0.10.3，`edition = "2021"`。
- 注释与用户可见文案为**中文**，标识符为英文。

## 分层（最硬的规则，违反即返工）

```
game-core（纯逻辑）  ←  platform（cfg(target_os) 适配）  ←  app（唯一能 use bevy 的 crate）
```

- `crates/game-core`：禁止出现 `bevy`。布雷/邻接计数/翻开泛洪/插旗/判胜负（`board.rs`）、关卡表（`levels.rs`）、主题素材数据（`theme.rs`）、坐标换算 `cell_center`（`lib.rs`）都属于这里。
- `crates/platform`：禁止第三方依赖。窗口尺寸、输入方式（`InputMode`）、系统字体候选路径。
- `crates/app`：`main.rs` 只装配 `DefaultPlugins` + `MinesweeperPlugin`；`plugin.rs` 是系统调度唯一入口；`state.rs` 放 bevy 资源；`assets.rs` 是 `Theme/Rgb/字体路径 → bevy 类型` 的唯一出口；`board.rs` 生成/销毁实体；`input.rs` 输入；`render.rs` 状态→外观同步。
- bevy 类型（`Color`/`FontSource`/`Sprite`）**不得**出现在核心层或平台层的公开 API 上。

## 常用命令

```bash
cargo run -p app                    # 开发运行（debug）
cargo run -p app --release          # 流畅运行 / 演示
cargo check --workspace --all-targets
cargo test --workspace              # 15 game-core + 2 platform（app 无测试）
cargo clippy --workspace --all-targets -- -D warnings   # 必须 0 告警（提交前跑）
cargo fmt --all && cargo fmt --all -- --check           # 必须 0 差异
```

- 改核心逻辑：先 `cargo test -p game-core`（秒级），再动 `app`。
- 全量 `target/` 约 10 GB；增量编译过慢时 `cargo clean` 或开 `dynamic_linking`（见 `docs/bevy-engine.md` §8）。
- **查 bevy 用法先看本机官方示例**，不要凭记忆或抄博客：`~/.cargo/registry/src/*/bevy-0.19.1/examples/<主题>/`，索引见 `docs/bevy-engine.md` §9。

## 改动纪律

- **不要按 ECS 角色拆文件**（不要 `components.rs`/`systems.rs`）；按领域拆，一个模块一个 `Plugin`。
- **游戏事实的唯一真源是 `Board`**：不要从 `Sprite.color`/`Visibility` 反推状态，不要在 `app` 层复制判定逻辑。
- 渲染同步必须进 `render::Signature`（含 `revision`/`generation`/hover/status/flags/revealed/flagged/level）；新增渲染维度不进签名，就会出现“状态变了画面没变”。
- 系统顺序固定：`(mouse_click, hover_highlight).chain() → (on_level_change, sync_board, update_info).chain()`。新系统按职责挂到对应链上，不要依赖 Bevy 默认并行顺序。
- 0.19 特有坑（`#[derive(Resource)]` 已含 `Component`；`Font::from_bytes`；`Query<Entity>` 与 `Res<T>` 冲突；`despawn_recursive()` 与 `Query::get_single()` 均已移除；rand 0.10 改名）见 `docs/bevy-engine.md` §4。别照抄 0.15 前的教程/博客。
- 实体/资源取用一律 `if let Ok(..)` / `.ok()?` / `single().ok()?`，**系统里禁止 `unwrap()`/`expect()`**。
- 系统参数超过 7 个用 `#[derive(SystemParam)]` 分组（参考 `render::BoardView`），**不要 `#[allow(clippy::too_many_arguments)]`**；写法与两个生命周期坑见 `docs/coding-standards.md` §9。
- 边界用 `saturating_sub()` / `.min()`，不要 panic 兜底。

## 测试要求

- 核心规则改动必须在 `crates/game-core/src/**` 的 `#[cfg(test)] mod tests` 加测试；固定雷位用 `board_with(cols, rows, &[(col, row)])`，随机相关断言写成不变量（如“重开 50 次雷数恒为 10”）。
- 新增关卡必须让 `levels.rs` 的可玩性/难度单调测试继续通过。
- 交互改动跑一遍手动清单：开窗口 → 中文不乱码 → 左键翻开 → 右键插旗 → 空白泛洪 → 踩雷判负 → 翻完判胜 → 过关进下一关且棋盘按新尺寸重建。

## 已知偏差（改动时顺手清理，但不要扩大）

- `src/main.rs`（仓库根）是重构前的 607 行单文件版本，未被 workspace 编译，仅作历史参考。
- 尚未初始化 CI；Windows / Android 路径为“已配置未验证”。
- `target/` 已长到约 10 GB，`cargo clean` 前心里有数（全量重建要重新编译 bevy）。

lint / fmt 基线目前是**全绿**：`cargo fmt --all -- --check` 与
`cargo clippy --workspace --all-targets -- -D warnings` 都必须保持 0 输出。

## 提交

- 提交信息与正文用中文，标题为“做了什么”一句话（参考 `git log`）。不新建无关分支。
- `Cargo.lock` 一起提交；`target/`、`.DS_Store`、`*.bak` 已被 `.gitignore` 排除，不要 `git add -f`。
- API、关卡、操作方式、Bevy 版本发生变化时，同一批改动里同步更新 `README.md` 与 `docs/*.md`。
