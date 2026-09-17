use lsp_server::{Connection, Message};
use lsp_types::{
    notification::{PublishDiagnostics, Notification}, Diagnostic, DiagnosticSeverity, Position, PublishDiagnosticsParams, Range, Url,
};
use parser::Parser;
use typechecker::TypeChecker;

pub fn publish_diagnostics(connection: &Connection, uri: Url, text: String) {
    let mut diagnostics = Vec::new();

    let mut parser = Parser::new(&text);
    match parser.parse_program() {
        Ok(program) => {
            let mut checker = TypeChecker::new();
            if let Err(type_err) = checker.check_program(&program) {
                // Map byte span to line/col
                let start = byte_offset_to_position(&text, type_err.span.start);
                let end = byte_offset_to_position(&text, type_err.span.end);
                
                let diag = Diagnostic {
                    range: Range { start, end },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Type Error: {}", type_err.message),
                    ..Default::default()
                };
                diagnostics.push(diag);
            }
        }
        Err(parse_err) => {
            // Map byte span to line/col
            let start = byte_offset_to_position(&text, parse_err.span.start);
            let end = byte_offset_to_position(&text, parse_err.span.end);
            
            let diag = Diagnostic {
                range: Range { start, end },
                severity: Some(DiagnosticSeverity::ERROR),
                message: parse_err.message,
                ..Default::default()
            };
            diagnostics.push(diag);
        }
    }

    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    
    let not = lsp_server::Notification::new(
        PublishDiagnostics::METHOD.to_string(),
        params,
    );
    connection.sender.send(Message::Notification(not)).unwrap();
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in text.chars().enumerate() {
        if i == offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position { line, character: col }
}
