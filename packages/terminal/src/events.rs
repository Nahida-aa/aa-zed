use std::{
    fmt::{self, Formatter},
    path::PathBuf,
    process::ExitStatus,
    sync::Arc,
};

use crate::{
    alacritty::HyperlinkMatch,
    grid::geometry::bounds::TerminalBounds,
    grid::geometry::point::Point,
    input::scroll::{Scroll, ViMotion},
    input::selection::Selection,
};
use alacritty_terminal::vte::ansi::Rgb;
use gpui::{Pixels, Point as GpuiPoint};

///Upward flowing events, for changing the title and such \
/// Terminal 报给 视图 的事件 \
/// `Terminal` 不持有视图句柄，不能直接改 UI； \
/// 它唯一能做的就是把"发生了什么" `cx.emit` 出去 \
/// 。视图层在 `packages/terminal_view/src/view/events.rs:22` \
/// 用 `cx.subscribe_in(terminal, ...)` **一处**订阅全部，按变体分支
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    TitleChanged,       // shell 改窗口标题（OSC 0/2）
    BreadcrumbsChanged, // 面包屑目录变了
    OutputReplaced,     // ACP 远程会话**权威替换**整屏输出
    CloseTerminal,
    // 程序响铃
    Bell,
    // **"屏幕脏了，请重画"**——最高频核心
    Wakeup,
    // 光标闪烁开关态
    BlinkChanged(bool),
    // 文本选中发生/变化/清除
    SelectionsChanged,
    // 当前鼠标悬停的链接：`Some` 有链接、`None` 清除
    NewNavigationTarget(Option<MaybeNavigationTarget>),
    // 用户**点了**链接
    Open(MaybeNavigationTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathLikeTarget {
    /// File system path, absolute or relative, existing or not.
    /// Might have line and column number(s) attached as `file.rs:1:23`
    pub maybe_path: String,
    /// Current working directory of the terminal
    pub working_directory: Option<PathBuf>,
}

/// A string inside terminal, potentially useful as a URI that can be opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaybeNavigationTarget {
    /// HTTP, git, etc. string determined by the `URL_REGEX` regex.
    Url(String),
    /// File system path, absolute or relative, existing or not.
    /// Might have line and column number(s) attached as `file.rs:1:23`
    PathLike(PathLikeTarget),
}

// 视图输入回调（input/ 各模块：键盘、鼠标、滚轮、选中、vi 按键…）
//    │  只做一件事：入队意图，不立即执行
//    │  self.events.push_back(InternalEvent::SetSelection(...))
//    ▼
// Terminal.events : VecDeque<InternalEvent>      （pty/mod.rs 的字段，主线程队列）
//    │
//    ▼  视图每次重绘前的 pre-paint 统一排空
// Terminal::sync()（grid/content/content.rs:175）
//    │  while let Some(e) = self.events.pop_front() { process_internal_event(e) }
//    ▼
// 真正执行（锁 term、滚 grid、发剪贴板、查链接……）
#[derive(Clone)]
pub enum InternalEvent {
    Resize(TerminalBounds),
    Clear,
    // FocusNextMatch,
    Scroll(Scroll),
    ScrollToPoint(Point),
    SetSelection(Option<Selection>),
    UpdateSelection(GpuiPoint<Pixels>),
    FindHyperlink(GpuiPoint<Pixels>, bool),
    ProcessHyperlink(HyperlinkMatch, bool),
    // Whether keep selection when copy
    Copy(Option<bool>),
    // Vi mode events
    ToggleViMode,
    ViMotion(ViMotion),
    MoveViCursorToPoint(Point),
}

#[derive(Clone)]
pub enum TerminalBackendEvent {
    MouseCursorDirty,
    Title(String),
    ResetTitle,
    ClipboardStore(String),
    ClipboardLoad(ClipboardFormatter),
    ColorRequest(usize, ColorFormatter),
    PtyWrite(String),
    TextAreaSizeRequest(TextAreaSizeFormatter),
    CursorBlinkingChange,
    Wakeup,
    Bell,
    Exit,
    ChildExit(ExitStatus),
}
impl fmt::Debug for TerminalBackendEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MouseCursorDirty => f.write_str("MouseCursorDirty"),
            Self::Title(title) => write!(f, "Title({title})"),
            Self::ResetTitle => f.write_str("ResetTitle"),
            Self::ClipboardStore(data) => write!(f, "ClipboardStore({data})"),
            Self::ClipboardLoad(_) => f.write_str("ClipboardLoad"),
            Self::ColorRequest(index, _) => write!(f, "ColorRequest({index})"),
            Self::PtyWrite(output) => write!(f, "PtyWrite({output})"),
            Self::TextAreaSizeRequest(_) => f.write_str("TextAreaSizeRequest"),
            Self::CursorBlinkingChange => f.write_str("CursorBlinkingChange"),
            Self::Wakeup => f.write_str("Wakeup"),
            Self::Bell => f.write_str("Bell"),
            Self::Exit => f.write_str("Exit"),
            Self::ChildExit(status) => write!(f, "ChildExit({status})"),
        }
    }
}

/// 终端输出后端与前台 [`Terminal`] 之间通道上的消息类型：上游 zed #57483
/// 把这些 backend-neutral 的事件类型从 alacritty 里拆了出来。
///
/// 刻意的单变体 enum：将来要加新消息种类，加变体即可，通道签名不用动。
///
/// `u64` 是事件产生时的输出代（upstream #64976 “authoritative terminal display
/// updates”）。[`Terminal::process_pty_event`] 会丢弃 generation 与终端当前
/// `output_generation` 不一致的事件——防止在整体替换显示输出
/// （[`Terminal::replace_display_output`]，ACP 驱动的终端用）之前
/// 已在通道里积压的陈旧事件污染新画面。本地 pty 后端永远发 generation 0。
pub enum PtyEvent {
    Event(TerminalBackendEvent, u64),
}

type ClipboardFormatter = Arc<dyn Fn(&str) -> String + Sync + Send + 'static>;
type ColorFormatter = Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>;
type TextAreaSizeFormatter = Arc<dyn Fn(TerminalBounds) -> String + Sync + Send + 'static>;
