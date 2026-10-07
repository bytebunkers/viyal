#[cfg(test)]
mod fuzz_tests {
    use lexer::Lexer;
    use parser::Parser;
    use typechecker::TypeChecker;

    // A simple, fast xorshift PRNG for fuzzing without external dependencies
    struct XorShift {
        state: u32,
    }

    impl XorShift {
        fn new(seed: u32) -> Self {
            Self { state: seed }
        }

        fn next(&mut self) -> u32 {
            let mut x = self.state;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.state = x;
            x
        }

        fn next_bytes(&mut self, len: usize) -> Vec<u8> {
            let mut bytes = Vec::with_capacity(len);
            for _ in 0..len {
                // Ensure we occasionally generate ASCII/UTF8 characters, but also random gibberish
                let val = self.next();
                if val % 3 == 0 {
                    bytes.push((val % 128) as u8); // ASCII
                } else {
                    bytes.push((val % 256) as u8); // Arbitrary byte
                }
            }
            bytes
        }
    }

    #[test]
    fn test_lexer_fuzz_no_panic() {
        let mut prng = XorShift::new(0x12345678);
        for _ in 0..1000 {
            let bytes = prng.next_bytes(100);
            let s = String::from_utf8_lossy(&bytes);
            let lexer = Lexer::new(&s);
            for _ in lexer {}
        }
    }

    #[test]
    fn test_parser_fuzz_no_panic() {
        let mut prng = XorShift::new(0x87654321);
        for _ in 0..1000 {
            let bytes = prng.next_bytes(150);
            let s = String::from_utf8_lossy(&bytes);
            let mut parser = Parser::new(&s);
            // Parse, ensure no panic
            let _ = parser.parse_program();
        }
    }

    #[test]
    fn test_typechecker_fuzz_no_panic() {
        let mut prng = XorShift::new(0xDEADBEEF);
        // We will generate structurally somewhat-valid syntax but random identifiers
        // to stress the typechecker logic.
        for _ in 0..100 {
            let bytes = prng.next_bytes(50);
            // Just mutate a valid program template
            let ident = String::from_utf8_lossy(&bytes)
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            if ident.is_empty() {
                continue;
            }
            let s = format!(
                "class Main {{ void run() {{ var {} = 10; print({}); }} }}",
                ident, ident
            );

            let mut parser = Parser::new(&s);
            if let Ok(prog) = parser.parse_program() {
                let mut tc = TypeChecker::new();
                let _ = tc.check_program(&prog); // Should not panic
            }
        }
    }
}
