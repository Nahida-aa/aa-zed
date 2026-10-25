<!-- intent-skills:start -->

## Skill Loading

Before editing files for a substantial task:

- Run `bunx @tanstack/intent@latest list` from the workspace root to see available local skills.
- If a listed skill matches the task, run `bunx @tanstack/intent@latest load <package>#<skill>` before changing files.
- Use the loaded `SKILL.md` guidance while making the change.
- Monorepos: when working across packages, run the skill check from the workspace root and prefer the local skill for the package being changed.
- Multiple matches: prefer the most specific local skill for the package or concern you are changing; load additional skills only when the task spans multiple packages or concerns.

<!-- intent-skills:end -->

- 修改代码后, 如果认为适合提交, 就自行提交
- 禁止删除或手动修改 `Cargo.lock`

## Debug

- 使用 tracing

## Code Navigation

Use `ast-grep outline` before reading full source files when exploring code.

- Use `ast-grep outline <file>` to inspect a candidate file.
- Use `ast-grep outline <dir> --items exports` to find public entry points.

After finding candidate files, use `ast-grep outline` to decide which source
range to read. If your agent supports skills, use the `ast-grep-outline`
skill for more detailed guidance.

## ⚠️ 主线声明

**GPUI 桌面（packages/app + packages/workspace + packages/ui）是唯一主线。**

以下前端方案是历史遗留或实验性的，**不保证能编译、能运行或与主线兼容**：

- `packages/cli/` — Ratatui TUI（旧 CLI 入口）
- `packages/tui/` — OpenTUI + Solid.js TUI（Bun）
- `packages/webui/` — Solid.js Web 前端（TanStack Router）
- `packages/ui-solid/` — Kobalte UI 组件库
- `packages/shared/` — TS 共享 lib

`cargo check --workspace` 只要求主线相关包通过（`kernel`、`core`、`llm`、`extensions`、`extension-sdk`、`extension-mcp`、`function-tools`、`config`、`session`、`server`、`app`、`workspace`、`ui`）。其他包即使编译失败也不阻塞提交。

---

所有代码统一放在 `packages/` 下，不分 Rust/TS：

**非主线（仅参考/历史，不保证编译）：**

- `packages/cli/` 旧 CLI 入口（`aa run` 行模式、Ratatui TUI）
- `packages/tui/` OpenTUI + Solid.js TUI（独立 Bun 进程）
- `packages/webui/` Solid.js + TanStack Router Web 前端
- `packages/ui-solid/` Kobalte 共享 UI 组件库
- `packages/shared/` TS 共享 lib（utils、i18n）
- `packages/sdk-ts/` 自动生成的 TypeScript SDK（openapi-ts）

**Rust edition**: 2024（`set_var`/`remove_var` 需 `unsafe` 块）

📁 **目录名 vs Cargo.toml name 规则**：目录名用简洁形式（如 `kernel/`），`package.name` 用 `aa-` 前缀（如 `aa-kernel`）。不要混用。

## 架构决策

- **Server 层不单独拆 C/S**，内嵌在 `packages/server/` 作为同进程调用边界
- **主线（GPUI 桌面）直接调 `session::run_turn()`**，不走网络、不走 SSE
- `packages/server/` 可选暴露 HTTP/SSE（`aa serve`）给远程客户端（非主线前端用）

## GPUI 桌面架构（主线）

- `packages/app/` — 应用入口：主窗口、标题栏（SSD/CSD 条件渲染）、Agent 面板
- `packages/workspace/` — Workspace 框架：StatusBar（left/right/hidden items）、DockSide、面板切换
- `packages/ui/` — 共享 GPUI UI 组件
- **GPUI 来源**：Zed git rev `bd747337`（pin 在根 Cargo.toml，共 6 个 `gpui*` + `collections` 等 29 个 zed crate 一起锁）
- **UI 组件来源**：`ui` = `aa_gpui_kit_ui`，从 `gpui_learn` 的 git rev 拉（`Cargo.toml` 约 536 行）→ 改 gpui rev 后要同步这个 rev
- **WindowControlArea**：`Drag` + `Close/Minimize/Maximize` 按钮布局（吸收自 aa-player）
- **服务器调用**：同进程 `session::run_turn()`，不经过 HTTP

## 非主线架构（仅供参考，不保证工作）

### 旧 CLI（Ratatui）

- `aa run` — 行模式对话
- `aa` — Ratatui TUI

### OpenTUI + Solid.js TUI

