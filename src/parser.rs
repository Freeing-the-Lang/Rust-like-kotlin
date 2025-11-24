use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Int,
    String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    StringLiteral(String),
    Var(String),
    Binary(Box<Expr>, String, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, TypeName, Expr),
    ExprStmt(Expr),
    Return(Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, TypeName)>,
    pub ret_type: TypeName,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub funcs: Vec<Function>,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn next(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) {
        let tok = self.next();
        if tok != expected {
            panic!("Expected {:?}, got {:?}", expected, tok);
        }
    }

    fn expect_ident(&mut self) -> String {
        match self.next() {
            Token::Ident(name) => name.clone(),
            other => panic!("Expected identifier, got {:?}", other),
        }
    }

    fn parse_type(&mut self) -> TypeName {
        match self.next() {
            Token::IntType => TypeName::Int,
            Token::StringType => TypeName::String,
            other => panic!("Expected type, got {:?}", other),
        }
    }

    // =====================================================
    // PROGRAM
    // =====================================================
    pub fn parse_program(&mut self) -> Program {
        let mut funcs = Vec::new();

        while !matches!(self.peek(), Token::EOF) {
            funcs.push(self.parse_function());
        }

        Program { funcs }
    }

    // =====================================================
    // FUNCTION
    // =====================================================
    fn parse_function(&mut self) -> Function {
        match self.next() {
            Token::Func => {}
            other => panic!("Expected 'func', got {:?}", other),
        }

        let name = self.expect_ident();

        self.expect(&Token::LParen);

        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen) {
            let pname = self.expect_ident();
            self.expect(&Token::Colon);
            let ptype = self.parse_type();
            params.push((pname, ptype));

            if matches!(self.peek(), Token::Comma) {
                self.next();
            }
        }

        self.expect(&Token::RParen);
        self.expect(&Token::Colon);
        let ret_type = self.parse_type();

        self.expect(&Token::LBrace);

        let mut body = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            body.push(self.parse_stmt());
        }

        self.expect(&Token::RBrace);

        Function {
            name,
            params,
            ret_type,
            body,
        }
    }

    // =====================================================
    // STATEMENTS
    // =====================================================
    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.next(); // let

        let name = self.expect_ident();
        self.expect(&Token::Colon);
        let t = self.parse_type();

        self.expect(&Token::Assign);
        let expr = self.parse_expr();
        self.expect(&Token::Semicolon);

        Stmt::Let(name, t, expr)
    }

    fn parse_return(&mut self) -> Stmt {
        self.next(); // return
        let expr = self.parse_expr();
        self.expect(&Token::Semicolon);
        Stmt::Return(expr)
    }

    fn parse_if(&mut self) -> Stmt {
        self.next(); // if

        let cond = self.parse_expr();

        // THEN BLOCK
        self.expect(&Token::LBrace);
        let mut then_body = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            then_body.push(self.parse_stmt());
        }
        self.expect(&Token::RBrace);

        // ELSE BLOCK
        self.expect(&Token::Else);
        self.expect(&Token::LBrace);
        let mut else_body = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            else_body.push(self.parse_stmt());
        }
        self.expect(&Token::RBrace);

        Stmt::If(cond, then_body, else_body)
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expr();
        self.expect(&Token::Semicolon);
        Stmt::ExprStmt(expr)
    }

    // =====================================================
    // EXPRESSIONS
    // =====================================================
    fn parse_expr(&mut self) -> Expr {
        self.parse_binary()
    }

    fn parse_binary(&mut self) -> Expr {
        let mut left = self.parse_primary();

        loop {
            let op = match self.peek() {
                Token::Plus => "+",
                Token::Minus => "-",
                Token::Star => "*",
                Token::Slash => "/",
                Token::Greater => ">",
                Token::Less => "<",
                Token::EqualEqual => "==",
                Token::NotEqual => "!=",
                _ => break,
            }
            .to_string();

            self.next(); // consume operator

            let right = self.parse_primary();
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        left
    }

    // =====================================================
    // PRIMARY (fixed version)
    // =====================================================
    fn parse_primary(&mut self) -> Expr {
        match self.next() {
            Token::Number(n) => Expr::Number(*n),

            Token::StringLiteral(s) => Expr::StringLiteral(s.clone()),

            Token::Ident(name) => {
                let ident = name.clone();

                // 먼저 함수 호출인지 확인
                let is_call = matches!(self.peek(), Token::LParen);

                // 변수
                if !is_call {
                    return Expr::Var(ident);
                }

                // 함수 호출
                self.next(); // '('

                let mut args = Vec::new();
                while !matches!(self.peek(), Token::RParen) {
                    args.push(self.parse_expr());
                    if matches!(self.peek(), Token::Comma) {
                        self.next(); // consume comma
                    }
                }

                self.expect(&Token::RParen);
                Expr::Call(ident, args)
            }

            Token::LParen => {
                let expr = self.parse_expr();
                self.expect(&Token::RParen);
                expr
            }

            other => panic!("Unexpected token in primary: {:?}", other),
        }
    }
}
