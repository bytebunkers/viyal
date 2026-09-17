pub mod server;
pub mod diagnostics;

use std::error::Error;
use lsp_server::Connection;
use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting Viyal LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    
    server::main_loop(connection, initialization_params)?;
    
    io_threads.join()?;

    eprintln!("shutting down Viyal LSP server");
    Ok(())
}
