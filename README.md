# Bevy 扫雷（多平台）

基于 [Bevy](https://bevy.org) 游戏引擎的 2D 扫雷，参考
[qongzi 的 Bevy Minesweeper 教程](https://dev.to/qongzi/bevy-minesweeper-introduction-4l7f)。
当前使用 **Bevy 0.19**，依赖全部为最新版。

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
└── crates/
    ├── game-core/          # 纯逻辑核心，无 bevy 依赖，可跨平台复用 + 单测
    │   └── src/
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
- **中文显示**：加载 macOS 自带简体中文黑体（Hiragino Sans GB），并设
  `LineBreak::AnyCharacter` 绕开 parley/icu_segmenter 的中日文词切分，
  消除 `ICU4X data error: No segmentation model...` 提示。

## 平台状态

| 平台 | 输入 | 字体 | 状态 |
|------|------|------|------|
| macOS | 鼠标 | Hiragino Sans GB | ✅ 已验证运行 |
| Windows | 鼠标 | 微软雅黑 | 已配置，待验证 |
| Android | 触摸 | Noto Sans CJK | 已配置，待验证 |

## 测试

```bash
cargo test          # 全部：15 core + 2 platform
cargo test -p game-core
```
