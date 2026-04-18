mod url;

use std::io::Write;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

use url::AgentUrl;

static NEXT_ID: AtomicI64 = AtomicI64::new(1);
const RPC_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const SHORT_TIMEOUT: Duration = Duration::from_secs(30);

fn next_id() -> i64 {
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

// ── CLI ──

#[derive(Parser)]
#[command(name = "agc", about = "Agent protocol client — curl for agent://")]
struct Cli {
    /// agent:// URL (e.g. agent://id.relay.yinnho.cn/claude)
    url: String,

    /// Message to send (use -- before message if it starts with -)
    message: Option<String>,

    /// Auth token
    #[arg(short, long)]
    token: Option<String>,

    /// Working directory for the session
    #[arg(short, long)]
    cwd: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

// ── Connection ──

enum Connection {
    Tls {
        reader: BufReader<ReadHalf<TlsStream<TcpStream>>>,
        writer: WriteHalf<TlsStream<TcpStream>>,
    },
    Tcp {
        reader: BufReader<ReadHalf<TcpStream>>,
        writer: WriteHalf<TcpStream>,
    },
}

impl Connection {
    async fn connect(parsed: &AgentUrl) -> anyhow::Result<Self> {
        let stream = tokio::time::timeout(
            CONNECT_TIMEOUT,
            TcpStream::connect((parsed.relay_host.as_str(), parsed.port)),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Connection timeout to {}:{}", parsed.relay_host, parsed.port))??;

        let mut conn = if parsed.use_tls {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let config = rustls::ClientConfig::builder_with_provider(
                Arc::new(rustls::crypto::ring::default_provider()),
            )
            .with_safe_default_protocol_versions()?
            .with_root_certificates(root_store)
            .with_no_client_auth();
            let connector = TlsConnector::from(Arc::new(config));
            let domain = rustls_pki_types::ServerName::try_from(parsed.tls_domain.clone())
                .map_err(|e| anyhow::anyhow!("Invalid TLS domain: {}", e))?;
            let tls_stream = connector.connect(domain, stream).await?;
            let (r, w) = tokio::io::split(tls_stream);
            Connection::Tls {
                reader: BufReader::new(r),
                writer: w,
            }
        } else {
            let (r, w) = tokio::io::split(stream);
            Connection::Tcp {
                reader: BufReader::new(r),
                writer: w,
            }
        };

        // Relay handshake (only for relay URLs)
        if let Some(ref target) = parsed.relay_target {
            conn.send(json!({
                "type": "connect",
                "target": target
            }))
            .await?;
            let resp = conn.recv().await?;
            match resp.get("type").and_then(|v| v.as_str()) {
                Some("connected") => {}
                Some("error") => {
                    let msg = resp["message"].as_str().unwrap_or("Unknown relay error");
                    anyhow::bail!("Relay error: {}", msg);
                }
                other => anyhow::bail!("Unexpected relay response: {:?}", other),
            }
        }

        Ok(conn)
    }

    async fn send(&mut self, msg: Value) -> anyhow::Result<()> {
        let mut data = serde_json::to_string(&msg)?;
        data.push('\n');
        match self {
            Connection::Tls { writer, .. } => {
                writer.write_all(data.as_bytes()).await?;
                writer.flush().await?;
            }
            Connection::Tcp { writer, .. } => {
                writer.write_all(data.as_bytes()).await?;
                writer.flush().await?;
            }
        }
        Ok(())
    }

    async fn recv(&mut self) -> anyhow::Result<Value> {
        loop {
            let line = match self {
                Connection::Tls { reader, .. } => read_line(reader).await?,
                Connection::Tcp { reader, .. } => read_line(reader).await?,
            };
            let line = match line {
                Some(l) => l,
                None => anyhow::bail!("Connection closed"),
            };
            if line.is_empty() {
                continue;
            }
            // Skip ping/pong heartbeats and other control messages
            if let Ok(val) = serde_json::from_str::<Value>(&line) {
                match val.get("type").and_then(|v| v.as_str()) {
                    Some("ping") | Some("pong") => continue,
                    _ => return Ok(val),
                }
            } else {
                continue;
            }
        }
    }

    /// Send a JSON-RPC request and collect streaming response
    async fn rpc(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = next_id();
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))
        .await?;

        let mut text_parts: Vec<String> = Vec::new();
        let timeout = if method == "session/prompt" { RPC_TIMEOUT } else { SHORT_TIMEOUT };

        loop {
            let resp = tokio::time::timeout(timeout, self.recv())
                .await
                .map_err(|_| anyhow::anyhow!("Timeout waiting for response to {}", method))??;

            // Final response has matching id
            if resp.get("id").and_then(|v| v.as_i64()) == Some(id) {
                if let Some(error) = resp.get("error") {
                    if !error.is_null() {
                        let msg = error["message"].as_str().unwrap_or("Unknown error");
                        let code = error["code"].as_i64().unwrap_or(0);
                        anyhow::bail!("RPC error ({}): {}", code, msg);
                    }
                }
                let result = resp.get("result").cloned().unwrap_or(json!({}));

                // Streaming response — return collected text
                if result.get("stopReason").is_some() {
                    return Ok(json!({
                        "stopReason": result["stopReason"],
                        "text": text_parts.join("")
                    }));
                }
                return Ok(result);
            }

            // Collect streaming notifications
            if resp.get("method").and_then(|v| v.as_str()) == Some("session/update") {
                if let Some(update) = resp.pointer("/params/update") {
                    if update.get("sessionUpdate").and_then(|v| v.as_str()) == Some("agent_message_chunk") {
                        if let Some(text) = update.pointer("/content/text").and_then(|v| v.as_str()) {
                            text_parts.push(text.to_string());
                            print!("{}", text);
                            std::io::stdout().flush()?;
                        }
                    }
                }
            }
        }
    }
}

async fn read_line<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> anyhow::Result<Option<String>> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

// ── Main ──

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let parsed = AgentUrl::parse(&cli.url)?;

    if cli.verbose {
        eprintln!("[agc] Connecting to {}", parsed);
    }

    let mut conn = Connection::connect(&parsed).await?;

    // 1. Initialize
    let mut init_params = json!({
        "protocolVersion": "1",
        "clientInfo": {"name": "agc", "version": env!("CARGO_PKG_VERSION")}
    });
    if let Some(ref token) = cli.token {
        // Put token in both _meta (standard) and top-level (relay compat)
        init_params["_meta"] = json!({"authToken": token});
        init_params["authToken"] = json!(token);
    }
    let init_result = conn.rpc("initialize", init_params).await?;
    let authenticated = init_result
        .get("authenticated")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if cli.verbose {
        eprintln!("[agc] Initialized (authenticated: {})", authenticated);
    }
    if !authenticated && cli.token.is_some() {
        anyhow::bail!("Authentication failed. Check your token.");
    }

    // 2. Create session
    let mut session_params = json!({});
    if let Some(ref agent) = parsed.agent {
        session_params["_meta"] = json!({"aginx/agentId": agent});
    }
    if let Some(ref cwd) = cli.cwd {
        session_params["cwd"] = json!(cwd);
    }
    session_params["mcpServers"] = json!([]);
    let session_result = conn.rpc("session/new", session_params).await?;
    let session_id = session_result["sessionId"].as_str().unwrap_or("unknown");
    if cli.verbose {
        eprintln!("[agc] Session: {}", session_id);
    }

    // 3. Send prompt
    let message = match cli.message {
        Some(msg) => msg,
        None => {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if message.is_empty() {
        anyhow::bail!("No message provided");
    }

    if cli.verbose {
        eprintln!("[agc] Sending prompt...");
    }

    let result = conn
        .rpc(
            "session/prompt",
            json!({
                "sessionId": session_id,
                "prompt": [{"type": "text", "text": message}]
            }),
        )
        .await?;

    // Only print trailing newline if streaming text doesn't already end with one
    if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
        if !text.ends_with('\n') {
            println!();
        }
    } else {
        println!();
    }

    Ok(())
}
