## 先花 5 分钟建立全景（不用读代码，只看这张图）

```
                    【输入方向】                              【输出方向】
  TerminalView(独立crate)                                    alacritty 事件循环 (后台线程)
        │ 按键/鼠标/滚轮/选中                                        │ 标题/退出/Wakeup…
        ▼                                                          ▼
   input/ (keyboard, mouse, scroll, selection, bytes)      alacritty/listener.rs  ZedListener
        │ write_input  ──▶ alacritty/pty.rs PtySender::notify ──▶ term 模拟器
        │                                                    alacritty/ops.rs · grid/*
                                                                 │
  pty/builder.rs subscribe() 的事件泵 ◀── UnboundedSender ───────┘
        │
        ▼
   pty/process/dispatch.rs  process_pty_event → process_event / process_internal_event
        │
        ▼
   Terminal 实体状态更新 → 通知 TerminalView 重新渲染
```

记住一句话：**`pty/` 是骨架（Terminal 本体 + 接线），`alacritty/` 是引擎（模拟器 + 后台线程），`input/` 是双手，`grid/` 是词典（纯数据类型，查就好）**。

## 推荐阅读顺序（由轻到重）

| 步  | 文件                                         | 看什么                                                                                                            | 时间  |
| --- | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | ----- |
| 1   | `lib.rs`                                     | 模块树 + 全部 re-export = **公共 API 面**，一张地图                                                               | 10min |
| 2   | `events.rs`                                  | `PtyEvent` / `TerminalBackendEvent` / `InternalEvent` / `Event` —— 这是**通信协议词汇表**，后面每个模块都围着它转 | 15min |
| 3   | `pty/mod.rs` 的 `Terminal` struct（L41 起）  | 字段全是注释自解释，读字段名 + 注释 = 读状态机                                                                    | 15min |
| 4   | `pty/builder.rs` 的 `new()` / `subscribe()`  | 装配 + 通道接线（上两轮已经聊透，扫一眼确认）                                                                     | 20min |
| 5   | `pty/process/dispatch.rs`（原 process.rs）   | **事件消费端**：`process_pty_event → process_event → process_internal_event`，"每种事件把终端状态改成什么样"      | 30min |
| 6   | `alacritty/listener.rs` + `alacritty/pty.rs` | 引擎侧：`ZedListener`（sink）+ 开 pty / 起事件循环                                                                | 20min |
| 7   | 按需                                         | `input/*`（双手）、`alacritty/ops.rs`（grid 操作）、`grid/*`（查词典）                                            | —     |

## 两个提效技巧

- **别按目录顺序读，按场景读**：挑一个场景从头追到尾，比逐文件啃快得多。最好的第一个场景是**"打一个字符"**：`TerminalView 的 handle_input → input/bytes write_input → PtySender::notify → alacritty term → Wakeup 事件 → 通道 → subscribe 泵 → dispatch → 渲染`。第二场景追 **"shell 退出"**（`ChildExit → Event::Exit`）。两个场景走完，主干全通。
- 项目 AGENTS.md 也推荐了：**读源码前先 `ast-grep outline <dir>`** 拿结构骨架，比如 `ast-grep outline packages/terminal/src/pty --items exports`，能省掉大量无效翻读。

## 一个阅读时容易卡住的点（提前打预防针）

`dispatch.rs` 里对 `TerminalType::Pty { .. }` vs `DisplayOnly` 的分叉非常多（本地 pty / 无头子进程 / ACP 远程显示三种形态共用一套状态机）。第一遍**只看 Pty 分支**，DisplayOnly 分支留到读 `acp_thread` / `terminal_view` 的远程终端时再回来，否则会三条线混在一起。
