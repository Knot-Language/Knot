use crate::lexer::num::Num;
use crate::lexer::token::Token;

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

const VALID_SUFFIXES: &[&str] = &[
    "f32", "f64", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64",
];

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        (self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn peek_suffix(&self) -> (String, bool) {
        let mut suffix = String::new();
        let mut valid = true;
        let mut i = self.pos;

        while i < self.chars.len() {
            let c = self.chars[i];
            if c.is_alphanumeric() {
                suffix.push(c);
                i += 1;

                let lower: String = suffix.chars().map(|ch| ch.to_ascii_lowercase()).collect();
                let is_prefix = VALID_SUFFIXES.iter().any(|s| s.starts_with(&lower));
                if !is_prefix {
                    valid = false;
                    break;
                }
            } else {
                break;
            }
        }

        let lower: String = suffix.chars().map(|ch| ch.to_ascii_lowercase()).collect();
        if valid && !suffix.is_empty() && !VALID_SUFFIXES.contains(&lower.as_str()) {
            valid = false;
        }

        (suffix, valid)
    }

    fn skip_whitespace(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') => {
                    self.advance();
                }
                Some('/') if self.peek_at(1) == Some('/') => {
                    self.skip_line_comment();
                }
                Some('/') if self.peek_at(1) == Some('*') => {
                    if let Some(err) = self.skip_block_comment() {
                        return Some(err);
                    }
                }
                _ => return None,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Option<Token> {
        self.advance();
        self.advance();
        while self.peek().is_some() {
            if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                self.advance();
                self.advance();
                return None;
            }
            self.advance();
        }
        Some(Token::Error("unterminated block comment".to_string()))
    }

    fn read_number(&mut self) -> Token {
        let radix = if self.peek() == Some('0') {
            match self.peek_at(1) {
                Some('x') | Some('X') => {
                    self.advance();
                    self.advance();
                    16
                }
                Some('b') | Some('B') => {
                    self.advance();
                    self.advance();
                    2
                }
                Some('o') | Some('O') => {
                    self.advance();
                    self.advance();
                    8
                }
                _ if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) => {
                    self.advance();
                    8
                }
                _ => 10,
            }
        } else {
            10
        };

        let mut has_digit = false;
        let mut has_dot = false;
        let mut raw = String::new();

        loop {
            match self.peek() {
                Some(c) if is_digit(c, radix) || c == '_' => {
                    has_digit = true;
                    raw.push(self.advance().unwrap());
                }
                Some('.') if radix == 10 && !has_dot => {
                    if self.peek_at(1) == Some('.') {
                        break;
                    }
                    has_dot = true;
                    raw.push(self.advance().unwrap());
                }
                _ => break,
            }
        }

        if !has_digit {
            return Token::Error("numeric literal without digits".to_string());
        }

        if radix != 10 {
            let clean: String = raw.chars().filter(|&c| c != '_').collect();
            if clean.is_empty() {
                return Token::Error("numeric literal without digits".to_string());
            }
            let val = match i64::from_str_radix(&clean, radix) {
                Ok(v) => v,
                Err(_) => return Token::Error(format!("invalid {} literal: {}", radix_name(radix), raw)),
            };

            let (suffix, valid) = self.peek_suffix();
            if !valid {
                return Token::Error(format!("invalid numeric suffix: {}", suffix));
            }
            for _ in 0..suffix.len() {
                self.advance();
            }

            return match suffix.to_lowercase().as_str() {
                "f32" => Token::Number(Num::F32(val as f32)),
                "f64" => Token::Number(Num::F64(val as f64)),
                "i8" => Token::Number(Num::I8(val as i8)),
                "i16" => Token::Number(Num::I16(val as i16)),
                "i32" | "" => Token::Number(Num::I32(val as i32)),
                "i64" => Token::Number(Num::I64(val)),
                "u8" => Token::Number(Num::U8(val as u8)),
                "u16" => Token::Number(Num::U16(val as u16)),
                "u32" => Token::Number(Num::U32(val as u32)),
                "u64" => Token::Number(Num::U64(val as u64)),
                _ => unreachable!(),
            };
        }

        let is_float = has_dot;
        let (suffix, valid) = self.peek_suffix();
        if !valid {
            return Token::Error(format!("invalid numeric suffix: {}", suffix));
        }
        for _ in 0..suffix.len() {
            self.advance();
        }

        let clean: String = raw.chars().filter(|&c| c != '_').collect();

        let num = match suffix.to_lowercase().as_str() {
            "f32" => Num::F32(clean.parse().unwrap()),
            "f64" => Num::F64(clean.parse().unwrap()),
            "i8" => Num::I8(clean.parse().unwrap()),
            "i16" => Num::I16(clean.parse().unwrap()),
            "i32" => Num::I32(clean.parse().unwrap()),
            "i64" => Num::I64(clean.parse().unwrap()),
            "u8" => Num::U8(clean.parse().unwrap()),
            "u16" => Num::U16(clean.parse().unwrap()),
            "u32" => Num::U32(clean.parse().unwrap()),
            "u64" => Num::U64(clean.parse().unwrap()),
            "" => {
                if is_float {
                    Num::F64(clean.parse().unwrap())
                } else {
                    Num::I32(clean.parse().unwrap())
                }
            }
            _ => unreachable!(),
        };

        Token::Number(num)
    }

    fn read_ident_or_keyword(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if s == "as" && self.peek() == Some('!') {
            s.push(self.advance().unwrap());
        }

        let keywords = [
            "abstract", "args", "as", "as!", "assert", "break", "catch", "class",
            "continue", "else", "enum", "false", "for", "func", "if", "import",
            "in", "kwargs", "match", "mixin", "null", "operator", "private",
            "return", "static", "throw", "true", "try", "while", "wrap",
            "new", "delete", "Any",
        ];

        if keywords.contains(&s.as_str()) {
            Token::Keyword(s)
        } else {
            Token::Identifier(s)
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                return Token::String(s);
            }
            if c == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => { self.advance(); s.push('\n'); }
                    Some('t') => { self.advance(); s.push('\t'); }
                    Some('r') => { self.advance(); s.push('\r'); }
                    Some('\\') => { self.advance(); s.push('\\'); }
                    Some('"') => { self.advance(); s.push('"'); }
                    Some('{') => { self.advance(); s.push('{'); }
                    Some(ch) => { self.advance(); s.push(ch); }
                    None => break,
                }
            } else {
                s.push(self.advance().unwrap());
            }
        }
        Token::Error("unterminated string literal".to_string())
    }

    fn read_raw_string(&mut self, quote: char) -> Token {
        self.advance();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == quote {
                self.advance();
                return Token::String(s);
            }
            s.push(self.advance().unwrap());
        }
        Token::Error("unterminated string literal".to_string())
    }

    fn read_operator(&mut self) -> Token {
        let c = self.advance().unwrap();
        let op = match c {
            '+' => match self.peek() {
                Some('=') => { self.advance(); "+=" }
                Some('+') => { self.advance(); "++" }
                _ => "+",
            },
            '-' => match self.peek() {
                Some('>') => { self.advance(); "->" }
                Some('=') => { self.advance(); "-=" }
                Some('-') => { self.advance(); "--" }
                _ => "-",
            },
            '*' => match self.peek() {
                Some('=') => { self.advance(); "*=" }
                _ => "*",
            },
            '/' => match self.peek() {
                Some('=') => { self.advance(); "/=" }
                _ => "/",
            },
            '%' => match self.peek() {
                Some('=') => { self.advance(); "%=" }
                _ => "%",
            },
            '=' => match self.peek() {
                Some('=') => {
                    self.advance();
                    "=="
                }
                Some('>') => {
                    self.advance();
                    "=>"
                }
                _ => "=",
            },
            '!' => match self.peek() {
                Some('=') => {
                    self.advance();
                    "!="
                }
                _ => "!",
            },
            '<' => match self.peek() {
                Some('=') => {
                    self.advance();
                    "<="
                }
                Some('<') => {
                    self.advance();
                    "<<"
                }
                _ => "<",
            },
            '>' => match self.peek() {
                Some('=') => {
                    self.advance();
                    ">="
                }
                Some('>') => {
                    self.advance();
                    ">>"
                }
                _ => ">",
            },
            '&' => match self.peek() {
                Some('&') => {
                    self.advance();
                    "&&"
                }
                Some('=') => {
                    self.advance();
                    "&="
                }
                _ => "&",
            },
            '|' => match self.peek() {
                Some('|') => {
                    self.advance();
                    "||"
                }
                Some('=') => {
                    self.advance();
                    "|="
                }
                _ => "|",
            },
            '^' => match self.peek() {
                Some('=') => {
                    self.advance();
                    "^="
                }
                _ => "^",
            },
            '~' => "~",
            '.' => match self.peek() {
                Some('.') => {
                    self.advance();
                    ".."
                }
                _ => ".",
            },
            ':' => match self.peek() {
                Some(':') => {
                    self.advance();
                    "::"
                }
                _ => ":",
            },
            '?' => match self.peek() {
                Some('?') => {
                    self.advance();
                    "??"
                }
                _ => "?",
            },
            '(' => "(",
            ')' => ")",
            '[' => "[",
            ']' => "]",
            '{' => "{",
            '}' => "}",
            ',' => ",",
            '@' => "@",
            _ => {
                return Token::Error(format!("unexpected character: '{}'", c));
            }
        };
        Token::Operator(op.to_string())
    }

    pub fn next(&mut self) -> (Token, (usize, usize)) {
        if let Some(err) = self.skip_whitespace() {
            let pos = self.position();
            return (err, pos);
        }

        let pos = self.position();
        let token = match self.peek() {
            None => Token::Eof,
            Some('\n') => {
                self.advance();
                Token::NewLine
            }
            Some(c) if c.is_ascii_digit() => self.read_number(),
            Some(c) if c.is_alphabetic() || c == '_' => self.read_ident_or_keyword(),
            Some('"') => self.read_string(),
            Some('\'') => self.read_raw_string('\''),
            Some('`') => self.read_raw_string('`'),
            Some(_) => self.read_operator(),
        };
        (token, pos)
    }
}

pub fn tokenize(source: &str) -> (Vec<Token>, Vec<(usize, usize)>) {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut positions = Vec::new();
    loop {
        let (t, pos) = lexer.next();
        let done = matches!(t, Token::Eof);
        tokens.push(t);
        positions.push(pos);
        if done {
            break;
        }
    }
    (tokens, positions)
}

fn is_digit(c: char, radix: u32) -> bool {
    match radix {
        2 => c == '0' || c == '1',
        8 => ('0'..='7').contains(&c),
        10 => c.is_ascii_digit(),
        16 => c.is_ascii_hexdigit(),
        _ => false,
    }
}

fn radix_name(radix: u32) -> &'static str {
    match radix {
        2 => "binary",
        8 => "octal",
        16 => "hex",
        _ => "decimal",
    }
}
