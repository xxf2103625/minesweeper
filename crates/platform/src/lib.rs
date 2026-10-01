//! # platform
//!
//! 各平台适配层。把“跟平台有关的东西”集中在这里，按 `cfg(target_os)` 分派：
//! - 窗口尺寸 / 标题
//! - 系统字体路径（用于渲染中文，各平台字体位置不同）
//! - 输入方式（桌面用鼠标，移动端用触摸）
//!
//! 无第三方依赖，可在 macOS / Windows / Android / Linux 上编译。

/// 输入方式：桌面是鼠标（左键翻开、右键插旗），移动端是触摸（点按翻开、长按插旗）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Mouse,
    Touch,
}

/// 平台描述，供渲染/输入层查询。
pub struct Platform {
    pub name: &'static str,
    pub input: InputMode,
    /// 主窗口逻辑分辨率（宽, 高）。移动端按竖屏给一个适合手机的尺寸。
    pub window_size: (u32, u32),
    /// 系统字体候选路径（按顺序尝试，第一个存在且能加载的胜出）。
    /// 空数组表示该平台用引擎默认字体即可。
    pub font_candidates: &'static [&'static str],
}

/// 编译期确定当前平台。
pub const fn current() -> Platform {
    #[cfg(target_os = "macos")]
    {
        Platform {
            name: "macOS",
            input: InputMode::Mouse,
            window_size: (520, 620),
            // macOS 自带简体中文黑体（GB 版，非日文字体）
            font_candidates: &[
                "/System/Library/Fonts/Hiragino Sans GB.ttc",
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
            ],
        }
    }
    #[cfg(target_os = "windows")]
    {
        Platform {
            name: "Windows",
            input: InputMode::Mouse,
            window_size: (520, 620),
            // Windows 自带微软雅黑
            font_candidates: &[
                r"C:\Windows\Fonts\msyh.ttc",
                r"C:\Windows\Fonts\simhei.ttf",
                r"C:\Windows\Fonts\simsun.ttc",
            ],
        }
    }
    #[cfg(target_os = "android")]
    {
        Platform {
            name: "Android",
            input: InputMode::Touch,
            // 竖屏手机比例
            window_size: (720, 1280),
            // Android 通常内嵌 Noto Sans CJK；无则回退引擎默认
            font_candidates: &["/system/fonts/NotoSansCJK-Regular.ttc"],
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "android")))]
    {
        Platform {
            name: "other",
            input: InputMode::Mouse,
            window_size: (520, 620),
            font_candidates: &[],
        }
    }
}

/// 返回第一个真实存在的字体路径；都没有则 None（渲染层回退默认字体）。
pub fn available_font() -> Option<&'static str> {
    current()
        .font_candidates
        .iter()
        .copied()
        .find(|p| std::path::Path::new(p).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_is_coherent() {
        let p = current();
        assert!(!p.name.is_empty());
        assert!(p.window_size.0 > 0 && p.window_size.1 > 0);
        // 桌面平台用鼠标，Android 用触摸
        if p.name == "Android" {
            assert_eq!(p.input, InputMode::Touch);
        } else {
            assert_eq!(p.input, InputMode::Mouse);
        }
    }

    #[test]
    fn available_font_is_either_none_or_existing() {
        if let Some(f) = available_font() {
            assert!(std::path::Path::new(f).exists());
        }
    }
}