- 独立 Bun 进程，localhost HTTP/SSE 连 Rust server
- `@opentui/*` 必须 0.3.4（0.4.1 Solid context 不传播）
- JSX 通过 Babel 插件转换，需 `bunfig.toml` + `--conditions=browser`

### WebUI（Solid.js + TanStack）

- 通过 HTTP/SSE 连接 Rust server

## 已验证

- **Ollama 端到端**：`aa run --provider ollama --model gemma4:31b-cloud` 成功返回 "Hello"（462 prompt + 2 completion tokens）
- **TUI 编译通过**（无终端时 panic 属预期）
- **`Config::resolve()` 正确性**：cli 传 `Option`，不覆盖 provider 默认值（ollama→`http://localhost:11434`）
- **`RunArgs` 改为 `Option<String>`**：避免 clap default 覆盖 provider 特定默认值

纠错记录见 `.agents/CORRECTIONS.md`。

## 从 zed 搬代码

**搬 zed 的包之前，先读 `.agents/zed-port.md`。**

zed 参照仓库在 `~/repos/ide_ls/learn_ls/zed`，单文件 crate 拆分后的可见性转发、
`collections::` vs `std::collections::`、`RelPath` vs `std::path::Path`、
模块遮蔽要用 `::rpc::` 绝对路径等坑都记在那儿，照搬时直接套用，别重新推演。

## rev→rev fork 同步（**必须先读 `.agents/zed-modifications.md`**）

同步 zed rev（afecd6d → c019950 这类区间）或 fork crate 时，**第一步必须读
`.agents/zed-modifications.md`**，尤其是：

**每次同步的批次计划与进度在 `.agents/fork-sync/<old7>_<new7>.md`**
（`just fork-sync new` 生成、`just fork-sync list` 查看）。开始同步前先看对应批次
文件的「进度追踪」表，接手别人的活儿时尤其要看「踩的坑」。

- **第 1 节 命名空间/命名映射** —— `zed_actions`→`aacode_actions`、
  `cloud_llm_client`→`aa_cloud_llm_client`/`zed_cloud_llm_client` 等。
  cp 上游源码前先查这张表，确认目标 crate 名在 aacode Cargo.toml 里的对应关系。
- **第 2 节 结构性重组** —— 上游单文件拆成 aacode 多文件（`terminal_panel`→`panel`、
  `conversation_view.rs`→目录等）。模块路径可能不同。

**典型踩坑**（已发生多次）：

- cp `edit_prediction` 时源码里是 `cloud_llm_client`，但 aacode Cargo.toml 里根本没有
  这个 crate —— 必须先 grep workspace.dependencies 里的实际名字
- cp `agent_ui` 时源码里是 `zed_actions`，但 aacode 已 rename 成 `aacode_actions` ——
  源码里所有 crate 引用都要手动改
- 这两项在 zed-modifications.md 第 1 节里**明明白白列出来了**，只是没读

**正确流程**：

1. 读 `.agents/zed-modifications.md` 第 1 节 + 第 2 节
2. **清单必须从上游客观反推，不许靠记忆列举**（见下）
3. 对每个要同步的 crate，先 `grep '<crate>' packages/<path>/Cargo.toml` 验证依赖名
4. cp 上游 diff，按第 1 节映射表改 crate 名、按第 2 节改模块路径
5. 每个 crate 同步完跑 `cargo check -p <crate>`，不过不进下一个
6. **收尾做「消费方验收」**（见下）
7. 最后 `cargo check --workspace` exit=0 才算完成

### 清单反推原则（第 2 步）

**不要凭印象列"这次要同步哪些 crate"** —— 2026-10 的 `afecd6d→c019950`
就因为清单里漏了 `edit_prediction_ui`，导致 commit `bee7a3f7fe` 的 8 处改动
（4 处菜单文案 + 4 处逻辑）**一项都没落地**，且进度表上 `edit_prediction`
还标着 ✅，看不出来漏了。

先从上游客观拿到完整文件清单，再逐个映射：

```bash
# 区间内所有改动文件（按 crate 聚合）
git -C ~/repos/ide_ls/learn_ls/zed diff --stat <OLD>..<NEW> -- crates/

# 单个提交（cherry-pick 尤其要看，容易被"已处理过"的印象跳过）
git -C ~/repos/ide_ls/learn_ls/zed show --stat <sha>
```

两个要点：

