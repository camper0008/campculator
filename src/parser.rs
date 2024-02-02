use crate::lexer::{Lexer, Token, TokenVariant};

pub enum Expr {
    Int(i64),
    Float(f64),
    Id(String),
    Binary(BinaryVariant, Box<Expr>, Box<Expr>),
}

pub enum BinaryVariant {
    Fn,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

pub struct Error {
    pub from: usize,
    pub to: usize,
    pub message: String,
}

pub struct Parser<'text> {
    text: &'text str,
    lexer: Lexer,
    current: Option<Token>,
}

impl<'text> Parser<'text> {
    pub fn new(text: &'text str, lexer: Lexer) -> Self {
        return Self {
            text,
            lexer,
            current: None,
        };
    }

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.parse_operand()
    }

    fn parse_operand(&mut self) -> Result<Expr, Error> {
        let Some(current) = &self.current else {
            panic!("break shit get hit");
        };
        match current.variant {
            TokenVariant::Int => Ok(Expr::Int(
                self.text[current.from..=current.to]
                    .parse()
                    .expect("should not tokenize incorrect int"),
            )),
            TokenVariant::Float => Ok(Expr::Float(
                self.text[current.from..=current.to]
                    .parse()
                    .expect("should not tokenize incorrect float"),
            )),
            TokenVariant::Id => Ok(Expr::Id(self.text[current.from..=current.to].to_string())),
            TokenVariant::LParen => {
                self.step();
                let expr = self.parse_expr();
                let Some(closing) = &self.current else {
                    self.step();
                    return Err(Error {
                        from: 0,
                        to: 0,
                        message: format!("expected LParen got None"),
                    });
                };
                if !matches!(closing.variant, TokenVariant::LParen) {
                    let err = Error {
                        from: closing.from,
                        to: closing.to,
                        message: format!("expected LParen got {:?}", closing.variant),
                    };
                    self.step();
                    return Err(err);
                }
                self.step();
                expr
            }
            TokenVariant::Fn => {
                self.step();
                let id = self.eat(TokenVariant::Id)?;
                let id = Expr::Id(self.text[id.from..=id.to].to_string());
                let expr = self.parse_expr()?;
                return Ok(Expr::Binary(
                    BinaryVariant::Fn,
                    Box::new(id),
                    Box::new(expr),
                ));
            }

            TokenVariant::Add
            | TokenVariant::Sub
            | TokenVariant::Mul
            | TokenVariant::Div
            | TokenVariant::Pow
            | TokenVariant::Equal
            | TokenVariant::Arrow
            | TokenVariant::RParen
            | TokenVariant::Invalid => {
                let err = Error {
                    from: current.from,
                    to: current.to,
                    message: format!(
                        "expected Int | Float | Id | RParen | Fn got '{:?}'",
                        current.variant
                    ),
                };
                self.step();
                Err(err)
            }
        }
    }

    fn eat(&mut self, variant: TokenVariant) -> Result<Token, Error> {
        let Some(current) = self.current.take() else {
            return Err(Error {
                from: 0,
                to: 0,
                message: format!("expected {variant:?} got None"),
            });
        };

        self.step();

        if current.variant != variant {
            return Err(Error {
                from: current.from,
                to: current.to,
                message: format!("expected {:?} got {:?}", variant, current.variant),
            });
        }

        Ok(current)
    }

    fn parse_expr(&mut self) -> Result<Expr, Error> {
        self.parse_eq()
    }

    fn parse_eq(&mut self) -> Result<Expr, Error> {
        let left = self.parse_add_sub()?;
        if self.current.is_none() {
            return Ok(left);
        }
        let _ = self.eat(TokenVariant::Equal)?;
        let right = self.parse_add_sub()?;
        Ok(Expr::Binary(
            BinaryVariant::Eq,
            Box::new(left),
            Box::new(right),
        ))
    }

    fn parse_add_sub(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_mul_div()?;
        loop {
            let variant = self.current.as_ref().map(|v| &v.variant).cloned();
            if !matches!(variant, Some(TokenVariant::Add | TokenVariant::Sub)) {
                break Ok(left);
            }
            self.step();
            let right = self.parse_mul_div()?;
            if matches!(variant, Some(TokenVariant::Add)) {
                left = Expr::Binary(BinaryVariant::Add, Box::new(left), Box::new(right))
            } else {
                left = Expr::Binary(BinaryVariant::Sub, Box::new(left), Box::new(right))
            }
        }
    }

    fn parse_mul_div(&mut self) -> Result<Expr, Error> {
        let mut left = self.parse_unary()?;
        loop {
            let variant = self.current.as_ref().map(|v| &v.variant).cloned();
            if !matches!(variant, Some(TokenVariant::Mul | TokenVariant::Div)) {
                break Ok(left);
            }
            self.step();
            let right = self.parse_unary()?;
            if matches!(variant, Some(TokenVariant::Mul)) {
                left = Expr::Binary(BinaryVariant::Mul, Box::new(left), Box::new(right))
            } else {
                left = Expr::Binary(BinaryVariant::Div, Box::new(left), Box::new(right))
            }
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, Error> {
        todo!()
    }

    fn step(&mut self) {
        self.current = self.lexer.next();
    }
}
