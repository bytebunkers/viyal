pub mod server;
pub mod diagnostics;

use std::error::Error;
use lsp_server::Connection;
use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind};

pub fn start_server() -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting Viyal LSP server");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
    }).unwrap();
    
    let initialization_params = connection.initialize(server_capabilities)?;
    
    server::main_loop(connection, initialization_params)?;
    
    io_threads.join()?;

    eprintln!("shutting down Viyal LSP server");
    Ok(())
}