- **一个功能常跨多个 crate**。不要按功能名猜归属，按**实际文件路径**映射。
  当时的错误就是把 `edit_prediction_ui/.../edit_prediction_button.rs`（状态栏按钮
  和点击后的菜单）记成了 `packages/edit_prediction`，于是跟着错误的路径去核对，
  自然"看起来没问题"。
- **依赖 crate 已同步 ≠ 消费 crate 已同步**。

### 消费方验收（第 6 步）

上游若在 crate A 新增了 API，通常会在 crate B 调用它。**只把 A 同步了、B 没同步
是典型的静默漏同步** —— 编译照过、运行时看不出，只有对照上游才发现。

机械可查：上游新增/改动的公开函数，在 aacode 里若**存在定义却没有任何调用方**，
基本就是消费方漏了。

```bash
# 例：上游 commit 新增 zed_edit_predictions_excluded_from_plan
grep -rn "zed_edit_predictions_excluded_from_plan" packages/   # 定义在哪
grep -rn "excluded_from_plan" packages/edit_prediction_ui/     # 消费方有没有用
```

2026-10 的实例：`packages/edit_prediction` 和 `packages/client` 里的
`excluded_from_plan` 相关函数**都在** ✅，但消费它们的
`packages/edit_prediction_ui/src/edit_prediction_button.rs` **一处都没用** ❌。
按上面两条 grep 当场就能发现。

## 查上游版本/release：优先用 `tools/gh-releases.ts`

问「zed 最新到哪个版本 / 某 rev 对应什么 / 某功能在哪个版本进的」时，
**不要用网页搜索**，直接跑：

```bash
bun tools/gh-releases.ts zed-industries/zed --latest      # 最新稳定版 tag + sha
bun tools/gh-releases.ts zed-industries/zed --limit 15     # 按时间倒序列表
```

它走 GitHub GraphQL 拿 tag / commit sha / publishedAt，**比网页搜索准确**：
搜索结果常是缓存的旧快照，曾据此误判「不存在 1.23」，而实际 1.23.2 已于
2026-10-07 发布。核对 rev 时还能直接对着 sha 走本地 zed 仓库。

网页搜索只用于「release notes 说了什么」「官方公告/定价」这类 GraphQL 拿不到的内容。

## `.agents/` 内容索引

`.agents/` 是本项目的知识库，按用途分：

| 路径                         | 用途                                                                                                                     |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `zed-port.md`                | **搬 zed 包之前必读**，§1~§15 是坑，§16 是 rev→rev 同步索引                                                              |
| `zed-modifications.md`       | aacode 相对 zed 的**有意改动**（命名映射、结构重组、各 crate 改动）                                                      |
| `CORRECTIONS.md`             | 架构纠错记录与技术细节                                                                                                   |
| `fork-sync/<old7>_<new7>.md` | **每次 rev→rev 同步的计划与记录**，一个批次一个文件                                                                      |
| `memory/`                    | 排查方法论与项目缺口清单                                                                                                 |
| `zed/`                       | zed 相关研究笔记（agent、workspace、gpui action 派发等）                                                                 |
| `learn.local.md`             | 本地参考项目路径清单                                                                                                     |
| `skills/`                    | 自定义 skill 定义（ast-grep / cargo / gpui-test / outline）                                                              |
| `toolcell.md`                | 各家 agent 工具对照（opencode / codex / zed / claude code），目前多为待填                                                |
| `issues/`                    | 单个问题/PR 记录：上游 zed issue（`pr-*` / `review-*` / `pr-issue-*`）与自研问题（`YYYY-MM-DD-*`，如工具调用流截断排查） |
| `fake-provider/`             | Bun 写的 OpenAI 兼容假端点，复现流式 tool call 缺 id/name（zed #65655），`mode` 文件切换正常/异常响应 |

> 新增 `.agents/` 下的文档时，回来更新这张表，避免内容变成孤儿。

## 排查经验（`.agents/memory/`）

- **UI「内容不显示」类 bug**：先在数据源函数入口打一行 `len()` 确认数据非空，再碰渲染层。
  指标全正常时立即停止、回头复核用户诉求。详见 `.agents/memory/debug-ui-empty.md`。
- **Zed init 链缺口**：`init` 链缺失不报编译错、大多不 panic，只表现为功能静默失效
  （如缺 provider 注册 → 某个设置子页全白且零报错）。移植新功能时 diff 一遍上游
  `zed/src/main.rs` 的 init 链。已知缺口与排查法见 `.agents/memory/zed-init-chain-gaps.md`。
