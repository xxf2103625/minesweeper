# Bevy 扫雷（多平台）

基于 [Bevy](https://bevy.org) 游戏引擎的 2D 扫雷，参考
[qongzi 的 Bevy Minesweeper 教程](https://dev.to/qongzi/bevy-minesweeper-introduction-4l7f)。
当前使用 **Bevy 0.19.1**（Rust 1.99 stable 验证通过），依赖全部为最新版。

## 文档

| 文件 | 内容 |
|------|------|
| `AGENTS.md` | AI/自动化改动的入口约束：分层规则、常用命令、改动纪律、已知偏差 |
| `docs/bevy-engine.md` | Bevy 引擎使用说明：0.19 API 清单、feature 取舍、旧写法→0.19 对照表、中文字体、性能与升级流程 |
| `docs/coding-standards.md` | 代码规范：依赖方向、模块组织、状态先行、Bevy 用法约定、Rust 风格、测试规范、lint 基线 |

## 运行

```bash
cargo run -p app            # debug
cargo run -p app --release  # release（推荐，流畅）
```

## 操作

- **左键**：翻开格子（空白格自动泛洪展开）
- **右键**：插旗 / 取消旗（插旗的格子不会被翻开）
- 过关后点击 → 进入下一关；踩雷后点击 → 重开本关

## 工程结构（Cargo workspace）

```
minesweeper/
├── Cargo.toml              # workspace 根，统一依赖版本
├── AGENTS.md               # 自动化/协作者改动约束（入口）
├── docs/                   # 引擎使用说明 + 代码规范
├── src/main.rs             # 重构前的单文件旧版，不参与编译，仅作历史参考
└── crates/
    ├── game-core/          # 纯逻辑核心，无 bevy 依赖，可跨平台复用 + 单测
    │   └── src/
    │       ├── lib.rs      # 对外导出 + 格子坐标→世界坐标换算（全项目唯一一份）
    │       ├── board.rs    # 棋盘：布雷 / 相邻计数 / 翻开泛洪 / 插旗 / 判胜负
    │       ├── levels.rs   # 多关卡设计（尺寸 + 雷数，难度递增）
    │       └── theme.rs    # 素材抽象（配色 / 字体 / 尺寸，换肤=换 Theme）
    ├── platform/           # 各平台适配（按 cfg(target_os) 分派）
    │   └── src/lib.rs      # macOS / Windows / Android：窗口、输入、字体路径
    └── app/                # Bevy 应用层，把上面两者接到一起渲染
        └── src/
            ├── main.rs     # 入口：DefaultPlugins + 窗口 + 日志过滤
            ├── plugin.rs   # 插件装配：资源初始化 + 系统调度
            ├── state.rs    # bevy 资源（Game / 关卡进度 / 悬停 / 主题 / 字体）
            ├── assets.rs   # 素材适配：Theme→Color、字体路径→Font
            ├── board.rs    # 棋盘实体生成 / 重建
            ├── input.rs    # 鼠标输入（桌面）
            └── render.rs   # 状态→外观同步（签名缓存去重，不每帧写）
```

### 设计要点

- **逻辑与渲染分离**：`game-core` 完全不含 bevy，15 个单元测试覆盖布雷、相邻计数、
  点雷判负、空白泛洪、插旗保护、判胜等，`cargo test -p game-core` 即可跑。
- **多关卡**：`game-core/src/levels.rs` 的 `LEVELS` 数组，新增关卡只需追加一项。
- **素材抽象**：`game-core/src/theme.rs` 的 `Theme` 用纯数据（`Rgb`/f32）描述皮肤，
  渲染层负责映射成 bevy 的 `Color`/`FontSize`。
- **平台适配**：`crates/platform` 按 `cfg(target_os)` 返回窗口尺寸、输入方式
  （桌面鼠标 / 移动端触摸）、系统字体候选路径。
- **中文显示**：加载 macOS 自带简体中文黑体（Hiragino Sans GB），`TextLayout` 设
  `LineBreak::AnyCharacter` 按字符断行（中文无空格分词，避免整词被折行）。
- **日志干净**：parley 排版 CJK 文本时 `icu_segmenter` 缺少中文断行字典，debug 构建会
  每 100ms 打一条 `ICU4X data error: ... Chinese/Japanese`。修法是把 `icu_provider` 的
  `logging` feature 打开（走 tracing 管线）再用 `LogPlugin.filter` 的 `icu_provider=off`
  精准屏蔽；debug stderr 已验证为 0 行。原因与排查过程见
  [`docs/bevy-engine.md`](docs/bevy-engine.md) §7.2。
- **引擎用法与规范**：本仓库锁定 Bevy 0.19 的 `features = ["2d"]`（不带 3d/ui/audio），
  0.19 的 API 与网上多数旧教程不同，改动前先看
  [`docs/bevy-engine.md`](docs/bevy-engine.md)（API 清单 + 旧写法对照表）和
  [`docs/coding-standards.md`](docs/coding-standards.md)（依赖方向 + 状态先行 + 测试规范）。

## 平台状态

| 平台 | 输入 | 字体 | 状态 |
|------|------|------|------|
| macOS | 鼠标 | Hiragino Sans GB | ✅ 已验证运行 |
| Windows | 鼠标 | 微软雅黑 | 已配置，待验证 |
| Android | 触摸 | Noto Sans CJK | 已配置，待验证 |

## 测试

```bash
cargo test                    # 全部：15 game-core + 2 platform（app 层暂无测试）
cargo test -p game-core       # 只跑核心逻辑（秒级）
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings   # 基线全绿，0 告警（见 docs/coding-standards.md §8）
cargo fmt --all -- --check                              # 基线全绿，0 差异
```

核心逻辑不依赖 bevy，因此单元测试无需 GPU/窗口即可运行；交互改动请另外跑一遍
`docs/bevy-engine.md` §10 的手动验证清单。
