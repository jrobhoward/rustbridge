//! Erlang port driver for rustbridge plugins.
//!
//! This binary is spawned by an Erlang port and communicates via stdin/stdout
//! using `{packet, 4}` framing (4-byte big-endian length prefix, JSON payload).
//!
//! It loads a single rustbridge plugin and dispatches commands to it.

mod error;
mod handler;
mod protocol;

use handler::Handler;
use protocol::{Command, LogMessage};

use std::io::{BufWriter, Read, Write};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Read a 4-byte big-endian length prefix, then that many bytes.
fn read_frame(reader: &mut impl Read) -> Result<Option<Vec<u8>>, std::io::Error> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(Some(buf))
}

/// Write a frame with 4-byte big-endian length prefix.
fn write_frame(writer: &Arc<Mutex<BufWriter<std::io::Stdout>>>, data: &[u8]) {
    #[allow(clippy::unwrap_used)] // Fatal: if stdout is broken, we must exit
    let mut w = writer.lock().unwrap();
    let len = data.len() as u32;
    // Ignore write errors - if stdout is closed, the port is shutting down
    let _ = w.write_all(&len.to_be_bytes());
    let _ = w.write_all(data);
    let _ = w.flush();
}

/// Spawn a thread that drains log messages and writes them to stdout.
fn spawn_log_writer(
    rx: mpsc::Receiver<LogMessage>,
    writer: Arc<Mutex<BufWriter<std::io::Stdout>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for log_msg in rx {
            if let Ok(json) = serde_json::to_vec(&log_msg) {
                write_frame(&writer, &json);
            }
        }
    })
}

fn main() {
    let (log_tx, log_rx) = mpsc::channel::<LogMessage>();
    let stdout = Arc::new(Mutex::new(BufWriter::new(std::io::stdout())));

    let _log_thread = spawn_log_writer(log_rx, stdout.clone());

    let mut handler = Handler::new(log_tx);
    let mut stdin = std::io::stdin().lock();

    loop {
        let frame = match read_frame(&mut stdin) {
            Ok(Some(data)) => data,
            Ok(None) => break, // EOF - Erlang closed the port
            Err(_) => break,   // Read error - exit cleanly
        };

        let response = match serde_json::from_slice::<Command>(&frame) {
            Ok(command) => handler.dispatch(command),
            Err(e) => {
                // Can't determine the id from a malformed command, use 0
                protocol::Response::error(
                    0,
                    error::CODE_PROTOCOL_ERROR,
                    format!("failed to parse command: {e}"),
                )
            }
        };

        if let Ok(json) = serde_json::to_vec(&response) {
            write_frame(&stdout, &json);
        }
    }
}
