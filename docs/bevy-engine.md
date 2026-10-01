# Bevy 引擎使用说明（本项目适用）

> 本仓库锁定的引擎版本：**Bevy `0.19.1`**（见 `Cargo.lock`），Rust toolchain `1.99.0 stable`。
> 下文所有 API 均按 0.19 的签名书写；0.15 以前的写法（含网上大多数扫雷教程）在本仓库**不成立**，改动前先对照本文件。
> 权威来源：
> - 入门/插件/资源：<https://bevy.org/learn/quick-start/getting-started/ecs/>、`.../plugins/`、`.../resources/`
> - 项目组织与插件开发：<https://bevy.org/learn/book/modular-architecture/project-organization/>、`/learn/book/modular-architecture/plugins/`、`/learn/quick-start/plugin-development/`
> - 测试实践：<https://bevy.org/learn/book/development-practices/testing/>
> - 升级必看：<https://bevy.org/learn/migration-guides/0-18-to-0-19/>
> - API 细节以 `~/.cargo/registry/src/*/bevy_*-0.19.1/` 里的实际签名为准。

---

## 1. 引擎心智模型（App / World / ECS）

- **App**：每个 Bevy 程序都是一个 `App`，负责外层循环；只在启动阶段以 builder 方式链式装配（`add_plugins` / `insert_resource` / `add_systems`）。运行期业务逻辑不要塞 `App`。
- **World**：`App` 持有 `World`，`World` 存放全部游戏数据。
- **ECS 三件套**：
  - `Entity`：只带唯一 id 的“东西”。
  - `Component`：普通 Rust `struct` + `#[derive(Component)]`，描述数据，不带方法调用逻辑。
  - `System`：普通 Rust 函数，参数即“这个系统在哪些组件上跑”，由 Bevy 调度并行执行。
- **Resource**：全局唯一数据。访问用 `Res<T>`（只读）/ `ResMut<T>`（可写）。本项目把棋盘、主题、关卡进度、悬停状态都做成 Resource（见 `crates/app/src/state.rs`）。

```rust
// 本项目的装配入口：crates/app/src/plugin.rs（MinesweeperPlugin::build）
app.init_resource::<Hover>()
   .init_resource::<Revision>()
   .insert_resource(AppTheme(Theme::default()))
   .insert_resource(LevelProgress(levels::FIRST_LEVEL))
   .add_systems(Startup, setup)
   .add_systems(
       Update,
       ((mouse_click, hover_highlight).chain(),
        (on_level_change, sync_board, update_info).chain()).chain(),
   );
```

**分工口径**：`main.rs` 只配 `DefaultPlugins`（窗口 + 日志过滤）并挂插件；`plugin.rs` 是唯一登记资源与系统顺序的地方；新增系统请在那里挂载，不要在 `main.rs` 里 `add_systems`。

## 2. 本仓库用到的 0.19 API 清单

