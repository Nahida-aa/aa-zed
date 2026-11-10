# PR 草稿（#65655）

- base：`zed-industries/zed:main`
- head：`Nahida-aa:fix/65655-openai-tool-call-empty-name-or-id`
- 标题：`language_model_core: Fail streamed tool calls with an empty id or name`
- 贴之前：按 `CONTRIBUTING.md` 的 AI Policy 用你自己的话改写；先签 https://zed.dev/cla。
  - 英文若是模型改写/翻译的：按政策把英文放进引用块（`>`），中文原文附在后面。
  - 政策要求 build & run 后手动测试，结果写进 Testing。#65859 就是死在没自审。
- 时机：#65859 已于 2026-10-25 被 `zed-community-bot` 以 `no self-review` 关闭（不可重开），可直接开。
- 若开：PR 描述里保留 `Closes #65655`（squash 合并时以 PR 描述为准，commit 里的会被丢掉）。

## 正文

## Summary

`ChatCompletionEventMapper::map_event` only emits a streamed tool call — a
`tool_calls` entry in the provider's response — once it has both a non-empty id
and a name, but the `finish_reason == "tool_calls"` drain pushed every
accumulated call unconditionally. A call whose id never arrived was still
emitted with an empty `tool_call_id`, stored in the thread, and rejected by the
provider on the next request; retrying resent the same history and left the
thread stuck.

This mirrors the per-delta guard on the drain: well-formed siblings are still
emitted, and if any call lacks an id or a name the response fails with a new
`InvalidProviderOutput` error instead of ending the turn as a successful tool
call. The error is not transient, so the agent does not retry it automatically.
The malformed call is never executed, because it is rejected before any
`ToolUse` event is emitted.

I deliberately did not synthesize an id. That would store data the provider
never issued, and deriving it from the stream `index` is not reliable because
providers reuse indices (`separates_parallel_tool_calls_sharing_an_index`).
If you would rather keep the turn going, a `{name}-{position}`-style id would
be unique within the response and easy to trace.

One behavior change worth noting: a rejected response ends with the error and no
`Stop` event. That is documented on `ChatCompletionEventMapper`. The diagnostic
log also truncates the tool call arguments to a 200-character preview: the
arguments are not the subject of this check, and logging them in full could be
large.

Closes #65655.

## Testing

`cargo test -p language_model_core --lib` (75 pass), plus
`cargo clippy -p language_model_core --all-targets -- -D warnings`,
`cargo fmt -p language_model_core -p agent -- --check`, and
`cargo check -p agent`.

New tests drive the mapper directly with a finish chunk and cover:

- an empty or absent id, an empty or absent name, and both missing (six shapes);
- the malformed call is never emitted as a tool use, and no `ToolUse` stop is
  reported;
- a well-formed sibling is emitted, in order, before the error.

Both `issue_65655_*` tests fail if the drain guard is removed, so they cover
the regression rather than the surrounding code.

Manual verification: I built Zed from this branch and pointed a custom
OpenAI-compatible provider at a local stub whose response streams a single tool
call with an empty id and name and finishes with `finish_reason: "tool_calls"`.

- The turn ended with the existing error callout and no tool call was executed.
- The next request in the same thread carried no replayed tool call, and the
  thread kept working normally; the stub never received an empty
  `tool_call_id` (it would have answered 400).
- Both recovery paths work: retrying the failed turn, and sending another
  message. Either request carried the partial assistant text but no tool call,
  and the response completed normally.
- A stream that ends without a `tool_calls` finish still takes the old path, so
  this change only affects responses that actually report tool calls.

## Self-review

- [x] I've reviewed my diff for quality, security, reliability, and performance.
- [x] UI changes follow the checklist. (No `agent_ui` changes; the error renders
      through the existing error callout.)
- [x] Tests cover the new or changed behavior.

Release Notes:

- Fixed agent threads becoming unusable when a streamed tool call arrived
  without an id or a name.

---

## 中文版（自己核对用，不要贴到 PR；贴之前仍按 `CONTRIBUTING.md:48` 用你自己的话改写）

### Summary

`ChatCompletionEventMapper::map_event` 只在一条流式 tool call（provider 响应里的
`tool_calls` 条目）同时拿到非空 id 和 name 之后才发出 `ToolUse`；但
`finish_reason == "tool_calls"` 的 drain 会把
accumulator 里的所有调用无条件发出。于是 id 始终没到的调用会带着空
`tool_call_id` 被发出、写进会话，并在下一次请求 replay 时被 provider 拒绝；重试
（resume）会带上同样的历史，线程就此卡死。

本次把逐 delta 的守卫同样加到 drain 上：格式正确的兄弟调用照常发出；只要任一调用
缺 id 或 name，就让这一轮响应以新的 `InvalidProviderOutput` 错误结束，而不是当成
一次成功的 tool call 收尾。该错误不是 transient，agent 不会自动重试；坏调用在
发出 `ToolUse` 之前就被拒绝，因此从未被执行。

我刻意没有合成 id：那会把 provider 从未下发过的数据写进会话；而拿流里的 `index`
来推导也不可靠，因为 provider 会复用 index（见
`separates_parallel_tool_calls_sharing_an_index`）。如果维护者更希望这一轮继续，
`{name}-{position}` 形式的 id 在响应内唯一，也便于排障。

有一处行为变化值得注意：被拒绝的响应以错误结束，不再产生 `Stop` 事件；这一点已
写进 `ChatCompletionEventMapper` 的文档。诊断日志里对工具调用的参数只保留前 200
字符的预览——参数不是这次检查的重点，完整打印可能很长。

Closes #65655.

### Testing

`cargo test -p language_model_core --lib`（75 通过），以及
`cargo clippy -p language_model_core --all-targets -- -D warnings`、
`cargo fmt -p language_model_core -p agent -- --check`、`cargo check -p agent`。

新增测试直接用 finish chunk 驱动 mapper，覆盖：

- id 为空/缺失、name 为空/缺失、以及两者都缺（六种形态）；
- 坏调用绝不作为 tool use 发出，也不报告 `ToolUse` 的 stop；
- 格式正确的兄弟调用仍在报错之前按顺序发出。

去掉 drain 守卫后，两个 `issue_65655_*` 用例都会失败，说明它们覆盖的是这次回归，
而不是无关代码。

手动验证：用本分支构建 Zed，把自定义 OpenAI 兼容 provider 指向本地 stub，让它流出一条
id 和 name 都为空的 tool call，并以 `finish_reason: "tool_calls"` 结束。

- 这一轮以错误 callout 结束，工具没有被执行；
- 同一 thread 的下一次请求里没有任何被 replay 的 tool call，线程继续正常工作；stub 也
  从未收到空 `tool_call_id`（收到会直接回 400）；
- 两种恢复方式都正常：点 Retry，或直接发下一条消息。两种请求里都只有上次的部分
  assistant 文本、没有 tool call，响应正常完成；
- 流在没有 `tool_calls` finish 的情况下结束仍走旧路径，所以本次只影响真正报告了 tool
  call 的响应。

### Self-review

- [x] 已检查 diff 的质量、安全、可靠性与性能。
- [x] UI 变更符合 checklist（无 `agent_ui` 改动；错误走现有 error callout）。
- [x] 测试覆盖新增或变更的行为。

Release Notes:

- 修复流式 tool call 缺少 id 或 name 时 agent 线程不可用的问题。
