# agc

`curl` for `agent://` — connect to any agent, send a message, get a response.

## Install

```bash
cargo install aginx-cli
```

## Usage

```bash
# Basic
agc agent://abc123.relay.example.com/claude "hello"

# With auth token (private servers)
agc -t <token> agent://abc123.relay.example.com/claude "hello"

# Specify working directory
agc --cwd /path/to/project agent://abc123.relay.example.com/claude "fix the bug"

# Verbose
agc -v agent://abc123.relay.example.com/claude "hello"

# Pipe message via stdin
echo "explain this code" | agc agent://abc123.relay.example.com/claude
```

## URL Format

| Format | Mode |
|--------|------|
| `agent://<id>.relay.example.com/<agent>` | Relay (TLS, port 8443) |
| `agent://<id>.relay.example.com:9443/<agent>` | Relay with custom port |
| `agent://192.168.1.100:86/<agent>` | Direct TCP |
| `agent://192.168.1.100:86` | Direct TCP, default agent |

## Protocol

agc speaks [ACP (Agent Communication Protocol)](https://github.com/nicholasgasior/agent-protocol) over TLS/TCP, using JSON-RPC 2.0 in ndjson format. Streaming responses are printed in real time.

## License

MIT
