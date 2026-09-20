pub mod analyzer;
pub mod rules;

use ast::Span;
use parser::Parser;
use analyzer::Analyzer;

#[derive(Debug, PartialEq, Clone)]
pub struct AnalyzerWarning {
    pub message: String,
    pub span: Span,
}

pub fn analyze(source: &str) -> Result<Vec<AnalyzerWarning>, String> {
    let mut parser = Parser::new(source);
    match parser.parse_program() {
        Ok(program) => {
            let mut analyzer = Analyzer::new();
            Ok(analyzer.analyze_program(&program))
        }
        Err(errors) => {
            let messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            Err(format!("Parse errors: {}", messages.join(", ")))
        },
    }
}
