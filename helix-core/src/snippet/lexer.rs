use std::str::Chars;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TokenKind {
    Int,
    Dollar,
    LBrace,
    RBrace,
    Comma,
    Pipe,
    Colon,
    Eof,
    Text,
}

struct Token {
    kind: TokenKind,
    len: u32,
}

struct Lexer<'a> {
    chars: Chars<'a>,
    input_len: u32,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            chars: input.chars(),
            input_len: input.len() as u32,
        }
    }

    fn rest_len(&self) -> u32 {
        self.chars.as_str().len() as u32
    }

    /// Returns amount of already consumed chars.
    fn pos(&self) -> u32 {
        self.input_len - self.rest_len()
    }

    fn nth(&self, n: u32) -> Option<char> {
        self.chars.clone().nth(n as usize)
    }

    fn peek(&self) -> Option<char> {
        self.nth(0)
    }

    fn accept_while<F: FnMut(char) -> bool>(&mut self, mut pred: F) {
        while let Some(c) = self.peek() {
            if pred(c) {
                self.chars.next().unwrap();
            } else {
                break;
            }
        }
    }

    fn is_important(c: char) -> bool {
        matches!(c, '$' | '{' | '}' | ',' | '|' | ':') || c.is_numeric()
    }

    fn lex(&mut self) -> Option<Token> {
        let start = self.pos();
        let kind = self.lex_impl()?;
        let end = self.pos();
        Some(Token { kind, len: start - end})
    }

    fn lex_impl(&mut self) -> Option<TokenKind> {
        use TokenKind::*;

        Some(match self.chars.next()? {
            '$' => Dollar,
            '{' => LBrace,
            '}' => RBrace,
            ',' => Comma,
            '|' => Pipe,
            ':' => Colon,
            c if c.is_numeric() => {
                self.chars.next().unwrap();
                self.accept_while(char::is_numeric);
                Int
            }
            _ => {
                self.chars.next().unwrap();
                self.accept_while(|c| !Lexer::is_important(c));
                Text
            }
        })
    }

    // fn lex_text(&mut self) -> TokenKind {
    //     match self.chars.next() {
    //         '\\' => {
    //             // match self.chars.ne
    //         }
    //     }
    // }
}
