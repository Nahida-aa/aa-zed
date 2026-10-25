# fake-provider

给 zed 的 `openai_compatible` provider 用的本地假端点，用来复现「流式 tool call 缺 id/name」
（zed issue #65655）。不依赖真实模型，也不会改你的 zed 配置。

## 切换正确 / 错误

```bash
cd .agents/fake-provider
echo malformed > mode   # 返回：思考内容 + 一条 id 和 name 都为空的 tool call（触发报错）
echo normal    > mode   # 返回：普通文本 "stub ok"
```

服务每次请求都重新读 `mode`，**不用重启**。

## 启动 stub

```bash
systemd-run --user --unit=stubbun --collect bun .agents/fake-provider/stub_server.ts
journalctl --user -u stubbun -f          # 看启动/异常
systemctl --user stop stubbun             # 停止
```

默认 `127.0.0.1:8765`，日志写在 `/tmp/opencode/stub_requests.jsonl`（每次请求一行 JSON）。

## 启动测试用 zed

```bash
.agents/fake-provider/run-zed.sh          # 默认用 zed repo 的 debug 二进制
STUB_PROXY=http://127.0.0.1:8080 .agents/fake-provider/run-zed.sh   # 顺便走 mitmweb
systemctl --user stop zedtest
```

要点：

- 配置会被拷到 `<data>/config/settings.json`（`--user-data-dir` 会改配置目录），
  数据目录默认 `/tmp/opencode/zedtest/data`，和日常 zed 完全隔离。
- 密钥走环境变量：`STUB_API_KEY=stub`。名字来自 provider id 的 upper-snake 形式
  （`stub` → `STUB_API_KEY`），**必须在启动前设置**；启动后再热加 provider 不会认证，
  会报 "No API key is configured"。
- 用新的 thread 测；错误 callout 不会自动消失，点 ✕ 或 Retry（Retry 会清掉并重发）。
- 想看完整请求体：`tail -1 /tmp/opencode/stub_requests.jsonl | python3 -m json.tool`，
  `empty_tool_call_id` 为 `true` 时 stub 会直接回 400 —— 出现即说明空 id 被写进了历史。
