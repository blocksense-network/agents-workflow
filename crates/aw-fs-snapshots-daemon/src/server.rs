use crate::operations::process_request;
use crate::types::{Request, Response};
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::{wrappers::UnixListenerStream, StreamExt};
use tracing::{debug, error, info, warn};

// TODO: Replace with proper SSZ encoding/decoding
pub fn encode_length_prefixed_json<T: serde::Serialize>(data: &T) -> Result<Vec<u8>> {
    let json_bytes = serde_json::to_vec(data)?;
    let len = json_bytes.len() as u32;
    let mut result = Vec::with_capacity(4 + json_bytes.len());
    result.extend_from_slice(&len.to_le_bytes());
    result.extend_from_slice(&json_bytes);
    Ok(result)
}

pub fn decode_length_prefixed_json<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
    if data.len() < 4 {
        return Err(anyhow!("Data too short for length prefix"));
    }
    let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len {
        return Err(anyhow!("Data too short for payload"));
    }
    let json_data = &data[4..4 + len];
    Ok(serde_json::from_slice(json_data)?)
}

pub struct DaemonServer {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
}

impl DaemonServer {
    pub fn new(socket_path: PathBuf) -> Result<Self> {
        // Ensure socket directory exists
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Remove existing socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        // Set permissions to allow anyone to connect (since tests run as regular user)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&socket_path)?.permissions();
            perms.set_mode(0o666);
            std::fs::set_permissions(&socket_path, perms)?;
        }

        info!("Daemon listening on socket: {}", socket_path.display());

        Ok(Self {
            socket_path,
            listener: Some(listener),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = self.listener.take().ok_or_else(|| anyhow!("Server not initialized"))?;
        let mut stream = UnixListenerStream::new(listener);

        info!("AW filesystem snapshots daemon started. Press Ctrl+C to stop.");

        while let Some(stream) = stream.next().await {
            match stream {
                Ok(socket) => {
                    let socket_path = self.socket_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(socket).await {
                            error!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        info!("Shutting down daemon...");

        // Remove the socket file
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        Ok(())
    }
}

async fn handle_client(mut socket: UnixStream) -> Result<()> {
    debug!("Handling new client connection");

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read one line (the request)
    let n = reader.read_line(&mut line).await?;
    if n == 0 {
        // Client disconnected
        return Ok(());
    }

    // Parse length-prefixed JSON request from hex string
    let request_bytes = hex::decode(line.trim())?;
    let request: Request = decode_length_prefixed_json(&request_bytes)?;

    // Process the request
    let response = process_request(request).await;

    // Encode response as length-prefixed JSON and send as hex
    let response_bytes = encode_length_prefixed_json(&response)?;
    let response_hex = hex::encode(&response_bytes);

    // Write response followed by newline
    writer.write_all(response_hex.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    debug!("Handled client request successfully");

    Ok(())
}
