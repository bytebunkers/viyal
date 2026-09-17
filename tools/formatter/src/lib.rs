pub mod formatter;

use parser::Parser;

pub fn format(source: &str) -> Result<String, String> {
    let mut parser = Parser::new(source);
    match parser.parse_program() {
        Ok(program) => {
            let mut f = formatter::Formatter::new();
            f.format_program(&program);
            Ok(f.into_string())
        }
        Err(e) => Err(format!("Parse error: {}", e.message)),
    }
}
