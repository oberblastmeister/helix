use super::lexer::Lexer;

struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    // fn parse(&mut self) -> 
}