| 用途 | 0.19 写法 | 备注 |
| --- | --- | --- |
| 2D 相机 | `commands.spawn(Camera2d);` | `Camera2d` 是单元结构体，会启用 2D 渲染主流程 |
| 纯色方块 | `Sprite::from_color(impl Into<Color>, Vec2::new(w, h))` | 格子/背景底色，第二个参数是尺寸 |
| 颜色 | `Color::srgb(r, g, b)`，分量 `0.0..=1.0` | 由 `assets::to_color(Rgb)` 从核心层 `Rgb` 转换 |
| 位置/层级 | `Transform::from_xyz(x, y, z)` | `z` 越大越靠前景；标签用 `1.0`，信息栏用 `10.0` |
| 父子关系 | `ChildOf(child_entity_component)` | 数字/雷/旗文本是格子的子实体 |
| 世界坐标文本 | `Text2d::new(String)` + `TextFont { font, font_size: FontSize::Px(px), ..default() }`（`TextFont` 共 8 个字段，`..default()` 必要）+ `TextColor` + `TextLayout { justify: Justify::Center, linebreak: LineBreak::AnyCharacter }` + `Visibility` | 0.19 文本后端已换成 **Parley**（不再是 cosmic-text）；`TextLayout` 只有 `justify`/`linebreak` 两个字段，**别写 `..default()`** |
| 字体 | `bevy::text::Font::from_bytes(bytes)`（不是 `try_from_bytes`），配合 `FontSource::Handle(handle)` / `FontSource::default()` | `FontSource::default()` 在 `default_font` feature 下回退内置 FiraMono（只够 ASCII） |
| 窗口 | `WindowPlugin { primary_window: Some(Window { title, resolution: (w, h).into(), ..default() }), ..default() }` | `Window::resolution` 是 `WindowResolution`，元组 `.into()` 即可 |
| 鼠标位置 | `window.cursor_position() -> Option<Vec2>`（逻辑像素） | 无光标（光标还在窗外等）返回 `None`，必须用 `?` 提前返回 |
| 屏幕→世界 | `camera.viewport_to_world_2d(camera_transform, cursor) -> Result<Vec2, ViewportConversionError>` | 官方写法 `camera_query: Single<(&Camera, &GlobalTransform)>, window: Single<&Window>`（`bevy-0.19.1/examples/2d/2d_viewport_to_world.rs`）；本仓库用 `Query` + `.single().ok()?` |
| 按键输入 | `Res<ButtonInput<MouseButton>>` + `just_pressed(MouseButton::Left / Right)` | 边沿触发，不写状态机时最简单可靠 |
| 删除实体 | `commands.entity(e).despawn()`（0.19 起自动**递归**级联 `Children`）；只删子级用 `despawn_children()`；实体可能已消失用 `commands.get_entity(e).ok()?.despawn()` 或 `try_despawn()` | `despawn_recursive()` / `despawn_descendants()` 自 0.16 起已废弃并在 0.19 删除，本仓库 `despawn_board` 直接 `despawn()` |
| 日志降噪 | `.set(bevy::log::LogPlugin { filter: "error".into(), ..default() })` | 本仓库用它屏蔽 parley/icu_segmenter 关于中日文分词模型的无害提示与 winit 退出刷屏（`crates/app/src/main.rs`） |

## 3. Cargo feature：为什么是 `features = ["2d"]`

0.19 的 feature 划分比早期版本细（`default = ["2d", "3d", "ui", "audio"]`）：

- `2d` 会带出 `default_app`（`bevy_asset`、`bevy_log`、`bevy_state`…）、`default_platform`（`bevy_winit`、`default_font`、`multi_threaded`、`x11/wayland`）、`2d_bevy_render`（`bevy_sprite` 等）、`scene`、`picking`。
- `bevy_text` / `bevy_window` 由 `common_api`（`2d` → `2d_api` → `common_api`）带出，所以 **`Sprite` + `Text2d` + 窗口输入全都够用**。
- **`audio` 与 `ui` 在 0.19 不再被 `2d`/`3d` 隐式开启**（官方 0.18→0.19 迁移指南明确说明）。本项目不需要，保持关闭。
- `default-features = false` + 显式 `["2d"]` 是刻意为之：编译更短、依赖更少，也避免把 3D/音频拖进一个扫雷小游戏。
- 想看某个符号是否可用：`rg -n "^pub " ~/.cargo/registry/src/*/bevy_<crate>-0.19.1/src/`，或 `cargo tree -p app -d`。

## 4. 版本决策表（0.15 前教程 → 0.19）

写代码/抄教程时最先踩的几个坑：

