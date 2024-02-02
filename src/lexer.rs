#[derive(PartialEq, Debug, Clone)]
pub enum TokenVariant {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Equal,
    Int,
    Arrow,
    Float,
    Id,
    RParen,
    LParen,
    Fn,
    Invalid,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub from: usize,
    pub to: usize,
    pub variant: TokenVariant,
}

pub struct Lexer {
    text: Vec<char>,
    position: usize,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.make_token()
    }
}

impl<'a> Lexer {
    pub fn new<T: Into<Vec<char>>>(text: T) -> Self {
        Self {
            text: text.into(),
            position: 0,
        }
    }

    fn current(&self) -> Option<&char> {
        self.text.get(self.position)
    }

    fn step(&mut self) {
        self.position += 1;
    }

    fn make_identifier(&'a mut self) -> Option<Token> {
        let from = self.position;
        let mut to = self.position;
        let mut variant = TokenVariant::Id;
        loop {
            match self.current() {
                Some('A'..='Z' | 'a'..='z' | '_' | '0'..='9') => {
                    to = self.position;
                    self.step();
                }
                None | Some(_) => {
                    if &self.text[from..=to] == &['f', 'n'] {
                        variant = TokenVariant::Fn;
                    }
                    break Some(Token { from, to, variant });
                }
            }
        }
    }

    fn make_single(&'a mut self, variant: TokenVariant) -> Token {
        let from = self.position;
        let to = self.position;
        self.step();
        Token { from, to, variant }
    }

    fn make_number(&'a mut self) -> Option<Token> {
        let from = self.position;
        let mut to = self.position;
        let mut variant = TokenVariant::Int;
        loop {
            match self.current() {
                Some('.') => {
                    to = self.position;
                    if matches!(variant, TokenVariant::Float) {
                        break Some(Token {
                            from,
                            to,
                            variant: TokenVariant::Invalid,
                        });
                    }
                    variant = TokenVariant::Float;
                    self.step();
                }
                Some('0'..='9') => {
                    to = self.position;
                    self.step();
                }
                None | Some(_) => break Some(Token { from, to, variant }),
            }
        }
    }

    fn make_double(
        &'a mut self,
        double_character: char,
        single_variant: TokenVariant,
        double_variant: TokenVariant,
    ) -> Token {
        let from = self.position;
        let mut to = self.position;
        let mut variant = single_variant;
        self.step();
        if self.current().is_some_and(|&v| v == double_character) {
            to = self.position;
            variant = double_variant;
            self.step();
        }
        Token { from, to, variant }
    }

    fn make_token(&'a mut self) -> Option<Token> {
        let current = self.current()?;
        match current {
            '0'..='9' | '.' => self.make_number(),
            'A'..='Z' | 'a'..='z' | '_' => self.make_identifier(),
            '+' => Some(self.make_single(TokenVariant::Add)),
            '-' => Some(self.make_single(TokenVariant::Sub)),
            '^' => Some(self.make_single(TokenVariant::Pow)),
            '*' => Some(self.make_single(TokenVariant::Mul)),
            '=' => Some(self.make_double('>', TokenVariant::Equal, TokenVariant::Arrow)),
            '/' => Some(self.make_single(TokenVariant::Div)),
            '(' => Some(self.make_single(TokenVariant::LParen)),
            ')' => Some(self.make_single(TokenVariant::RParen)),
            c if c.is_whitespace() => {
                self.step();
                self.make_token()
            }
            _ => Some(self.make_single(TokenVariant::Invalid)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    struct TokenFactory {
        index: usize,
    }

    impl TokenFactory {
        pub fn new() -> Self {
            Self { index: 0 }
        }

        pub fn skip(&mut self, text: &str) -> Option<Token> {
            for c in text.chars() {
                self.make(&c.to_string(), TokenVariant::Fn);
            }
            None
        }

        pub fn make(&mut self, text: &str, variant: TokenVariant) -> Option<Token> {
            let token = Token {
                variant,
                from: self.index,
                to: self.index + text.len() - 1,
            };
            self.index += text.len();
            Some(token)
        }
    }
    #[test]
    fn does_not_crash_and_burn() {
        let text = "ABC_1+-*/^()==>123 123.4fn";
        let given_tokens: Vec<_> = Lexer::new(text.chars().collect::<Vec<_>>()).collect();
        let mut factory = TokenFactory::new();
        let expected_tokens: Vec<_> = vec![
            factory.make("ABC_1", TokenVariant::Id),
            factory.make("+", TokenVariant::Add),
            factory.make("-", TokenVariant::Sub),
            factory.make("*", TokenVariant::Mul),
            factory.make("/", TokenVariant::Div),
            factory.make("^", TokenVariant::Pow),
            factory.make("(", TokenVariant::LParen),
            factory.make(")", TokenVariant::RParen),
            factory.make("=", TokenVariant::Equal),
            factory.make("=>", TokenVariant::Arrow),
            factory.make("123", TokenVariant::Int),
            factory.skip(" "),
            factory.make("123.4", TokenVariant::Float),
            factory.make("fn", TokenVariant::Fn),
        ]
        .into_iter()
        .filter_map(std::convert::identity)
        .collect();

        assert_eq!(given_tokens, expected_tokens)
    }
}
