use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

use super::{PaintParams, PaintType};

/// Request to the Node.js server
#[derive(Debug, Serialize, Deserialize)]
struct NodeRequest {
    /// The type of content to render
    #[serde(rename = "method")]
    ty: String,
    /// Whether the equation is inline or display mode (for MathJax)
    inline: bool,
    /// The content to render
    content: String,
}

/// Configuration for the Node.js server
#[derive(Clone)]
pub struct NodeServer {
    /// Path to the Node.js script
    script_path: String,
    /// Child process handle
    process: Arc<AsyncMutex<Option<ChildProcess>>>,
}

impl std::fmt::Debug for NodeServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.script_path)
    }
}

/// Wrapper for the child process
struct ChildProcess {
    child: Child,
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        // Attempt to kill the process when it's dropped
        let _ = self.child.kill();
    }
}

impl NodeServer {
    /// Create a new Node.js server connection using stdio
    pub fn new(script_path: String) -> Self {
        NodeServer {
            script_path,
            process: Arc::new(AsyncMutex::new(None)),
        }
    }

    /// Start the Node.js server process if not already running
    async fn ensure_process_started(&self) -> Result<()> {
        let mut process_guard = self.process.lock().await;

        if process_guard.is_none() {
            // Start the Node.js process with stdio mode
            let mut child = Command::new(&self.script_path)
                .arg("stdio")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("Failed to start Node.js server process")?;

            // Read initial stderr output to confirm process started
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();

                // Read the first line which should contain startup message
                if reader.read_line(&mut line).is_ok() {
                    if !line.contains("Running in stdio mode") {
                        return Err(anyhow::anyhow!(
                            "Unexpected output from Node.js server: {}",
                            line
                        ));
                    }
                }

                // Put stderr back
                child.stderr = Some(reader.into_inner());
            }

            *process_guard = Some(ChildProcess { child });
        }

        Ok(())
    }

    /// Paint content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        // Create the request based on the paint type
        let request = match params.ty {
            PaintType::InlineTeX => NodeRequest {
                ty: "mathjax".to_string(),
                inline: true,
                content: params.content,
            },
            PaintType::Equation => NodeRequest {
                ty: "mathjax".to_string(),
                inline: false,
                content: params.content,
            },
            PaintType::Mermaid => NodeRequest {
                ty: "mermaid".to_string(),
                inline: false, // Not used for Mermaid
                content: params.content,
            },
        };

        // Ensure the process is started
        self.ensure_process_started().await?;

        // Get a lock on the process
        let mut process_guard = self.process.lock().await;
        let process = process_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Node.js server process not started"))?;

        // Get stdin and stdout handles
        let stdin = process
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin handle"))?;
        let stdout = process
            .child
            .stdout
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout handle"))?;

        // Serialize the request to JSON and send it
        let request_json =
            serde_json::to_string(&request).context("Failed to serialize Node.js request")?;

        // Write the request to stdin
        stdin
            .write_all(request_json.as_bytes())
            .context("Failed to write to Node.js server stdin")?;
        stdin
            .write_all(b"\n")
            .context("Failed to write newline to Node.js server stdin")?;
        stdin
            .flush()
            .context("Failed to flush Node.js server stdin")?;

        // Read the response from stdout
        let mut reader = BufReader::new(stdout);

        // let mut response = String::new();
        // reader.read_line(&mut response)
        //     .context("Failed to read from Node.js server stdout")?;
        let mut buf = vec![0; 1024 * 1024];
        let bytes_read = reader.read(&mut buf)?;
        let mut string_bytes = Vec::with_capacity(bytes_read);
        string_bytes.extend_from_slice(&buf[..bytes_read]);
        let response = String::from_utf8(string_bytes)?;

        // Trim any whitespace
        let svg_content = response.trim().to_string();

        // Return the SVG content
        Ok(svg_content)
    }
}