| 旧写法 | 0.19 写法 |
| --- | --- |
| `Res<Windows>` / `windows.primary()` | `window: Single<&Window>`（或 `Query<&Window>` + `.single().ok()?`）。**0.19 的 `Query` 只有 `single()` / `single_mut()`（返回 `Result`），`get_single()` / `get_single_mut()` 已不存在**（`bevy_ecs-0.19.1` 里搜不到）；“没有就跳过、多个也跳过”用 `Option<Single<&T>>` 系统参数 |
| `Camera2dBundle { .. }` | 直接 `Camera2d`（其余靠默认组件） |
| `SpriteBundle { sprite, transform, .. }` | `Sprite::from_color(...)` + `Transform::from_xyz(...)` 作为 tuple bundle |
| `TextBundle { .. }` | `Text2d` + `TextFont` + `TextColor` + `TextLayout` |
| `Text` 结构体（`sections`/`Layout`） | `Text2d`；分段文本走新引入的 `TextSection`（`TextRoot`/`TextSpanAccess` 已删除） |
| `Font::try_from_bytes(data)` | `Font::from_bytes(data)` |
| `#[derive(Component, Resource)] struct X;` | **只 derive 一个**：0.19 起 `Resource` 是 `Component` 的子 trait，`#[derive(Resource)]` 已自动实现 `Component`，同一类型不能既是资源又是组件 |
| `Query<Entity>` / `Query<()>` 与 `Res<T>` 同系统 | 会报冲突（资源现在也是挂在抽象实体上的组件）。加 `Without<IsResource>` 或收窄查询 |
| `rand::thread_rng()` / `rngs::StdRng` | `rand::rng()`；`RngCore` 改名 `Rng`，原 `Rng` 改名 `RngExt`（rand 0.10）。`indices.shuffle(&mut rand::rng())` 仍走 `rand::seq::SliceRandom` |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()`（0.16 起自动级联子实体，0.19 里旧名已彻底移除） |
| 单文件 `main.rs` 里 `App::new()...` | 拆成 `Plugin`（见下） |
| `TextLayout { justify, linebreak, ..default() }` | `TextLayout { justify, linebreak }` 或 `TextLayout::justify(Justify::Center)`——只有这两个字段，`..default()` 会被 clippy 判 `needless_update` |

## 5. 调度与执行顺序

- **系统默认并行**，同帧内谁先谁后不保证。只要“A 改状态、B 读状态”就必须显式定序：元组 + `.chain()`。
- 本仓库 `Update` 的固定顺序（`crates/app/src/plugin.rs`）：
  `(mouse_click, hover_highlight).chain()` → `(on_level_change, sync_board, update_info).chain()`，两段之间再 `.chain()`。
  即：先吃输入改 `Board`，再按尺寸变化重建实体，最后同步外观。原因见 `docs/coding-standards.md` §3（状态先行）。
- `Startup` 只跑一次（生成相机、字体、首屏实体）；`Update` 每帧跑。
- **渲染同步要做“脏检查”**：把一帧的输入压成一个签名（`crates/app/src/render.rs` 的 `Signature`：
  `revision` + `generation` + hover + `status` + `flags` + `revealed`/`flagged` + `level`），签名不变直接 `return`，
  避免每帧写组件导致全板重排。Bevy 的组件写入是变更检测敏感操作，全板每帧写会明显掉帧。
- 同规格重开（踩雷后重开本关）时 `Board::generation` 会回到同一数值，单靠它无法让签名失效，
  因此额外有 `Revision(pub u32)` 资源：输入层在每次重开/换关时 `+= 1`，渲染层把它并入签名。加新的“必须强制重画”事件时同样 bump 它。
- `Local<T>` 适合放这种“上一帧缓存”；跨系统共享才升格成 `Resource`。
- `Single<T>` 系统参数在“匹配数 ≠ 1”时**会让系统 panic**；本仓库统一用 `Query` + `.single().ok()?` / `if let Ok(..)`，需要“零个也正常”再考虑 `Option<Single<T>>`（见 `examples/ecs/fallible_params.rs`）。

## 6. 坐标与布局约定

- 世界坐标 **Y 轴向上**（屏幕向上），逻辑像素；`Transform.translation.z` 决定遮挡顺序。
- 格子 → 世界的换算集中在 `game_core::cell_center(theme, cols, rows, col, row)`：一个函数被生成实体、鼠标拾取、渲染三处复用，别在别处写第二份。
- 反查（世界坐标 → 格子）用 `input::pick_cell`，中心 ± `cell_size/2` 的方形命中判定，比射线/碰撞体简单且够用。
- 悬停高亮存的是 `Option<Entity>` 而非格子下标：实体在换关时会重建，务必每次从 query 现取，不要跨帧缓存 Entity 去 `get_mut`（用 `if let Ok(...)` 兜底）。

## 7. 中文渲染：字体加载 + CJK 断行警告刷屏

### 7.1 字体（豆腐块问题）

Bevy 内置字体只含 ASCII，中文会变豆腐块。本项目的方案：

1. `platform::available_font()` 按平台候选路径顺序找实际存在的字体（macOS 用 `Hiragino Sans GB.ttc` 等，Windows 用 `msyh.ttc` 等，Android 用 `NotoSansCJK`）。
2. `crates/app/src/assets.rs` 读出字节 → `Font::from_bytes` → 注册进 `Assets<Font>` → 存成 `AppFont(FontSource)` 复用。
3. 字体只加载一次并共享 handle；`FontSize::Px` 会再乘窗口 scale factor 与 `UiScale`，同一字号反复组合会重建字体图集，别每帧改字号。

### 7.2 `ICU4X data error: ... Chinese/Japanese` 刷屏（真因）

**症状**：debug 运行每 ~100ms 往 stderr 打一行
`ICU4X data error: No segmentation model for complex script: Chinese/Japanese`（实测 6 秒 ≈ 280 行）。

**真因链**（逐层查过源码，别再按旧结论改）：

- 0.19 文本后端换成 **parley 0.9**；排版任何文本都会调
  `icu_segmenter` 的 `LineSegmenter`/`WordSegmenter`，对 CJK 走
  `ComplexPayloadsBorrowed::select(ComplexScript::ChineseJapanese)`。
- `icu_segmenter` 的 `compiled_data` **没打包 CJK 断行字典**，取不到就 `DataError`，
  经 `icu_provider::DataError::with_display_context()` 打一条 **WARN**。
- **`TextLayout.linebreak` 改不了它**：`AnyCharacter` 只是把 parley 的 `WordBreak` 设成
  `BreakAll`，分段器照旧调用。实测 `NoWrap` 同样刷屏 → 网上“改 linebreak 就能消除”的说法是错的。
- **`LogPlugin.filter` 也压不住**：`icu_provider` 未开 `logging` feature 时，
  debug 构建走 `pub mod log { pub use std::eprintln as warn; }`（裸 `eprintln!`），
  根本进不了 tracing；`RUST_LOG=off` 也照打。release 构建是空实现（所以 release 看不到），
  但仍无法按需打开诊断。

**修法（已落地，两处必须成对）**：

1. 根 `Cargo.toml` 的 `[workspace.dependencies]` 里显式依赖 `icu_provider` 并开 `logging` feature，
   让它走 `log` → `LogTracer` → tracing 管线：
   `icu_provider = { version = "2.3", features = ["logging"] }`（`crates/app/Cargo.toml` 用 `workspace = true` 继承）。
2. `crates/app/src/main.rs` 的 `LogPlugin.filter` 加指令把它关掉：
   `filter: "error,icu_provider=off".into()`。

实测：debug stderr 从 ~280 行/6s 变成 **0 行**；`RUST_LOG=info` 时 5 行 INFO 全保留、CJK 噪声 0 行
（即诊断能力没被牺牲，只精准屏蔽这一条）。`Cargo.lock` 只多两行（`log` 与 app 的依赖边），不新增第三方包。

**为什么不用 bevy_feathers / bevy_ui 换掉文本**：`bevy_ui::TextNode` 和 Feathers 控件的文字
同样经 `bevy_text` → parley 排版，照样触发这条警告，换 UI 层不解决问题；
而格子是**世界坐标**里的 `Sprite` + `Text2d`，改成 UI 要重做整套坐标/拾取，纯属绕路。
需要正式排版的中文界面时再引入 `ui`/`bevy_feathers` feature，别为消警告引入。

**想彻底消除警告本身**（而非屏蔽）只有一条路：给 parley 提供更完整的 CJK 数据，
或等 `icu_segmenter` 的 `compiled_data` 打包 CJK 字典；升级 Bevy 时重查这一节。

## 8. 性能与编译速度

- `Cargo.toml` 已按官方推荐配置 `opt-level`：`[profile.dev] opt-level = 1`、`[profile.dev.package."*"] opt-level = 3`（官方 Setup 页原样建议，dev 构建才能流畅跑）。别删。
- 嫌 Bevy 增量编译慢时开动态链接（官方称影响最大）：在 `crates/app/Cargo.toml` 加 `dynamic_linking = ["bevy/dynamic_linking"]` 这个 feature 再 `cargo run -p app --features dynamic_linking`。注意 `bevy_dylib` 目前**不在 `Cargo.lock` 里**，首次开启要联网拉包；发布构建不带它。
- 全量构建 `target/` 会长到 **~10 GB**，`cargo clean` 是最直接的回血手段。
- 需要量化时用 `cargo build --timings`；Rust 按 crate 并行编译，拆 crate（本仓库的 3 crate 结构）本身就是减增量编译时间的杠杆。
- 运行期省帧的三件套：脏检查渲染（见 §5）、共享字体 handle、只保留需要的 bevy feature。

## 9. 官方示例速查（离线，就在本机）

Bevy 把 `examples/` 一起打进 crates.io 包，**不用联网就能查官方用法**，比翻博客可靠：

```bash
E=~/.cargo/registry/src/index.crates.io-*/bevy-0.19.1/examples
sed -n '1,60p' $E/2d/text2d.rs                    # Text2d + TextFont + TextLayout
sed -n '1,60p' $E/2d/2d_viewport_to_world.rs      # 光标 → 世界坐标
sed -n '1,60p' $E/2d/sprite.rs                    # Camera2d + Sprite::from_image
sed -n '1,80p' $E/input/mouse_input.rs            # ButtonInput<MouseButton>
sed -n '1,80p' $E/ecs/hierarchy.rs                # ChildOf / Children 与级联 despawn
sed -n '1,80p' $E/ecs/change_detection.rs         # 脏检查/变更检测
sed -n '1,60p' $E/ecs/fallible_params.rs         # Single / Option<Single> 的失败语义
sed -n '1,50p' $E/window/window_settings.rs      # Window 字段（标题、分辨率、可缩放）
cat $E/app/plugin_group.rs                        # DefaultPlugins 与 PluginGroup::set
```

目录：`2d 3d animation app asset audio camera dev_tools diagnostics ecs gizmos gltf input math
movement no_std picking reflection remote scene shader shader_advanced showcase state
stress_tests time transforms ui usage wasm window`。
先 `ls $E/<主题>` 找名字，再 `sed -n` 局部读，别整文件 cat。

## 10. 测试相关引擎能力

- **核心逻辑测试优先**：`crates/game-core` 完全不依赖 bevy，布雷/泛洪/胜负这些纯逻辑直接 `#[test]`，跑得比编译 bevy 快几个数量级。新增规则先在这里加测试。
- `crates/platform` 同样无第三方依赖，可单测。
- 官方对小游戏的态度（Testing Bevy Apps 章节）：CI 通常没 GPU、游戏需要输入推进、断言视觉状态容易脆弱；**小团队用“核心单元测试 + 手动验证”即可**，不必为了 bevy 集成测试引入 App 测试框架。若确实要做 bevy 端自动化，再考虑 `bevy::app::App` 手动 `update()` 驱动帧。
- 手动验证清单写进 PR：能开窗口、中文不乱码、左键翻开、右键插旗、空白格泛洪、踩雷判负、翻完判胜、换关后棋盘尺寸正确重建。

## 11. 升级 Bevy 的流程

1. 先读对应版本的迁移指南（如 `/learn/migration-guides/0-18-to-0-19/`），按编译报错逐条改，不要凭记忆。
2. 改 `Cargo.toml` 里 `workspace.dependencies.bevy` 的版本与 feature 列表。feature 名最容易在升级时搬走：0.19 里场景序列化 crate 从 `bevy_scene` 更名为 `bevy_world_serialization`（feature `scene` 现在同时带这两个），`audio`/`ui` 也不再被 `2d`/`3d` 隐式开启。
3. `cargo clean` 后 `cargo check --workspace --all-targets`。
4. 升级文档：本文件 §2/§4/§9 的 API 表与示例索引、§7.2 的 CJK 断行噪声结论、`README.md` 版本说明、必要时更新 `AGENTS.md` 的禁止事项。
