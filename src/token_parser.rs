use core::panic;
use std::iter::Peekable;

use crate::ast_nodes::*;
use crate::lang_errors::LangError;
use crate::spans::*;
use crate::token_lexer::Lexer;
use crate::tokens::*;
use colored::*;
#[derive(Clone)]
pub struct TokenIter<'input> {
    lexer: Lexer<'input>,
}

impl<'input> TokenIter<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
}

impl<'input> Iterator for TokenIter<'input> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next()
    }
}
#[derive(Clone)]
pub struct Parser<'input, I>
where
    I: Iterator<Item = Token>,
{
    input: &'input str,
    tokens: Peekable<I>,
    err_out: LangError,
}

impl<'input> Parser<'input, TokenIter<'input>> {
    pub fn new(input: &'input str) -> Parser<'input, TokenIter<'input>> {
        Parser {
            input,
            tokens: TokenIter::new(input).peekable(),
            err_out: LangError {
                input: input.to_string(),
            },
        }
    }
    pub fn text(&mut self, token: &Token) -> String {
        return self.input[token.span.0..token.span.1].to_string();
    }
    fn peek(&mut self) -> Option<Token> {
        self.tokens.peek().cloned()
    }
    fn peek_some(&mut self) -> Result<Token, ()> {
        let Some(peeked) = self.tokens.peek().cloned() else {
            println!(
                "{} Expected to find another token but none was found",
                "ERROR!".red()
            );
            return Err(());
        };
        return Ok(peeked);
    }
    fn next(&mut self) -> Option<Token> {
        self.tokens.next()
    }
    fn check_valid(&mut self, expected: TokenType, token: Token) -> Result<(), ()> {
        if token.kind != expected {
            println!();
            let err = self.err_out.build(
                format!("expected token {expected:?} but got token {:?}", token.kind).as_str(),
                token.span,
            );
            println!("{err}");
            println!();
            return Err(());
        }
        Ok(())
    }

    fn expect(&mut self, expected: TokenType) -> Result<Token, ()> {
        let token: Token = self.peek_some()?;
        self.check_valid(expected, token.clone())?;
        return Ok(token);
    }
    fn parse_vardef(&mut self) -> Result<NodeSpan, ()> {
        let first = dbg!(self.peek());
        let ident: Token = self.expect(TokenType::IDENTIFIER)?;
        let var_name = self.text(&ident);
        self.next();
        match self.peek_some()?.kind {
            TokenType::EOL => {
                let Some(last) = self.peek() else {todo!()};
                return Ok(Declaration {
                    var_name: var_name,
                    value: Box::new(Value::Null.to_nodespan(ident.span)),
                }
                .to_nodespan((first.expect("idk").span.0, last.span.1)));
            }
            TokenType::EQUAL => {
                let last = self.next();
                let val = self.parse_expr()?;
                return Ok(Declaration {
                    var_name: var_name,
                    value: Box::new(val),
                }
                .to_nodespan((first.expect("idk").span.0, last.expect("idk").span.1)));
            }
            _ => {
                let span = self.peek_some()?.span;
                self.err_out.emit("Invalid variable declaration", span);
                return Err(());
            }
        }
    }

    fn parse_paren(&mut self, paren: Token) -> Result<NodeSpan, ()> {
        let expr = self.parse_expr()?;
        let err = self.err_out.build("Unterminated parentheses", paren.span);
        let Some(end) = self.peek() else {
            println!();
            println!("{err}");
            println!();
            return Err(());
        };
        self.check_valid(TokenType::RPAREN, end)?;
        self.next();
        return Ok(expr);
    }
    fn simple_parse(&mut self, peeked: &Option<Token>) -> Result<NodeSpan, ()> {
        let Some(value) = peeked.clone() else {todo!()};

        match value.kind {
            TokenType::STR => {
                return Ok(Value::Str(self.text(&value)).to_nodespan(value.span));
            }
            TokenType::VAR => {
                return self.parse_vardef();
            }
            TokenType::NUM => {
                return Ok(Value::Num(self.text(&value).parse().unwrap()).to_nodespan(value.span));
            }
            TokenType::FALSE => {
                return Ok(Value::Bool(false).to_nodespan(value.span));
            }
            TokenType::TRUE => {
                return Ok(Value::Bool(true).to_nodespan(value.span));
            }
            TokenType::IDENTIFIER => {
                return Ok(Variable {
                    name: self.text(&value),
                }
                .to_nodespan(value.span));
            }
            TokenType::DO => {
                let first = self.expect(TokenType::LBRACE)?;
                let block = self.parse_block()?;
                let span = block.span.clone();
                return Ok(DoBlock {
                    body: Box::new(block),
                }
                .to_nodespan((first.span.0, span.1)));
            }
            TokenType::LOOP => {
                let first = self.expect(TokenType::LBRACE)?;
                let block = self.parse_block()?;
                let span = block.span.clone();
                return Ok(Loop {
                    proc: Box::new(block),
                }
                .to_nodespan((first.span.0, span.1)));
            }
            TokenType::NOT | TokenType::BANG => return self.unary_operator(UnaryOp::NOT),
            TokenType::MINUS => return self.unary_operator(UnaryOp::NEGATIVE),
            TokenType::LPAREN => return self.parse_paren(value),
            unexpected => {
                panic!("{unexpected:?}");
            }
        };
    }
    fn parse_call(&mut self, callee: NodeSpan) -> Result<NodeSpan, ()> {
        let mut params: NodeStream = vec![];
        let first = self.next().expect("");
        let mut token = dbg!(self.peek_some()?);
        if token.kind == TokenType::RPAREN {
            let last = self.next().expect("");
            return Ok(Call {
                args: Box::new(params),
                callee: Box::new(callee),
            }
            .to_nodespan((first.span.0, last.span.1)));
        }
        while token.kind != TokenType::RPAREN {
            let expr = dbg!(self.parse_expr()?);
            token = dbg!(self.peek_some()?);
            match token.kind {
                TokenType::RPAREN => {
                    params.push(expr);
                    break;
                }
                TokenType::COMMA => {
                    params.push(expr);
                    self.next();
                }
                _ => {
                    self.err_out.emit("Invalid Token", token.span);
                    return Err(());
                }
            }
        }
        let last = self.peek_some()?;
        self.next();

        let val = Call {
            args: Box::new(params),
            callee: Box::new(callee),
        }
        .to_nodespan((first.span.0, last.span.1));
        self.parse_operator(token, val)
    }

    pub fn parse_block(&mut self) -> Result<NodeSpan, ()> {
        let first = self.expect(TokenType::LBRACE)?;
        let mut body: NodeStream = vec![];
        self.next().expect("");
        let mut token = dbg!(self.peek_some()?);

        loop {
            if self.peek_some()?.kind == TokenType::RBRACE {
                break;
            }
            let expr = self.parse_expr()?;
            token = dbg!(self.peek_some()?);

            match token.kind {
                TokenType::EOL => {
                    self.next();
                    body.push(expr);
                    continue;
                }
                TokenType::RBRACE => {
                    body.push(expr.wrap_in_result());
                    break;
                }
                unexpected => self.err_out.emit(
                    format!("Unexpected Token{unexpected:?}").as_str(),
                    token.span,
                ),
            }
        }
        return Ok(Block {
            body: Box::new(body),
        }
        .to_nodespan((first.span.0, token.span.1)));
    }
    fn unary_operator(&mut self, kind: UnaryOp) -> Result<NodeSpan, ()> {
        let token = self.peek();
        self.next();
        let right = self.simple_parse(&token)?;
        return Ok(UnaryNode {
            kind,
            object: Box::new(right),
        }
        .to_nodespan((token.expect("").span)));
    }
    fn binary_operator(&mut self, left: NodeSpan, kind: BinaryOp) -> Result<NodeSpan, ()> {
        let last = self.next();

        return Ok(BinaryNode {
            kind,
            left: Box::new(left),
            right: Box::new(self.parse_expr()?),
        }
        .to_nodespan(last.expect("fuck").span));
    }
    fn parse_assignment(&mut self, previous: Token, token: Token) -> Result<NodeSpan, ()> {
        self.check_valid(TokenType::IDENTIFIER, previous.clone())?;
        let var_name = self.text(&previous);
        let last = self.next();
        return Ok(Assignment {
            var_name,
            value: Box::new(self.parse_expr()?),
        }
        .to_nodespan((token.span.0, last.expect("").span.1)));
    }
    fn parse_operator(&mut self, previous: Token, left: NodeSpan) -> Result<NodeSpan, ()> {
        let Some(token) = self.peek() else {return Ok(left);};
        match token.kind {
            TokenType::EQUAL => return self.parse_assignment(previous, token),
            TokenType::LPAREN => return self.parse_call(left),
            TokenType::PLUS => return self.binary_operator(left, BinaryOp::ADD),
            TokenType::MINUS => return self.binary_operator(left, BinaryOp::SUBTRACT),
            TokenType::STAR => return self.binary_operator(left, BinaryOp::MULTIPLY),
            TokenType::SLASH => return self.binary_operator(left, BinaryOp::DIVIDE),
            TokenType::PERCENT => return self.binary_operator(left, BinaryOp::MODULO),
            TokenType::GREATER_EQUAL => return self.binary_operator(left, BinaryOp::GREATER_EQUAL),
            TokenType::GREATER => return self.binary_operator(left, BinaryOp::GREATER),
            TokenType::LESSER_EQUAL => return self.binary_operator(left, BinaryOp::LESSER_EQUAL),
            TokenType::LESSER => return self.binary_operator(left, BinaryOp::LESSER),
            TokenType::DOUBLE_EQUAL => return self.binary_operator(left, BinaryOp::ISEQUAL),
            TokenType::BANG_EQUAL => return self.binary_operator(left, BinaryOp::ISDIFERENT),

            TokenType::AND | TokenType::AMPERSAND => {
                return self.binary_operator(left, BinaryOp::AND)
            }
            TokenType::OR | TokenType::PIPE => return self.binary_operator(left, BinaryOp::OR),
            _ => {
                return Ok(left);
            }
        }
    }

    pub fn parse_expr(&mut self) -> Result<NodeSpan, ()> {
        let value = dbg!(self.peek());
        self.next();
        let left = self.simple_parse(&value)?;
        let Some(peeked) = value else {todo!()};
        self.parse_operator(peeked, left)
    }
    pub fn parse_top(&mut self) -> Option<Result<NodeSpan, ()>> {
        match self.peek()?.kind {
            TokenType::VAR => {
                self.next();
                let var = Some(self.parse_vardef());
                self.next();
                var
            }
            TokenType::FUNC => todo!(),
            token => panic!("Invalid Token at toplevel: {token:?}"),
        }
    }
    pub fn batch_parse(&mut self) -> Block {
        let mut body: NodeStream = vec![];
        loop {
            let Some(parsed) = self.parse_top() else {break;};
            let Ok(parsed_2) = parsed else {break;};
            body.push(parsed_2);
        }
        return Block {
            body: Box::new(body),
        };
    }
}
