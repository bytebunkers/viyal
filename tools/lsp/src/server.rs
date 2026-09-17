use lsp_server::{Connection, Message};
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument, Notification},
    DidChangeTextDocumentParams, DidOpenTextDocumentParams,
};
use std::error::Error;
use crate::diagnostics;

pub fn main_loop(
    connection: Connection,
    _initialization_params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    eprintln!("starting main loop");
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
            }
            Message::Response(_resp) => {}
            Message::Notification(not) => {
                match not.method.as_str() {
                    DidOpenTextDocument::METHOD => {
                        let params: DidOpenTextDocumentParams = serde_json::from_value(not.params).unwrap();
                        let uri = params.text_document.uri;
                        let text = params.text_document.text;
                        diagnostics::publish_diagnostics(&connection, uri, text);
                    }
                    DidChangeTextDocument::METHOD => {
                        let params: DidChangeTextDocumentParams = serde_json::from_value(not.params).unwrap();
                        let uri = params.text_document.uri;
                        // Since we requested Full sync, the content changes will just be the full text
                        if let Some(change) = params.content_changes.first() {
                            let text = change.text.clone();
                            diagnostics::publish_diagnostics(&connection, uri, text);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
