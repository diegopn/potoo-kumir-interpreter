//! A source-grounded interpreter core for school algorithms written in Kumir 2 syntax.

use std::collections::BTreeMap;
use std::fmt;

const MAX_STEPS: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Boolean(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Real(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{}", if *value { "да" } else { "нет" }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Integer,
    Real,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutorKind {
    Robot,
    Draftsman,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExecutorCommand {
    PaintCell,
    MoveUp,
    PenDown,
    PenUp,
    MoveVector { x: f64, y: f64 },
}

/// A host implementation for one of Kumir's independently configured executors.
pub trait ExecutorHost {
    fn execute(&mut self, executor: ExecutorKind, command: ExecutorCommand) -> Result<(), String>;
}

/// Services required by the algorithmic language. Executor support is optional.
pub trait Host {
    fn input(&mut self) -> Result<String, String>;
    fn output(&mut self, text: &str) -> Result<(), String>;

    fn executor_host(&mut self) -> Option<&mut dyn ExecutorHost> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    name: Option<String>,
    executor: Option<ExecutorKind>,
    declarations: BTreeMap<String, Type>,
    body: Vec<Stmt>,
}

impl Program {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn executor(&self) -> Option<ExecutorKind> {
        self.executor
    }
}

#[derive(Clone, Debug)]
enum Stmt {
    Assign {
        name: String,
        value: Expr,
        span: Span,
    },
    Input {
        names: Vec<String>,
        span: Span,
    },
    Output {
        values: Vec<Expr>,
        span: Span,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        name: String,
        start: Expr,
        end: Expr,
        step: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Executor {
        command: ExecutorCommandExpr,
        span: Span,
    },
}

#[derive(Clone, Debug)]
enum ExecutorCommandExpr {
    PaintCell,
    MoveUp,
    PenDown,
    PenUp,
    MoveVector(Expr, Expr),
}

#[derive(Clone, Debug)]
enum Expr {
    Integer(i64),
    Real(f64),
    Variable(String, Span),
    UnaryMinus(Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Clone, Copy, Debug)]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeError {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

#[derive(Clone, Debug)]
pub struct Execution {
    pub variables: BTreeMap<String, Value>,
    pub steps: usize,
}

#[derive(Clone, Debug)]
struct SourceLine {
    number: usize,
    text: String,
}

pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>> {
    Parser::new(source).parse().map_err(|error| vec![error])
}

struct Parser {
    lines: Vec<SourceLine>,
    at: usize,
}

impl Parser {
    fn new(source: &str) -> Self {
        Self {
            lines: source
                .lines()
                .enumerate()
                .map(|(index, line)| SourceLine {
                    number: index + 1,
                    text: line.trim().to_owned(),
                })
                .collect(),
            at: 0,
        }
    }

    fn parse(mut self) -> Result<Program, Diagnostic> {
        self.skip_blank();
        let executor = if let Some(line) = self.current().cloned() {
            if starts_with_keyword(&line.text, "использовать") {
                let span = self.span();
                let name = line.text["использовать".len()..].trim().to_owned();
                self.at += 1;
                match normalize(&name).as_str() {
                    "робот" => Some(ExecutorKind::Robot),
                    "чертежник" | "чертёжник" => Some(ExecutorKind::Draftsman),
                    _ => {
                        return Err(error(
                            span,
                            "only the documented Робот and Чертежник modules are supported",
                        ));
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        self.skip_blank();
        let header = self
            .current()
            .ok_or_else(|| error(Span { line: 1, column: 1 }, "expected `алг` header"))?;
        if !starts_with_keyword(&header.text, "алг") {
            return Err(error(self.span(), "expected `алг` header"));
        }
        let name = header.text["алг".len()..].trim();
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_owned())
        };
        self.at += 1;

        self.skip_blank();
        if !self
            .current()
            .is_some_and(|line| is_keyword(&line.text, "нач"))
        {
            return Err(error(
                self.span(),
                "expected `нач` after the algorithm header",
            ));
        }
        self.at += 1;

        let mut declarations = BTreeMap::new();
        self.skip_blank();
        while self.at < self.lines.len() {
            let Some((ty, names)) = parse_declaration(&self.lines[self.at].text) else {
                break;
            };
            let span = self.span();
            for name in split_names(names) {
                if !valid_identifier(name) {
                    return Err(error(span, format!("invalid identifier `{name}`")));
                }
                let key = normalize(name);
                if declarations.insert(key.clone(), ty).is_some() {
                    return Err(error(span, format!("duplicate declaration `{name}`")));
                }
            }
            self.at += 1;
            self.skip_blank();
        }

        let body = self.block_until(&["кон"])?;
        if !self
            .current()
            .is_some_and(|line| is_keyword(&line.text, "кон"))
        {
            return Err(error(self.span(), "expected `кон` to finish the algorithm"));
        }
        self.at += 1;
        self.skip_blank();
        if self.at != self.lines.len() {
            return Err(error(self.span(), "unexpected text after `кон`"));
        }

        for statement in &body {
            validate_statement(statement, &declarations, executor)?;
        }
        Ok(Program {
            name,
            executor,
            declarations,
            body,
        })
    }

    fn block_until(&mut self, terminators: &[&str]) -> Result<Vec<Stmt>, Diagnostic> {
        let mut body = Vec::new();
        loop {
            self.skip_blank();
            let Some(line) = self.current().cloned() else {
                return Err(error(self.span(), "unexpected end of source"));
            };
            if terminators.iter().any(|end| is_keyword(&line.text, end)) {
                return Ok(body);
            }
            if ["кон", "иначе", "все", "кц"]
                .iter()
                .any(|end| is_keyword(&line.text, end))
            {
                return Err(error(
                    self.span(),
                    format!("unexpected block terminator `{}`", line.text),
                ));
            }
            body.push(self.statement()?);
        }
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        self.skip_blank();
        let line = self
            .current()
            .cloned()
            .ok_or_else(|| error(self.span(), "expected a statement"))?;
        let span = self.span();
        let lower = normalize(&line.text);

        if starts_with_keyword(&line.text, "если") {
            let header = line.text["если".len()..].trim();
            let Some(condition_text) = strip_keyword_suffix(header, "то") else {
                return Err(error(span, "expected `то` after the condition"));
            };
            let condition = parse_expression(condition_text, span)?;
            self.at += 1;
            let then_body = self.block_until(&["иначе", "все"])?;
            let else_body = if self
                .current()
                .is_some_and(|end| is_keyword(&end.text, "иначе"))
            {
                self.at += 1;
                self.block_until(&["все"])?
            } else {
                Vec::new()
            };
            if !self
                .current()
                .is_some_and(|end| is_keyword(&end.text, "все"))
            {
                return Err(error(self.span(), "expected `все` to close `если`"));
            }
            self.at += 1;
            return Ok(Stmt::If {
                condition,
                then_body,
                else_body,
                span,
            });
        }

        if starts_with_keyword(&line.text, "нц пока") {
            let condition_text = line.text["нц пока".len()..].trim();
            let condition = parse_expression(condition_text, span)?;
            self.at += 1;
            let body = self.block_until(&["кц"])?;
            self.consume("кц")?;
            return Ok(Stmt::While {
                condition,
                body,
                span,
            });
        }

        if starts_with_keyword(&line.text, "нц для") {
            let rest = line.text["нц для".len()..].trim();
            let Some((name, rest)) = split_keyword(rest, "от") else {
                return Err(error(span, "expected `от` in counted loop"));
            };
            let Some((start_text, rest)) = split_keyword(rest, "до") else {
                return Err(error(span, "expected `до` in counted loop"));
            };
            if !valid_identifier(name.trim()) {
                return Err(error(span, "invalid counted-loop variable"));
            }
            let (end_text, step_text) =
                split_keyword(rest, "шаг").map_or((rest, None), |(end, step)| (end, Some(step)));
            let start = parse_expression(start_text.trim(), span)?;
            let end = parse_expression(end_text.trim(), span)?;
            let step = match step_text {
                Some(text) => parse_expression(text.trim(), span)?,
                None => Expr::Integer(1),
            };
            self.at += 1;
            let body = self.block_until(&["кц"])?;
            self.consume("кц")?;
            return Ok(Stmt::For {
                name: normalize(name.trim()),
                start,
                end,
                step,
                body,
                span,
            });
        }

        if starts_with_keyword(&line.text, "ввод") {
            let names = split_names(line.text["ввод".len()..].trim())
                .into_iter()
                .map(normalize)
                .collect::<Vec<_>>();
            if names.is_empty() || names.iter().any(|name| !valid_identifier(name)) {
                return Err(error(span, "`ввод` requires declared variable names"));
            }
            self.at += 1;
            return Ok(Stmt::Input { names, span });
        }

        if starts_with_keyword(&line.text, "вывод") {
            let values = parse_expression_list(line.text["вывод".len()..].trim(), span)?;
            self.at += 1;
            return Ok(Stmt::Output { values, span });
        }

        if let Some((left, right)) = line.text.split_once(":=") {
            let name = left.trim();
            if !valid_identifier(name) {
                return Err(error(span, "assignment target must be a variable name"));
            }
            let value = parse_expression(right.trim(), span)?;
            self.at += 1;
            return Ok(Stmt::Assign {
                name: normalize(name),
                value,
                span,
            });
        }

        if let Some(executor) = self.executor_statement(&lower, span)? {
            self.at += 1;
            return Ok(Stmt::Executor {
                command: executor,
                span,
            });
        }

        Err(error(
            span,
            format!("unsupported statement `{}`", line.text),
        ))
    }

    fn executor_statement(
        &self,
        line: &str,
        span: Span,
    ) -> Result<Option<ExecutorCommandExpr>, Diagnostic> {
        let Some(kind) = self.executor_kind_before_program() else {
            return Ok(None);
        };
        let command = match (kind, line) {
            (ExecutorKind::Robot, "закрасить") => ExecutorCommandExpr::PaintCell,
            (ExecutorKind::Robot, "вверх") => ExecutorCommandExpr::MoveUp,
            (ExecutorKind::Draftsman, "опустить перо") => ExecutorCommandExpr::PenDown,
            (ExecutorKind::Draftsman, "поднять перо") => ExecutorCommandExpr::PenUp,
            (ExecutorKind::Draftsman, _)
                if line.starts_with("сместиться на вектор(") && line.ends_with(')') =>
            {
                let args = &line["сместиться на вектор(".len()..line.len() - 1];
                let values = parse_expression_list(args, span)?;
                if values.len() != 2 {
                    return Err(error(
                        span,
                        "`сместиться на вектор` requires two coordinates",
                    ));
                }
                let mut values = values.into_iter();
                ExecutorCommandExpr::MoveVector(
                    values.next().expect("length checked"),
                    values.next().expect("length checked"),
                )
            }
            _ => return Ok(None),
        };
        Ok(Some(command))
    }

    fn executor_kind_before_program(&self) -> Option<ExecutorKind> {
        self.lines.iter().find_map(|line| {
            if starts_with_keyword(&line.text, "использовать") {
                match normalize(line.text["использовать".len()..].trim()).as_str() {
                    "робот" => Some(ExecutorKind::Robot),
                    "чертежник" | "чертёжник" => Some(ExecutorKind::Draftsman),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    fn consume(&mut self, keyword: &str) -> Result<(), Diagnostic> {
        if self
            .current()
            .is_some_and(|line| is_keyword(&line.text, keyword))
        {
            self.at += 1;
            Ok(())
        } else {
            Err(error(self.span(), format!("expected `{keyword}`")))
        }
    }

    fn skip_blank(&mut self) {
        while self.at < self.lines.len() && self.lines[self.at].text.is_empty() {
            self.at += 1;
        }
    }

    fn current(&self) -> Option<&SourceLine> {
        self.lines.get(self.at)
    }

    fn span(&self) -> Span {
        Span {
            line: self
                .current()
                .map_or_else(|| self.lines.len() + 1, |line| line.number),
            column: 1,
        }
    }
}

fn validate_statement(
    statement: &Stmt,
    declarations: &BTreeMap<String, Type>,
    executor: Option<ExecutorKind>,
) -> Result<(), Diagnostic> {
    match statement {
        Stmt::Assign { name, value, span } => {
            if !declarations.contains_key(name) {
                return Err(error(*span, format!("undefined variable `{name}`")));
            }
            validate_expression(value, declarations)?;
        }
        Stmt::Input { names, span } => {
            for name in names {
                if !declarations.contains_key(name) {
                    return Err(error(*span, format!("undefined variable `{name}`")));
                }
            }
        }
        Stmt::Output { values, .. } => {
            for value in values {
                validate_expression(value, declarations)?;
            }
        }
        Stmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            validate_expression(condition, declarations)?;
            for child in then_body.iter().chain(else_body) {
                validate_statement(child, declarations, executor)?;
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            validate_expression(condition, declarations)?;
            for child in body {
                validate_statement(child, declarations, executor)?;
            }
        }
        Stmt::For {
            name,
            start,
            end,
            step,
            body,
            span,
        } => {
            if declarations.get(name) != Some(&Type::Integer) {
                return Err(error(
                    *span,
                    "counted-loop variable must be declared with `цел`",
                ));
            }
            validate_expression(start, declarations)?;
            validate_expression(end, declarations)?;
            validate_expression(step, declarations)?;
            for child in body {
                validate_statement(child, declarations, executor)?;
            }
        }
        Stmt::Executor { span, .. } if executor.is_none() => {
            return Err(error(*span, "executor command requires `использовать`"));
        }
        Stmt::Executor { command, .. } => {
            if let ExecutorCommandExpr::MoveVector(x, y) = command {
                validate_expression(x, declarations)?;
                validate_expression(y, declarations)?;
            }
        }
    }
    Ok(())
}

fn validate_expression(
    expr: &Expr,
    declarations: &BTreeMap<String, Type>,
) -> Result<(), Diagnostic> {
    match expr {
        Expr::Variable(name, span) if !declarations.contains_key(name) => {
            Err(error(*span, format!("undefined variable `{name}`")))
        }
        Expr::UnaryMinus(value) => validate_expression(value, declarations),
        Expr::Binary(left, _, right) => {
            validate_expression(left, declarations)?;
            validate_expression(right, declarations)
        }
        _ => Ok(()),
    }
}

fn parse_declaration(line: &str) -> Option<(Type, &str)> {
    if starts_with_keyword(line, "цел") {
        Some((Type::Integer, line["цел".len()..].trim()))
    } else if starts_with_keyword(line, "вещ") {
        Some((Type::Real, line["вещ".len()..].trim()))
    } else {
        None
    }
}

fn parse_expression_list(source: &str, span: Span) -> Result<Vec<Expr>, Diagnostic> {
    let parts = split_top_level(source, ',');
    if parts.is_empty() || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(error(span, "expected one or more expressions"));
    }
    parts
        .into_iter()
        .map(|part| parse_expression(part.trim(), span))
        .collect()
}

fn parse_expression(source: &str, span: Span) -> Result<Expr, Diagnostic> {
    let tokens = lex_expression(source, span)?;
    let mut parser = ExpressionParser {
        tokens,
        at: 0,
        base: span,
    };
    let expr = parser.comparison()?;
    if !matches!(parser.peek(), ExprTokenKind::End) {
        return Err(error(
            parser.current_span(span),
            "unexpected token in expression",
        ));
    }
    Ok(expr)
}

#[derive(Clone, Debug)]
struct ExprToken {
    kind: ExprTokenKind,
    column: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum ExprTokenKind {
    Integer(i64),
    Real(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    LeftParen,
    RightParen,
    End,
}

fn lex_expression(source: &str, span: Span) -> Result<Vec<ExprToken>, Diagnostic> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < chars.len() {
        let ch = chars[at];
        if ch.is_whitespace() {
            at += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = at;
            while at < chars.len() && chars[at].is_ascii_digit() {
                at += 1;
            }
            if at < chars.len() && chars[at] == '.' {
                at += 1;
                let fractional = at;
                while at < chars.len() && chars[at].is_ascii_digit() {
                    at += 1;
                }
                if fractional == at {
                    return Err(error(
                        Span {
                            column: start + 1,
                            ..span
                        },
                        "real literal requires digits after the decimal point",
                    ));
                }
                let text = chars[start..at].iter().collect::<String>();
                let number = text.parse::<f64>().map_err(|_| {
                    error(
                        Span {
                            column: start + 1,
                            ..span
                        },
                        "invalid real literal",
                    )
                })?;
                tokens.push(ExprToken {
                    kind: ExprTokenKind::Real(number),
                    column: start + 1,
                });
            } else {
                let text = chars[start..at].iter().collect::<String>();
                let number = text.parse::<i64>().map_err(|_| {
                    error(
                        Span {
                            column: start + 1,
                            ..span
                        },
                        "integer literal is out of range",
                    )
                })?;
                tokens.push(ExprToken {
                    kind: ExprTokenKind::Integer(number),
                    column: start + 1,
                });
            }
            continue;
        }
        if is_identifier_start(ch) {
            let start = at;
            at += 1;
            while at < chars.len() && is_identifier_continue(chars[at]) {
                at += 1;
            }
            tokens.push(ExprToken {
                kind: ExprTokenKind::Identifier(normalize(
                    &chars[start..at].iter().collect::<String>(),
                )),
                column: start + 1,
            });
            continue;
        }
        let column = at + 1;
        let next = chars.get(at + 1).copied();
        let kind = match (ch, next) {
            ('<', Some('=')) => {
                at += 2;
                ExprTokenKind::LessEqual
            }
            ('>', Some('=')) => {
                at += 2;
                ExprTokenKind::GreaterEqual
            }
            ('<', Some('>')) | ('!', Some('=')) => {
                at += 2;
                ExprTokenKind::NotEqual
            }
            ('+', _) => {
                at += 1;
                ExprTokenKind::Plus
            }
            ('-', _) => {
                at += 1;
                ExprTokenKind::Minus
            }
            ('*', _) | ('×', _) => {
                at += 1;
                ExprTokenKind::Star
            }
            ('/', _) | ('÷', _) => {
                at += 1;
                ExprTokenKind::Slash
            }
            ('=', _) => {
                at += 1;
                ExprTokenKind::Equal
            }
            ('<', _) => {
                at += 1;
                ExprTokenKind::Less
            }
            ('>', _) => {
                at += 1;
                ExprTokenKind::Greater
            }
            ('(', _) => {
                at += 1;
                ExprTokenKind::LeftParen
            }
            (')', _) => {
                at += 1;
                ExprTokenKind::RightParen
            }
            _ => {
                return Err(error(
                    Span { column, ..span },
                    format!("unsupported character `{ch}` in expression"),
                ));
            }
        };
        tokens.push(ExprToken { kind, column });
    }
    tokens.push(ExprToken {
        kind: ExprTokenKind::End,
        column: source.chars().count() + 1,
    });
    Ok(tokens)
}

struct ExpressionParser {
    tokens: Vec<ExprToken>,
    at: usize,
    base: Span,
}

impl ExpressionParser {
    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.additive()?;
        loop {
            let op = match self.peek() {
                ExprTokenKind::Equal => BinaryOp::Equal,
                ExprTokenKind::NotEqual => BinaryOp::NotEqual,
                ExprTokenKind::Less => BinaryOp::Less,
                ExprTokenKind::LessEqual => BinaryOp::LessEqual,
                ExprTokenKind::Greater => BinaryOp::Greater,
                ExprTokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.at += 1;
            let rhs = self.additive()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn additive(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                ExprTokenKind::Plus => BinaryOp::Add,
                ExprTokenKind::Minus => BinaryOp::Subtract,
                _ => break,
            };
            self.at += 1;
            let rhs = self.multiplicative()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.unary()?;
        loop {
            let op = match self.peek() {
                ExprTokenKind::Star => BinaryOp::Multiply,
                ExprTokenKind::Slash => BinaryOp::Divide,
                _ => break,
            };
            self.at += 1;
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.peek(), ExprTokenKind::Minus) {
            self.at += 1;
            return Ok(Expr::UnaryMinus(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.tokens[self.at].clone();
        match token.kind {
            ExprTokenKind::Integer(value) => {
                self.at += 1;
                Ok(Expr::Integer(value))
            }
            ExprTokenKind::Real(value) => {
                self.at += 1;
                Ok(Expr::Real(value))
            }
            ExprTokenKind::Identifier(name) => {
                self.at += 1;
                Ok(Expr::Variable(
                    name,
                    Span {
                        line: self.base.line,
                        column: token.column,
                    },
                ))
            }
            ExprTokenKind::LeftParen => {
                self.at += 1;
                let expr = self.comparison()?;
                if !matches!(self.peek(), ExprTokenKind::RightParen) {
                    return Err(self.error_at_current("expected `)`"));
                }
                self.at += 1;
                Ok(expr)
            }
            _ => {
                Err(self
                    .error_at_current("expected a number, variable, or parenthesized expression"))
            }
        }
    }

    fn peek(&self) -> &ExprTokenKind {
        &self.tokens[self.at].kind
    }

    fn error_at_current(&self, message: &str) -> Diagnostic {
        Diagnostic {
            span: Span {
                line: self.base.line,
                column: self.tokens[self.at].column,
            },
            message: message.to_owned(),
        }
    }

    fn current_span(&self, base: Span) -> Span {
        Span {
            line: base.line,
            column: self.tokens[self.at].column,
        }
    }
}

pub struct Interpreter<H: Host> {
    host: H,
    steps: usize,
}

impl<H: Host> Interpreter<H> {
    pub fn new(host: H) -> Self {
        Self { host, steps: 0 }
    }

    pub fn run(&mut self, program: &Program) -> Result<Execution, RuntimeError> {
        self.steps = 0;
        let mut variables = program
            .declarations
            .iter()
            .map(|(name, ty)| {
                (
                    name.clone(),
                    match ty {
                        Type::Integer => Value::Integer(0),
                        Type::Real => Value::Real(0.0),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        self.execute_block(&program.body, program, &mut variables)?;
        Ok(Execution {
            variables,
            steps: self.steps,
        })
    }

    pub fn into_host(self) -> H {
        self.host
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        program: &Program,
        variables: &mut BTreeMap<String, Value>,
    ) -> Result<(), RuntimeError> {
        for statement in statements {
            self.tick(statement_span(statement))?;
            match statement {
                Stmt::Assign { name, value, span } => {
                    let result = self.evaluate(value, variables, *span)?;
                    let declared = program.declarations.get(name).ok_or_else(|| {
                        runtime_error(*span, format!("undefined variable `{name}`"))
                    })?;
                    let result = coerce_for_assignment(result, *declared, *span)?;
                    variables.insert(name.clone(), result);
                }
                Stmt::Input { names, span } => {
                    for name in names {
                        let raw = self
                            .host
                            .input()
                            .map_err(|message| runtime_error(*span, message))?;
                        let declared = program.declarations.get(name).ok_or_else(|| {
                            runtime_error(*span, format!("undefined variable `{name}`"))
                        })?;
                        let value = match declared {
                            Type::Integer => Value::Integer(raw.trim().parse().map_err(|_| {
                                runtime_error(
                                    *span,
                                    format!("input for `{name}` is not an integer"),
                                )
                            })?),
                            Type::Real => Value::Real(raw.trim().parse().map_err(|_| {
                                runtime_error(
                                    *span,
                                    format!("input for `{name}` is not a real number"),
                                )
                            })?),
                        };
                        variables.insert(name.clone(), value);
                    }
                }
                Stmt::Output { values, span } => {
                    let mut rendered = Vec::with_capacity(values.len());
                    for expr in values {
                        rendered.push(self.evaluate(expr, variables, *span)?.to_string());
                    }
                    self.host
                        .output(&rendered.join(" "))
                        .map_err(|message| runtime_error(*span, message))?;
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                    span,
                } => {
                    let branch = match self.evaluate(condition, variables, *span)? {
                        Value::Boolean(true) => then_body,
                        Value::Boolean(false) => else_body,
                        _ => return Err(runtime_error(*span, "condition must be a comparison")),
                    };
                    self.execute_block(branch, program, variables)?;
                }
                Stmt::While {
                    condition,
                    body,
                    span,
                } => loop {
                    self.tick(*span)?;
                    match self.evaluate(condition, variables, *span)? {
                        Value::Boolean(true) => self.execute_block(body, program, variables)?,
                        Value::Boolean(false) => break,
                        _ => {
                            return Err(runtime_error(
                                *span,
                                "loop condition must be a comparison",
                            ));
                        }
                    }
                },
                Stmt::For {
                    name,
                    start,
                    end,
                    step,
                    body,
                    span,
                } => {
                    if program.declarations.get(name) != Some(&Type::Integer) {
                        return Err(runtime_error(
                            *span,
                            "counted-loop variable must be declared with `цел`",
                        ));
                    }
                    let start = integer_value(self.evaluate(start, variables, *span)?, *span)?;
                    let end = integer_value(self.evaluate(end, variables, *span)?, *span)?;
                    let step = integer_value(self.evaluate(step, variables, *span)?, *span)?;
                    if step == 0 {
                        return Err(runtime_error(*span, "counted-loop step cannot be zero"));
                    }
                    let mut value = start;
                    while if step > 0 { value <= end } else { value >= end } {
                        self.tick(*span)?;
                        variables.insert(name.clone(), Value::Integer(value));
                        self.execute_block(body, program, variables)?;
                        let Some(next) = value.checked_add(step) else {
                            return Err(runtime_error(*span, "counted-loop value overflowed"));
                        };
                        value = next;
                    }
                }
                Stmt::Executor { command, span } => {
                    let executor_kind = program.executor.ok_or_else(|| {
                        runtime_error(*span, "executor command requires `использовать`")
                    })?;
                    let command = match command {
                        ExecutorCommandExpr::PaintCell => ExecutorCommand::PaintCell,
                        ExecutorCommandExpr::MoveUp => ExecutorCommand::MoveUp,
                        ExecutorCommandExpr::PenDown => ExecutorCommand::PenDown,
                        ExecutorCommandExpr::PenUp => ExecutorCommand::PenUp,
                        ExecutorCommandExpr::MoveVector(x, y) => ExecutorCommand::MoveVector {
                            x: number_value(self.evaluate(x, variables, *span)?, *span)?,
                            y: number_value(self.evaluate(y, variables, *span)?, *span)?,
                        },
                    };
                    let executor = self.host.executor_host().ok_or_else(|| {
                        runtime_error(
                            *span,
                            "this program requires a configured Kumir executor host",
                        )
                    })?;
                    executor
                        .execute(executor_kind, command)
                        .map_err(|message| runtime_error(*span, message))?;
                }
            }
        }
        Ok(())
    }

    fn evaluate(
        &self,
        expr: &Expr,
        variables: &BTreeMap<String, Value>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Integer(value) => Ok(Value::Integer(*value)),
            Expr::Real(value) => Ok(Value::Real(*value)),
            Expr::Variable(name, location) => variables.get(name).cloned().ok_or_else(|| {
                runtime_error(
                    Span {
                        column: location.column,
                        ..span
                    },
                    format!("undefined variable `{name}`"),
                )
            }),
            Expr::UnaryMinus(value) => match self.evaluate(value, variables, span)? {
                Value::Integer(value) => value
                    .checked_neg()
                    .map(Value::Integer)
                    .ok_or_else(|| runtime_error(span, "integer negation overflowed")),
                Value::Real(value) => Ok(Value::Real(-value)),
                Value::Boolean(_) => Err(runtime_error(span, "cannot negate a logical value")),
            },
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(left, variables, span)?;
                let right = self.evaluate(right, variables, span)?;
                evaluate_binary(left, *op, right, span)
            }
        }
    }

    fn tick(&mut self, span: Span) -> Result<(), RuntimeError> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            Err(runtime_error(span, "execution step limit exceeded"))
        } else {
            Ok(())
        }
    }
}

fn evaluate_binary(
    left: Value,
    op: BinaryOp,
    right: Value,
    span: Span,
) -> Result<Value, RuntimeError> {
    if matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
    ) {
        let (left, right) = (number_value(left, span)?, number_value(right, span)?);
        let result = match op {
            BinaryOp::Equal => left == right,
            BinaryOp::NotEqual => left != right,
            BinaryOp::Less => left < right,
            BinaryOp::LessEqual => left <= right,
            BinaryOp::Greater => left > right,
            BinaryOp::GreaterEqual => left >= right,
            _ => unreachable!(),
        };
        return Ok(Value::Boolean(result));
    }

    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => match op {
            BinaryOp::Add => a.checked_add(b).map(Value::Integer),
            BinaryOp::Subtract => a.checked_sub(b).map(Value::Integer),
            BinaryOp::Multiply => a.checked_mul(b).map(Value::Integer),
            BinaryOp::Divide => {
                if b == 0 {
                    return Err(runtime_error(span, "division by zero"));
                }
                Some(Value::Real(a as f64 / b as f64))
            }
            _ => None,
        }
        .ok_or_else(|| runtime_error(span, "integer arithmetic overflowed")),
        (a, b) if is_number(&a) && is_number(&b) => {
            let (a, b) = (number_value(a, span)?, number_value(b, span)?);
            match op {
                BinaryOp::Add => Ok(Value::Real(a + b)),
                BinaryOp::Subtract => Ok(Value::Real(a - b)),
                BinaryOp::Multiply => Ok(Value::Real(a * b)),
                BinaryOp::Divide if b != 0.0 => Ok(Value::Real(a / b)),
                BinaryOp::Divide => Err(runtime_error(span, "division by zero")),
                _ => Err(runtime_error(span, "invalid arithmetic operation")),
            }
        }
        _ => Err(runtime_error(span, "arithmetic requires numeric values")),
    }
}

fn coerce_for_assignment(value: Value, ty: Type, span: Span) -> Result<Value, RuntimeError> {
    match (ty, value) {
        (Type::Integer, value @ Value::Integer(_)) => Ok(value),
        (Type::Real, Value::Integer(value)) => Ok(Value::Real(value as f64)),
        (Type::Real, value @ Value::Real(_)) => Ok(value),
        (Type::Integer, Value::Real(_)) => Err(runtime_error(
            span,
            "real value cannot be assigned to `цел`",
        )),
        (_, Value::Boolean(_)) => Err(runtime_error(
            span,
            "logical value cannot be assigned to a numeric variable",
        )),
    }
}

fn number_value(value: Value, span: Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(value as f64),
        Value::Real(value) => Ok(value),
        Value::Boolean(_) => Err(runtime_error(span, "expected a numeric value")),
    }
}

fn integer_value(value: Value, span: Span) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(value),
        _ => Err(runtime_error(
            span,
            "counted-loop bounds and step must be integers",
        )),
    }
}

fn is_number(value: &Value) -> bool {
    matches!(value, Value::Integer(_) | Value::Real(_))
}

fn statement_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Assign { span, .. }
        | Stmt::Input { span, .. }
        | Stmt::Output { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Executor { span, .. } => *span,
    }
}

fn split_top_level(source: &str, delimiter: char) -> Vec<&str> {
    let mut output = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                output.push(&source[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    output.push(&source[start..]);
    output
}

fn split_names(source: &str) -> Vec<&str> {
    source
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}

fn split_keyword<'a>(source: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let normalized = normalize(source);
    let keyword = normalize(keyword);
    let needle = format!(" {keyword} ");
    let index = normalized.find(&needle)?;
    let end = index + needle.len();
    Some((&source[..index], &source[end..]))
}

fn strip_keyword_suffix<'a>(source: &'a str, keyword: &str) -> Option<&'a str> {
    let trimmed = source.trim_end();
    let lower = normalize(trimmed);
    let keyword = normalize(keyword);
    if lower.ends_with(&keyword) {
        let index = trimmed.len().checked_sub(keyword.len())?;
        let prefix = trimmed[..index].trim_end();
        if prefix.is_empty() || prefix.ends_with(char::is_whitespace) {
            return Some(prefix);
        }
    }
    None
}

fn starts_with_keyword(source: &str, keyword: &str) -> bool {
    let source = source.trim_start();
    let lower = normalize(source);
    let keyword = normalize(keyword);
    lower
        .strip_prefix(&keyword)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
}

fn is_keyword(source: &str, keyword: &str) -> bool {
    normalize(source.trim()) == normalize(keyword)
}

fn normalize(source: &str) -> String {
    source.chars().flat_map(char::to_lowercase).collect()
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}

fn error(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        span,
        message: message.into(),
    }
}

fn runtime_error(span: Span, message: impl Into<String>) -> RuntimeError {
    RuntimeError {
        span,
        message: message.into(),
    }
}
