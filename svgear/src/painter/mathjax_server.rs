use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

use super::{PaintParams, PaintType};

/// Request to the MathJax server
#[derive(Debug, Serialize, Deserialize)]
struct MathJaxRequest {
    /// Whether the equation is inline or display mode
    inline: bool,
    /// The equation content to render
    content: String,
}

/// Configuration for the MathJax server
#[derive(Clone)]
pub struct MathjaxServer {
    /// Path to the Node.js script
    script_path: String,
    /// Child process handle
    process: Arc<AsyncMutex<Option<ChildProcess>>>,
}

impl std::fmt::Debug for MathjaxServer {
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

impl MathjaxServer {
    /// Create a new MathJax server connection using stdio
    pub fn new(script_path: String) -> Self {
        MathjaxServer {
            script_path,
            process: Arc::new(AsyncMutex::new(None)),
        }
    }
    
    /// Start the MathJax server process if not already running
    async fn ensure_process_started(&self) -> Result<()> {
        let mut process_guard = self.process.lock().await;
        
        if process_guard.is_none() {
            // Start the Node.js process with stdio mode
            let mut child = Command::new("node")
                .arg(&self.script_path)
                .arg("stdio")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("Failed to start MathJax server process")?;

            
            // Read initial stderr output to confirm process started
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                
                // Read the first line which should contain startup message
                if reader.read_line(&mut line).is_ok() {
                    if !line.contains("Running in stdio mode") {
                        return Err(anyhow::anyhow!("Unexpected output from MathJax server: {}", line));
                    }
                }
                
                // Put stderr back
                child.stderr = Some(reader.into_inner());
            }
            
            *process_guard = Some(ChildProcess { child });
        }
        
        Ok(())
    }
    
    /// Paint TeX content to SVG
    pub async fn paint(&self, params: PaintParams) -> Result<String> {
        // Determine if the content is inline or display mode based on the paint type
        let inline = match params.ty {
            PaintType::InlineTeX => true,
            PaintType::Equation => false,
            _ => return Err(anyhow::anyhow!("Unsupported paint type for MathJax: {:?}", params.ty)),
        };

        // Create the request
        let request = MathJaxRequest {
            inline,
            content: params.content,
        };

        // Ensure the process is started
        self.ensure_process_started().await?;
        
        // Get a lock on the process
        let mut process_guard = self.process.lock().await;
        let process = process_guard.as_mut()
            .ok_or_else(|| anyhow::anyhow!("MathJax server process not started"))?;
        
        // Get stdin and stdout handles
        let stdin = process.child.stdin.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdin handle"))?;
        let stdout = process.child.stdout.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Failed to get stdout handle"))?;
        
        // Serialize the request to JSON and send it
        let request_json = serde_json::to_string(&request)
            .context("Failed to serialize MathJax request")?;
        
        // Write the request to stdin
        stdin.write_all(request_json.as_bytes())
            .context("Failed to write to MathJax server stdin")?;
        stdin.write_all(b"\n")
            .context("Failed to write newline to MathJax server stdin")?;
        stdin.flush()
            .context("Failed to flush MathJax server stdin")?;
        
        // Read the response from stdout
        let mut reader = BufReader::new(stdout);
        let mut response = String::new();
        reader.read_line(&mut response)
            .context("Failed to read from MathJax server stdout")?;
        
        // Trim any whitespace
        let svg_content = response.trim().to_string();
        
        // Return the SVG content
        Ok(svg_content)
    }
}
