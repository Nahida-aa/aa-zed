import { appendFileSync } from "node:fs";

const MODEL = "stub-model";
const PORT = Number(process.env.STUB_PORT ?? 8765);
const DIR = import.meta.dir;
const MODE_FILE = `${DIR}/mode`;
const LOG = process.env.STUB_LOG ?? "/tmp/opencode/stub_requests.jsonl";

type ToolCall = {
  index: number;
  id?: string;
  type?: "function";
  function?: { name?: string; arguments?: string };
};

type Delta = {
  role?: "assistant";
  content?: string;
  reasoning_content?: string;
  tool_calls?: ToolCall[];
};

type Chunk = {
  id: string;
  object: "chat.completion.chunk";
  created: number;
  model: string;
  choices: Array<{ index: number; delta: Delta; finish_reason: string | null }>;
};

const chunks = (...items: Chunk[]) =>
  items.map((item) => `data: ${JSON.stringify(item)}\n\n`).join("");
const now = () => Math.floor(Date.now() / 1000);

const event = (delta: Delta, finish_reason: string | null = null): Chunk => ({
  id: "chatcmpl-stub",
  object: "chat.completion.chunk",
  created: now(),
  model: MODEL,
  choices: [{ index: 0, delta, finish_reason }],
});

const done = "data: [DONE]\n\n";

function malformedStream(): string {
  return (
    chunks(
      event({
        role: "assistant",
        reasoning_content: "I should list the project directory first.",
      }),
      event({
        role: "assistant",
        tool_calls: [
          {
            index: 0,
            id: "",
            type: "function",
            function: { name: "", arguments: '{"path":"."}' },
          },
        ],
      }),
      event({}, "tool_calls"),
    ) + done
  );
}

function normalStream(text = "stub ok"): string {
  return chunks(event({ role: "assistant", content: text }), event({}, "stop")) + done;
}

type Message = {
  role?: string;
  tool_call_id?: string;
  tool_calls?: Array<{ id?: string }>;
  content?: unknown;
};

function hasEmptyToolCallId(messages: Message[]): boolean {
  for (const message of messages) {
    if (message.role === "tool" && !message.tool_call_id) return true;
    for (const call of message.tool_calls ?? []) {
      if (!call.id) return true;
    }
  }
  return false;
}

async function readMode(url: URL): Promise<string> {
  const override = url.searchParams.get("mode");
  if (override) return override;
  try {
    return (await Bun.file(MODE_FILE).text()).trim() || "malformed";
  } catch {
    return "malformed";
  }
}

const json = (status: number, body: unknown) => Response.json(body, { status });

Bun.serve({
  port: PORT,
  hostname: "127.0.0.1",
  async fetch(request) {
    const url = new URL(request.url);

    if (request.method === "GET") {
      if (url.pathname.replace(/\/+$/, "").endsWith("/models")) {
        return json(200, { object: "list", data: [{ id: MODEL, object: "model" }] });
      }
      return json(404, { error: { message: "not found" } });
    }

    if (request.method !== "POST") {
      return json(405, { error: { message: "method not allowed" } });
    }

    let body: { messages?: Message[] } = {};
    try {
      body = await request.json();
    } catch {
      body = {};
    }
    const messages = body.messages ?? [];
    const mode = await readMode(url);
    const emptyId = hasEmptyToolCallId(messages);

    appendFileSync(
      LOG,
      JSON.stringify({ path: url.pathname, mode, empty_tool_call_id: emptyId, messages }) + "\n",
    );

    if (emptyId) {
      return json(400, {
        error: {
          message: "Invalid parameter: 'tool_call_id' must not be empty",
          type: "invalid_request_error",
        },
      });
    }

    const payload = mode === "malformed" ? malformedStream() : normalStream();

    return new Response(payload, {
      headers: { "content-type": "text/event-stream", "cache-control": "no-cache" },
    });
  },
});

console.log(`stub listening on http://127.0.0.1:${PORT} (mode file: ${MODE_FILE})`);
