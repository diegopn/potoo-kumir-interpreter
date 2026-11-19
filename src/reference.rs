//! Profile-aware Kumir 2.1 language implementation and host contracts.
//!
//! The reference API targets the versioned 2.1.0-rc11 classic and high-grade
//! manuals. Graphics, files, keyboard input, and executor environments remain
//! host services; the interpreter never accesses those resources directly.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const MAX_INTEGER: i32 = i32::MAX;
pub const MIN_INTEGER: i32 = i32::MIN;
pub const MAX_STEPS: usize = 1_000_000;
pub const MAX_TABLE_ELEMENTS: usize = 1_000_000;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Profile {
    #[default]
    Classic,
    HighGrade,
}

/// Selects which source section is currently being executed.
///
/// Kumir executor manuals restrict their `@@` inspection algorithms to the
/// hidden teacher-authored section. Student-visible code is the default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ExecutionContext {
    #[default]
    Student,
    HiddenTeacherCode,
}

impl Profile {
    pub fn parse(value: &str) -> Result<Self, String> {
        match normalize(value).as_str() {
            "classic" | "классический" | "классика" => Ok(Self::Classic),
            "high-grade" | "highgrade" | "olympiad" | "высокий" => Ok(Self::HighGrade),
            _ => Err(format!(
                "unknown Kumir profile `{value}`; use classic or high-grade"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Module {
    Robot,
    Draftsman,
    Grasshopper,
    IsometricRobot,
    Turtle,
    Keyboard,
    Painter,
    Vodoley,
    ComplexNumbers,
    Files,
    Strings,
}

impl Module {
    pub fn from_name(name: &str) -> Option<Self> {
        match normalize(name).as_str() {
            "робот" => Some(Self::Robot),
            "чертежник" | "чертёжник" | "рисование" => {
                Some(Self::Draftsman)
            }
            "кузнечик" => Some(Self::Grasshopper),
            "вертун" | "изометрический робот" => {
                Some(Self::IsometricRobot)
            }
            "черепаха" => Some(Self::Turtle),
            "клавиатура" => Some(Self::Keyboard),
            "рисователь" | "художник" => Some(Self::Painter),
            "водолей" => Some(Self::Vodoley),
            "комплексные числа" | "компл" => Some(Self::ComplexNumbers),
            "файлы" => Some(Self::Files),
            "строки" => Some(Self::Strings),
            _ => None,
        }
    }

    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::Robot => "Робот",
            Self::Draftsman => "Чертежник",
            Self::Grasshopper => "Кузнечик",
            Self::IsometricRobot => "Вертун",
            Self::Turtle => "Черепаха",
            Self::Keyboard => "Клавиатура",
            Self::Painter => "Рисователь",
            Self::Vodoley => "Водолей",
            Self::ComplexNumbers => "Комплексные числа",
            Self::Files => "Файлы",
            Self::Strings => "Строки",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Integer,
    Real,
    Boolean,
    Character,
    String,
    Complex,
    Color,
    KeyCode,
    File,
    Table {
        element: Box<Type>,
        bounds: Vec<(i32, i32)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imaginary: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayValue {
    pub bounds: Vec<(i32, i32)>,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i32),
    Real(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Complex(Complex),
    Color(String),
    KeyCode(i32),
    File(String),
    Table(ArrayValue),
    Void,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) | Self::KeyCode(value) => write!(f, "{value}"),
            Self::Real(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{}", if *value { "да" } else { "нет" }),
            Self::Character(value) => write!(f, "{value}"),
            Self::String(value) | Self::Color(value) | Self::File(value) => f.write_str(value),
            Self::Complex(value) => {
                if value.imaginary == 0.0 {
                    write!(f, "{}", value.real)
                } else if value.real == 0.0 {
                    write!(f, "{}i", value.imaginary)
                } else if value.real > 0.0 {
                    write!(f, "{}*i+{}", value.imaginary, value.real)
                } else {
                    write!(f, "{}*i{}", value.imaginary, value.real)
                }
            }
            Self::Table(_) => f.write_str("<таблица>"),
            Self::Void => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub mode: ParameterMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterMode {
    Argument,
    Result,
    ArgumentResult,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub profile: Profile,
    pub modules: Vec<Module>,
    pub main: String,
    global_declarations: BTreeMap<String, Declaration>,
    global_initializers: Vec<(String, Declaration)>,
    preamble: Vec<Stmt>,
    algorithms: BTreeMap<String, Algorithm>,
}

#[derive(Clone, Debug)]
struct Declaration {
    ty: Type,
    initial: Option<Expr>,
    constant: bool,
}

#[derive(Clone, Debug)]
struct Algorithm {
    name: String,
    parameters: Vec<Parameter>,
    result: Option<Type>,
    preconditions: Vec<Expr>,
    postconditions: Vec<Expr>,
    declarations: BTreeMap<String, Declaration>,
    body_declarations: BTreeSet<String>,
    body: Vec<Stmt>,
    span: Span,
}

impl Program {
    pub fn algorithms(&self) -> impl Iterator<Item = &str> {
        self.algorithms.keys().map(String::as_str)
    }

    pub fn has_module(&self, module: Module) -> bool {
        self.modules.contains(&module)
    }
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
pub enum FileRequest {
    Open {
        path: String,
        mode: String,
    },
    Close {
        handle: Value,
    },
    /// Reads the next line of text. The interpreter splits its values according
    /// to `ввод` arguments and `нс` line separators.
    Read {
        handle: Value,
    },
    Write {
        handle: Value,
        text: String,
    },
    Append {
        path: String,
        text: String,
    },
    Seek {
        handle: Value,
        position: i32,
    },
    Position {
        handle: Value,
    },
    Call {
        name: String,
        arguments: Vec<Value>,
    },
}

/// A host boundary for Kumir file commands. Implementations own handles and
/// must decide which paths and access modes their application permits.
pub trait FileHost {
    fn file_request(&mut self, request: FileRequest) -> Result<Value, String>;

    fn finish(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ModuleCall {
    pub module: Module,
    pub name: String,
    pub arguments: Vec<ModuleArgument>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleArgumentMode {
    Argument,
    Result,
    ArgumentResult,
}

#[derive(Clone, Debug)]
pub struct ModuleArgument {
    pub value: Value,
    pub mode: ModuleArgumentMode,
}

#[derive(Clone, Debug)]
pub struct ModuleCallResult {
    pub value: Value,
    /// Updated argument values. Result positions are `Some`; argument-only
    /// positions are `None`. The vector is empty when the call has no outputs.
    pub arguments: Vec<Option<Value>>,
}

/// Adapter for Kumir's separately configured actor modules. The generic call
/// preserves the language command and its values; rendering and environment
/// behavior belong to the embedding application.
pub trait ModuleHost {
    fn call(&mut self, call: ModuleCall) -> Result<ModuleCallResult, String>;
}

pub trait Host {
    fn input(&mut self) -> Result<String, String>;
    fn output(&mut self, text: &str) -> Result<(), String>;

    fn file_host(&mut self) -> Option<&mut dyn FileHost> {
        None
    }

    fn module_host(&mut self) -> Option<&mut dyn ModuleHost> {
        None
    }

    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn wait_millis(&mut self, millis: i32) -> Result<(), String> {
        let _ = millis;
        self.pause()
    }

    fn time_millis(&mut self) -> Result<i32, String> {
        Err("clock service is unavailable".to_owned())
    }

    /// Returns a random sample from the half-open range `[low, high)`.
    /// Integer library functions request an upper bound one greater than the
    /// largest value allowed by the Kumir call.
    fn random_number(&mut self, low: f64, high: f64) -> Result<f64, String> {
        let _ = (low, high);
        Err("random-number service is unavailable".to_owned())
    }
}

#[derive(Clone, Debug)]
struct SourceLine {
    number: usize,
    text: String,
}

#[derive(Clone, Debug)]
enum Stmt {
    Declare {
        declarations: Vec<(String, Declaration)>,
        span: Span,
    },
    Assign {
        target: Expr,
        value: Expr,
        span: Span,
    },
    Input {
        targets: Vec<InputPart>,
        span: Span,
    },
    Output {
        values: Vec<OutputPart>,
        span: Span,
    },
    Newline {
        span: Span,
    },
    If {
        condition: Expr,
        yes: Vec<Stmt>,
        no: Vec<Stmt>,
        span: Span,
    },
    Choose {
        value: Option<Expr>,
        branches: Vec<(Vec<Expr>, Vec<Stmt>)>,
        otherwise: Vec<Stmt>,
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
    Repeat {
        count: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    RepeatUntil {
        body: Vec<Stmt>,
        condition: Expr,
        span: Span,
    },
    Loop {
        body: Vec<Stmt>,
        span: Span,
    },
    Exit {
        span: Span,
    },
    Assert {
        condition: Expr,
        span: Span,
    },
    Stop {
        span: Span,
    },
    Pause {
        span: Span,
    },
    Call {
        name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    ModuleCall {
        module: Module,
        name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    File {
        request: FileExpr,
        span: Span,
    },
}

#[derive(Clone, Debug)]
enum OutputPart {
    Value(Expr),
    Newline,
}

#[derive(Clone, Debug)]
enum InputPart {
    Target(Expr),
    Newline,
}

#[derive(Clone, Debug)]
enum FileExpr {
    Call(String, Vec<Expr>),
}

#[derive(Clone, Debug)]
enum Expr {
    Literal(Value),
    Variable(String, Span),
    Index(Box<Expr>, Vec<Expr>),
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
    ModuleCall(Module, String, Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

#[derive(Clone, Copy, Debug)]
enum UnaryOp {
    Negate,
    Not,
    Positive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

pub fn parse(source: &str, profile: Profile) -> Result<Program, Vec<Diagnostic>> {
    Parser::new(source, profile)
        .parse()
        .map_err(|error| vec![error])
}

pub fn parse_classic(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse(source, Profile::Classic)
}

pub fn parse_high_grade(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse(source, Profile::HighGrade)
}

struct Parser {
    lines: Vec<SourceLine>,
    at: usize,
    profile: Profile,
    modules: Vec<Module>,
}

impl Parser {
    fn new(source: &str, profile: Profile) -> Self {
        let mut layout_lines = Vec::new();
        for (index, line) in source.lines().enumerate() {
            for part in split_source_line(line) {
                for text in split_implicit_layout(&part) {
                    if text.is_empty() || text.starts_with('#') {
                        continue;
                    }
                    layout_lines.push(SourceLine {
                        number: index + 1,
                        text,
                    });
                }
            }
        }
        let layout_lines = join_implicit_pri_lines(layout_lines);
        let mut lines = Vec::new();
        for line in layout_lines {
            let text = line.text;
            let number = line.number;
            let lower = normalize(&text);
            if starts_keyword(&text, "иначе") && lower != "иначе" {
                lines.push(SourceLine {
                    number,
                    text: "иначе".to_owned(),
                });
                lines.push(SourceLine {
                    number,
                    text: text["иначе".len()..].trim().to_owned(),
                });
            } else if starts_keyword(&text, "то") && lower != "то" {
                lines.push(SourceLine {
                    number,
                    text: text["то".len()..].trim().to_owned(),
                });
            } else if starts_keyword(&text, "если") {
                if let Some((header, inline)) = split_keyword(&text, "то") {
                    lines.push(SourceLine {
                        number,
                        text: format!("{} то", header.trim()),
                    });
                    if !inline.trim().is_empty() {
                        lines.push(SourceLine {
                            number,
                            text: inline.trim().to_owned(),
                        });
                    }
                } else {
                    lines.push(SourceLine { number, text });
                }
            } else if starts_keyword(&text, "при") {
                if let Some((header, inline)) = split_top_level_once(&text, ':') {
                    lines.push(SourceLine {
                        number,
                        text: header.trim().to_owned(),
                    });
                    if !inline.trim().is_empty() {
                        lines.push(SourceLine {
                            number,
                            text: inline.trim().to_owned(),
                        });
                    }
                } else {
                    lines.push(SourceLine { number, text });
                }
            } else {
                lines.push(SourceLine { number, text });
            }
        }
        Self {
            lines,
            at: 0,
            profile,
            modules: vec![Module::Files],
        }
    }

    fn parse(mut self) -> Result<Program, Diagnostic> {
        let mut global_declarations = BTreeMap::new();
        let mut global_initializers = Vec::new();
        let mut preamble = Vec::new();
        if self.profile == Profile::HighGrade && !self.modules.contains(&Module::Strings) {
            self.modules.push(Module::Strings);
        }
        while let Some(line) = self.current().cloned() {
            if starts_keyword(&line.text, "использовать") {
                let name = line.text["использовать".len()..].trim();
                let module = Module::from_name(name)
                    .ok_or_else(|| self.error_here("unknown module in `использовать`"))?;
                if module == Module::Strings && self.profile == Profile::Classic {
                    return Err(self.error_here(
                        "the Строки library is available only in the high-grade profile",
                    ));
                }
                if !self.modules.contains(&module) {
                    self.modules.push(module);
                }
                self.at += 1;
            } else if starts_keyword(&line.text, "алг") {
                break;
            } else {
                if let Some(declarations) =
                    parse_declaration(&line.text, self.span(), &self.modules)?
                {
                    for (name, declaration) in declarations {
                        if global_declarations
                            .insert(name.clone(), declaration.clone())
                            .is_some()
                        {
                            return Err(
                                self.error_here(&format!("duplicate global declaration `{name}`"))
                            );
                        }
                        global_initializers.push((name, declaration));
                    }
                    self.at += 1;
                } else {
                    let statement = self.statement()?;
                    if matches!(
                        statement,
                        Stmt::If { .. }
                            | Stmt::Choose { .. }
                            | Stmt::While { .. }
                            | Stmt::For { .. }
                            | Stmt::Repeat { .. }
                            | Stmt::RepeatUntil { .. }
                            | Stmt::Loop { .. }
                            | Stmt::Declare { .. }
                    ) {
                        return Err(self.error_here(
                            "program preamble accepts declarations and non-branching commands only",
                        ));
                    }
                    preamble.push(statement);
                }
            }
        }
        let mut algorithms = BTreeMap::new();
        let mut main = None;
        while self.at < self.lines.len() {
            let algorithm = self.algorithm()?;
            if main.is_none() {
                main = Some(algorithm.name.clone());
            }
            let key = normalize(&algorithm.name);
            if algorithms.insert(key.clone(), algorithm).is_some() {
                return Err(self.error_here(&format!("duplicate algorithm `{key}`")));
            }
        }
        let main = main.ok_or_else(|| {
            self.error_at(Span { line: 1, column: 1 }, "expected an `алг` definition")
        })?;
        for declaration in global_declarations.values() {
            if let Some(module) = required_type_module(&declaration.ty) {
                if !self.modules.contains(&module) {
                    return Err(error(
                        Span { line: 1, column: 1 },
                        format!(
                            "type `{}` requires `использовать {}`",
                            type_name(&declaration.ty),
                            module.canonical_name()
                        ),
                    ));
                }
            }
            if let Some(initial) = &declaration.initial {
                validate_expr(initial, &global_declarations)?;
                validate_assignment_type(
                    &declaration.ty,
                    initial,
                    &global_declarations,
                    &algorithms,
                    self.profile,
                    &self.modules,
                    Span { line: 1, column: 1 },
                )?;
            }
        }
        for algorithm in algorithms.values() {
            if let Some(name) = algorithm
                .declarations
                .keys()
                .find(|name| global_declarations.contains_key(*name))
            {
                return Err(error(
                    algorithm.span,
                    format!("local declaration `{name}` conflicts with a global declaration"),
                ));
            }
            for declaration in algorithm.declarations.values() {
                if let Some(module) = required_type_module(&declaration.ty) {
                    if !self.modules.contains(&module) {
                        return Err(error(
                            algorithm_span(algorithm),
                            format!(
                                "type `{}` requires `использовать {}`",
                                type_name(&declaration.ty),
                                module.canonical_name()
                            ),
                        ));
                    }
                }
                if let Some(initial) = &declaration.initial {
                    let mut scope = global_declarations.clone();
                    scope.extend(algorithm.declarations.clone());
                    validate_expr(initial, &scope)?;
                    validate_assignment_type(
                        &declaration.ty,
                        initial,
                        &scope,
                        &algorithms,
                        self.profile,
                        &self.modules,
                        algorithm_span(algorithm),
                    )?;
                }
            }
            let mut scope = global_declarations.clone();
            scope.extend(
                algorithm
                    .declarations
                    .iter()
                    .filter(|(name, _)| !algorithm.body_declarations.contains(*name))
                    .map(|(name, declaration)| (name.clone(), declaration.clone())),
            );
            for condition in algorithm
                .preconditions
                .iter()
                .chain(algorithm.postconditions.iter())
            {
                validate_expr(condition, &scope)?;
                validate_expression_expected(
                    condition,
                    &Type::Boolean,
                    &scope,
                    &algorithms,
                    self.profile,
                    &self.modules,
                    algorithm_span(algorithm),
                )?;
            }
            self.validate_block(&algorithm.body, &algorithms, &scope)?;
            self.validate_types_block(&algorithm.body, &algorithms, &scope)?;
        }
        let preamble_scope = global_declarations.clone();
        self.validate_block(&preamble, &algorithms, &preamble_scope)?;
        self.validate_types_block(&preamble, &algorithms, &preamble_scope)?;
        Ok(Program {
            profile: self.profile,
            modules: self.modules,
            main: normalize(&main),
            global_declarations,
            global_initializers,
            preamble,
            algorithms,
        })
    }

    fn algorithm(&mut self) -> Result<Algorithm, Diagnostic> {
        let header = self
            .current()
            .cloned()
            .ok_or_else(|| self.error_here("expected `алг`"))?;
        let span = self.span();
        if !starts_keyword(&header.text, "алг") {
            return Err(self.error_here("expected `алг`"));
        }
        let (name, result, parameters) =
            parse_algorithm_header(header.text["алг".len()..].trim(), span)?;
        self.at += 1;
        let mut declarations = BTreeMap::new();
        for parameter in &parameters {
            declarations.insert(
                normalize(&parameter.name),
                Declaration {
                    ty: parameter.ty.clone(),
                    initial: None,
                    constant: parameter.mode == ParameterMode::Argument,
                },
            );
        }
        if let Some(ty) = result.clone() {
            declarations.insert(
                "знач".to_owned(),
                Declaration {
                    ty,
                    initial: None,
                    constant: false,
                },
            );
        }
        let mut preconditions = Vec::new();
        let mut postconditions = Vec::new();
        while let Some(line) = self.current() {
            if is_keyword(&line.text, "нач") {
                self.at += 1;
                break;
            }
            if starts_keyword(&line.text, "дано") {
                let condition =
                    parse_expression(line.text["дано".len()..].trim(), self.span(), &self.modules)?;
                preconditions.push(condition);
                self.at += 1;
                continue;
            }
            if starts_keyword(&line.text, "надо") {
                let condition =
                    parse_expression(line.text["надо".len()..].trim(), self.span(), &self.modules)?;
                postconditions.push(condition);
                self.at += 1;
                continue;
            }
            if let Some(decls) = parse_declaration(&line.text, span, &self.modules)? {
                for (name, declaration) in decls {
                    let key = normalize(&name);
                    if declarations.insert(key.clone(), declaration).is_some() {
                        return Err(self.error_here(&format!("duplicate declaration `{key}`")));
                    }
                }
                self.at += 1;
            } else {
                return Err(
                    self.error_here("expected a declaration or `нач` before the algorithm body")
                );
            }
        }
        while let Some(line) = self.current() {
            if let Some(decls) = parse_declaration(&line.text, self.span(), &self.modules)? {
                for (name, declaration) in decls {
                    let key = normalize(&name);
                    if declarations.insert(key.clone(), declaration).is_some() {
                        return Err(self.error_here(&format!("duplicate declaration `{key}`")));
                    }
                }
                self.at += 1;
            } else {
                break;
            }
        }
        let body = self.block_until(&["кон"])?;
        let mut body_declarations = BTreeSet::new();
        collect_local_declarations(&body, &mut declarations, &mut body_declarations, span)?;
        self.consume("кон")?;
        Ok(Algorithm {
            name: normalize(&name),
            parameters,
            result,
            preconditions,
            postconditions,
            declarations,
            body_declarations,
            body,
            span,
        })
    }

    fn block_until(&mut self, ends: &[&str]) -> Result<Vec<Stmt>, Diagnostic> {
        let mut body = Vec::new();
        loop {
            let Some(line) = self.current().cloned() else {
                return Err(self.error_here("unexpected end of source"));
            };
            if ends.iter().any(|end| is_terminator(&line.text, end)) {
                return Ok(body);
            }
            if ["кон", "иначе", "все", "кц", "при", "кц_при"]
                .iter()
                .any(|end| is_keyword(&line.text, end))
            {
                return Err(
                    self.error_here(&format!("unexpected block terminator `{}`", line.text))
                );
            }
            body.push(self.statement()?);
        }
    }

    fn block_until_repeat_close(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut body = Vec::new();
        loop {
            let Some(line) = self.current() else {
                return Err(self.error_here("unexpected end of source"));
            };
            if is_keyword(&line.text, "кц") || is_repeat_close(&line.text) {
                return Ok(body);
            }
            body.push(self.statement()?);
        }
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        let line = self
            .current()
            .cloned()
            .ok_or_else(|| self.error_here("expected a statement"))?;
        let span = self.span();
        let text = line.text.trim();
        if let Some(declarations) = parse_declaration(text, span, &self.modules)? {
            self.at += 1;
            return Ok(Stmt::Declare { declarations, span });
        }
        if starts_keyword(text, "если") {
            let header = text["если".len()..].trim();
            let condition_text = strip_suffix_keyword(header, "то").unwrap_or(header);
            let condition = parse_expression(condition_text, span, &self.modules)?;
            self.at += 1;
            if strip_suffix_keyword(header, "то").is_none()
                && self
                    .current()
                    .is_some_and(|line| is_keyword(&line.text, "то"))
            {
                self.at += 1;
            }
            let yes = self.block_until(&["иначе", "все"])?;
            let no = if self
                .current()
                .is_some_and(|line| is_keyword(&line.text, "иначе"))
            {
                self.at += 1;
                self.block_until(&["все"])?
            } else {
                Vec::new()
            };
            self.consume("все")?;
            return Ok(Stmt::If {
                condition,
                yes,
                no,
                span,
            });
        }
        if starts_keyword(text, "выбор") {
            let value_text = text["выбор".len()..].trim();
            if !value_text.is_empty() {
                return Err(self.error_here("`выбор` takes condition branches and no selector"));
            }
            let value = None;
            self.at += 1;
            let mut branches = Vec::new();
            let mut otherwise = Vec::new();
            while self
                .current()
                .is_some_and(|line| starts_keyword(&line.text, "при"))
            {
                let label = self.current().unwrap().text["при".len()..]
                    .trim()
                    .to_owned();
                if split_top_level(&label, ',').len() != 1 {
                    return Err(self.error_here("each `при` branch takes one condition"));
                }
                let labels = vec![parse_expression(&label, span, &self.modules)?];
                self.at += 1;
                branches.push((labels, self.block_until(&["при", "иначе", "все"])?));
            }
            if self
                .current()
                .is_some_and(|line| is_keyword(&line.text, "иначе"))
            {
                self.at += 1;
                otherwise = self.block_until(&["все"])?;
            }
            self.consume("все")?;
            return Ok(Stmt::Choose {
                value,
                branches,
                otherwise,
                span,
            });
        }
        if starts_keyword(text, "нц пока") {
            let condition = parse_expression(text["нц пока".len()..].trim(), span, &self.modules)?;
            self.at += 1;
            let body = self.block_until(&["кц"])?;
            self.consume("кц")?;
            return Ok(Stmt::While {
                condition,
                body,
                span,
            });
        }
        if starts_keyword(text, "нц для") {
            let rest = text["нц для".len()..].trim();
            let (name, rest) = split_keyword(rest, "от")
                .ok_or_else(|| self.error_here("expected `от` in counted loop"))?;
            let (start, rest) = split_keyword(rest, "до")
                .ok_or_else(|| self.error_here("expected `до` in counted loop"))?;
            let (end, step) =
                split_keyword(rest, "шаг").map_or((rest, None), |(end, step)| (end, Some(step)));
            let name = normalize(name.trim());
            let start = parse_expression(start.trim(), span, &self.modules)?;
            let end = parse_expression(end.trim(), span, &self.modules)?;
            let step = match step {
                Some(step) => parse_expression(step.trim(), span, &self.modules)?,
                None => Expr::Literal(Value::Integer(1)),
            };
            self.at += 1;
            let body = self.block_until(&["кц"])?;
            self.consume("кц")?;
            return Ok(Stmt::For {
                name,
                start,
                end,
                step,
                body,
                span,
            });
        }
        if starts_keyword(text, "нц") {
            let suffix = text["нц".len()..].trim();
            self.at += 1;
            if suffix.is_empty() {
                let body = self.block_until_repeat_close()?;
                if self
                    .current()
                    .is_some_and(|line| is_repeat_close(&line.text))
                {
                    let close = self.current().unwrap().text.clone();
                    let condition = parse_expression(
                        repeat_close_condition(&close).unwrap_or_default(),
                        self.span(),
                        &self.modules,
                    )?;
                    self.at += 1;
                    return Ok(Stmt::RepeatUntil {
                        body,
                        condition,
                        span,
                    });
                }
                self.consume("кц")?;
                return Ok(Stmt::Loop { body, span });
            }
            let count = parse_expression(
                strip_suffix_keyword(suffix, "раз").unwrap_or(suffix),
                span,
                &self.modules,
            )?;
            let body = self.block_until(&["кц"])?;
            self.consume("кц")?;
            return Ok(Stmt::Repeat { count, body, span });
        }
        if is_repeat_close(text) {
            return Err(self.error_here("`кц при` must close a `нц` repeat-until loop"));
        }
        if is_keyword(text, "нс") {
            self.at += 1;
            return Ok(Stmt::Newline { span });
        }
        if is_keyword(text, "выход") {
            self.at += 1;
            return Ok(Stmt::Exit { span });
        }
        if is_keyword(text, "стоп") {
            self.at += 1;
            return Ok(Stmt::Stop { span });
        }
        if is_keyword(text, "пауза") {
            self.at += 1;
            return Ok(Stmt::Pause { span });
        }
        for keyword in ["утв", "дано", "надо"] {
            if starts_keyword(text, keyword) {
                let condition =
                    parse_expression(text[keyword.len()..].trim(), span, &self.modules)?;
                self.at += 1;
                return Ok(Stmt::Assert { condition, span });
            }
        }
        if starts_keyword(text, "ввод") {
            let source = text["ввод".len()..].trim();
            let targets = split_top_level(source, ',')
                .into_iter()
                .map(|part| {
                    if is_keyword(part.trim(), "нс") {
                        Ok(InputPart::Newline)
                    } else {
                        parse_expression(part.trim(), span, &self.modules).map(InputPart::Target)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            self.at += 1;
            return Ok(Stmt::Input { targets, span });
        }
        if starts_keyword(text, "вывод") {
            let source = text["вывод".len()..].trim();
            let values = if source.is_empty() {
                Vec::new()
            } else {
                split_top_level(source, ',')
                    .into_iter()
                    .map(|part| {
                        if is_keyword(part.trim(), "нс") {
                            Ok(OutputPart::Newline)
                        } else {
                            parse_expression(part.trim(), span, &self.modules)
                                .map(OutputPart::Value)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
            };
            self.at += 1;
            return Ok(Stmt::Output { values, span });
        }
        if let Some((left, right)) = split_assignment(text) {
            let target = parse_expression(left.trim(), span, &self.modules)?;
            let value = parse_expression(right.trim(), span, &self.modules)?;
            self.at += 1;
            return Ok(Stmt::Assign {
                target,
                value,
                span,
            });
        }
        if let Some(module_call) = parse_module_statement(text, span, &self.modules)? {
            self.at += 1;
            return Ok(Stmt::ModuleCall {
                module: module_call.0,
                name: module_call.1,
                arguments: module_call.2,
                span,
            });
        }
        if let Some(file) = parse_file_statement(text, span, &self.modules)? {
            self.at += 1;
            return Ok(Stmt::File {
                request: file,
                span,
            });
        }
        if let Some((name, arguments)) = parse_call_statement(text, span, &self.modules)? {
            self.at += 1;
            return Ok(Stmt::Call {
                name,
                arguments,
                span,
            });
        }
        Err(self.error_here(&format!("unsupported statement `{text}`")))
    }

    fn validate_block(
        &self,
        body: &[Stmt],
        algorithms: &BTreeMap<String, Algorithm>,
        declarations: &BTreeMap<String, Declaration>,
    ) -> Result<(), Diagnostic> {
        let mut visible = declarations.clone();
        for statement in body {
            match statement {
                Stmt::Call {
                    name,
                    arguments,
                    span,
                } => {
                    if !algorithms.contains_key(&normalize(name))
                        && !is_builtin(name, self.profile, &self.modules)
                        && !(self.modules.contains(&Module::Files) && is_file_builtin(name))
                    {
                        return Err(error(
                            *span,
                            format!("unknown algorithm or procedure `{name}`"),
                        ));
                    }
                    if is_string_output_procedure(name) {
                        let (expected, result_index) = match normalize(name).as_str() {
                            "вставить" => (3, 1),
                            "заменить" => (4, 0),
                            "удалить" => (3, 0),
                            _ => unreachable!(),
                        };
                        if arguments.len() != expected {
                            return Err(error(
                                *span,
                                format!("`{name}` expects {expected} argument(s)"),
                            ));
                        }
                        for (index, argument) in arguments.iter().enumerate() {
                            if index == result_index {
                                validate_lvalue(argument, &visible)?;
                                if let Expr::Variable(name, _) = argument {
                                    if visible.get(name).is_some_and(|symbol| symbol.constant) {
                                        return Err(error(
                                            *span,
                                            format!("cannot pass read-only `{name}` as an output"),
                                        ));
                                    }
                                }
                            } else {
                                validate_expr(argument, &visible)?;
                            }
                        }
                    } else if let Some(callee) = algorithms.get(&normalize(name)) {
                        if arguments.len() != callee.parameters.len() {
                            return Err(error(
                                *span,
                                format!(
                                    "algorithm `{name}` expects {} argument(s)",
                                    callee.parameters.len()
                                ),
                            ));
                        }
                        for (parameter, argument) in callee.parameters.iter().zip(arguments) {
                            if parameter.mode == ParameterMode::Argument {
                                validate_expr(argument, &visible)?;
                            } else {
                                validate_lvalue(argument, &visible)?;
                            }
                        }
                    } else {
                        for arg in arguments {
                            validate_expr(arg, &visible)?;
                        }
                    }
                }
                Stmt::Declare {
                    declarations: declared,
                    ..
                } => {
                    for (name, declaration) in declared {
                        if visible.contains_key(name) {
                            return Err(error(
                                statement_span(statement),
                                format!("duplicate declaration `{name}`"),
                            ));
                        }
                        if let Some(initial) = &declaration.initial {
                            validate_expr(initial, &visible)?;
                        }
                        visible.insert(name.clone(), declaration.clone());
                    }
                }
                Stmt::Assign { target, value, .. } => {
                    validate_lvalue(target, &visible)?;
                    validate_expr(value, &visible)?;
                }
                Stmt::Input { targets, .. } => {
                    for target in targets {
                        if let InputPart::Target(target) = target {
                            validate_lvalue(target, &visible)?;
                        }
                    }
                }
                Stmt::Output { values, .. } => {
                    for value in values {
                        if let OutputPart::Value(expr) = value {
                            validate_expr(expr, &visible)?;
                        }
                    }
                }
                Stmt::If {
                    condition, yes, no, ..
                } => {
                    validate_expr(condition, &visible)?;
                    self.validate_block(yes, algorithms, &visible)?;
                    self.validate_block(no, algorithms, &visible)?;
                }
                Stmt::Choose {
                    value,
                    branches,
                    otherwise,
                    ..
                } => {
                    if let Some(value) = value {
                        validate_expr(value, &visible)?;
                    }
                    for (labels, body) in branches {
                        for label in labels {
                            validate_expr(label, &visible)?;
                        }
                        self.validate_block(body, algorithms, &visible)?;
                    }
                    self.validate_block(otherwise, algorithms, &visible)?;
                }
                Stmt::While {
                    condition, body, ..
                }
                | Stmt::RepeatUntil {
                    body, condition, ..
                } => {
                    validate_expr(condition, &visible)?;
                    self.validate_block(body, algorithms, &visible)?;
                }
                Stmt::For {
                    name,
                    start,
                    end,
                    step,
                    body,
                    span,
                } => {
                    let Some(declaration) = visible.get(name) else {
                        return Err(error(*span, format!("undefined loop variable `{name}`")));
                    };
                    if declaration.ty != Type::Integer {
                        return Err(error(*span, "counted-loop variable must have integer type"));
                    }
                    if declaration.constant {
                        return Err(error(*span, "counted-loop variable is read-only"));
                    }
                    validate_expr(start, &visible)?;
                    validate_expr(end, &visible)?;
                    validate_expr(step, &visible)?;
                    self.validate_block(body, algorithms, &visible)?;
                }
                Stmt::Repeat { count, body, .. } => {
                    validate_expr(count, &visible)?;
                    self.validate_block(body, algorithms, &visible)?;
                }
                Stmt::Loop { body, .. } => self.validate_block(body, algorithms, &visible)?,
                Stmt::ModuleCall {
                    module,
                    name,
                    arguments,
                    span,
                    ..
                } => {
                    if !self.modules.contains(module) {
                        return Err(error(
                            *span,
                            format!(
                                "module `{}` must be enabled with `использовать`",
                                module.canonical_name()
                            ),
                        ));
                    }
                    if let Some(expected) = module_expected_arity(*module, name) {
                        if arguments.len() != expected {
                            return Err(error(
                                *span,
                                format!("`{name}` expects {expected} argument(s)"),
                            ));
                        }
                    }
                    for (arg, mode) in
                        arguments
                            .iter()
                            .zip(module_argument_modes(*module, name, arguments.len()))
                    {
                        if mode == ModuleArgumentMode::Argument {
                            validate_expr(arg, &visible)?;
                        } else {
                            validate_lvalue(arg, &visible)?;
                        }
                    }
                }
                Stmt::File { request, span } => {
                    if !self.modules.contains(&Module::Files) {
                        return Err(error(
                            *span,
                            "file commands require the Файлы module or high-grade profile",
                        ));
                    }
                    for arg in file_exprs(request) {
                        validate_expr(arg, &visible)?;
                    }
                }
                Stmt::Assert { condition, .. } => validate_expr(condition, &visible)?,
                Stmt::Newline { .. }
                | Stmt::Exit { .. }
                | Stmt::Stop { .. }
                | Stmt::Pause { .. } => {}
            }
        }
        Ok(())
    }

    fn validate_types_block(
        &self,
        body: &[Stmt],
        algorithms: &BTreeMap<String, Algorithm>,
        declarations: &BTreeMap<String, Declaration>,
    ) -> Result<(), Diagnostic> {
        let mut visible = declarations.clone();
        for statement in body {
            let span = statement_span(statement);
            match statement {
                Stmt::Declare {
                    declarations: declared,
                    ..
                } => {
                    for (name, declaration) in declared {
                        if let Some(initial) = &declaration.initial {
                            validate_assignment_type(
                                &declaration.ty,
                                initial,
                                &visible,
                                algorithms,
                                self.profile,
                                &self.modules,
                                span,
                            )?;
                        }
                        visible.insert(name.clone(), declaration.clone());
                    }
                }
                Stmt::Assign { target, value, .. } => {
                    validate_mutable_target(target, &visible, span)?;
                    let target_type = infer_target_type(
                        target,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                    )?;
                    let value_type = infer_expression_type(
                        value,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                    )?;
                    if let (Some(expected), Some(actual)) = (target_type, value_type) {
                        require_assignable(&expected, &actual, span)?;
                    }
                }
                Stmt::Input { targets, .. } => {
                    for target in targets {
                        if let InputPart::Target(target) = target {
                            validate_mutable_target(target, &visible, span)?;
                            infer_target_type(
                                target,
                                &visible,
                                algorithms,
                                self.profile,
                                &self.modules,
                                span,
                            )?;
                        }
                    }
                }
                Stmt::Output { values, .. } => {
                    for value in values {
                        if let OutputPart::Value(expression) = value {
                            infer_expression_type(
                                expression,
                                &visible,
                                algorithms,
                                self.profile,
                                &self.modules,
                                span,
                            )?;
                        }
                    }
                }
                Stmt::If {
                    condition, yes, no, ..
                } => {
                    validate_expression_expected(
                        condition,
                        &Type::Boolean,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                    )?;
                    self.validate_types_block(yes, algorithms, &visible)?;
                    self.validate_types_block(no, algorithms, &visible)?;
                }
                Stmt::Choose {
                    branches,
                    otherwise,
                    ..
                } => {
                    for (conditions, branch) in branches {
                        for condition in conditions {
                            validate_expression_expected(
                                condition,
                                &Type::Boolean,
                                &visible,
                                algorithms,
                                self.profile,
                                &self.modules,
                                span,
                            )?;
                        }
                        self.validate_types_block(branch, algorithms, &visible)?;
                    }
                    self.validate_types_block(otherwise, algorithms, &visible)?;
                }
                Stmt::While {
                    condition, body, ..
                }
                | Stmt::RepeatUntil {
                    body, condition, ..
                } => {
                    validate_expression_expected(
                        condition,
                        &Type::Boolean,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                    )?;
                    self.validate_types_block(body, algorithms, &visible)?;
                }
                Stmt::For {
                    start,
                    end,
                    step,
                    body,
                    ..
                } => {
                    for expression in [start, end, step] {
                        validate_expression_expected(
                            expression,
                            &Type::Integer,
                            &visible,
                            algorithms,
                            self.profile,
                            &self.modules,
                            span,
                        )?;
                    }
                    self.validate_types_block(body, algorithms, &visible)?;
                }
                Stmt::Repeat { count, body, .. } => {
                    validate_expression_expected(
                        count,
                        &Type::Integer,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                    )?;
                    self.validate_types_block(body, algorithms, &visible)?;
                }
                Stmt::Loop { body, .. } => {
                    self.validate_types_block(body, algorithms, &visible)?;
                }
                Stmt::Call {
                    name, arguments, ..
                } => {
                    validate_call_types(
                        name,
                        arguments,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                        false,
                    )?;
                }
                Stmt::ModuleCall {
                    module,
                    name,
                    arguments,
                    span: module_span,
                } => {
                    validate_module_call_types(
                        *module,
                        name,
                        arguments,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        *module_span,
                        false,
                    )?;
                }
                Stmt::File { request, .. } => {
                    let FileExpr::Call(name, arguments) = request;
                    validate_builtin_call_types(
                        name,
                        arguments,
                        &visible,
                        algorithms,
                        self.profile,
                        &self.modules,
                        span,
                        false,
                    )?;
                }
                Stmt::Assert { condition, .. } => validate_expression_expected(
                    condition,
                    &Type::Boolean,
                    &visible,
                    algorithms,
                    self.profile,
                    &self.modules,
                    span,
                )?,
                Stmt::Newline { .. }
                | Stmt::Exit { .. }
                | Stmt::Stop { .. }
                | Stmt::Pause { .. } => {}
            }
        }
        Ok(())
    }

    fn consume(&mut self, keyword: &str) -> Result<(), Diagnostic> {
        if self
            .current()
            .is_some_and(|line| is_keyword(&line.text, keyword))
        {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error_here(&format!("expected `{keyword}`")))
        }
    }

    fn current(&self) -> Option<&SourceLine> {
        self.lines.get(self.at)
    }
    fn span(&self) -> Span {
        Span {
            line: self
                .current()
                .map_or(self.lines.last().map_or(1, |l| l.number + 1), |line| {
                    line.number
                }),
            column: 1,
        }
    }
    fn error_here(&self, message: &str) -> Diagnostic {
        error(self.span(), message)
    }
    fn error_at(&self, span: Span, message: &str) -> Diagnostic {
        error(span, message)
    }
}

fn join_implicit_pri_lines(lines: Vec<SourceLine>) -> Vec<SourceLine> {
    let mut joined = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        if is_keyword(&line.text, "кц")
            && lines
                .get(index + 1)
                .is_some_and(|next| starts_keyword(&next.text, "при"))
        {
            let next = &lines[index + 1];
            let mut condition = next.text["при".len()..].trim().to_owned();
            index += 2;
            if condition.is_empty()
                && lines.get(index).is_some_and(|next| {
                    !["кон", "все", "иначе", "кц", "при", "то"]
                        .iter()
                        .any(|keyword| is_keyword(&next.text, keyword))
                })
            {
                condition = lines[index].text.clone();
                index += 1;
            }
            let text = if condition.is_empty() {
                "кц при".to_owned()
            } else {
                format!("кц при {condition}")
            };
            joined.push(SourceLine {
                number: line.number,
                text,
            });
        } else if is_keyword(&line.text, "при")
            && lines.get(index + 1).is_some_and(|next| {
                !["кон", "все", "иначе", "кц", "при", "то"]
                    .iter()
                    .any(|keyword| is_keyword(&next.text, keyword))
            })
        {
            let next = &lines[index + 1];
            joined.push(SourceLine {
                number: line.number,
                text: format!("при {}", next.text),
            });
            index += 2;
        } else {
            joined.push(line.clone());
            index += 1;
        }
    }
    joined
}

fn split_source_line(line: &str) -> Vec<String> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for ch in line.chars() {
        if let Some(delimiter) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '|' => break,
            ';' => {
                output.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    output.push(current);
    output
}

fn split_implicit_layout(source: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut word_start = None;
    for (byte, ch) in source.char_indices() {
        if let Some(delimiter) = quote {
            if word_start.take().is_some() {
                words.push((byte, byte, String::new()));
            }
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            if let Some(start) = word_start.take() {
                words.push((start, byte, normalize(&source[start..byte])));
            }
            quote = Some(ch);
            continue;
        }
        if is_identifier_continue(ch) {
            word_start.get_or_insert(byte);
        } else if let Some(start) = word_start.take() {
            words.push((start, byte, normalize(&source[start..byte])));
        }
    }
    if let Some(start) = word_start {
        words.push((start, source.len(), normalize(&source[start..])));
    }
    words.retain(|(_, _, word)| !word.is_empty());

    let mut boundaries = vec![0, source.len()];
    for (index, (start, end, word)) in words.iter().enumerate() {
        if matches!(
            word.as_str(),
            "все" | "кц" | "кц_при" | "то" | "иначе" | "при"
        ) && *start > 0
        {
            boundaries.push(*start);
        }
        if matches!(word.as_str(), "нач" | "выбор" | "раз" | "то" | "иначе") && *end < source.len()
        {
            boundaries.push(*end);
        }
        if word == "нц" && *end < source.len() {
            let next = words.get(index + 1).map(|(_, _, next)| next.as_str());
            let first_loop_boundary = words[index + 1..]
                .iter()
                .map(|(_, _, candidate)| candidate.as_str())
                .find(|candidate| {
                    matches!(
                        *candidate,
                        "раз"
                            | "нц"
                            | "кц"
                            | "кц_при"
                            | "все"
                            | "то"
                            | "иначе"
                            | "при"
                            | "выбор"
                            | "нач"
                    )
                });
            let is_counted_loop = first_loop_boundary == Some("раз");
            if !matches!(next, Some("пока" | "для")) && !is_counted_loop {
                boundaries.push(*end);
            }
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
        .windows(2)
        .map(|pair| source[pair[0]..pair[1]].trim().to_owned())
        .filter(|part| !part.is_empty())
        .collect()
}

fn parse_algorithm_header(
    source: &str,
    span: Span,
) -> Result<(String, Option<Type>, Vec<Parameter>), Diagnostic> {
    let (signature, result) = if let Some((ty, rest)) = take_type_prefix(source) {
        (rest.trim(), Some(ty))
    } else {
        (source.trim(), None)
    };
    if signature.is_empty() {
        return Ok(("main".to_owned(), result, Vec::new()));
    }
    let (name, parameter_text) = if let Some(open) = signature.find('(') {
        let close = signature
            .rfind(')')
            .ok_or_else(|| error(span, "expected `)` in algorithm header"))?;
        if close < open {
            return Err(error(span, "malformed algorithm parameter list"));
        }
        (&signature[..open], Some(&signature[open + 1..close]))
    } else {
        (signature, None)
    };
    let name = normalize(name.trim());
    if !valid_name(&name) {
        return Err(error(span, "invalid algorithm name"));
    }
    let mut parameters = Vec::new();
    if let Some(parameter_text) = parameter_text {
        for item in split_top_level(parameter_text, ',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let (mode, typed) = if starts_keyword(item, "аргрез") {
                (ParameterMode::ArgumentResult, item["аргрез".len()..].trim())
            } else if starts_keyword(item, "арг рез") {
                (
                    ParameterMode::ArgumentResult,
                    item["арг рез".len()..].trim(),
                )
            } else if starts_keyword(item, "арг") {
                (ParameterMode::Argument, item["арг".len()..].trim())
            } else if starts_keyword(item, "рез") {
                (ParameterMode::Result, item["рез".len()..].trim())
            } else {
                (ParameterMode::Argument, item)
            };
            let (ty, names) = take_type_prefix(typed)
                .ok_or_else(|| error(span, format!("parameter `{item}` needs a declared type")))?;
            for name in split_names(names) {
                if !valid_name(name) {
                    return Err(error(span, format!("invalid parameter name `{name}`")));
                }
                parameters.push(Parameter {
                    name: normalize(name),
                    ty: ty.clone(),
                    mode,
                });
            }
        }
    }
    Ok((name, result, parameters))
}

fn parse_declaration(
    line: &str,
    span: Span,
    modules: &[Module],
) -> Result<Option<Vec<(String, Declaration)>>, Diagnostic> {
    let mut source = line.trim();
    let constant = if starts_keyword(source, "конст") {
        source = source["конст".len()..].trim();
        true
    } else {
        false
    };
    let Some((ty, names)) = take_type_prefix(source) else {
        return Ok(None);
    };
    let names = if starts_keyword(names, "таб") {
        names["таб".len()..].trim()
    } else {
        names
    };
    let mut result = Vec::new();
    for item in split_top_level(names, ',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (name_with_shape, initial) = match split_declaration_initializer(item) {
            Some((name, value)) => (
                name.trim(),
                Some(parse_expression(value.trim(), span, modules)?),
            ),
            None => (item, None),
        };
        let (name, dims) =
            if let Some(open) = name_with_shape.find('[') {
                if !name_with_shape.ends_with(']') {
                    return Err(error(span, "malformed table bounds"));
                }
                let bounds = &name_with_shape[open + 1..name_with_shape.len() - 1];
                let mut dims = Vec::new();
                for bound in bounds.split(',') {
                    let (lo, hi) = bound
                        .split_once(':')
                        .ok_or_else(|| error(span, "table bounds must use `lower:upper`"))?;
                    let lo = lo.trim().parse::<i32>().map_err(|_| {
                        error(span, "table lower bound must be an integer constant")
                    })?;
                    let hi = hi.trim().parse::<i32>().map_err(|_| {
                        error(span, "table upper bound must be an integer constant")
                    })?;
                    if hi < lo {
                        return Err(error(span, "table upper bound is below its lower bound"));
                    }
                    dims.push((lo, hi));
                }
                if !(1..=3).contains(&dims.len()) {
                    return Err(error(span, "Kumir tables have one to three dimensions"));
                }
                (name_with_shape[..open].trim(), Some(dims))
            } else {
                (name_with_shape.trim(), None)
            };
        if !valid_name(name) {
            return Err(error(span, format!("invalid identifier `{name}`")));
        }
        let ty = match (&ty, dims) {
            (Type::Table { element, .. }, Some(bounds)) => Type::Table {
                element: element.clone(),
                bounds,
            },
            (Type::Table { element, .. }, None) => {
                let _ = element;
                return Err(error(span, "table declaration needs index bounds"));
            }
            (_, Some(bounds)) => Type::Table {
                element: Box::new(ty.clone()),
                bounds,
            },
            (_, None) => ty.clone(),
        };
        if let Type::Table { bounds, .. } = &ty {
            if table_element_count(bounds).is_none_or(|count| count > MAX_TABLE_ELEMENTS) {
                return Err(error(
                    span,
                    format!("table declaration exceeds the {MAX_TABLE_ELEMENTS}-element limit"),
                ));
            }
        }
        result.push((
            normalize(name),
            Declaration {
                ty,
                initial,
                constant,
            },
        ));
    }
    Ok(Some(result))
}

fn collect_local_declarations(
    body: &[Stmt],
    declarations: &mut BTreeMap<String, Declaration>,
    body_declarations: &mut BTreeSet<String>,
    span: Span,
) -> Result<(), Diagnostic> {
    for statement in body {
        match statement {
            Stmt::Declare {
                declarations: items,
                ..
            } => {
                for (name, declaration) in items {
                    let mut symbol = declaration.clone();
                    symbol.initial = None;
                    if declarations.insert(name.clone(), symbol).is_some() {
                        return Err(error(span, format!("duplicate declaration `{name}`")));
                    }
                    body_declarations.insert(name.clone());
                }
            }
            Stmt::If { yes, no, .. } => {
                collect_local_declarations(yes, declarations, body_declarations, span)?;
                collect_local_declarations(no, declarations, body_declarations, span)?;
            }
            Stmt::Choose {
                branches,
                otherwise,
                ..
            } => {
                for (_, nested) in branches {
                    collect_local_declarations(nested, declarations, body_declarations, span)?;
                }
                collect_local_declarations(otherwise, declarations, body_declarations, span)?;
            }
            Stmt::While { body, .. }
            | Stmt::For { body, .. }
            | Stmt::Repeat { body, .. }
            | Stmt::RepeatUntil { body, .. }
            | Stmt::Loop { body, .. } => {
                collect_local_declarations(body, declarations, body_declarations, span)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn take_type_prefix(source: &str) -> Option<(Type, &str)> {
    let source = source.trim_start();
    let word_end = source.find(char::is_whitespace).unwrap_or(source.len());
    let word = normalize(&source[..word_end]);
    let scalar = match word.as_str() {
        "цел" | "целтаб" => Type::Integer,
        "вещ" | "вещтаб" => Type::Real,
        "лог" | "логтаб" => Type::Boolean,
        "сим" | "симтаб" => Type::Character,
        "лит" | "литтаб" => Type::String,
        "компл" => Type::Complex,
        "цвет" => Type::Color,
        "сканкод" => Type::KeyCode,
        "файл" => Type::File,
        _ => return None,
    };
    let attached_table = word.ends_with("таб");
    let rest = source[word_end..].trim_start();
    if attached_table {
        return Some((
            Type::Table {
                element: Box::new(scalar),
                bounds: Vec::new(),
            },
            rest,
        ));
    }
    if starts_keyword(rest, "таб") {
        return Some((
            Type::Table {
                element: Box::new(scalar),
                bounds: Vec::new(),
            },
            rest["таб".len()..].trim_start(),
        ));
    }
    Some((scalar, rest))
}

fn valid_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch == '_' || ch.is_whitespace())
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
}

fn split_assignment(source: &str) -> Option<(&str, &str)> {
    let mut quote = None;
    let mut depth = 0usize;
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let (byte, ch) = chars[i];
        if let Some(q) = quote {
            if ch == q && (i == 0 || chars[i - 1].1 != '\\') {
                quote = None;
            }
        } else {
            match ch {
                '"' | '\'' => quote = Some(ch),
                '(' | '[' => depth += 1,
                ')' | ']' => depth = depth.saturating_sub(1),
                ':' if depth == 0 && chars.get(i + 1).is_some_and(|(_, next)| *next == '=') => {
                    return Some((&source[..byte], &source[byte + 2..]));
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

fn split_declaration_initializer(source: &str) -> Option<(&str, &str)> {
    if let Some(found) = split_assignment(source) {
        return Some(found);
    }
    let mut quote = None;
    let mut depth = 0usize;
    for (byte, ch) in source.char_indices() {
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => return Some((&source[..byte], &source[byte + 1..])),
            _ => {}
        }
    }
    None
}

fn parse_expression_list(
    source: &str,
    span: Span,
    modules: &[Module],
) -> Result<Vec<Expr>, Diagnostic> {
    let parts = split_top_level(source, ',');
    if parts.is_empty() || parts.iter().any(|part| part.trim().is_empty()) {
        return Err(error(span, "expected one or more values"));
    }
    parts
        .into_iter()
        .map(|part| parse_expression(part.trim(), span, modules))
        .collect()
}

fn parse_expression(source: &str, span: Span, modules: &[Module]) -> Result<Expr, Diagnostic> {
    let trimmed = source.trim();
    if let Some((name, args)) = parse_named_call(trimmed, span, modules)? {
        return Ok(Expr::Call(name, args));
    }
    if let Some(name) = parse_zero_argument_call(trimmed, modules) {
        return Ok(Expr::Call(name, Vec::new()));
    }
    if let Some((module, name)) = match_module_query(trimmed, modules) {
        return Ok(Expr::ModuleCall(module, name.to_owned(), Vec::new()));
    }
    let tokens = lex_expression(trimmed, span, modules)?;
    let mut parser = ExpressionParser {
        tokens,
        at: 0,
        base: span,
    };
    let expr = parser.logical_or()?;
    if !matches!(parser.peek(), TokenKind::End) {
        return Err(parser.error_here("unexpected token in expression"));
    }
    Ok(expr)
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    column: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Literal(Value),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Power,
    IntegerDivide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    End,
}

fn lex_expression(source: &str, span: Span, modules: &[Module]) -> Result<Vec<Token>, Diagnostic> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut at = 0usize;
    while at < chars.len() {
        let ch = chars[at];
        if ch.is_whitespace() {
            at += 1;
            continue;
        }
        let column = source_column(source, at);
        if ch == '@' {
            let remaining = chars[at..].iter().collect::<String>();
            let Some((module, command)) = match_prefixed_module_name(&remaining, modules) else {
                return Err(error(
                    Span { column, ..span },
                    "unknown or unavailable `@` module function",
                ));
            };
            at += command.chars().count();
            let mut after_name = at;
            while after_name < chars.len() && chars[after_name].is_whitespace() {
                after_name += 1;
            }
            let has_arguments = chars.get(after_name) == Some(&'(');
            if !has_arguments
                && !module_zero_argument_names(module)
                    .iter()
                    .any(|candidate| normalize(candidate) == normalize(command))
            {
                return Err(error(
                    Span { column, ..span },
                    format!("`{command}` requires a parenthesized argument list"),
                ));
            }
            tokens.push(Token {
                kind: TokenKind::Identifier(format!(
                    "@{}:{}",
                    module.canonical_name(),
                    normalize(command)
                )),
                column,
            });
            continue;
        }
        if ch == '-'
            && chars
                .get(at + 1..at + 11)
                .is_some_and(|digits| digits.iter().copied().eq("2147483648".chars()))
            && chars.get(at + 11).is_none_or(|next| {
                !is_identifier_continue(*next) && !matches!(*next, '.' | 'е' | 'Е')
            })
            && tokens.last().is_none_or(|token| {
                matches!(
                    token.kind,
                    TokenKind::Plus
                        | TokenKind::Minus
                        | TokenKind::Star
                        | TokenKind::Slash
                        | TokenKind::Power
                        | TokenKind::IntegerDivide
                        | TokenKind::Modulo
                        | TokenKind::Equal
                        | TokenKind::NotEqual
                        | TokenKind::Less
                        | TokenKind::LessEqual
                        | TokenKind::Greater
                        | TokenKind::GreaterEqual
                        | TokenKind::And
                        | TokenKind::Or
                        | TokenKind::Not
                        | TokenKind::LeftParen
                        | TokenKind::LeftBracket
                        | TokenKind::Comma
                        | TokenKind::Colon
                )
            })
        {
            tokens.push(Token {
                kind: TokenKind::Literal(Value::Integer(i32::MIN)),
                column,
            });
            at += 11;
            continue;
        }
        if ch == '"' || ch == '\'' {
            let quote = ch;
            at += 1;
            let mut value = String::new();
            while at < chars.len() && chars[at] != quote {
                if chars[at] == '\\' && at + 1 < chars.len() {
                    at += 1;
                    value.push(match chars[at] {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                } else {
                    value.push(chars[at]);
                }
                at += 1;
            }
            if at == chars.len() {
                return Err(error(Span { column, ..span }, "unterminated text literal"));
            }
            at += 1;
            let v = if quote == '\'' && value.chars().count() == 1 {
                Value::Character(value.chars().next().unwrap())
            } else {
                Value::String(value)
            };
            tokens.push(Token {
                kind: TokenKind::Literal(v),
                column,
            });
            continue;
        }
        if ch == '$' {
            at += 1;
            let start = at;
            while at < chars.len() && chars[at].is_ascii_hexdigit() {
                at += 1;
            }
            if start == at {
                return Err(error(
                    Span { column, ..span },
                    "hexadecimal integer needs digits after `$`",
                ));
            }
            let raw = chars[start..at].iter().collect::<String>();
            let value = i32::from_str_radix(&raw, 16).map_err(|_| {
                error(
                    Span { column, ..span },
                    "integer literal is outside the Kumir 32-bit range",
                )
            })?;
            tokens.push(Token {
                kind: TokenKind::Literal(Value::Integer(value)),
                column,
            });
            continue;
        }
        if ch.is_ascii_digit() {
            let start = at;
            while at < chars.len() && chars[at].is_ascii_digit() {
                at += 1;
            }
            if at < chars.len()
                && chars[at] == '.'
                && chars.get(at + 1).is_some_and(char::is_ascii_digit)
            {
                at += 1;
                while at < chars.len() && chars[at].is_ascii_digit() {
                    at += 1;
                }
            }
            if at < chars.len() && matches!(chars[at], 'e' | 'E' | 'е' | 'Е') {
                at += 1;
                if at < chars.len() && matches!(chars[at], '+' | '-') {
                    at += 1;
                }
                let exp_start = at;
                while at < chars.len() && chars[at].is_ascii_digit() {
                    at += 1;
                }
                if exp_start == at {
                    return Err(error(Span { column, ..span }, "real exponent needs digits"));
                }
            }
            let raw = chars[start..at]
                .iter()
                .collect::<String>()
                .replace(['е', 'Е'], "e");
            let number = if raw.contains('.') || raw.contains('e') {
                Value::Real(
                    raw.parse()
                        .map_err(|_| error(Span { column, ..span }, "invalid real literal"))?,
                )
            } else {
                Value::Integer(raw.parse::<i32>().map_err(|_| {
                    error(
                        Span { column, ..span },
                        "integer literal is outside the Kumir 32-bit range",
                    )
                })?)
            };
            if at < chars.len() && chars[at] == 'i' && modules.contains(&Module::ComplexNumbers) {
                at += 1;
                let imaginary = match &number {
                    Value::Integer(v) => *v as f64,
                    Value::Real(v) => *v,
                    _ => 0.0,
                };
                tokens.push(Token {
                    kind: TokenKind::Literal(Value::Complex(Complex {
                        real: 0.0,
                        imaginary,
                    })),
                    column,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Literal(number),
                    column,
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
            let mut words = vec![chars[start..at].iter().collect::<String>()];
            loop {
                let whitespace = at;
                while at < chars.len() && chars[at].is_whitespace() {
                    at += 1;
                }
                if at == whitespace || at >= chars.len() || !is_identifier_start(chars[at]) {
                    at = whitespace;
                    break;
                }
                let next_start = at;
                at += 1;
                while at < chars.len() && is_identifier_continue(chars[at]) {
                    at += 1;
                }
                let next_word = chars[next_start..at].iter().collect::<String>();
                let normalized_word = normalize(&next_word);
                if normalized_word == "не" && !words.iter().any(|word| normalize(word) == "не")
                {
                    while at < chars.len() && chars[at].is_whitespace() {
                        at += 1;
                    }
                    if at < chars.len() && is_identifier_start(chars[at]) {
                        words.push(next_word);
                        continue;
                    }
                    at = whitespace;
                    break;
                }
                if ["и", "или", "не", "div", "mod"].contains(&normalized_word.as_str()) {
                    at = whitespace;
                    break;
                }
                words.push(next_word);
            }
            let name = normalize(&words.join(" "));
            let call_follows = chars[at..]
                .iter()
                .skip_while(|ch| ch.is_whitespace())
                .next()
                == Some(&'(');
            let kind = match name.as_str() {
                "да" => TokenKind::Literal(Value::Boolean(true)),
                "нет" => TokenKind::Literal(Value::Boolean(false)),
                "истина" => TokenKind::Literal(Value::Boolean(true)),
                "ложь" => TokenKind::Literal(Value::Boolean(false)),
                "и" => TokenKind::And,
                "или" => TokenKind::Or,
                "не" => TokenKind::Not,
                "pi" => TokenKind::Literal(Value::Real(std::f64::consts::PI)),
                "e" => TokenKind::Literal(Value::Real(std::f64::consts::E)),
                "максцел" => TokenKind::Literal(Value::Integer(MAX_INTEGER)),
                "максвещ" => TokenKind::Literal(Value::Real(f64::MAX)),
                "i" if modules.contains(&Module::ComplexNumbers) => {
                    TokenKind::Literal(Value::Complex(Complex {
                        real: 0.0,
                        imaginary: 1.0,
                    }))
                }
                "красный" | "красная" | "красное" | "red" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("red".to_owned()))
                }
                "синий" | "синяя" | "синее" | "blue" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("blue".to_owned()))
                }
                "зеленый" | "зелёный" | "зеленая" | "зелёная" | "green"
                    if modules.contains(&Module::Painter) =>
                {
                    TokenKind::Literal(Value::Color("green".to_owned()))
                }
                "черный" | "чёрный" | "black" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("black".to_owned()))
                }
                "белый" | "белая" | "белое" | "white" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("white".to_owned()))
                }
                "желтый" | "жёлтый" | "yellow" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("yellow".to_owned()))
                }
                "прозрачный" | "прозрачная" | "прозрачное"
                    if modules.contains(&Module::Painter) =>
                {
                    TokenKind::Literal(Value::Color("transparent".to_owned()))
                }
                "серый" | "серая" | "серое" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("gray".to_owned()))
                }
                "фиолетовый" | "фиолетовая" | "фиолетовое"
                    if modules.contains(&Module::Painter) =>
                {
                    TokenKind::Literal(Value::Color("violet".to_owned()))
                }
                "голубой" | "голубая" | "голубое" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("cyan".to_owned()))
                }
                "оранжевый" | "оранжевая" | "оранжевое" if modules.contains(&Module::Painter) => {
                    TokenKind::Literal(Value::Color("orange".to_owned()))
                }
                _ if modules.contains(&Module::Keyboard) && keyboard_key_code(&name).is_some() => {
                    TokenKind::Literal(Value::KeyCode(keyboard_key_code(&name).unwrap()))
                }
                "div" if !call_follows => TokenKind::IntegerDivide,
                "mod" if !call_follows => TokenKind::Modulo,
                _ => {
                    if let Some(module) = modules_for_name(&name, modules).filter(|module| {
                        call_follows
                            || module_zero_argument_names(*module)
                                .iter()
                                .any(|item| normalize(item) == name)
                    }) {
                        TokenKind::Identifier(format!("@{}:{name}", module.canonical_name()))
                    } else {
                        TokenKind::Identifier(name)
                    }
                }
            };
            tokens.push(Token { kind, column });
            continue;
        }
        let next = chars.get(at + 1).copied();
        let (kind, width) = match (ch, next) {
            ('<', Some('=')) => (TokenKind::LessEqual, 2),
            ('>', Some('=')) => (TokenKind::GreaterEqual, 2),
            ('<', Some('>')) | ('!', Some('=')) => (TokenKind::NotEqual, 2),
            ('*', Some('*')) => (TokenKind::Power, 2),
            ('+', _) => (TokenKind::Plus, 1),
            ('-', _) => (TokenKind::Minus, 1),
            ('*' | '×', _) => (TokenKind::Star, 1),
            ('/' | '÷', _) => (TokenKind::Slash, 1),
            ('=', _) => (TokenKind::Equal, 1),
            ('<', _) => (TokenKind::Less, 1),
            ('>', _) => (TokenKind::Greater, 1),
            ('(', _) => (TokenKind::LeftParen, 1),
            (')', _) => (TokenKind::RightParen, 1),
            ('[', _) => (TokenKind::LeftBracket, 1),
            (']', _) => (TokenKind::RightBracket, 1),
            (':', _) => (TokenKind::Colon, 1),
            (',', _) => (TokenKind::Comma, 1),
            _ => {
                return Err(error(
                    Span { column, ..span },
                    format!("unsupported character `{ch}` in expression"),
                ));
            }
        };
        at += width;
        tokens.push(Token { kind, column });
    }
    tokens.push(Token {
        kind: TokenKind::End,
        column: source.chars().count() + 1,
    });
    Ok(tokens)
}

struct ExpressionParser {
    tokens: Vec<Token>,
    at: usize,
    base: Span,
}

impl ExpressionParser {
    fn logical_or(&mut self) -> Result<Expr, Diagnostic> {
        self.binary_chain(Self::logical_and, &[(TokenKind::Or, BinaryOp::Or)])
    }
    fn logical_and(&mut self) -> Result<Expr, Diagnostic> {
        self.binary_chain(Self::comparison, &[(TokenKind::And, BinaryOp::And)])
    }
    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        self.binary_chain(
            Self::additive,
            &[
                (TokenKind::Equal, BinaryOp::Equal),
                (TokenKind::NotEqual, BinaryOp::NotEqual),
                (TokenKind::Less, BinaryOp::Less),
                (TokenKind::LessEqual, BinaryOp::LessEqual),
                (TokenKind::Greater, BinaryOp::Greater),
                (TokenKind::GreaterEqual, BinaryOp::GreaterEqual),
            ],
        )
    }
    fn additive(&mut self) -> Result<Expr, Diagnostic> {
        self.binary_chain(
            Self::multiplicative,
            &[
                (TokenKind::Plus, BinaryOp::Add),
                (TokenKind::Minus, BinaryOp::Subtract),
            ],
        )
    }
    fn multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        self.binary_chain(
            Self::unary,
            &[
                (TokenKind::Star, BinaryOp::Multiply),
                (TokenKind::Slash, BinaryOp::Divide),
                (TokenKind::IntegerDivide, BinaryOp::IntegerDivide),
                (TokenKind::Modulo, BinaryOp::Modulo),
            ],
        )
    }
    fn binary_chain(
        &mut self,
        next: fn(&mut Self) -> Result<Expr, Diagnostic>,
        ops: &[(TokenKind, BinaryOp)],
    ) -> Result<Expr, Diagnostic> {
        let mut expr = next(self)?;
        loop {
            let Some((_, op)) = ops.iter().find(|(token, _)| {
                std::mem::discriminant(token) == std::mem::discriminant(self.peek())
            }) else {
                break;
            };
            let op = *op;
            self.at += 1;
            let rhs = next(self)?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        let op = match self.peek() {
            TokenKind::Minus => Some(UnaryOp::Negate),
            TokenKind::Plus => Some(UnaryOp::Positive),
            TokenKind::Not => Some(UnaryOp::Not),
            _ => None,
        };
        if let Some(op) = op {
            self.at += 1;
            return Ok(Expr::Unary(op, Box::new(self.unary()?)));
        }
        self.power()
    }
    fn power(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.primary()?;
        if matches!(self.peek(), TokenKind::Power) {
            self.at += 1;
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), BinaryOp::Power, Box::new(rhs));
        }
        Ok(expr)
    }
    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.tokens[self.at].clone();
        let mut expr = match token.kind {
            TokenKind::Literal(value) => {
                self.at += 1;
                Expr::Literal(value)
            }
            TokenKind::Identifier(name) => {
                self.at += 1;
                if matches!(self.peek(), TokenKind::LeftParen) {
                    self.at += 1;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), TokenKind::RightParen) {
                        loop {
                            args.push(self.logical_or()?);
                            if !matches!(self.peek(), TokenKind::Comma) {
                                break;
                            }
                            self.at += 1;
                        }
                    }
                    self.expect(TokenKind::RightParen, "expected `)` after arguments")?;
                    if name.starts_with('@') {
                        let (module, command) = split_module_marker(&name);
                        Expr::ModuleCall(module, command, args)
                    } else {
                        Expr::Call(name, args)
                    }
                } else if name.starts_with('@') {
                    let (module, command) = split_module_marker(&name);
                    Expr::ModuleCall(module, command, Vec::new())
                } else if name == "время" {
                    Expr::Call(name, Vec::new())
                } else {
                    Expr::Variable(
                        name.clone(),
                        Span {
                            column: token.column,
                            ..self.base
                        },
                    )
                }
            }
            TokenKind::LeftParen => {
                self.at += 1;
                let inner = self.logical_or()?;
                self.expect(TokenKind::RightParen, "expected `)`")?;
                inner
            }
            _ => {
                return Err(self.error_here(
                    "expected a value, variable, function call, or parenthesized expression",
                ));
            }
        };
        while matches!(self.peek(), TokenKind::LeftBracket) {
            self.at += 1;
            let first = self.logical_or()?;
            if matches!(self.peek(), TokenKind::Colon) {
                self.at += 1;
                let end = self.logical_or()?;
                self.expect(TokenKind::RightBracket, "expected `]` after string slice")?;
                expr = Expr::Slice(Box::new(expr), Box::new(first), Box::new(end));
                continue;
            }
            let mut indices = vec![first];
            while matches!(self.peek(), TokenKind::Comma) {
                self.at += 1;
                indices.push(self.logical_or()?);
            }
            self.expect(TokenKind::RightBracket, "expected `]` after table indices")?;
            expr = Expr::Index(Box::new(expr), indices);
        }
        Ok(expr)
    }
    fn expect(&mut self, token: TokenKind, message: &str) -> Result<(), Diagnostic> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(&token) {
            self.at += 1;
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }
    fn peek(&self) -> &TokenKind {
        &self.tokens[self.at].kind
    }
    fn error_here(&self, message: &str) -> Diagnostic {
        error(
            Span {
                column: self.tokens[self.at].column,
                ..self.base
            },
            message,
        )
    }
}

fn modules_for_name(name: &str, enabled: &[Module]) -> Option<Module> {
    let name = normalize(name);
    enabled.iter().copied().find(|module| {
        module_expression_names(*module)
            .iter()
            .any(|command| normalize(command) == name)
    })
}

fn match_prefixed_module_name(source: &str, enabled: &[Module]) -> Option<(Module, &'static str)> {
    let mut names = enabled
        .iter()
        .flat_map(|module| {
            module_expression_names(*module)
                .iter()
                .filter(|name| name.starts_with('@'))
                .map(move |name| (*module, *name))
        })
        .collect::<Vec<_>>();
    names.sort_by_key(|(_, name)| std::cmp::Reverse(name.chars().count()));
    names
        .into_iter()
        .find(|(_, name)| starts_prefixed_name(source, name))
}

fn starts_prefixed_name(source: &str, name: &str) -> bool {
    let source = normalize(source);
    let name = normalize(name);
    source == name
        || source.strip_prefix(&name).is_some_and(|rest| {
            rest.chars().next().is_some_and(|next| {
                next.is_whitespace()
                    || matches!(
                        next,
                        '(' | ')' | '[' | ']' | ',' | ':' | '=' | '<' | '>' | '+' | '-' | '*' | '/'
                    )
            })
        })
}

fn module_expression_names(module: Module) -> &'static [&'static str] {
    match module {
        Module::Robot => &[
            "слева стена",
            "справа стена",
            "сверху стена",
            "снизу стена",
            "слева свободно",
            "справа свободно",
            "сверху свободно",
            "снизу свободно",
            "клетка закрашена",
            "клетка чистая",
            "радиация",
            "температура",
            "@@закрашена",
            "@@метка",
            "@@верхняя буква",
            "@@нижняя буква",
            "@@температура",
            "@@радиация",
        ],
        Module::Draftsman => &["@есть отрезок в области"],
        Module::IsometricRobot => &[
            "впереди стена",
            "впереди свободно",
            "клетка закрашена",
            "клетка чистая",
            "@@есть точка",
            "@@клетка закрашена",
            "@@положение робота x",
            "@@положение робота y",
            "@@размер поля x",
            "@@размер поля y",
        ],
        Module::Keyboard => &["сигнал клав", "код клав"],
        Module::Painter => &[
            "CMYK",
            "CMYKA",
            "RGB",
            "RGBA",
            "HSL",
            "HSLA",
            "HSV",
            "HSVA",
            "ширина листа",
            "высота листа",
            "центр x",
            "центр y",
            "ширина текста",
            "значение в точке",
        ],
        Module::Vodoley => &[
            "размер A",
            "размер B",
            "размер C",
            "размер А",
            "размер В",
            "размер С",
            "в сосуде A",
            "в сосуде B",
            "в сосуде C",
            "в сосуде А",
            "в сосуде В",
            "в сосуде С",
            "@решено",
        ],
        Module::Grasshopper
        | Module::Turtle
        | Module::ComplexNumbers
        | Module::Files
        | Module::Strings => &[],
    }
}

fn module_zero_argument_names(module: Module) -> &'static [&'static str] {
    match module {
        Module::Robot => &[
            "слева стена",
            "справа стена",
            "сверху стена",
            "снизу стена",
            "слева свободно",
            "справа свободно",
            "сверху свободно",
            "снизу свободно",
            "клетка закрашена",
            "клетка чистая",
            "радиация",
            "температура",
        ],
        Module::IsometricRobot => &[
            "впереди стена",
            "впереди свободно",
            "клетка закрашена",
            "клетка чистая",
            "@@положение робота x",
            "@@положение робота y",
            "@@размер поля x",
            "@@размер поля y",
        ],
        Module::Keyboard => &["сигнал клав", "код клав"],
        Module::Painter => &["ширина листа", "высота листа", "центр x", "центр y"],
        Module::Vodoley => &[
            "размер A",
            "размер B",
            "размер C",
            "размер А",
            "размер В",
            "размер С",
            "в сосуде A",
            "в сосуде B",
            "в сосуде C",
            "в сосуде А",
            "в сосуде В",
            "в сосуде С",
            "@решено",
        ],
        Module::Draftsman
        | Module::Grasshopper
        | Module::Turtle
        | Module::ComplexNumbers
        | Module::Files
        | Module::Strings => &[],
    }
}

fn keyboard_key_code(name: &str) -> Option<i32> {
    let name = normalize(name);
    let key = name.strip_prefix("кл_")?;
    let special = match key {
        "назад" | "backspace" => Some(0x0100_0003),
        "tab" => Some(0x0100_0001),
        "ввод" | "enter" | "return" => Some(0x0100_0004),
        "пробел" | "space" => Some(32),
        "pageup" | "pgup" => Some(0x0100_0016),
        "pagedown" | "pgdown" => Some(0x0100_0017),
        "home" => Some(0x0100_0010),
        "end" => Some(0x0100_0011),
        "insert" => Some(0x0100_0006),
        "delete" => Some(0x0100_0007),
        "влево" => Some(0x0100_0012),
        "вверх" => Some(0x0100_0013),
        "вправо" => Some(0x0100_0014),
        "вниз" => Some(0x0100_0015),
        "х" => Some('[' as i32),
        "ъ" => Some(']' as i32),
        "ж" => Some(';' as i32),
        "э" => Some('\'' as i32),
        "б" => Some(',' as i32),
        "ю" => Some('.' as i32),
        _ => None,
    };
    if special.is_some() {
        return special;
    }
    if let Some(number) = key.strip_prefix('f').and_then(|n| n.parse::<i32>().ok()) {
        if (1..=12).contains(&number) && key == format!("f{number}") {
            return Some(0x0100_0030 + number - 1);
        }
    }
    if key.len() == 1 {
        let ch = key.chars().next()?;
        if ch.is_ascii_digit() {
            return Some(ch as i32);
        }
        if ch.is_ascii_lowercase() {
            return Some(ch.to_ascii_uppercase() as i32);
        }
    }
    let latin = match key {
        "й" => 'Q',
        "ц" => 'W',
        "у" => 'E',
        "к" => 'R',
        "е" => 'T',
        "н" => 'Y',
        "г" => 'U',
        "ш" => 'I',
        "щ" => 'O',
        "з" => 'P',
        "ф" => 'A',
        "ы" => 'S',
        "в" => 'D',
        "а" => 'F',
        "п" => 'G',
        "р" => 'H',
        "о" => 'J',
        "л" => 'K',
        "д" => 'L',
        "я" => 'Z',
        "ч" => 'X',
        "с" => 'C',
        "м" => 'V',
        "и" => 'B',
        "т" => 'N',
        "ь" => 'M',
        _ => return None,
    };
    Some(latin as i32)
}

fn match_module_query(source: &str, enabled: &[Module]) -> Option<(Module, &'static str)> {
    let key = normalize(source);
    let module = modules_for_name(&key, enabled)?;
    let name = module_zero_argument_names(module)
        .iter()
        .find(|name| normalize(name) == key)?;
    Some((module, *name))
}

fn split_module_marker(value: &str) -> (Module, String) {
    let value = value.trim_start_matches('@');
    let (module_name, command) = value.split_once(':').unwrap_or(("", value));
    let module = Module::from_name(module_name)
        .or_else(|| {
            [
                Module::Robot,
                Module::Draftsman,
                Module::Grasshopper,
                Module::IsometricRobot,
                Module::Turtle,
                Module::Keyboard,
                Module::Painter,
                Module::Vodoley,
                Module::ComplexNumbers,
            ]
            .into_iter()
            .find(|module| normalize(module.canonical_name()) == normalize(module_name))
        })
        .unwrap_or(Module::Robot);
    (module, command.to_owned())
}

fn parse_module_statement(
    source: &str,
    span: Span,
    enabled: &[Module],
) -> Result<Option<(Module, String, Vec<Expr>)>, Diagnostic> {
    let normalized = normalize(source);
    let mut commands = enabled
        .iter()
        .flat_map(|module| {
            module_command_names(*module)
                .iter()
                .map(move |name| (*module, *name))
        })
        .collect::<Vec<_>>();
    commands.sort_by_key(|(_, name)| std::cmp::Reverse(name.chars().count()));
    for (module, name) in commands {
        if !enabled.contains(&module) {
            continue;
        }
        let prefix = normalize(name);
        if normalized == prefix
            || normalized.starts_with(&(prefix.clone() + " "))
            || normalized.starts_with(&(prefix.clone() + "("))
        {
            let rest = source[name.len()..].trim();
            let args_text = if rest.starts_with('(') && rest.ends_with(')') {
                &rest[1..rest.len() - 1]
            } else {
                rest
            };
            let args = if args_text.is_empty() {
                Vec::new()
            } else {
                parse_expression_list(args_text, span, enabled)?
            };
            return Ok(Some((module, name.to_owned(), args)));
        }
    }
    Ok(None)
}

fn parse_file_statement(
    source: &str,
    span: Span,
    enabled: &[Module],
) -> Result<Option<FileExpr>, Diagnostic> {
    if !enabled.contains(&Module::Files) {
        return Ok(None);
    }
    let names = file_function_names();
    for command in names {
        if let Some((name, args)) = parse_specific_call(source, command, span, enabled)? {
            return Ok(Some(FileExpr::Call(name, args)));
        }
    }
    Ok(None)
}

fn parse_named_call(
    source: &str,
    span: Span,
    modules: &[Module],
) -> Result<Option<(String, Vec<Expr>)>, Diagnostic> {
    let mut names = file_function_names().to_vec();
    names.extend([
        "верний регистр",
        "нижний регистр",
        "позиция после",
        "поз после",
    ]);
    names.sort_by_key(|name| std::cmp::Reverse(name.chars().count()));
    for name in names {
        if file_function_names().contains(&name) && !modules.contains(&Module::Files) {
            continue;
        }
        if [
            "верний регистр",
            "нижний регистр",
            "позиция",
            "поз",
            "позиция после",
            "поз после",
            "вставить",
            "заменить",
            "удалить",
        ]
        .contains(&name)
            && !modules.contains(&Module::Strings)
        {
            continue;
        }
        if let Some(call) = parse_specific_call(source, name, span, modules)? {
            return Ok(Some(call));
        }
    }
    Ok(None)
}

fn parse_zero_argument_call(source: &str, modules: &[Module]) -> Option<String> {
    let name = normalize(source);
    if name == "время" {
        return Some(name);
    }
    if modules.contains(&Module::Files)
        && ["рабочий каталог", "каталог программы", "консоль"].contains(&name.as_str())
    {
        return Some(name);
    }
    None
}

fn parse_specific_call(
    source: &str,
    name: &str,
    span: Span,
    modules: &[Module],
) -> Result<Option<(String, Vec<Expr>)>, Diagnostic> {
    let source = source.trim();
    if !starts_keyword(source, name) {
        return Ok(None);
    }
    let rest = source[name.len()..].trim();
    if !rest.starts_with('(') || !rest.ends_with(')') {
        return Ok(None);
    }
    let args_source = &rest[1..rest.len() - 1];
    let args = if args_source.trim().is_empty() {
        Vec::new()
    } else {
        parse_expression_list(args_source, span, modules)?
    };
    Ok(Some((normalize(name), args)))
}

fn file_function_names() -> &'static [&'static str] {
    &[
        "открыть на чтение",
        "открыть на запись",
        "открыть на добавление",
        "закрыть",
        "начать чтение",
        "конец файла",
        "есть данные",
        "установить кодировку",
        "можно открыть на чтение",
        "можно открыть на запись",
        "существует",
        "является каталогом",
        "создать каталог",
        "удалить_файл",
        "удалить_каталог",
        "полный путь",
        "рабочий каталог",
        "каталог программы",
        "назначить ввод",
        "назначить вывод",
        "консоль",
    ]
}

fn parse_call_statement(
    source: &str,
    span: Span,
    modules: &[Module],
) -> Result<Option<(String, Vec<Expr>)>, Diagnostic> {
    let Some(open) = source.find('(') else {
        return Ok(None);
    };
    if !source.ends_with(')') {
        return Ok(None);
    }
    let name = normalize(source[..open].trim());
    if !valid_name(&name) {
        return Ok(None);
    }
    let args_text = &source[open + 1..source.len() - 1];
    let args = if args_text.trim().is_empty() {
        Vec::new()
    } else {
        parse_expression_list(args_text, span, modules)?
    };
    Ok(Some((name, args)))
}

fn module_command_names(module: Module) -> &'static [&'static str] {
    match module {
        Module::Robot => &[
            "закрасить",
            "влево",
            "вправо",
            "вверх",
            "вниз",
            "слева стена",
            "справа стена",
            "сверху стена",
            "снизу стена",
            "слева свободно",
            "справа свободно",
            "сверху свободно",
            "снизу свободно",
            "клетка закрашена",
            "клетка чистая",
            "радиация",
            "температура",
            "@@размер поля",
            "@@робот",
        ],
        Module::Draftsman => &[
            "опустить перо",
            "поднять перо",
            "выбрать чернила",
            "сместиться в точку",
            "сместиться на вектор",
            "написать",
        ],
        Module::Grasshopper => &["вперед", "назад", "перекрасить"],
        Module::IsometricRobot => &[
            "вперед",
            "повернуть налево",
            "повернуть направо",
            "закрасить",
            "впереди стена",
            "впереди свободно",
            "клетка закрашена",
            "клетка чистая",
        ],
        Module::Turtle => &[
            "опустить хвост",
            "поднять хвост",
            "вперед",
            "назад",
            "влево",
            "вправо",
        ],
        Module::Keyboard => &["сигнал клав", "код клав", "сброс клав"],
        Module::Painter => &[
            "CMYK",
            "CMYKA",
            "RGB",
            "RGBA",
            "HSL",
            "HSLA",
            "HSV",
            "HSVA",
            "разложить в CMYK",
            "разложить в RGB",
            "разложить в RGBA",
            "разложить в HSL",
            "разложить в HSLA",
            "разложить в HSV",
            "разложить в HSVA",
            "новый лист",
            "загрузить лист",
            "сохранить лист",
            "перо",
            "кисть",
            "убрать кисть",
            "плотность",
            "шрифт",
            "в точку",
            "линия в точку",
            "линия",
            "многоугольник",
            "пиксель",
            "прямоугольник",
            "окружность",
            "эллипс",
            "надпись",
            "залить",
            "ширина листа",
            "высота листа",
            "центр x",
            "центр y",
            "ширина текста",
            "значение в точке",
        ],
        Module::Vodoley => &[
            "наполни A",
            "наполни B",
            "наполни C",
            "наполни А",
            "наполни В",
            "наполни С",
            "вылей A",
            "вылей B",
            "вылей C",
            "вылей А",
            "вылей В",
            "вылей С",
            "перелей из A в B",
            "перелей из A в C",
            "перелей из B в A",
            "перелей из B в C",
            "перелей из C в A",
            "перелей из C в B",
            "перелей из А в В",
            "перелей из А в С",
            "перелей из В в А",
            "перелей из В в С",
            "перелей из С в А",
            "перелей из С в В",
            "размер A",
            "размер B",
            "размер C",
            "размер А",
            "размер В",
            "размер С",
            "в сосуде A",
            "в сосуде B",
            "в сосуде C",
            "в сосуде А",
            "в сосуде В",
            "в сосуде С",
        ],
        Module::ComplexNumbers | Module::Files | Module::Strings => &[],
    }
}

fn module_argument_modes(
    module: Module,
    name: &str,
    argument_count: usize,
) -> Vec<ModuleArgumentMode> {
    let mut modes = vec![ModuleArgumentMode::Argument; argument_count];
    if module == Module::Painter && normalize(name).starts_with("разложить в ") {
        for mode in modes.iter_mut().skip(1) {
            *mode = ModuleArgumentMode::ArgumentResult;
        }
    } else if module == Module::Robot
        && matches!(normalize(name).as_str(), "@@размер поля" | "@@робот")
    {
        modes.fill(ModuleArgumentMode::Result);
    }
    modes
}

fn module_expected_arity(module: Module, name: &str) -> Option<usize> {
    let name = normalize(name);
    let matches = |names: &[&str]| names.iter().any(|candidate| normalize(candidate) == name);
    match module {
        Module::Robot => {
            if matches(&["@@размер поля", "@@робот"]) {
                Some(2)
            } else if matches(&[
                "@@закрашена",
                "@@метка",
                "@@верхняя буква",
                "@@нижняя буква",
                "@@температура",
                "@@радиация",
            ]) {
                Some(2)
            } else if matches(&[
                "закрасить",
                "влево",
                "вправо",
                "вверх",
                "вниз",
                "слева стена",
                "справа стена",
                "сверху стена",
                "снизу стена",
                "слева свободно",
                "справа свободно",
                "сверху свободно",
                "снизу свободно",
                "клетка закрашена",
                "клетка чистая",
                "радиация",
                "температура",
            ]) {
                Some(0)
            } else {
                None
            }
        }
        Module::Draftsman => {
            if name == "@есть отрезок в области" {
                Some(3)
            } else if name == "выбрать чернила" {
                Some(1)
            } else if matches(&["сместиться в точку", "сместиться на вектор"])
            {
                Some(2)
            } else if name == "написать" {
                Some(2)
            } else if matches(&["опустить перо", "поднять перо"]) {
                Some(0)
            } else {
                None
            }
        }
        Module::Grasshopper => {
            if matches(&["вперед", "назад"]) {
                Some(1)
            } else if name == "перекрасить" {
                Some(0)
            } else {
                None
            }
        }
        Module::IsometricRobot => {
            if matches(&["@@есть точка", "@@клетка закрашена"]) {
                Some(2)
            } else if matches(&[
                "@@положение робота x",
                "@@положение робота y",
                "@@размер поля x",
                "@@размер поля y",
            ]) {
                Some(0)
            } else if matches(&[
                "вперед",
                "повернуть налево",
                "повернуть направо",
                "закрасить",
                "впереди стена",
                "впереди свободно",
                "клетка закрашена",
                "клетка чистая",
                "@@положение робота x",
                "@@положение робота y",
                "@@размер поля x",
                "@@размер поля y",
            ]) {
                Some(0)
            } else {
                None
            }
        }
        Module::Turtle => {
            if matches(&["вперед", "назад", "влево", "вправо"]) {
                Some(1)
            } else if matches(&["опустить хвост", "поднять хвост"]) {
                Some(0)
            } else {
                None
            }
        }
        Module::Keyboard => matches(&["сигнал клав", "код клав", "сброс клав"]).then_some(0),
        Module::Painter => match name.as_str() {
            "hsl" | "hsv" | "rgb" => Some(3),
            "cmyk" => Some(4),
            "cmyka" => Some(5),
            "rgba" | "hsla" | "hsva" => Some(4),
            "разложить в cmyk" | "разложить в hsla" | "разложить в hsva" => {
                Some(5)
            }
            "разложить в rgb" | "разложить в hsl" | "разложить в hsv" => {
                Some(4)
            }
            "разложить в rgba" => Some(5),
            "новый лист" => Some(3),
            "загрузить лист" | "сохранить лист" | "плотность" => {
                Some(1)
            }
            "перо" | "шрифт" => Some(if name == "перо" { 2 } else { 4 }),
            "кисть" => Some(1),
            "убрать кисть" | "ширина листа" | "высота листа" | "центр x" | "центр y" => {
                Some(0)
            }
            "в точку" | "линия в точку" | "значение в точке" | "залить" => {
                Some(2)
            }
            "линия" | "прямоугольник" | "эллипс" => Some(4),
            "многоугольник" => Some(3),
            "пиксель" => Some(3),
            "окружность" => Some(3),
            "надпись" => Some(3),
            "ширина текста" => Some(1),
            _ => None,
        },
        Module::Vodoley => matches(&[
            "наполни A",
            "наполни B",
            "наполни C",
            "наполни А",
            "наполни В",
            "наполни С",
            "вылей A",
            "вылей B",
            "вылей C",
            "вылей А",
            "вылей В",
            "вылей С",
            "перелей из A в B",
            "перелей из A в C",
            "перелей из B в A",
            "перелей из B в C",
            "перелей из C в A",
            "перелей из C в B",
            "перелей из А в В",
            "перелей из А в С",
            "перелей из В в А",
            "перелей из В в С",
            "перелей из С в А",
            "перелей из С в В",
            "размер A",
            "размер B",
            "размер C",
            "размер А",
            "размер В",
            "размер С",
            "в сосуде A",
            "в сосуде B",
            "в сосуде C",
            "в сосуде А",
            "в сосуде В",
            "в сосуде С",
            "@решено",
        ])
        .then_some(0),
        Module::ComplexNumbers | Module::Files | Module::Strings => None,
    }
}

#[derive(Clone)]
struct ModuleSignature {
    arguments: Vec<Option<Type>>,
    result: ModuleResultType,
}

#[derive(Clone)]
enum ModuleResultType {
    Procedure,
    Value(Type),
    Unspecified,
}

fn module_signature(module: Module, name: &str) -> Option<ModuleSignature> {
    let name = normalize(name);
    let int = Type::Integer;
    let real = Type::Real;
    let boolean = Type::Boolean;
    let string = Type::String;
    let color = Type::Color;
    let integer_table = || Type::Table {
        element: Box::new(Type::Integer),
        bounds: Vec::new(),
    };
    let typed = |arguments: Vec<Type>, result: ModuleResultType| ModuleSignature {
        arguments: arguments.into_iter().map(Some).collect(),
        result,
    };
    let unknown_result_args = |arguments: Vec<Type>| ModuleSignature {
        arguments: arguments.into_iter().map(Some).collect(),
        result: ModuleResultType::Unspecified,
    };
    let procedure = |arguments: Vec<Type>| typed(arguments, ModuleResultType::Procedure);
    let value =
        |arguments: Vec<Type>, result: Type| typed(arguments, ModuleResultType::Value(result));

    let signature = match module {
        Module::Robot => {
            if matches!(
                name.as_str(),
                "закрасить" | "влево" | "вправо" | "вверх" | "вниз"
            ) {
                Some(procedure(Vec::new()))
            } else if matches!(
                name.as_str(),
                "слева стена"
                    | "справа стена"
                    | "сверху стена"
                    | "снизу стена"
                    | "слева свободно"
                    | "справа свободно"
                    | "сверху свободно"
                    | "снизу свободно"
                    | "клетка закрашена"
                    | "клетка чистая"
            ) {
                Some(value(Vec::new(), boolean))
            } else if matches!(name.as_str(), "температура" | "радиация") {
                Some(value(Vec::new(), real))
            } else if matches!(name.as_str(), "@@размер поля" | "@@робот") {
                Some(procedure(vec![int.clone(), int.clone()]))
            } else if matches!(name.as_str(), "@@закрашена" | "@@метка") {
                Some(value(vec![int.clone(), int.clone()], boolean))
            } else if matches!(
                name.as_str(),
                "@@верхняя буква" | "@@нижняя буква" | "@@температура" | "@@радиация"
            ) {
                Some(unknown_result_args(vec![int.clone(), int.clone()]))
            } else {
                None
            }
        }
        Module::Draftsman => match name.as_str() {
            "опустить перо" | "поднять перо" => Some(procedure(Vec::new())),
            "выбрать чернила" => Some(procedure(vec![color])),
            "сместиться в точку" | "сместиться на вектор" => {
                Some(procedure(vec![real.clone(), real.clone()]))
            }
            "написать" => Some(procedure(vec![real, string])),
            "@есть отрезок в области" => {
                Some(value(vec![real.clone(), real.clone(), real], boolean))
            }
            _ => None,
        },
        Module::Grasshopper => match name.as_str() {
            "вперед" | "назад" => Some(procedure(vec![int])),
            "перекрасить" => Some(procedure(Vec::new())),
            _ => None,
        },
        Module::IsometricRobot => match name.as_str() {
            "вперед" | "повернуть налево" | "повернуть направо" | "закрасить" => {
                Some(procedure(Vec::new()))
            }
            "впереди стена" | "впереди свободно" | "клетка закрашена" | "клетка чистая" => {
                Some(value(Vec::new(), boolean))
            }
            "@@есть точка" | "@@клетка закрашена" => {
                Some(value(vec![int.clone(), int.clone()], boolean))
            }
            "@@положение робота x"
            | "@@положение робота y"
            | "@@размер поля x"
            | "@@размер поля y" => Some(value(Vec::new(), int)),
            _ => None,
        },
        Module::Turtle => match name.as_str() {
            "опустить хвост" | "поднять хвост" => {
                Some(procedure(Vec::new()))
            }
            "вперед" | "назад" | "влево" | "вправо" => {
                Some(procedure(vec![int]))
            }
            _ => None,
        },
        Module::Keyboard => match name.as_str() {
            "сигнал клав" => Some(value(Vec::new(), boolean)),
            "код клав" => Some(value(Vec::new(), int)),
            "сброс клав" => Some(procedure(Vec::new())),
            _ => None,
        },
        Module::Painter => match name.as_str() {
            "cmyk" => Some(value(
                vec![int.clone(), int.clone(), int.clone(), int],
                color,
            )),
            "cmyka" => Some(value(
                vec![int.clone(), int.clone(), int.clone(), int.clone(), int],
                color,
            )),
            "rgb" | "hsl" | "hsv" => Some(value(vec![int.clone(), int.clone(), int], color)),
            "rgba" | "hsla" | "hsva" => Some(value(
                vec![int.clone(), int.clone(), int.clone(), int],
                color,
            )),
            "разложить в cmyk" => Some(procedure(vec![
                color,
                int.clone(),
                int.clone(),
                int.clone(),
                int,
            ])),
            "разложить в cmyka" => Some(procedure(vec![
                color,
                int.clone(),
                int.clone(),
                int.clone(),
                int.clone(),
            ])),
            "разложить в rgb" | "разложить в hsl" | "разложить в hsv" => {
                Some(procedure(vec![color, int.clone(), int.clone(), int]))
            }
            "разложить в rgba" | "разложить в hsla" | "разложить в hsva" => {
                Some(procedure(vec![
                    color,
                    int.clone(),
                    int.clone(),
                    int.clone(),
                    int,
                ]))
            }
            "новый лист" => Some(procedure(vec![int.clone(), int.clone(), color])),
            "загрузить лист" | "сохранить лист" => {
                Some(procedure(vec![string]))
            }
            "перо" => Some(procedure(vec![int.clone(), color])),
            "кисть" => Some(procedure(vec![color])),
            "убрать кисть" => Some(procedure(Vec::new())),
            "плотность" => Some(procedure(vec![int])),
            "шрифт" => Some(procedure(vec![string, int, boolean.clone(), boolean])),
            "в точку" | "линия в точку" | "залить" => {
                Some(procedure(vec![int.clone(), int]))
            }
            "линия" | "прямоугольник" | "эллипс" => {
                Some(procedure(vec![int.clone(), int.clone(), int.clone(), int]))
            }
            "многоугольник" => {
                Some(procedure(vec![int, integer_table(), integer_table()]))
            }
            "пиксель" => Some(procedure(vec![int.clone(), int.clone(), color])),
            "окружность" => Some(procedure(vec![int.clone(), int.clone(), int])),
            "надпись" => Some(procedure(vec![int.clone(), int, string])),
            "ширина листа" | "высота листа" | "центр x" | "центр y" => {
                Some(value(Vec::new(), int))
            }
            "ширина текста" => Some(value(vec![string], int)),
            "значение в точке" => Some(value(vec![int.clone(), int], color)),
            _ => None,
        },
        Module::Vodoley => {
            if matches!(
                name.as_str(),
                "наполни a"
                    | "наполни b"
                    | "наполни c"
                    | "наполни а"
                    | "наполни в"
                    | "наполни с"
                    | "вылей a"
                    | "вылей b"
                    | "вылей c"
                    | "вылей а"
                    | "вылей в"
                    | "вылей с"
                    | "перелей из a в b"
                    | "перелей из a в c"
                    | "перелей из b в a"
                    | "перелей из b в c"
                    | "перелей из c в a"
                    | "перелей из c в b"
                    | "перелей из а в в"
                    | "перелей из а в с"
                    | "перелей из в в а"
                    | "перелей из в в с"
                    | "перелей из с в а"
                    | "перелей из с в в"
            ) {
                Some(procedure(Vec::new()))
            } else if name == "@решено" {
                Some(value(Vec::new(), boolean))
            } else if name.starts_with("размер ") || name.starts_with("в сосуде ") {
                Some(value(Vec::new(), int))
            } else {
                None
            }
        }
        Module::ComplexNumbers | Module::Files | Module::Strings => None,
    };

    signature.or_else(|| {
        module_expected_arity(module, &name).map(|arity| ModuleSignature {
            arguments: vec![None; arity],
            result: ModuleResultType::Unspecified,
        })
    })
}

fn validate_module_call_types(
    module: Module,
    name: &str,
    arguments: &[Expr],
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
    expression_call: bool,
) -> Result<Option<Type>, Diagnostic> {
    if !modules.contains(&module) {
        return Err(error(
            span,
            &format!(
                "module `{}` must be enabled with `использовать`",
                module.canonical_name()
            ),
        ));
    }

    let signature = module_signature(module, name).ok_or_else(|| {
        error(
            span,
            &format!(
                "unknown or unavailable command `{name}` in module `{}`",
                module.canonical_name()
            ),
        )
    })?;
    if arguments.len() != signature.arguments.len() {
        return Err(error(
            span,
            &format!("`{name}` expects {} argument(s)", signature.arguments.len()),
        ));
    }

    let modes = module_argument_modes(module, name, arguments.len());
    for ((argument, expected), mode) in arguments
        .iter()
        .zip(&signature.arguments)
        .zip(modes.iter().copied())
    {
        if mode != ModuleArgumentMode::Argument {
            validate_mutable_target(argument, declarations, span)?;
        }
        let actual = if mode == ModuleArgumentMode::Argument {
            infer_expression_type(argument, declarations, algorithms, profile, modules, span)?
        } else {
            infer_target_type(argument, declarations, algorithms, profile, modules, span)?
        };
        if let (Some(expected), Some(actual)) = (expected, actual) {
            require_assignable(expected, &actual, span)?;
        }
    }

    if expression_call {
        if modes
            .iter()
            .any(|mode| *mode != ModuleArgumentMode::Argument)
        {
            return Err(error(
                span,
                "module output arguments cannot be used in an expression call",
            ));
        }
        match signature.result {
            ModuleResultType::Procedure => Err(error(
                span,
                &format!("`{name}` is a procedure and cannot be used as a value"),
            )),
            ModuleResultType::Value(ty) => Ok(Some(ty)),
            ModuleResultType::Unspecified => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn runtime_value_matches(expected: &Type, value: &Value) -> bool {
    !matches!(value, Value::Void) && type_accepts(expected, &value_type(value))
}

fn validate_module_result_value(
    signature: &ModuleSignature,
    name: &str,
    value: &Value,
    span: Span,
) -> Result<(), RuntimeError> {
    match &signature.result {
        ModuleResultType::Procedure if !matches!(value, Value::Void) => Err(runtime_error(
            span,
            format!("module procedure `{name}` returned a value"),
        )),
        ModuleResultType::Value(expected) if !runtime_value_matches(expected, value) => {
            Err(runtime_error(
                span,
                format!(
                    "module function `{name}` returned an incompatible value; expected `{}`",
                    type_name(expected)
                ),
            ))
        }
        _ => Ok(()),
    }
}

fn file_exprs(file: &FileExpr) -> Vec<&Expr> {
    match file {
        FileExpr::Call(_, args) => args.iter().collect(),
    }
}

fn is_file_builtin(name: &str) -> bool {
    file_function_names()
        .iter()
        .any(|item| normalize(item) == normalize(name))
}

fn is_string_output_procedure(name: &str) -> bool {
    matches!(
        normalize(name).as_str(),
        "вставить" | "заменить" | "удалить"
    )
}

fn validate_assignment_type(
    expected: &Type,
    expression: &Expr,
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
) -> Result<(), Diagnostic> {
    let actual =
        infer_expression_type(expression, declarations, algorithms, profile, modules, span)?;
    if let Some(actual) = actual {
        require_assignable(expected, &actual, span)?;
    }
    Ok(())
}

fn validate_expression_expected(
    expression: &Expr,
    expected: &Type,
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
) -> Result<(), Diagnostic> {
    let actual =
        infer_expression_type(expression, declarations, algorithms, profile, modules, span)?;
    if let Some(actual) = actual {
        require_assignable(expected, &actual, span)?;
    }
    Ok(())
}

fn require_assignable(expected: &Type, actual: &Type, span: Span) -> Result<(), Diagnostic> {
    if type_accepts(expected, actual) {
        Ok(())
    } else {
        Err(error(
            span,
            &format!(
                "value of type `{}` is incompatible with `{}`",
                type_name(actual),
                type_name(expected)
            ),
        ))
    }
}

fn validate_mutable_target(
    target: &Expr,
    declarations: &BTreeMap<String, Declaration>,
    span: Span,
) -> Result<(), Diagnostic> {
    let name = match target {
        Expr::Variable(name, _) => Some(name.as_str()),
        Expr::Index(array, _) => match array.as_ref() {
            Expr::Variable(name, _) => Some(name.as_str()),
            _ => None,
        },
        _ => None,
    };
    if let Some(name) = name {
        if declarations
            .get(name)
            .is_some_and(|declaration| declaration.constant)
        {
            return Err(error(span, &format!("cannot modify read-only `{name}`")));
        }
    }
    Ok(())
}

fn type_accepts(expected: &Type, actual: &Type) -> bool {
    match (expected, actual) {
        (Type::Real, Type::Integer) | (Type::Complex, Type::Integer | Type::Real) => true,
        (Type::String, Type::Character) => true,
        (
            Type::Table {
                element: expected_element,
                bounds: expected_bounds,
            },
            Type::Table {
                element: actual_element,
                bounds: actual_bounds,
            },
        ) => {
            (expected_bounds.is_empty()
                || actual_bounds.is_empty()
                || expected_bounds == actual_bounds)
                && type_accepts(expected_element, actual_element)
        }
        _ => expected == actual,
    }
}

fn infer_target_type(
    target: &Expr,
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
) -> Result<Option<Type>, Diagnostic> {
    match target {
        Expr::Variable(name, _) => Ok(declarations
            .get(name)
            .map(|declaration| declaration.ty.clone())),
        Expr::Index(array, indices) => {
            let Some(array_type) =
                infer_expression_type(array, declarations, algorithms, profile, modules, span)?
            else {
                return Ok(None);
            };
            for index in indices {
                validate_expression_expected(
                    index,
                    &Type::Integer,
                    declarations,
                    algorithms,
                    profile,
                    modules,
                    span,
                )?;
            }
            match array_type {
                Type::Table { element, bounds }
                    if bounds.is_empty() || indices.len() == bounds.len() =>
                {
                    Ok(Some(*element))
                }
                Type::Table { .. } => Err(error(span, "wrong number of table indices")),
                _ => Err(error(span, "indexed value must be a table")),
            }
        }
        _ => Ok(None),
    }
}

fn infer_expression_type(
    expression: &Expr,
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
) -> Result<Option<Type>, Diagnostic> {
    match expression {
        Expr::Literal(value) => Ok(Some(value_type(value))),
        Expr::Variable(name, variable_span) => {
            if let Some(declaration) = declarations.get(name) {
                return Ok(Some(declaration.ty.clone()));
            }
            if negated_logical_name(name)
                .and_then(|base| declarations.get(&base))
                .is_some_and(|declaration| declaration.ty == Type::Boolean)
            {
                return Ok(Some(Type::Boolean));
            }
            Err(error(
                *variable_span,
                &format!("undefined variable `{name}`"),
            ))
        }
        Expr::Index(_, _) => {
            infer_target_type(expression, declarations, algorithms, profile, modules, span)
        }
        Expr::Slice(target, start, end) => {
            let base =
                infer_expression_type(target, declarations, algorithms, profile, modules, span)?;
            validate_expression_expected(
                start,
                &Type::Integer,
                declarations,
                algorithms,
                profile,
                modules,
                span,
            )?;
            validate_expression_expected(
                end,
                &Type::Integer,
                declarations,
                algorithms,
                profile,
                modules,
                span,
            )?;
            if base.is_some_and(|ty| ty != Type::String) {
                return Err(error(span, "string slice requires a text value"));
            }
            Ok(Some(Type::String))
        }
        Expr::Call(name, arguments) => validate_call_types(
            name,
            arguments,
            declarations,
            algorithms,
            profile,
            modules,
            span,
            true,
        ),
        Expr::ModuleCall(module, name, arguments) => validate_module_call_types(
            *module,
            name,
            arguments,
            declarations,
            algorithms,
            profile,
            modules,
            span,
            true,
        ),
        Expr::Unary(operator, value) => {
            let Some(value_type) =
                infer_expression_type(value, declarations, algorithms, profile, modules, span)?
            else {
                return Ok(None);
            };
            match operator {
                UnaryOp::Not if value_type == Type::Boolean => Ok(Some(Type::Boolean)),
                UnaryOp::Not => Err(error(span, "logical negation requires a logical value")),
                UnaryOp::Negate | UnaryOp::Positive if is_numeric_type(&value_type) => {
                    Ok(Some(value_type))
                }
                UnaryOp::Negate | UnaryOp::Positive => {
                    Err(error(span, "numeric unary operator requires a number"))
                }
            }
        }
        Expr::Binary(left, operator, right) => {
            let left_type =
                infer_expression_type(left, declarations, algorithms, profile, modules, span)?;
            let right_type =
                infer_expression_type(right, declarations, algorithms, profile, modules, span)?;
            match (left_type, right_type) {
                (Some(left), Some(right)) => {
                    infer_binary_type(&left, *operator, &right, span).map(Some)
                }
                _ => Ok(None),
            }
        }
    }
}

fn validate_call_types(
    name: &str,
    arguments: &[Expr],
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
    value_context: bool,
) -> Result<Option<Type>, Diagnostic> {
    if let Some(algorithm) = algorithms.get(&normalize(name)) {
        if arguments.len() != algorithm.parameters.len() {
            return Err(error(
                span,
                &format!(
                    "algorithm `{name}` expects {} argument(s)",
                    algorithm.parameters.len()
                ),
            ));
        }
        for (parameter, argument) in algorithm.parameters.iter().zip(arguments) {
            let argument_type = if parameter.mode == ParameterMode::Argument {
                infer_expression_type(argument, declarations, algorithms, profile, modules, span)?
            } else {
                infer_target_type(argument, declarations, algorithms, profile, modules, span)?
            };
            if let Some(argument_type) = argument_type {
                require_assignable(&parameter.ty, &argument_type, span)?;
                if parameter.mode != ParameterMode::Argument
                    && !type_accepts(&argument_type, &parameter.ty)
                {
                    return Err(error(
                        span,
                        "result parameter type cannot be written back to the argument",
                    ));
                }
            }
            if parameter.mode != ParameterMode::Argument {
                validate_mutable_target(argument, declarations, span)?;
            }
        }
        if value_context && algorithm.result.is_none() {
            return Err(error(
                span,
                &format!("procedure `{name}` cannot be used as a value"),
            ));
        }
        return Ok(algorithm.result.clone());
    }

    if !is_builtin(name, profile, modules)
        && !(modules.contains(&Module::Files) && is_file_builtin(name))
    {
        return Err(error(
            span,
            &format!("unknown algorithm or function `{name}`"),
        ));
    }
    if is_string_output_procedure(name) && value_context {
        return Err(error(
            span,
            &format!("`{name}` is a procedure and cannot be used as a value"),
        ));
    }
    validate_builtin_call_types(
        name,
        arguments,
        declarations,
        algorithms,
        profile,
        modules,
        span,
        value_context,
    )
}

fn validate_builtin_call_types(
    name: &str,
    arguments: &[Expr],
    declarations: &BTreeMap<String, Declaration>,
    algorithms: &BTreeMap<String, Algorithm>,
    profile: Profile,
    modules: &[Module],
    span: Span,
    value_context: bool,
) -> Result<Option<Type>, Diagnostic> {
    let name = normalize(name);
    let arity = |allowed: &[usize]| {
        if allowed.contains(&arguments.len()) {
            Ok(())
        } else {
            Err(error(
                span,
                &format!(
                    "`{name}` expects {} argument(s)",
                    allowed
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(" or ")
                ),
            ))
        }
    };
    let argument_type = |index: usize| -> Result<Option<Type>, Diagnostic> {
        arguments.get(index).map_or(Ok(None), |argument| {
            infer_expression_type(argument, declarations, algorithms, profile, modules, span)
        })
    };
    let expect = |index: usize, expected: &Type| -> Result<(), Diagnostic> {
        if let Some(actual) = argument_type(index)? {
            require_assignable(expected, &actual, span)?;
        }
        Ok(())
    };
    let expect_numeric = |index: usize| -> Result<(), Diagnostic> {
        if let Some(actual) = argument_type(index)? {
            if !matches!(actual, Type::Integer | Type::Real) {
                return Err(error(span, "numeric argument expected"));
            }
        }
        Ok(())
    };
    let expect_text = |index: usize| -> Result<(), Diagnostic> {
        if let Some(actual) = argument_type(index)? {
            if !matches!(actual, Type::String | Type::Character) {
                return Err(error(span, "text argument expected"));
            }
        }
        Ok(())
    };

    for argument in arguments {
        infer_expression_type(argument, declarations, algorithms, profile, modules, span)?;
    }

    if modules.contains(&Module::Files) && is_file_builtin(&name) {
        let (allowed, result, parameter) = match name.as_str() {
            "открыть на чтение" | "открыть на запись" | "открыть на добавление" => {
                (&[1][..], Some(Type::File), Some(Type::String))
            }
            "закрыть" | "начать чтение" => (&[1][..], None, Some(Type::File)),
            "конец файла" | "есть данные" => {
                (&[1][..], Some(Type::Boolean), Some(Type::File))
            }
            "установить кодировку" => (&[1][..], None, Some(Type::String)),
            "можно открыть на чтение"
            | "можно открыть на запись"
            | "существует"
            | "является каталогом"
            | "создать каталог"
            | "удалить_файл"
            | "удалить_каталог" => {
                (&[1][..], Some(Type::Boolean), Some(Type::String))
            }
            "полный путь" => (&[1][..], Some(Type::String), Some(Type::String)),
            "рабочий каталог" | "каталог программы" => {
                (&[0][..], Some(Type::String), None)
            }
            "консоль" => (&[0][..], Some(Type::File), None),
            "назначить ввод" | "назначить вывод" => {
                (&[1][..], None, Some(Type::String))
            }
            _ => return Err(error(span, "unsupported file operation")),
        };
        arity(allowed)?;
        if let Some(parameter) = parameter {
            expect(0, &parameter)?;
        }
        if value_context && result.is_none() {
            return Err(error(
                span,
                &format!("`{name}` is a procedure and cannot be used as a value"),
            ));
        }
        return Ok(result);
    }

    match name.as_str() {
        "цел" => {
            arity(&[1, 2])?;
            if arguments.len() == 2 {
                expect(1, &Type::Integer)?;
            } else {
                expect_numeric(0)?;
            }
        }
        "вещ" => {
            arity(&[1, 2])?;
            if arguments.len() == 2 {
                expect_numeric(1)?;
            } else {
                expect_numeric(0)?;
            }
        }
        "лог" => {
            arity(&[2])?;
            expect(1, &Type::Boolean)?;
        }
        "abs" | "sign" | "sqrt" | "sin" | "cos" | "tg" | "ctg" | "arcsin" | "arccos" | "arctg"
        | "arcctg" | "ln" | "lg" | "exp" | "int" | "rnd" => {
            arity(&[1])?;
            expect_numeric(0)?;
        }
        "iabs" | "irnd" => {
            arity(&[1])?;
            expect(0, &Type::Integer)?;
        }
        "rand" => {
            arity(&[2])?;
            expect_numeric(0)?;
            expect_numeric(1)?;
        }
        "irand" | "div" | "mod" | "imin" | "imax" => {
            arity(&[2])?;
            expect(0, &Type::Integer)?;
            expect(1, &Type::Integer)?;
        }
        "min" | "max" => {
            arity(&[2])?;
            expect_numeric(0)?;
            expect_numeric(1)?;
        }
        "длин" => {
            arity(&[1])?;
            if let Some(actual) = argument_type(0)? {
                if !matches!(actual, Type::String | Type::Table { .. }) {
                    return Err(error(span, "`длин` expects text or a table"));
                }
            }
        }
        "код" | "юникод" => {
            arity(&[1])?;
            expect_text(0)?;
        }
        "символ" | "юнисимвол" => {
            arity(&[1])?;
            expect(0, &Type::Integer)?;
        }
        "цел_в_лит" => {
            arity(&[1])?;
            expect(0, &Type::Integer)?;
        }
        "вещ_в_лит" => {
            arity(&[1])?;
            expect_numeric(0)?;
        }
        "лит_в_цел" | "лит_в_вещ" => {
            arity(&[1, 2])?;
            expect_text(0)?;
            if arguments.len() == 2 {
                if !matches!(arguments[1], Expr::Variable(_, _)) {
                    return Err(error(
                        span,
                        "conversion result parameter must be a variable",
                    ));
                }
                validate_mutable_target(&arguments[1], declarations, span)?;
                let target_type = infer_target_type(
                    &arguments[1],
                    declarations,
                    algorithms,
                    profile,
                    modules,
                    span,
                )?;
                if let Some(target_type) = target_type {
                    require_assignable(&Type::Boolean, &target_type, span)?;
                }
            }
        }
        "позиция" | "поз" => {
            arity(&[2])?;
            expect_text(0)?;
            expect_text(1)?;
        }
        "позиция после" | "поз после" => {
            arity(&[3])?;
            expect(0, &Type::Integer)?;
            expect_text(1)?;
            expect_text(2)?;
        }
        "верний регистр" | "нижний регистр" => {
            arity(&[1])?;
            expect_text(0)?;
        }
        "вставить" => {
            arity(&[3])?;
            expect_text(0)?;
            validate_mutable_target(&arguments[1], declarations, span)?;
            let target_type = infer_target_type(
                &arguments[1],
                declarations,
                algorithms,
                profile,
                modules,
                span,
            )?;
            if let Some(target_type) = target_type {
                require_assignable(&Type::String, &target_type, span)?;
            }
            expect(2, &Type::Integer)?;
        }
        "заменить" => {
            arity(&[4])?;
            validate_mutable_target(&arguments[0], declarations, span)?;
            let target_type = infer_target_type(
                &arguments[0],
                declarations,
                algorithms,
                profile,
                modules,
                span,
            )?;
            if let Some(target_type) = target_type {
                require_assignable(&Type::String, &target_type, span)?;
            }
            expect_text(1)?;
            expect_text(2)?;
            expect(3, &Type::Boolean)?;
        }
        "удалить" => {
            arity(&[3])?;
            validate_mutable_target(&arguments[0], declarations, span)?;
            let target_type = infer_target_type(
                &arguments[0],
                declarations,
                algorithms,
                profile,
                modules,
                span,
            )?;
            if let Some(target_type) = target_type {
                require_assignable(&Type::String, &target_type, span)?;
            }
            expect(1, &Type::Integer)?;
            expect(2, &Type::Integer)?;
        }
        "re" | "im" => {
            arity(&[1])?;
            expect(0, &Type::Complex)?;
        }
        "время" => arity(&[0])?,
        "ждать" => {
            arity(&[1])?;
            expect(0, &Type::Integer)?;
            if value_context {
                return Err(error(
                    span,
                    "`ждать` is a procedure and cannot be used as a value",
                ));
            }
        }
        _ => {
            for argument in arguments {
                infer_expression_type(argument, declarations, algorithms, profile, modules, span)?;
            }
        }
    }
    Ok(builtin_result_type(&name))
}

fn builtin_result_type(name: &str) -> Option<Type> {
    let name = normalize(name);
    let integer = [
        "цел",
        "iabs",
        "sign",
        "div",
        "mod",
        "int",
        "irnd",
        "irand",
        "код",
        "юникод",
        "длин",
        "максцел",
        "время",
        "позиция",
        "поз",
        "позиция после",
        "поз после",
    ];
    let real = [
        "вещ",
        "abs",
        "sqrt",
        "sin",
        "cos",
        "tg",
        "ctg",
        "arcsin",
        "arccos",
        "arctg",
        "arcctg",
        "ln",
        "lg",
        "exp",
        "min",
        "max",
        "rand",
        "rnd",
        "максвещ",
        "re",
        "im",
    ];
    if integer.contains(&name.as_str()) {
        return Some(Type::Integer);
    }
    if real.contains(&name.as_str()) {
        return Some(Type::Real);
    }
    match name.as_str() {
        "лог" => Some(Type::Boolean),
        "символ" | "юнисимвол" => Some(Type::Character),
        "цел_в_лит" | "вещ_в_лит" | "верний регистр" | "нижний регистр" => {
            Some(Type::String)
        }
        "лит_в_цел" => Some(Type::Integer),
        "лит_в_вещ" => Some(Type::Real),
        "открыть на чтение" | "открыть на запись" | "открыть на добавление" | "консоль" => {
            Some(Type::File)
        }
        "конец файла"
        | "есть данные"
        | "можно открыть на чтение"
        | "можно открыть на запись"
        | "существует"
        | "является каталогом"
        | "создать каталог"
        | "удалить_файл"
        | "удалить_каталог" => Some(Type::Boolean),
        "полный путь" | "рабочий каталог" | "каталог программы" => {
            Some(Type::String)
        }
        "imin" | "imax" => Some(Type::Integer),
        "ждать" | "вставить" | "заменить" | "удалить" => None,
        _ => None,
    }
}

fn is_numeric_type(ty: &Type) -> bool {
    matches!(ty, Type::Integer | Type::Real | Type::Complex)
}

fn infer_binary_type(
    left: &Type,
    operator: BinaryOp,
    right: &Type,
    span: Span,
) -> Result<Type, Diagnostic> {
    use BinaryOp::*;
    if matches!(
        operator,
        Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual
    ) {
        let compatible = (is_numeric_type(left)
            && is_numeric_type(right)
            && *left != Type::Complex
            && *right != Type::Complex)
            || left == right
                && matches!(
                    left,
                    Type::Boolean | Type::Character | Type::String | Type::Color | Type::KeyCode
                )
            || matches!(
                (left, right),
                (Type::Integer, Type::KeyCode) | (Type::KeyCode, Type::Integer)
            );
        return if compatible {
            Ok(Type::Boolean)
        } else {
            Err(error(span, "values of these types cannot be compared"))
        };
    }
    match operator {
        And | Or if *left == Type::Boolean && *right == Type::Boolean => Ok(Type::Boolean),
        And | Or => Err(error(span, "logical operators require logical values")),
        Add if *left == Type::String && *right == Type::String => Ok(Type::String),
        Add if *left == Type::Character && *right == Type::Character => Ok(Type::String),
        Add | Subtract | Multiply | Divide | IntegerDivide | Modulo | Power
            if is_numeric_type(left) && is_numeric_type(right) =>
        {
            if *left == Type::Complex || *right == Type::Complex {
                if matches!(operator, Add | Subtract | Multiply | Divide) {
                    Ok(Type::Complex)
                } else {
                    Err(error(
                        span,
                        "this operator is not defined for complex values",
                    ))
                }
            } else {
                Ok(match operator {
                    Add | Subtract | Multiply
                        if *left == Type::Integer && *right == Type::Integer =>
                    {
                        Type::Integer
                    }
                    IntegerDivide => Type::Integer,
                    Modulo if *left == Type::Integer && *right == Type::Integer => Type::Integer,
                    Modulo => Type::Real,
                    _ => Type::Real,
                })
            }
        }
        _ => Err(error(span, "incompatible types for this operator")),
    }
}

fn validate_expr(
    expr: &Expr,
    declarations: &BTreeMap<String, Declaration>,
) -> Result<(), Diagnostic> {
    match expr {
        Expr::Variable(name, span) if !declarations.contains_key(name) => {
            if negated_logical_name(name)
                .and_then(|base| declarations.get(&base))
                .is_some_and(|declaration| declaration.ty == Type::Boolean)
            {
                Ok(())
            } else {
                Err(error(*span, &format!("undefined variable `{name}`")))
            }
        }
        Expr::Index(target, indices) => {
            validate_expr(target, declarations)?;
            for index in indices {
                validate_expr(index, declarations)?;
            }
            Ok(())
        }
        Expr::Slice(target, start, end) => {
            validate_expr(target, declarations)?;
            validate_expr(start, declarations)?;
            validate_expr(end, declarations)
        }
        Expr::Call(name, _) if is_string_output_procedure(name) => Err(error(
            expr_span(expr),
            format!("`{name}` is a procedure and cannot be used as a value"),
        )),
        Expr::ModuleCall(module, name, arguments) => {
            if let Some(expected) = module_expected_arity(*module, name) {
                if arguments.len() != expected {
                    return Err(error(
                        expr_span(expr),
                        format!("`{name}` expects {expected} argument(s)"),
                    ));
                }
            }
            for argument in arguments {
                validate_expr(argument, declarations)?;
            }
            Ok(())
        }
        Expr::Call(_, arguments) => {
            for arg in arguments {
                validate_expr(arg, declarations)?;
            }
            Ok(())
        }
        Expr::Unary(_, value) => validate_expr(value, declarations),
        Expr::Binary(left, _, right) => {
            validate_expr(left, declarations)?;
            validate_expr(right, declarations)
        }
        _ => Ok(()),
    }
}

fn validate_lvalue(
    target: &Expr,
    declarations: &BTreeMap<String, Declaration>,
) -> Result<(), Diagnostic> {
    match target {
        Expr::Variable(name, _) if declarations.contains_key(name) => Ok(()),
        Expr::Index(array, indices) => {
            let Expr::Variable(name, span) = array.as_ref() else {
                return Err(error(
                    expr_span(target),
                    "table target must be a table variable",
                ));
            };
            if !declarations
                .get(name)
                .is_some_and(|declaration| matches!(&declaration.ty, Type::Table { .. }))
            {
                return Err(error(*span, format!("`{name}` is not a table variable")));
            }
            for index in indices {
                validate_expr(index, declarations)?;
            }
            Ok(())
        }
        Expr::Variable(name, span) => Err(error(*span, format!("undefined variable `{name}`"))),
        _ => Err(error(
            expr_span(target),
            "target must be a variable or table element",
        )),
    }
}

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Variable(_, span) => *span,
        Expr::Index(target, _) => expr_span(target),
        _ => Span::default(),
    }
}

#[derive(Clone, Debug)]
pub struct Interpreter<H: Host> {
    host: H,
    execution_context: ExecutionContext,
    steps: usize,
    recursion: usize,
    globals: BTreeMap<String, Value>,
    last_variables: BTreeMap<String, Value>,
    stopped: bool,
    routed_input: Option<Value>,
    routed_output: Option<Value>,
}

#[derive(Clone, Debug)]
enum Flow {
    Continue,
    Exit,
    Stop,
}

struct AlgorithmInvocation {
    value: Value,
    outputs: Vec<Option<Value>>,
}

impl<H: Host> Interpreter<H> {
    pub fn new(host: H) -> Self {
        Self {
            host,
            execution_context: ExecutionContext::Student,
            steps: 0,
            recursion: 0,
            globals: BTreeMap::new(),
            last_variables: BTreeMap::new(),
            stopped: false,
            routed_input: None,
            routed_output: None,
        }
    }

    /// Sets the source section allowed to call context-restricted executor
    /// algorithms. The default is [`ExecutionContext::Student`].
    pub fn with_execution_context(mut self, context: ExecutionContext) -> Self {
        self.execution_context = context;
        self
    }

    pub fn run(&mut self, program: &Program) -> Result<Execution, RuntimeError> {
        self.steps = 0;
        self.recursion = 0;
        self.globals.clear();
        self.last_variables.clear();
        self.stopped = false;
        self.routed_input = None;
        self.routed_output = None;
        for (name, declaration) in &program.global_declarations {
            self.globals
                .insert(name.clone(), undefined_value(&declaration.ty));
        }
        let mut globals = self.globals.clone();
        for (name, declaration) in &program.global_initializers {
            if let Some(initial) = &declaration.initial {
                let value = self.evaluate(initial, &mut globals, program, Span::default())?;
                globals.insert(
                    name.clone(),
                    assign_value(value, &declaration.ty, Span::default())?,
                );
            }
        }
        self.globals = globals;
        let preamble = Algorithm {
            name: "".to_owned(),
            parameters: Vec::new(),
            result: None,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            declarations: program.global_declarations.clone(),
            body_declarations: BTreeSet::new(),
            body: program.preamble.clone(),
            span: Span::default(),
        };
        let mut globals = self.globals.clone();
        let _ = self.execute_block(&program.preamble, &preamble, program, &mut globals)?;
        self.globals = globals;
        let _result = self.run_algorithm(
            &program.main,
            Vec::new(),
            program,
            Span { line: 1, column: 1 },
        )?;
        if let Some(host) = self.host.file_host() {
            host.finish()
                .map_err(|message| runtime_error(Span { line: 1, column: 1 }, message))?;
        }
        let variables = self.last_variables.clone();
        Ok(Execution {
            variables,
            steps: self.steps,
        })
    }

    pub fn into_host(self) -> H {
        self.host
    }

    fn update_globals_from_frame(
        &mut self,
        variables: &BTreeMap<String, Value>,
        program: &Program,
    ) {
        for name in program.global_declarations.keys() {
            if let Some(value) = variables.get(name) {
                self.globals.insert(name.clone(), value.clone());
            }
        }
    }

    fn copy_globals_to_frame(&self, variables: &mut BTreeMap<String, Value>, program: &Program) {
        for name in program.global_declarations.keys() {
            if let Some(value) = self.globals.get(name) {
                variables.insert(name.clone(), value.clone());
            }
        }
    }

    fn run_algorithm(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        program: &Program,
        call_span: Span,
    ) -> Result<AlgorithmInvocation, RuntimeError> {
        if self.recursion >= 1024 {
            return Err(runtime_error(
                call_span,
                "algorithm recursion limit exceeded",
            ));
        }
        let algorithm = program
            .algorithms
            .get(&normalize(name))
            .ok_or_else(|| runtime_error(call_span, format!("unknown algorithm `{name}`")))?
            .clone();
        if arguments.len() != algorithm.parameters.len() {
            return Err(runtime_error(
                call_span,
                format!(
                    "algorithm `{name}` expects {} argument(s), got {}",
                    algorithm.parameters.len(),
                    arguments.len()
                ),
            ));
        }
        let outermost = self.recursion == 0;
        self.recursion += 1;
        let result = (|| {
            let mut variables = self.globals.clone();
            for (parameter, value) in algorithm.parameters.iter().zip(arguments) {
                let value =
                    if parameter.mode == ParameterMode::Result && matches!(value, Value::Void) {
                        Value::Void
                    } else {
                        assign_value(value, &parameter.ty, call_span)?
                    };
                variables.insert(normalize(&parameter.name), value);
            }
            for (key, declaration) in &algorithm.declarations {
                if algorithm.body_declarations.contains(key) {
                    continue;
                }
                if variables.contains_key(key) {
                    continue;
                }
                variables.insert(
                    key.clone(),
                    if key == "знач" && algorithm.result.is_some() {
                        Value::Void
                    } else {
                        undefined_value(&declaration.ty)
                    },
                );
                if let Some(initial) = &declaration.initial {
                    let value = self.evaluate(initial, &mut variables, program, call_span)?;
                    let value = assign_value(value, &declaration.ty, call_span)?;
                    variables.insert(key.clone(), value);
                }
            }
            for condition in &algorithm.preconditions {
                if !truthy(
                    self.evaluate(condition, &mut variables, program, call_span)?,
                    call_span,
                )? {
                    return Err(runtime_error(call_span, "algorithm precondition failed"));
                }
            }
            let executed = self.execute_block(&algorithm.body, &algorithm, program, &mut variables);
            self.update_globals_from_frame(&variables, program);
            if outermost {
                self.last_variables = variables.clone();
            }
            let flow = executed?;
            if !matches!(flow, Flow::Stop | Flow::Exit) {
                for condition in &algorithm.postconditions {
                    if !truthy(
                        self.evaluate(condition, &mut variables, program, call_span)?,
                        call_span,
                    )? {
                        return Err(runtime_error(call_span, "algorithm postcondition failed"));
                    }
                }
            }
            let value = if let Some(result_type) = &algorithm.result {
                let value = variables.get("знач").cloned().unwrap_or(Value::Void);
                assign_value(value, result_type, call_span)?
            } else {
                Value::Void
            };
            let outputs = algorithm
                .parameters
                .iter()
                .map(|parameter| match parameter.mode {
                    ParameterMode::Argument => None,
                    ParameterMode::Result | ParameterMode::ArgumentResult => {
                        variables.get(&normalize(&parameter.name)).cloned()
                    }
                })
                .collect();
            Ok(AlgorithmInvocation { value, outputs })
        })();
        self.recursion -= 1;
        result
    }

    fn invoke_algorithm(
        &mut self,
        name: &str,
        arguments: &[Expr],
        caller_algorithm: &Algorithm,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let callee = program
            .algorithms
            .get(&normalize(name))
            .ok_or_else(|| runtime_error(span, format!("unknown algorithm `{name}`")))?;
        if arguments.len() != callee.parameters.len() {
            return Err(runtime_error(
                span,
                format!(
                    "algorithm `{name}` expects {} argument(s), got {}",
                    callee.parameters.len(),
                    arguments.len()
                ),
            ));
        }
        let mut values = Vec::with_capacity(arguments.len());
        self.update_globals_from_frame(variables, program);
        for (parameter, argument) in callee.parameters.iter().zip(arguments) {
            let value = match parameter.mode {
                ParameterMode::Argument => self.evaluate(argument, variables, program, span)?,
                ParameterMode::ArgumentResult => {
                    self.evaluate(argument, variables, program, span)?
                }
                ParameterMode::Result => match argument {
                    Expr::Variable(_, _) | Expr::Index(_, _) => Value::Void,
                    _ => {
                        return Err(runtime_error(
                            span,
                            "result argument must be an assignable variable",
                        ));
                    }
                },
            };
            if matches!(
                parameter.mode,
                ParameterMode::Result | ParameterMode::ArgumentResult
            ) && !matches!(argument, Expr::Variable(_, _) | Expr::Index(_, _))
            {
                return Err(runtime_error(
                    span,
                    "result argument must be an assignable variable",
                ));
            }
            values.push(
                if parameter.mode == ParameterMode::Result && matches!(value, Value::Void) {
                    Value::Void
                } else {
                    assign_value(value, &parameter.ty, span)?
                },
            );
        }
        let invocation = self.run_algorithm(name, values, program, span)?;
        self.copy_globals_to_frame(variables, program);
        for ((parameter, argument), output) in callee
            .parameters
            .iter()
            .zip(arguments)
            .zip(invocation.outputs)
        {
            if let Some(output) = output {
                self.assign_target(argument, output, caller_algorithm, variables, program, span)?;
            }
            let _ = parameter;
        }
        Ok(invocation.value)
    }

    fn execute_block(
        &mut self,
        body: &[Stmt],
        algorithm: &Algorithm,
        program: &Program,
        variables: &mut BTreeMap<String, Value>,
    ) -> Result<Flow, RuntimeError> {
        for stmt in body {
            if self.stopped {
                return Ok(Flow::Stop);
            }
            self.tick(statement_span(stmt))?;
            let flow = match stmt {
                Stmt::Declare { declarations, span } => {
                    for (name, declaration) in declarations {
                        variables.insert(name.clone(), undefined_value(&declaration.ty));
                        if let Some(initial) = &declaration.initial {
                            let value = self.evaluate(initial, variables, program, *span)?;
                            variables
                                .insert(name.clone(), assign_value(value, &declaration.ty, *span)?);
                        }
                    }
                    Flow::Continue
                }
                Stmt::Assign {
                    target,
                    value,
                    span,
                } => {
                    let result = self.evaluate(value, variables, program, *span)?;
                    if let Expr::Variable(name, _) = target {
                        if normalize(name) == algorithm.name {
                            if let Some(ty) = &algorithm.result {
                                variables
                                    .insert("знач".to_owned(), assign_value(result, ty, *span)?);
                                continue;
                            }
                        }
                        if algorithm
                            .declarations
                            .get(name)
                            .or_else(|| program.global_declarations.get(name))
                            .is_some_and(|declaration| declaration.constant)
                        {
                            return Err(runtime_error(
                                *span,
                                format!("cannot assign to constant `{name}`"),
                            ));
                        }
                    }
                    self.assign_target(target, result, algorithm, variables, program, *span)?;
                    Flow::Continue
                }
                Stmt::Input { targets, span } => {
                    let explicit_file = match targets.first() {
                        Some(InputPart::Target(target))
                            if self
                                .target_type(target, algorithm, variables, program, *span)?
                                == Type::File =>
                        {
                            match self.evaluate(target, variables, program, *span)? {
                                Value::File(handle) => Some(Value::File(handle)),
                                _ => {
                                    return Err(runtime_error(
                                        *span,
                                        "file input target does not contain an open file",
                                    ));
                                }
                            }
                        }
                        _ => None,
                    };
                    let file_handle = explicit_file.clone().or_else(|| self.routed_input.clone());
                    let input_parts = if explicit_file.is_some() {
                        &targets[1..]
                    } else {
                        targets.as_slice()
                    };
                    let mut input_groups = Vec::new();
                    let mut current_group = Vec::new();
                    for part in input_parts {
                        match part {
                            InputPart::Target(target) => current_group.push(target.clone()),
                            InputPart::Newline if !current_group.is_empty() => {
                                input_groups.push(std::mem::take(&mut current_group));
                            }
                            InputPart::Newline => {}
                        }
                    }
                    if !current_group.is_empty() {
                        input_groups.push(current_group);
                    }
                    for group in input_groups {
                        let raw_value = if let Some(handle) = &file_handle {
                            self.host
                                .file_host()
                                .ok_or_else(|| {
                                    runtime_error(*span, "file input needs a configured file host")
                                })?
                                .file_request(FileRequest::Read {
                                    handle: handle.clone(),
                                })
                                .map_err(|message| runtime_error(*span, message))?
                        } else {
                            Value::String(
                                self.host
                                    .input()
                                    .map_err(|message| runtime_error(*span, message))?,
                            )
                        };
                        let values = split_input_values(&raw_value.to_string())
                            .map_err(|message| runtime_error(*span, message))?;
                        if values.len() < group.len() {
                            return Err(runtime_error(*span, "input line has too few values"));
                        }
                        for (target, raw) in group.into_iter().zip(values) {
                            let ty =
                                self.target_type(&target, algorithm, variables, program, *span)?;
                            let value = parse_input(&raw, &ty, *span)?;
                            self.assign_target(
                                &target, value, algorithm, variables, program, *span,
                            )?;
                        }
                    }
                    Flow::Continue
                }
                Stmt::Output { values, span } => {
                    let rendered = values
                        .iter()
                        .map(|part| match part {
                            OutputPart::Value(expr) => {
                                self.evaluate(expr, variables, program, *span).map(Some)
                            }
                            OutputPart::Newline => Ok(None),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let explicit_file = rendered.first().and_then(|value| match value {
                        Some(Value::File(handle)) => Some(Value::File(handle.clone())),
                        _ => None,
                    });
                    let routed_file = explicit_file.clone().or_else(|| self.routed_output.clone());
                    if let Some(handle) = routed_file {
                        let start = usize::from(explicit_file.is_some());
                        let text = render_output(&rendered[start..]);
                        self.host
                            .file_host()
                            .ok_or_else(|| {
                                runtime_error(*span, "file output needs a configured file host")
                            })?
                            .file_request(FileRequest::Write { handle, text })
                            .map_err(|message| runtime_error(*span, message))?;
                    } else {
                        let text = render_output(&rendered);
                        self.host
                            .output(&text)
                            .map_err(|message| runtime_error(*span, message))?;
                    }
                    Flow::Continue
                }
                Stmt::Newline { span } => {
                    self.host
                        .output("\n")
                        .map_err(|message| runtime_error(*span, message))?;
                    Flow::Continue
                }
                Stmt::If {
                    condition,
                    yes,
                    no,
                    span,
                } => {
                    let condition =
                        truthy(self.evaluate(condition, variables, program, *span)?, *span)?;
                    self.execute_block(
                        if condition { yes } else { no },
                        algorithm,
                        program,
                        variables,
                    )?
                }
                Stmt::Choose {
                    value,
                    branches,
                    otherwise,
                    span,
                } => {
                    let value = value
                        .as_ref()
                        .map(|value| self.evaluate(value, variables, program, *span))
                        .transpose()?;
                    let mut selected = None;
                    for (labels, body) in branches {
                        for label in labels {
                            let label = self.evaluate(label, variables, program, *span)?;
                            let matches = if let Some(value) = &value {
                                values_equal(value, &label)
                            } else {
                                truthy(label, *span)?
                            };
                            if matches {
                                selected = Some(body);
                                break;
                            }
                        }
                        if selected.is_some() {
                            break;
                        }
                    }
                    self.execute_block(
                        selected.unwrap_or(otherwise),
                        algorithm,
                        program,
                        variables,
                    )?
                }
                Stmt::While {
                    condition,
                    body,
                    span,
                } => {
                    loop {
                        self.tick(*span)?;
                        if !truthy(self.evaluate(condition, variables, program, *span)?, *span)? {
                            break;
                        }
                        match self.execute_block(body, algorithm, program, variables)? {
                            Flow::Exit => break,
                            Flow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Flow::Continue
                }
                Stmt::For {
                    name,
                    start,
                    end,
                    step,
                    body,
                    span,
                } => {
                    let start = integer(self.evaluate(start, variables, program, *span)?, *span)?;
                    let end = integer(self.evaluate(end, variables, program, *span)?, *span)?;
                    let step = integer(self.evaluate(step, variables, program, *span)?, *span)?;
                    if step == 0 {
                        return Err(runtime_error(*span, "counted-loop step cannot be zero"));
                    }
                    let mut value = start;
                    while if step > 0 { value <= end } else { value >= end } {
                        self.tick(*span)?;
                        variables.insert(name.clone(), Value::Integer(value));
                        match self.execute_block(body, algorithm, program, variables)? {
                            Flow::Exit => break,
                            Flow::Continue => {}
                            flow => return Ok(flow),
                        }
                        let Some(next) = value.checked_add(step) else {
                            return Err(runtime_error(*span, "counted-loop value overflowed"));
                        };
                        if !(i32::MIN..=i32::MAX).contains(&next) {
                            return Err(runtime_error(
                                *span,
                                "counted-loop value is outside the Kumir integer range",
                            ));
                        }
                        value = next;
                    }
                    Flow::Continue
                }
                Stmt::Repeat { count, body, span } => {
                    let count = integer(self.evaluate(count, variables, program, *span)?, *span)?;
                    for _ in 0..count.max(0) {
                        self.tick(*span)?;
                        match self.execute_block(body, algorithm, program, variables)? {
                            Flow::Exit => break,
                            Flow::Continue => {}
                            flow => return Ok(flow),
                        }
                    }
                    Flow::Continue
                }
                Stmt::RepeatUntil {
                    body,
                    condition,
                    span,
                } => loop {
                    self.tick(*span)?;
                    match self.execute_block(body, algorithm, program, variables)? {
                        Flow::Exit => break Flow::Continue,
                        Flow::Continue => {}
                        flow => return Ok(flow),
                    }
                    if truthy(self.evaluate(condition, variables, program, *span)?, *span)? {
                        break Flow::Continue;
                    }
                },
                Stmt::Loop { body, span } => loop {
                    self.tick(*span)?;
                    match self.execute_block(body, algorithm, program, variables)? {
                        Flow::Exit => break Flow::Continue,
                        Flow::Continue => {}
                        flow => return Ok(flow),
                    }
                },
                Stmt::Exit { .. } => return Ok(Flow::Exit),
                Stmt::Assert { condition, span } => {
                    if !truthy(self.evaluate(condition, variables, program, *span)?, *span)? {
                        return Err(runtime_error(*span, "assertion failed"));
                    }
                    Flow::Continue
                }
                Stmt::Stop { .. } => {
                    self.stopped = true;
                    return Ok(Flow::Stop);
                }
                Stmt::Pause { span } => {
                    self.host
                        .pause()
                        .map_err(|message| runtime_error(*span, message))?;
                    Flow::Continue
                }
                Stmt::Call {
                    name,
                    arguments,
                    span,
                } => {
                    if self
                        .call_by_reference_builtin(
                            name,
                            arguments,
                            Some(algorithm),
                            variables,
                            program,
                            *span,
                        )?
                        .is_some()
                    {
                        Flow::Continue
                    } else if program.algorithms.contains_key(&normalize(name)) {
                        let _ = self.invoke_algorithm(
                            name, arguments, algorithm, variables, program, *span,
                        )?;
                        Flow::Continue
                    } else {
                        let values = arguments
                            .iter()
                            .map(|arg| self.evaluate(arg, variables, program, *span))
                            .collect::<Result<Vec<_>, _>>()?;
                        if program.modules.contains(&Module::Files) && is_file_builtin(name) {
                            let _ = self.call_file(name, values, *span)?;
                        } else {
                            let _ = self.call_builtin(name, values, program, *span)?;
                        }
                        Flow::Continue
                    }
                }
                Stmt::ModuleCall {
                    module,
                    name,
                    arguments,
                    span,
                } => {
                    self.call_module_statement(
                        *module, name, arguments, algorithm, variables, program, *span,
                    )?;
                    Flow::Continue
                }
                Stmt::File { request, span } => {
                    let _ = self.file_request(request, variables, program, *span)?;
                    Flow::Continue
                }
            };
            match flow {
                Flow::Continue => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Continue)
    }

    fn evaluate(
        &mut self,
        expr: &Expr,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable(name, location) => {
                if let Some(value) = variables.get(name) {
                    if matches!(value, Value::Void) {
                        return Err(runtime_error(
                            *location,
                            format!("`{name}` has not been assigned a value"),
                        ));
                    }
                    return Ok(value.clone());
                }
                if let Some(base) = negated_logical_name(name) {
                    if let Some(Value::Boolean(value)) = variables.get(&base) {
                        return Ok(Value::Boolean(!value));
                    }
                    return Err(runtime_error(
                        *location,
                        format!("`{name}` does not negate a logical variable"),
                    ));
                }
                Err(runtime_error(
                    *location,
                    format!("undefined variable `{name}`"),
                ))
            }
            Expr::Index(target, indices) => {
                let array = self.evaluate(target, variables, program, span)?;
                let indices = indices
                    .iter()
                    .map(|i| {
                        self.evaluate(i, variables, program, span)
                            .and_then(|v| integer(v, span))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let value = get_index(array, &indices, span)?;
                if matches!(value, Value::Void) {
                    return Err(runtime_error(
                        span,
                        "table element has not been assigned a value",
                    ));
                }
                Ok(value)
            }
            Expr::Slice(target, start, end) => {
                let target = self.evaluate(target, variables, program, span)?;
                let start = integer(self.evaluate(start, variables, program, span)?, span)?;
                let end = integer(self.evaluate(end, variables, program, span)?, span)?;
                let Value::String(text) = target else {
                    return Err(runtime_error(span, "string slice requires a text value"));
                };
                let characters = text.chars().collect::<Vec<_>>();
                if start < 1 || end < start || end as usize > characters.len() {
                    return Err(runtime_error(span, "string slice is outside its bounds"));
                }
                Ok(Value::String(
                    characters[(start as usize - 1)..end as usize]
                        .iter()
                        .collect(),
                ))
            }
            Expr::Call(name, arguments) => {
                if let Some(value) =
                    self.call_by_reference_builtin(name, arguments, None, variables, program, span)?
                {
                    return Ok(value);
                }
                let values = arguments
                    .iter()
                    .map(|arg| self.evaluate(arg, variables, program, span))
                    .collect::<Result<Vec<_>, _>>()?;
                if program.algorithms.contains_key(&normalize(name)) {
                    let callee = &program.algorithms[&normalize(name)];
                    if callee
                        .parameters
                        .iter()
                        .any(|parameter| parameter.mode != ParameterMode::Argument)
                    {
                        return Err(runtime_error(
                            span,
                            "result parameters of user algorithms require a statement call",
                        ));
                    }
                    self.update_globals_from_frame(variables, program);
                    let invocation = self.run_algorithm(name, values, program, span)?;
                    self.copy_globals_to_frame(variables, program);
                    Ok(invocation.value)
                } else if program.modules.contains(&Module::Files) && is_file_builtin(name) {
                    self.call_file(name, values, span)
                } else {
                    self.call_builtin(name, values, program, span)
                }
            }
            Expr::ModuleCall(module, name, arguments) => {
                let values = arguments
                    .iter()
                    .map(|arg| self.evaluate(arg, variables, program, span))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call_module(*module, name, values, program, span)
            }
            Expr::Unary(op, value) => {
                let value = self.evaluate(value, variables, program, span)?;
                match (op, value) {
                    (UnaryOp::Negate, Value::Integer(value)) => value
                        .checked_neg()
                        .map(Value::Integer)
                        .ok_or_else(|| runtime_error(span, "integer negation overflowed")),
                    (UnaryOp::Negate, Value::Real(value)) => Ok(Value::Real(-value)),
                    (UnaryOp::Negate, Value::Complex(value)) => Ok(Value::Complex(Complex {
                        real: -value.real,
                        imaginary: -value.imaginary,
                    })),
                    (UnaryOp::Positive, value @ (Value::Integer(_) | Value::Real(_))) => Ok(value),
                    (UnaryOp::Positive, value @ Value::Complex(_)) => Ok(value),
                    (UnaryOp::Not, Value::Boolean(value)) => Ok(Value::Boolean(!value)),
                    (UnaryOp::Not, _) => Err(runtime_error(span, "`не` requires a logical value")),
                    _ => Err(runtime_error(span, "unary sign requires a numeric value")),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(left, variables, program, span)?;
                let right = self.evaluate(right, variables, program, span)?;
                eval_binary(left, *op, right, span)
            }
        }
    }

    fn call_module(
        &mut self,
        module: Module,
        name: &str,
        arguments: Vec<Value>,
        program: &Program,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        self.ensure_module_context(module, name, span)?;
        if !program.modules.contains(&module) {
            return Err(runtime_error(
                span,
                format!("module `{}` is not enabled", module.canonical_name()),
            ));
        }
        let signature = module_signature(module, name).ok_or_else(|| {
            runtime_error(
                span,
                format!(
                    "unknown or unavailable command `{name}` in module `{}`",
                    module.canonical_name()
                ),
            )
        })?;
        if arguments.len() != signature.arguments.len() {
            return Err(runtime_error(
                span,
                format!("`{name}` expects {} argument(s)", signature.arguments.len()),
            ));
        }
        if module_argument_modes(module, name, arguments.len())
            .iter()
            .any(|mode| *mode != ModuleArgumentMode::Argument)
        {
            return Err(runtime_error(
                span,
                "module output arguments cannot be used in an expression call",
            ));
        }
        if matches!(&signature.result, ModuleResultType::Procedure) {
            return Err(runtime_error(
                span,
                format!("`{name}` is a procedure and cannot be used as a value"),
            ));
        }
        for (argument, expected) in arguments.iter().zip(&signature.arguments) {
            if let Some(expected) = expected {
                if !runtime_value_matches(expected, argument) {
                    return Err(runtime_error(
                        span,
                        format!(
                            "argument to module function `{name}` has an incompatible type; expected `{}`",
                            type_name(expected)
                        ),
                    ));
                }
            }
        }
        let call = ModuleCall {
            module,
            name: normalize(name),
            arguments: arguments
                .into_iter()
                .map(|value| ModuleArgument {
                    value,
                    mode: ModuleArgumentMode::Argument,
                })
                .collect(),
        };
        let result = self
            .host
            .module_host()
            .ok_or_else(|| {
                runtime_error(
                    span,
                    format!(
                        "module `{}` needs a configured host",
                        module.canonical_name()
                    ),
                )
            })?
            .call(call)
            .map_err(|message| runtime_error(span, message))?;
        if !result.arguments.is_empty() {
            return Err(runtime_error(
                span,
                "module function returned output arguments for an expression call",
            ));
        }
        if matches!(&result.value, Value::Void) {
            return Err(runtime_error(
                span,
                format!("module function `{name}` did not return a value"),
            ));
        }
        validate_module_result_value(&signature, name, &result.value, span)?;
        Ok(result.value)
    }

    fn call_module_statement(
        &mut self,
        module: Module,
        name: &str,
        arguments: &[Expr],
        algorithm: &Algorithm,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<(), RuntimeError> {
        self.ensure_module_context(module, name, span)?;
        if !program.modules.contains(&module) {
            return Err(runtime_error(
                span,
                format!("module `{}` is not enabled", module.canonical_name()),
            ));
        }
        let signature = module_signature(module, name).ok_or_else(|| {
            runtime_error(
                span,
                format!(
                    "unknown or unavailable command `{name}` in module `{}`",
                    module.canonical_name()
                ),
            )
        })?;
        if arguments.len() != signature.arguments.len() {
            return Err(runtime_error(
                span,
                format!("`{name}` expects {} argument(s)", signature.arguments.len()),
            ));
        }
        let modes = module_argument_modes(module, name, arguments.len());
        let mut call_arguments = Vec::with_capacity(arguments.len());
        for ((argument, expected), mode) in arguments.iter().zip(&signature.arguments).zip(&modes) {
            let value = match mode {
                ModuleArgumentMode::Argument | ModuleArgumentMode::ArgumentResult => {
                    self.evaluate(argument, variables, program, span)?
                }
                ModuleArgumentMode::Result => match argument {
                    Expr::Variable(_, _) | Expr::Index(_, _) => Value::Void,
                    _ => {
                        return Err(runtime_error(
                            span,
                            "module result argument must be an assignable variable",
                        ));
                    }
                },
            };
            if *mode != ModuleArgumentMode::Result
                && expected
                    .as_ref()
                    .is_some_and(|expected| !runtime_value_matches(expected, &value))
            {
                return Err(runtime_error(
                    span,
                    format!("argument to module command `{name}` has an incompatible type"),
                ));
            }
            if *mode != ModuleArgumentMode::Argument
                && !matches!(argument, Expr::Variable(_, _) | Expr::Index(_, _))
            {
                return Err(runtime_error(
                    span,
                    "module result argument must be an assignable variable",
                ));
            }
            call_arguments.push(ModuleArgument { value, mode: *mode });
        }
        let result = self
            .host
            .module_host()
            .ok_or_else(|| {
                runtime_error(
                    span,
                    format!(
                        "module `{}` needs a configured host",
                        module.canonical_name()
                    ),
                )
            })?
            .call(ModuleCall {
                module,
                name: normalize(name),
                arguments: call_arguments,
            })
            .map_err(|message| runtime_error(span, message))?;
        validate_module_result_value(&signature, name, &result.value, span)?;
        if result.arguments.is_empty()
            && modes
                .iter()
                .any(|mode| *mode != ModuleArgumentMode::Argument)
        {
            return Err(runtime_error(span, "module host omitted result arguments"));
        }
        if !result.arguments.is_empty() && result.arguments.len() != arguments.len() {
            return Err(runtime_error(
                span,
                "module host returned the wrong number of arguments",
            ));
        }
        if !result.arguments.is_empty() {
            for (index, (mode, output)) in modes.iter().zip(result.arguments).enumerate() {
                if *mode == ModuleArgumentMode::Argument && output.is_some() {
                    return Err(runtime_error(
                        span,
                        "module host changed an input-only argument",
                    ));
                }
                if *mode != ModuleArgumentMode::Argument {
                    let output = output.ok_or_else(|| {
                        runtime_error(span, "module host omitted a result argument value")
                    })?;
                    if signature.arguments[index]
                        .as_ref()
                        .is_some_and(|expected| !runtime_value_matches(expected, &output))
                    {
                        return Err(runtime_error(
                            span,
                            format!(
                                "module output argument {} has an incompatible type",
                                index + 1
                            ),
                        ));
                    }
                    self.assign_target(
                        &arguments[index],
                        output,
                        algorithm,
                        variables,
                        program,
                        span,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn call_builtin(
        &mut self,
        name: &str,
        args: Vec<Value>,
        program: &Program,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        match normalize(name).as_str() {
            "время" if args.is_empty() => self
                .host
                .time_millis()
                .map(Value::Integer)
                .map_err(|message| runtime_error(span, message)),
            "ждать" if args.len() == 1 => self
                .host
                .wait_millis(integer(args[0].clone(), span)?)
                .map(|()| Value::Void)
                .map_err(|message| runtime_error(span, message)),
            "rnd" if args.len() == 1 => {
                let upper = number_f64(args[0].clone(), span)?;
                self.host
                    .random_number(0.0, upper)
                    .map(Value::Real)
                    .map_err(|message| runtime_error(span, message))
            }
            "rand" if args.len() == 2 => {
                let low = number_f64(args[0].clone(), span)?;
                let high = number_f64(args[1].clone(), span)?;
                self.host
                    .random_number(low, high)
                    .map(Value::Real)
                    .map_err(|message| runtime_error(span, message))
            }
            "irnd" if args.len() == 1 => {
                let upper = integer(args[0].clone(), span)?;
                let value = self
                    .host
                    .random_number(0.0, upper as f64 + 1.0)
                    .map_err(|message| runtime_error(span, message))?
                    .floor();
                checked_real_integer(value, span)
            }
            "irand" if args.len() == 2 => {
                let low = integer(args[0].clone(), span)?;
                let high = integer(args[1].clone(), span)?;
                let value = self
                    .host
                    .random_number(low as f64, high as f64 + 1.0)
                    .map_err(|message| runtime_error(span, message))?
                    .floor();
                checked_real_integer(value, span)
            }
            _ if is_string_module_builtin(name) && !program.modules.contains(&Module::Strings) => {
                Err(runtime_error(
                    span,
                    "string utility requires the Строки module in this profile",
                ))
            }
            _ => builtin(name, args, program.profile, span),
        }
    }

    fn call_file(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let normalized = normalize(name);
        let route_target = if matches!(normalized.as_str(), "назначить ввод" | "назначить вывод")
        {
            arguments
                .first()
                .cloned()
                .filter(|value| !value.to_string().is_empty())
        } else {
            None
        };
        let result = self
            .host
            .file_host()
            .ok_or_else(|| runtime_error(span, "file operation needs a configured file host"))?
            .file_request(FileRequest::Call {
                name: normalized.clone(),
                arguments,
            })
            .map_err(|message| runtime_error(span, message))?;
        let result = if let Some(expected) = builtin_result_type(&normalized) {
            assign_value(result, &expected, span)?
        } else if matches!(result, Value::Void) {
            result
        } else {
            return Err(runtime_error(
                span,
                format!("file host returned an unexpected value for `{normalized}`"),
            ));
        };
        if matches!(normalized.as_str(), "назначить ввод" | "назначить вывод")
        {
            if normalized == "назначить ввод" {
                self.routed_input = route_target;
            } else {
                self.routed_output = route_target;
            }
        }
        Ok(result)
    }

    fn call_by_reference_builtin(
        &mut self,
        name: &str,
        arguments: &[Expr],
        algorithm: Option<&Algorithm>,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<Option<Value>, RuntimeError> {
        let name = normalize(name);
        if is_string_module_builtin(&name) && !program.modules.contains(&Module::Strings) {
            return Err(runtime_error(
                span,
                "string utility requires the Строки module in this profile",
            ));
        }
        if matches!(name.as_str(), "лит_в_цел" | "лит_в_вещ") && arguments.len() == 2
        {
            let source = self
                .evaluate(&arguments[0], variables, program, span)?
                .to_string();
            let (value, success) = if name == "лит_в_цел" {
                match source.trim().parse::<i32>() {
                    Ok(v) => (Value::Integer(v), true),
                    Err(_) => (Value::Integer(0), false),
                }
            } else {
                match source.trim().parse::<f64>() {
                    Ok(v) => (Value::Real(v), true),
                    Err(_) => (Value::Real(0.0), false),
                }
            };
            let Expr::Variable(target, location) = &arguments[1] else {
                return Err(runtime_error(span, "result parameter must be a variable"));
            };
            let Some(old) = variables.get(target) else {
                return Err(runtime_error(
                    *location,
                    format!("undefined variable `{target}`"),
                ));
            };
            if !matches!(old, Value::Boolean(_)) {
                return Err(runtime_error(
                    *location,
                    "conversion result parameter must have logical type",
                ));
            }
            variables.insert(target.clone(), Value::Boolean(success));
            return Ok(Some(value));
        }
        if name == "вставить" && arguments.len() == 3 {
            let fragment = self
                .evaluate(&arguments[0], variables, program, span)?
                .to_string();
            let target = &arguments[1];
            let current = match self.evaluate(target, variables, program, span)? {
                Value::String(value) => value,
                _ => return Err(runtime_error(span, "string procedure output must be text")),
            };
            let position = integer(
                self.evaluate(&arguments[2], variables, program, span)?,
                span,
            )?;
            let updated = builtin(
                "вставить",
                vec![
                    Value::String(fragment),
                    Value::String(current),
                    Value::Integer(position),
                ],
                program.profile,
                span,
            )?;
            let algorithm = algorithm
                .ok_or_else(|| runtime_error(span, "string mutation requires a statement call"))?;
            self.assign_target(target, updated, algorithm, variables, program, span)?;
            return Ok(Some(Value::Void));
        }
        if name == "заменить" && arguments.len() == 4 {
            let target = &arguments[0];
            let current = match self.evaluate(target, variables, program, span)? {
                Value::String(value) => value,
                _ => return Err(runtime_error(span, "string procedure output must be text")),
            };
            let old = self
                .evaluate(&arguments[1], variables, program, span)?
                .to_string();
            let new = self
                .evaluate(&arguments[2], variables, program, span)?
                .to_string();
            let every = truthy(
                self.evaluate(&arguments[3], variables, program, span)?,
                span,
            )?;
            let updated = Value::String(if every {
                current.replace(&old, &new)
            } else {
                current.replacen(&old, &new, 1)
            });
            let algorithm = algorithm
                .ok_or_else(|| runtime_error(span, "string mutation requires a statement call"))?;
            self.assign_target(target, updated, algorithm, variables, program, span)?;
            return Ok(Some(Value::Void));
        }
        if name == "удалить" && arguments.len() == 3 {
            let target = &arguments[0];
            let current = match self.evaluate(target, variables, program, span)? {
                Value::String(value) => value,
                _ => return Err(runtime_error(span, "string procedure output must be text")),
            };
            let start = self.evaluate(&arguments[1], variables, program, span)?;
            let count = self.evaluate(&arguments[2], variables, program, span)?;
            let updated = builtin(
                "удалить",
                vec![Value::String(current), start, count],
                program.profile,
                span,
            )?;
            let algorithm = algorithm
                .ok_or_else(|| runtime_error(span, "string mutation requires a statement call"))?;
            self.assign_target(target, updated, algorithm, variables, program, span)?;
            return Ok(Some(Value::Void));
        }
        Ok(None)
    }

    fn file_request(
        &mut self,
        request: &FileExpr,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<Value, RuntimeError> {
        let evaluated = |expr: &Expr, variables: &mut BTreeMap<String, Value>, this: &mut Self| {
            this.evaluate(expr, variables, program, span)
        };
        let FileExpr::Call(name, arguments) = request;
        let values = arguments
            .iter()
            .map(|arg| evaluated(arg, variables, self))
            .collect::<Result<Vec<_>, _>>()?;
        self.call_file(name, values, span)
    }

    fn target_type(
        &self,
        target: &Expr,
        algorithm: &Algorithm,
        _variables: &BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<Type, RuntimeError> {
        match target {
            Expr::Variable(name, _) => algorithm
                .declarations
                .get(name)
                .or_else(|| program.global_declarations.get(name))
                .map(|d| d.ty.clone())
                .ok_or_else(|| runtime_error(span, format!("undefined variable `{name}`"))),
            Expr::Index(array, indices) => {
                let Expr::Variable(name, _) = array.as_ref() else {
                    return Err(runtime_error(
                        span,
                        "indexed target must be a table variable",
                    ));
                };
                let declaration = algorithm
                    .declarations
                    .get(name)
                    .or_else(|| program.global_declarations.get(name))
                    .ok_or_else(|| runtime_error(span, format!("undefined variable `{name}`")))?;
                let Type::Table { element, bounds } = &declaration.ty else {
                    return Err(runtime_error(span, "indexed target is not a table"));
                };
                if indices.len() > bounds.len() {
                    return Err(runtime_error(span, "too many table indices"));
                }
                if indices.len() == bounds.len() {
                    Ok((**element).clone())
                } else {
                    Ok(Type::Table {
                        element: element.clone(),
                        bounds: bounds[indices.len()..].to_vec(),
                    })
                }
            }
            _ => Err(runtime_error(
                span,
                "input target must be a variable or table element",
            )),
        }
    }

    fn assign_target(
        &mut self,
        target: &Expr,
        value: Value,
        algorithm: &Algorithm,
        variables: &mut BTreeMap<String, Value>,
        program: &Program,
        span: Span,
    ) -> Result<(), RuntimeError> {
        match target {
            Expr::Variable(name, _) => {
                let declaration = algorithm
                    .declarations
                    .get(name)
                    .or_else(|| program.global_declarations.get(name))
                    .ok_or_else(|| runtime_error(span, format!("undefined variable `{name}`")))?;
                if declaration.constant {
                    return Err(runtime_error(
                        span,
                        format!("cannot assign to constant `{name}`"),
                    ));
                }
                variables.insert(name.clone(), assign_value(value, &declaration.ty, span)?);
                Ok(())
            }
            Expr::Index(array_expr, index_exprs) => {
                let Expr::Variable(name, _) = array_expr.as_ref() else {
                    return Err(runtime_error(
                        span,
                        "table assignment target must be a table variable",
                    ));
                };
                let declaration = algorithm
                    .declarations
                    .get(name)
                    .or_else(|| program.global_declarations.get(name))
                    .ok_or_else(|| runtime_error(span, format!("undefined variable `{name}`")))?;
                if declaration.constant {
                    return Err(runtime_error(
                        span,
                        format!("cannot assign to constant `{name}`"),
                    ));
                }
                let mut indices = Vec::new();
                for index in index_exprs {
                    indices.push(integer(
                        self.evaluate(index, variables, program, span)?,
                        span,
                    )?);
                }
                let Some(Value::Table(mut table)) = variables.remove(name) else {
                    return Err(runtime_error(span, format!("`{name}` is not a table")));
                };
                let result = set_index(&mut table, &indices, value, span);
                variables.insert(name.clone(), Value::Table(table));
                result
            }
            _ => Err(runtime_error(
                span,
                "assignment target must be a variable or table element",
            )),
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

    fn ensure_module_context(
        &self,
        module: Module,
        name: &str,
        span: Span,
    ) -> Result<(), RuntimeError> {
        let name = normalize(name);
        let teacher_only = match module {
            Module::Robot | Module::IsometricRobot => name.starts_with("@@"),
            Module::Draftsman => name == "@есть отрезок в области",
            _ => false,
        };
        if teacher_only && self.execution_context != ExecutionContext::HiddenTeacherCode {
            return Err(runtime_error(
                span,
                format!(
                    "`{name}` is available only in hidden teacher code for module `{}`",
                    module.canonical_name()
                ),
            ));
        }
        Ok(())
    }
}

fn is_builtin(name: &str, profile: Profile, modules: &[Module]) -> bool {
    let name = normalize(name);
    let common = [
        "цел",
        "вещ",
        "лог",
        "abs",
        "iabs",
        "sign",
        "sqrt",
        "sin",
        "cos",
        "tg",
        "ctg",
        "arcsin",
        "arccos",
        "arctg",
        "arcctg",
        "ln",
        "lg",
        "exp",
        "min",
        "max",
        "imin",
        "imax",
        "div",
        "mod",
        "int",
        "rnd",
        "rand",
        "irnd",
        "irand",
        "максцел",
        "максвещ",
        "длин",
        "код",
        "юникод",
        "символ",
        "юнисимвол",
        "цел_в_лит",
        "вещ_в_лит",
        "лит_в_вещ",
        "лит_в_цел",
        "время",
        "ждать",
    ];
    common.contains(&name.as_str())
        || (modules.contains(&Module::ComplexNumbers) && matches!(name.as_str(), "re" | "im"))
        || (modules.contains(&Module::Strings) && is_string_module_builtin(&name))
        || (profile == Profile::HighGrade && is_string_module_builtin(&name))
}

fn is_string_module_builtin(name: &str) -> bool {
    [
        "верний регистр",
        "нижний регистр",
        "позиция",
        "поз",
        "позиция после",
        "поз после",
        "вставить",
        "заменить",
        "удалить",
    ]
    .contains(&normalize(name).as_str())
}

fn builtin(
    name: &str,
    args: Vec<Value>,
    _profile: Profile,
    span: Span,
) -> Result<Value, RuntimeError> {
    let name = normalize(name);
    let arity = |expected: usize| {
        if args.len() == expected {
            Ok(())
        } else {
            Err(runtime_error(
                span,
                format!(
                    "`{name}` expects {expected} argument(s), got {}",
                    args.len()
                ),
            ))
        }
    };
    let numeric = |index: usize| -> Result<f64, RuntimeError> {
        args.get(index)
            .cloned()
            .ok_or_else(|| runtime_error(span, "missing argument"))
            .and_then(|v| number_f64(v, span))
    };
    match name.as_str() {
        "цел" => {
            if args.len() == 2 {
                return fallback_integer(args[0].clone(), args[1].clone(), span);
            }
            arity(1)?;
            checked_real_integer(numeric(0)?.trunc(), span)
        }
        "вещ" => {
            if args.len() == 2 {
                return fallback_real(args[0].clone(), args[1].clone(), span);
            }
            arity(1)?;
            Ok(Value::Real(numeric(0)?))
        }
        "лог" => {
            arity(2)?;
            match normalize(&args[0].to_string()).as_str() {
                "да" | "1" | "истина" => Ok(Value::Boolean(true)),
                "нет" | "0" | "ложь" => Ok(Value::Boolean(false)),
                _ => assign_value(args[1].clone(), &Type::Boolean, span),
            }
        }
        "abs" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.abs()))
        }
        "iabs" => {
            arity(1)?;
            integer(args[0].clone(), span)?
                .checked_abs()
                .map(Value::Integer)
                .ok_or_else(|| runtime_error(span, "integer absolute value overflowed"))
        }
        "sign" => {
            arity(1)?;
            let v = numeric(0)?;
            Ok(Value::Integer(if v < 0.0 {
                -1
            } else if v > 0.0 {
                1
            } else {
                0
            }))
        }
        "min" | "max" => {
            arity(2)?;
            Ok(Value::Real(if name == "min" {
                numeric(0)?.min(numeric(1)?)
            } else {
                numeric(0)?.max(numeric(1)?)
            }))
        }
        "imin" | "imax" => {
            arity(2)?;
            let (a, b) = (
                integer(args[0].clone(), span)?,
                integer(args[1].clone(), span)?,
            );
            Ok(Value::Integer(if name == "imin" {
                a.min(b)
            } else {
                a.max(b)
            }))
        }
        "div" | "mod" => {
            arity(2)?;
            let (a, b) = (
                integer(args[0].clone(), span)?,
                integer(args[1].clone(), span)?,
            );
            if b == 0 {
                return Err(runtime_error(span, "division by zero"));
            };
            let result = if name == "div" {
                a.checked_div(b)
            } else {
                a.checked_rem(b)
            };
            result
                .map(Value::Integer)
                .ok_or_else(|| runtime_error(span, "integer division is outside the Kumir range"))
        }
        "int" => {
            arity(1)?;
            checked_real_integer(numeric(0)?.floor(), span)
        }
        "sqrt" => {
            arity(1)?;
            let v = numeric(0)?;
            if v < 0.0 {
                Err(runtime_error(span, "square root of a negative value"))
            } else {
                Ok(Value::Real(v.sqrt()))
            }
        }
        "sin" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.sin()))
        }
        "cos" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.cos()))
        }
        "tg" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.tan()))
        }
        "ctg" => {
            arity(1)?;
            let v = numeric(0)?.tan();
            if v == 0.0 {
                Err(runtime_error(
                    span,
                    "cotangent is undefined at this argument",
                ))
            } else {
                Ok(Value::Real(1.0 / v))
            }
        }
        "arcsin" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.asin()))
        }
        "arccos" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.acos()))
        }
        "arctg" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.atan()))
        }
        "arcctg" => {
            arity(1)?;
            Ok(Value::Real(
                std::f64::consts::FRAC_PI_2 - numeric(0)?.atan(),
            ))
        }
        "ln" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.ln()))
        }
        "lg" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.log10()))
        }
        "exp" => {
            arity(1)?;
            Ok(Value::Real(numeric(0)?.exp()))
        }
        "длин" => {
            arity(1)?;
            match &args[0] {
                Value::String(v) => Ok(Value::Integer(v.chars().count() as i32)),
                Value::Table(v) => Ok(Value::Integer(v.values.len() as i32)),
                _ => Err(runtime_error(span, "`длин` expects text or a table")),
            }
        }
        "код" => {
            arity(1)?;
            let c = as_char(&args[0], span)?;
            cp1251_code(c)
                .map(Value::Integer)
                .ok_or_else(|| runtime_error(span, "character is not representable in CP-1251"))
        }
        "юникод" => {
            arity(1)?;
            Ok(Value::Integer(as_char(&args[0], span)? as i32))
        }
        "символ" => {
            arity(1)?;
            let value = integer(args[0].clone(), span)?;
            cp1251_char(value)
                .map(Value::Character)
                .ok_or_else(|| runtime_error(span, "invalid CP-1251 character code"))
        }
        "юнисимвол" => {
            arity(1)?;
            let value = integer(args[0].clone(), span)?;
            char::from_u32(value as u32)
                .map(Value::Character)
                .ok_or_else(|| runtime_error(span, "invalid Unicode character code"))
        }
        "цел_в_лит" => {
            arity(1)?;
            Ok(Value::String(integer(args[0].clone(), span)?.to_string()))
        }
        "вещ_в_лит" => {
            arity(1)?;
            Ok(Value::String(
                number_f64(args[0].clone(), span)?.to_string(),
            ))
        }
        "лит_в_цел" => {
            if args.len() == 2 {
                let s = args[0].to_string();
                return Ok(Value::Integer(
                    s.trim()
                        .parse::<i32>()
                        .unwrap_or(integer(args[1].clone(), span)?),
                ));
            }
            arity(1)?;
            args[0]
                .to_string()
                .trim()
                .parse::<i32>()
                .map(Value::Integer)
                .map_err(|_| runtime_error(span, "text is not an integer"))
        }
        "лит_в_вещ" => {
            if args.len() == 2 {
                let s = args[0].to_string();
                return Ok(Value::Real(
                    s.trim()
                        .parse::<f64>()
                        .unwrap_or(number_f64(args[1].clone(), span)?),
                ));
            }
            arity(1)?;
            args[0]
                .to_string()
                .trim()
                .parse::<f64>()
                .map(Value::Real)
                .map_err(|_| runtime_error(span, "text is not a real number"))
        }
        "позиция" | "поз" => {
            arity(2)?;
            Ok(Value::Integer(find_position(
                &args[0].to_string(),
                &args[1].to_string(),
                1,
            )))
        }
        "позиция после" | "поз после" => {
            arity(3)?;
            Ok(Value::Integer(find_position(
                &args[1].to_string(),
                &args[2].to_string(),
                integer(args[0].clone(), span)?.max(1) as usize,
            )))
        }
        "верний регистр" => {
            arity(1)?;
            Ok(Value::String(args[0].to_string().to_uppercase()))
        }
        "нижний регистр" => {
            arity(1)?;
            Ok(Value::String(args[0].to_string().to_lowercase()))
        }
        "удалить" => {
            arity(3)?;
            let text = args[0].to_string();
            let chars = text.chars().collect::<Vec<_>>();
            let start = integer(args[1].clone(), span)?;
            let count = integer(args[2].clone(), span)?;
            if start < 1 || start as usize > chars.len() + 1 {
                return Err(runtime_error(
                    span,
                    "deletion position is outside the string",
                ));
            }
            if count < 0 {
                return Err(runtime_error(span, "deletion length cannot be negative"));
            }
            let start = start as usize - 1;
            let end = start.saturating_add(count as usize).min(chars.len());
            Ok(Value::String(
                chars
                    .iter()
                    .take(start)
                    .chain(chars.iter().skip(end))
                    .collect(),
            ))
        }
        "вставить" => {
            arity(3)?;
            let mut s = args[1].to_string();
            let position = integer(args[2].clone(), span)?;
            if position < 1 || position as usize > s.chars().count() + 1 {
                return Err(runtime_error(
                    span,
                    "insertion position is outside the string",
                ));
            };
            let index = s
                .char_indices()
                .nth(position as usize - 1)
                .map_or(s.len(), |(i, _)| i);
            s.insert_str(index, &args[0].to_string());
            Ok(Value::String(s))
        }
        "re" => {
            arity(1)?;
            match &args[0] {
                Value::Complex(v) => Ok(Value::Real(v.real)),
                _ => Err(runtime_error(span, "`Re` expects a complex number")),
            }
        }
        "im" => {
            arity(1)?;
            match &args[0] {
                Value::Complex(v) => Ok(Value::Real(v.imaginary)),
                _ => Err(runtime_error(span, "`Im` expects a complex number")),
            }
        }
        "rnd" | "rand" => Err(runtime_error(span, "random-number service is unavailable")),
        "время" | "ждать" => Err(runtime_error(span, "clock service is unavailable")),
        _ => Err(runtime_error(
            span,
            format!("unknown standard-library function `{name}`"),
        )),
    }
}

fn eval_binary(left: Value, op: BinaryOp, right: Value, span: Span) -> Result<Value, RuntimeError> {
    use BinaryOp::*;
    if matches!(
        op,
        Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual
    ) {
        let ordering = compare_values(&left, &right)
            .ok_or_else(|| runtime_error(span, "values cannot be compared"))?;
        return Ok(Value::Boolean(match op {
            Equal => ordering.is_eq(),
            NotEqual => !ordering.is_eq(),
            Less => ordering.is_lt(),
            LessEqual => !ordering.is_gt(),
            Greater => ordering.is_gt(),
            GreaterEqual => !ordering.is_lt(),
            _ => false,
        }));
    }
    match (left, right) {
        (Value::Boolean(a), Value::Boolean(b)) => match op {
            And => Ok(Value::Boolean(a && b)),
            Or => Ok(Value::Boolean(a || b)),
            _ => Err(runtime_error(
                span,
                "logical values support only `и` and `или`",
            )),
        },
        (Value::String(a), Value::String(b)) if op == Add => Ok(Value::String(a + &b)),
        (Value::Character(a), Value::Character(b)) if op == Add => {
            Ok(Value::String(format!("{a}{b}")))
        }
        (Value::Complex(a), Value::Complex(b)) => match op {
            Add => Ok(Value::Complex(Complex {
                real: a.real + b.real,
                imaginary: a.imaginary + b.imaginary,
            })),
            Subtract => Ok(Value::Complex(Complex {
                real: a.real - b.real,
                imaginary: a.imaginary - b.imaginary,
            })),
            Multiply => Ok(Value::Complex(Complex {
                real: a.real * b.real - a.imaginary * b.imaginary,
                imaginary: a.real * b.imaginary + a.imaginary * b.real,
            })),
            Divide if b.real != 0.0 || b.imaginary != 0.0 => {
                let d = b.real * b.real + b.imaginary * b.imaginary;
                Ok(Value::Complex(Complex {
                    real: (a.real * b.real + a.imaginary * b.imaginary) / d,
                    imaginary: (a.imaginary * b.real - a.real * b.imaginary) / d,
                }))
            }
            Divide => Err(runtime_error(span, "division by zero")),
            _ => Err(runtime_error(span, "unsupported complex operation")),
        },
        (Value::Complex(a), b) if is_numeric(&b) => complex_binary(
            a,
            Complex {
                real: number_f64(b, span)?,
                imaginary: 0.0,
            },
            op,
            span,
        ),
        (a, Value::Complex(b)) if is_numeric(&a) => complex_binary(
            Complex {
                real: number_f64(a, span)?,
                imaginary: 0.0,
            },
            b,
            op,
            span,
        ),
        (a, b) if is_numeric(&a) && is_numeric(&b) => numeric_binary(a, b, op, span),
        _ => Err(runtime_error(
            span,
            "incompatible values for this operation",
        )),
    }
}

fn complex_binary(a: Complex, b: Complex, op: BinaryOp, span: Span) -> Result<Value, RuntimeError> {
    use BinaryOp::*;
    match op {
        Add => Ok(Value::Complex(Complex {
            real: a.real + b.real,
            imaginary: a.imaginary + b.imaginary,
        })),
        Subtract => Ok(Value::Complex(Complex {
            real: a.real - b.real,
            imaginary: a.imaginary - b.imaginary,
        })),
        Multiply => Ok(Value::Complex(Complex {
            real: a.real * b.real - a.imaginary * b.imaginary,
            imaginary: a.real * b.imaginary + a.imaginary * b.real,
        })),
        Divide if b.real != 0.0 || b.imaginary != 0.0 => {
            let d = b.real * b.real + b.imaginary * b.imaginary;
            Ok(Value::Complex(Complex {
                real: (a.real * b.real + a.imaginary * b.imaginary) / d,
                imaginary: (a.imaginary * b.real - a.real * b.imaginary) / d,
            }))
        }
        Divide => Err(runtime_error(span, "division by zero")),
        _ => Err(runtime_error(span, "unsupported complex operation")),
    }
}

fn numeric_binary(a: Value, b: Value, op: BinaryOp, span: Span) -> Result<Value, RuntimeError> {
    let ai = matches!(&a, Value::Integer(_));
    let bi = matches!(&b, Value::Integer(_));
    let x = number_f64(a.clone(), span)?;
    let y = number_f64(b.clone(), span)?;
    use BinaryOp::*;
    if ai && bi && matches!(op, Add | Subtract | Multiply | IntegerDivide | Modulo) {
        let (a, b) = (integer(a, span)?, integer(b, span)?);
        let result = match op {
            Add => a.checked_add(b),
            Subtract => a.checked_sub(b),
            Multiply => a.checked_mul(b),
            IntegerDivide if b != 0 => a.checked_div(b),
            Modulo if b != 0 => a.checked_rem(b),
            IntegerDivide | Modulo => return Err(runtime_error(span, "division by zero")),
            _ => None,
        };
        return result
            .filter(|v| (i32::MIN..=i32::MAX).contains(v))
            .map(Value::Integer)
            .ok_or_else(|| {
                runtime_error(span, "integer arithmetic is outside the Kumir 32-bit range")
            });
    }
    match op {
        Add => Ok(Value::Real(x + y)),
        Subtract => Ok(Value::Real(x - y)),
        Multiply => Ok(Value::Real(x * y)),
        Divide if y != 0.0 => Ok(Value::Real(x / y)),
        Divide => Err(runtime_error(span, "division by zero")),
        IntegerDivide if y != 0.0 => checked_real_integer((x / y).trunc(), span),
        IntegerDivide => Err(runtime_error(span, "division by zero")),
        Modulo if y != 0.0 => Ok(Value::Real(x % y)),
        Modulo => Err(runtime_error(span, "division by zero")),
        Power => Ok(Value::Real(x.powf(y))),
        _ => Err(runtime_error(span, "invalid numeric operator")),
    }
}

fn compare_values(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b))
        | (Value::KeyCode(a), Value::KeyCode(b))
        | (Value::Integer(a), Value::KeyCode(b))
        | (Value::KeyCode(a), Value::Integer(b)) => Some(a.cmp(b)),
        (Value::Real(a), Value::Real(b)) => a.partial_cmp(b),
        (Value::Integer(a), Value::Real(b)) => (*a as f64).partial_cmp(b),
        (Value::Real(a), Value::Integer(b)) => a.partial_cmp(&(*b as f64)),
        (Value::Boolean(a), Value::Boolean(b)) => Some(a.cmp(b)),
        (Value::Character(a), Value::Character(b)) => Some(a.cmp(b)),
        (Value::String(a), Value::String(b)) | (Value::Color(a), Value::Color(b)) => Some(a.cmp(b)),
        _ => None,
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) | (Value::KeyCode(a), Value::KeyCode(b)) => a == b,
        (Value::Integer(a), Value::KeyCode(b)) | (Value::KeyCode(a), Value::Integer(b)) => a == b,
        (Value::Real(a), Value::Real(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Character(a), Value::Character(b)) => a == b,
        (Value::String(a), Value::String(b))
        | (Value::Color(a), Value::Color(b))
        | (Value::File(a), Value::File(b)) => a == b,
        (Value::Complex(a), Value::Complex(b)) => a.real == b.real && a.imaginary == b.imaginary,
        (Value::Table(a), Value::Table(b)) => a == b,
        (Value::Void, Value::Void) => true,
        (Value::Integer(a), Value::Real(b)) => *a as f64 == *b,
        (Value::Real(a), Value::Integer(b)) => *a == *b as f64,
        _ => false,
    }
}
fn is_numeric(v: &Value) -> bool {
    matches!(v, Value::Integer(_) | Value::Real(_))
}
fn number_f64(v: Value, span: Span) -> Result<f64, RuntimeError> {
    match v {
        Value::Integer(v) => Ok(v as f64),
        Value::Real(v) => Ok(v),
        _ => Err(runtime_error(span, "expected a numeric value")),
    }
}
fn integer(v: Value, span: Span) -> Result<i32, RuntimeError> {
    match v {
        Value::Integer(v) => Ok(v),
        _ => Err(runtime_error(span, "expected an integer value")),
    }
}
fn checked_real_integer(v: f64, span: Span) -> Result<Value, RuntimeError> {
    if !v.is_finite() || v < i32::MIN as f64 || v > i32::MAX as f64 {
        Err(runtime_error(
            span,
            "integer result is outside the Kumir 32-bit range",
        ))
    } else {
        Ok(Value::Integer(v as i32))
    }
}
fn truthy(v: Value, span: Span) -> Result<bool, RuntimeError> {
    match v {
        Value::Boolean(v) => Ok(v),
        _ => Err(runtime_error(span, "condition must have logical type")),
    }
}

fn undefined_value(ty: &Type) -> Value {
    match ty {
        Type::Integer
        | Type::KeyCode
        | Type::File
        | Type::Real
        | Type::Boolean
        | Type::Character
        | Type::String
        | Type::Complex
        | Type::Color => Value::Void,
        Type::Table { element, bounds } => {
            let len = table_element_count(bounds).unwrap_or(0);
            Value::Table(ArrayValue {
                bounds: bounds.clone(),
                values: vec![undefined_value(element); len],
            })
        }
    }
}

fn table_element_count(bounds: &[(i32, i32)]) -> Option<usize> {
    bounds.iter().try_fold(1usize, |count, (lower, upper)| {
        let dimension = usize::try_from(*upper as i64 - *lower as i64 + 1).ok()?;
        count.checked_mul(dimension)
    })
}

fn assign_value(value: Value, ty: &Type, span: Span) -> Result<Value, RuntimeError> {
    match (ty, value) {
        (Type::Integer, Value::Integer(v)) => Ok(Value::Integer(v)),
        (Type::Real, Value::Integer(v)) => Ok(Value::Real(v as f64)),
        (Type::Real, v @ Value::Real(_)) => Ok(v),
        (Type::Complex, Value::Integer(v)) => Ok(Value::Complex(Complex {
            real: v as f64,
            imaginary: 0.0,
        })),
        (Type::Complex, Value::Real(v)) => Ok(Value::Complex(Complex {
            real: v,
            imaginary: 0.0,
        })),
        (Type::Boolean, v @ Value::Boolean(_))
        | (Type::Character, v @ Value::Character(_))
        | (Type::String, v @ Value::String(_))
        | (Type::Complex, v @ Value::Complex(_))
        | (Type::Color, v @ Value::Color(_))
        | (Type::KeyCode, v @ Value::KeyCode(_))
        | (Type::File, v @ Value::File(_)) => Ok(v),
        (Type::String, Value::Character(v)) => Ok(Value::String(v.to_string())),
        (Type::Table { element, bounds }, Value::Table(mut table)) => {
            if !bounds.is_empty() && bounds != &table.bounds {
                return Err(runtime_error(
                    span,
                    "table bounds do not match the declared type",
                ));
            }
            for value in &mut table.values {
                if !matches!(value, Value::Void) {
                    *value = assign_value(value.clone(), element, span)?;
                }
            }
            Ok(Value::Table(table))
        }
        (expected, value) => Err(runtime_error(
            span,
            format!(
                "value `{value}` is incompatible with declared type `{}`",
                type_name(expected)
            ),
        )),
    }
}

fn type_name(ty: &Type) -> &'static str {
    match ty {
        Type::Integer => "цел",
        Type::Real => "вещ",
        Type::Boolean => "лог",
        Type::Character => "сим",
        Type::String => "лит",
        Type::Complex => "компл",
        Type::Color => "цвет",
        Type::KeyCode => "сканкод",
        Type::File => "файл",
        Type::Table { .. } => "таблица",
    }
}

fn required_type_module(ty: &Type) -> Option<Module> {
    match ty {
        Type::Complex => Some(Module::ComplexNumbers),
        Type::Color => Some(Module::Painter),
        Type::KeyCode => Some(Module::Keyboard),
        Type::File => Some(Module::Files),
        Type::Table { element, .. } => required_type_module(element),
        _ => None,
    }
}
fn algorithm_span(algorithm: &Algorithm) -> Span {
    algorithm.span
}

fn value_type(value: &Value) -> Type {
    match value {
        Value::Integer(_) => Type::Integer,
        Value::Real(_) => Type::Real,
        Value::Boolean(_) => Type::Boolean,
        Value::Character(_) => Type::Character,
        Value::String(_) => Type::String,
        Value::Complex(_) => Type::Complex,
        Value::Color(_) => Type::Color,
        Value::KeyCode(_) => Type::KeyCode,
        Value::File(_) => Type::File,
        Value::Table(v) => Type::Table {
            element: Box::new(v.values.first().map(value_type).unwrap_or(Type::Integer)),
            bounds: v.bounds.clone(),
        },
        Value::Void => Type::Integer,
    }
}

fn fallback_integer(value: Value, fallback: Value, span: Span) -> Result<Value, RuntimeError> {
    let fallback = integer(fallback, span)?;
    Ok(value
        .to_string()
        .trim()
        .parse::<i32>()
        .map(Value::Integer)
        .unwrap_or(Value::Integer(fallback)))
}
fn fallback_real(value: Value, fallback: Value, span: Span) -> Result<Value, RuntimeError> {
    let fallback = number_f64(fallback, span)?;
    Ok(value
        .to_string()
        .trim()
        .parse::<f64>()
        .map(Value::Real)
        .unwrap_or(Value::Real(fallback)))
}
fn as_char(value: &Value, span: Span) -> Result<char, RuntimeError> {
    match value {
        Value::Character(ch) => Ok(*ch),
        Value::String(text) if text.chars().count() == 1 => Ok(text.chars().next().unwrap()),
        _ => Err(runtime_error(span, "expected one character")),
    }
}
fn find_position(text: &str, fragment: &str, start: usize) -> i32 {
    let index = start.saturating_sub(1);
    let suffix = text.chars().skip(index).collect::<String>();
    suffix.find(fragment).map_or(0, |byte| {
        (index + suffix[..byte].chars().count() + 1) as i32
    })
}
fn cp1251_code(ch: char) -> Option<i32> {
    if (ch as u32) <= 0x7f {
        return Some(ch as i32);
    }
    if ('А'..='я').contains(&ch) {
        return Some(ch as i32 - 'А' as i32 + 0xC0);
    }
    match ch {
        'Ё' => Some(0xA8),
        'ё' => Some(0xB8),
        'Ђ' => Some(0x80),
        'Ѓ' => Some(0x81),
        '‚' => Some(0x82),
        'ѓ' => Some(0x83),
        '„' => Some(0x84),
        '…' => Some(0x85),
        '†' => Some(0x86),
        '‡' => Some(0x87),
        '€' => Some(0x88),
        '‰' => Some(0x89),
        'Љ' => Some(0x8A),
        '‹' => Some(0x8B),
        'Њ' => Some(0x8C),
        'Ќ' => Some(0x8D),
        'Ћ' => Some(0x8E),
        'Џ' => Some(0x8F),
        'ђ' => Some(0x90),
        '‘' => Some(0x91),
        '’' => Some(0x92),
        '“' => Some(0x93),
        '”' => Some(0x94),
        '•' => Some(0x95),
        '–' => Some(0x96),
        '—' => Some(0x97),
        '™' => Some(0x99),
        'љ' => Some(0x9A),
        '›' => Some(0x9B),
        'њ' => Some(0x9C),
        'ќ' => Some(0x9D),
        'ћ' => Some(0x9E),
        'џ' => Some(0x9F),
        ' ' => Some(0xA0),
        'Ў' => Some(0xA1),
        'ў' => Some(0xA2),
        'Ј' => Some(0xA3),
        '¤' => Some(0xA4),
        'Ґ' => Some(0xA5),
        '¦' => Some(0xA6),
        '§' => Some(0xA7),
        '©' => Some(0xA9),
        'Є' => Some(0xAA),
        '«' => Some(0xAB),
        '¬' => Some(0xAC),
        '­' => Some(0xAD),
        '®' => Some(0xAE),
        'Ї' => Some(0xAF),
        '°' => Some(0xB0),
        '±' => Some(0xB1),
        'ґ' => Some(0xB2),
        '³' => Some(0xB3),
        'µ' => Some(0xB5),
        '¶' => Some(0xB6),
        '·' => Some(0xB7),
        'є' => Some(0xBA),
        '»' => Some(0xBB),
        'ј' => Some(0xBC),
        'Ѕ' => Some(0xBD),
        'ѕ' => Some(0xBE),
        'ї' => Some(0xBF),
        _ => None,
    }
}
fn cp1251_char(code: i32) -> Option<char> {
    if (0..=0x7f).contains(&code) {
        return char::from_u32(code as u32);
    }
    if (0xC0..=0xFF).contains(&code) {
        return char::from_u32(('А' as i32 + code - 0xC0) as u32);
    }
    Some(match code {
        0x80 => 'Ђ',
        0x81 => 'Ѓ',
        0x82 => '‚',
        0x83 => 'ѓ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => '€',
        0x89 => '‰',
        0x8A => 'Љ',
        0x8B => '‹',
        0x8C => 'Њ',
        0x8D => 'Ќ',
        0x8E => 'Ћ',
        0x8F => 'Џ',
        0x90 => 'ђ',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x99 => '™',
        0x9A => 'љ',
        0x9B => '›',
        0x9C => 'њ',
        0x9D => 'ќ',
        0x9E => 'ћ',
        0x9F => 'џ',
        0xA0 => ' ',
        0xA1 => 'Ў',
        0xA2 => 'ў',
        0xA3 => 'Ј',
        0xA4 => '¤',
        0xA5 => 'Ґ',
        0xA6 => '¦',
        0xA7 => '§',
        0xA8 => 'Ё',
        0xA9 => '©',
        0xAA => 'Є',
        0xAB => '«',
        0xAC => '¬',
        0xAD => '­',
        0xAE => '®',
        0xAF => 'Ї',
        0xB0 => '°',
        0xB1 => '±',
        0xB2 => 'ґ',
        0xB3 => '³',
        0xB5 => 'µ',
        0xB6 => '¶',
        0xB7 => '·',
        0xB8 => 'ё',
        0xBA => 'є',
        0xBB => '»',
        0xBC => 'ј',
        0xBD => 'Ѕ',
        0xBE => 'ѕ',
        0xBF => 'ї',
        _ => return None,
    })
}

fn split_input_values(source: &str) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut token_started = false;
    for ch in source.chars() {
        if let Some(delimiter) = quote {
            if escaped {
                current.push(match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => other,
                });
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                token_started = true;
            }
            ch if ch.is_whitespace() => {
                if token_started {
                    values.push(std::mem::take(&mut current));
                    token_started = false;
                }
            }
            _ => {
                current.push(ch);
                token_started = true;
            }
        }
    }
    if quote.is_some() {
        return Err("input contains an unterminated quoted value".to_owned());
    }
    if token_started {
        values.push(current);
    }
    Ok(values)
}

fn parse_input(raw: &str, ty: &Type, span: Span) -> Result<Value, RuntimeError> {
    let raw = raw.trim();
    match ty {
        Type::Integer => raw
            .parse::<i32>()
            .map(Value::Integer)
            .map_err(|_| runtime_error(span, "input is outside the Kumir integer range")),
        Type::Real => raw
            .parse::<f64>()
            .map(Value::Real)
            .map_err(|_| runtime_error(span, "input is not a real number")),
        Type::Complex => parse_complex_value(raw)
            .map(Value::Complex)
            .ok_or_else(|| runtime_error(span, "input is not a complex number")),
        Type::Boolean => match normalize(raw).as_str() {
            "да" => Ok(Value::Boolean(true)),
            "нет" => Ok(Value::Boolean(false)),
            _ => Err(runtime_error(span, "logical input must be `да` or `нет`")),
        },
        Type::Character if raw.chars().count() == 1 => {
            Ok(Value::Character(raw.chars().next().unwrap()))
        }
        Type::Character => Err(runtime_error(
            span,
            "character input must contain one character",
        )),
        Type::String => Ok(Value::String(raw.to_owned())),
        _ => Err(runtime_error(span, "input for this type is not supported")),
    }
}

fn parse_complex_value(source: &str) -> Option<Complex> {
    let compact = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .replace(['е', 'Е'], "e");
    if compact.is_empty() {
        return None;
    }
    let mut terms = Vec::new();
    let chars = compact.chars().collect::<Vec<_>>();
    let mut start = 0;
    for index in 1..chars.len() {
        if matches!(chars[index], '+' | '-') && !matches!(chars[index - 1], 'e' | 'E') {
            terms.push(chars[start..index].iter().collect::<String>());
            start = index;
        }
    }
    terms.push(chars[start..].iter().collect::<String>());
    let mut value = Complex {
        real: 0.0,
        imaginary: 0.0,
    };
    for term in terms {
        if term.is_empty() {
            return None;
        }
        if let Some(coefficient) = term.strip_suffix('i') {
            let coefficient = coefficient.strip_suffix('*').unwrap_or(coefficient);
            let imaginary = match coefficient {
                "" | "+" => 1.0,
                "-" => -1.0,
                _ => coefficient.parse::<f64>().ok()?,
            };
            value.imaginary += imaginary;
        } else {
            value.real += term.parse::<f64>().ok()?;
        }
    }
    Some(value)
}

fn render_output(values: &[Option<Value>]) -> String {
    let mut text = String::new();
    for value in values {
        match value {
            Some(value) => text.push_str(&value.to_string()),
            None => text.push('\n'),
        }
    }
    text
}

fn get_index(value: Value, indices: &[i32], span: Span) -> Result<Value, RuntimeError> {
    let Value::Table(table) = value else {
        return Err(runtime_error(span, "indexed value is not a table"));
    };
    if indices.len() != table.bounds.len() {
        return Err(runtime_error(span, "wrong number of table indices"));
    }
    let mut flat = 0usize;
    let mut stride = 1usize;
    for (index, (lo, hi)) in indices.iter().zip(&table.bounds).rev() {
        if index < lo || index > hi {
            return Err(runtime_error(
                span,
                "table index is outside its declared bounds",
            ));
        }
        flat += (index - lo) as usize * stride;
        stride *= (hi - lo + 1) as usize;
    }
    table
        .values
        .get(flat)
        .cloned()
        .ok_or_else(|| runtime_error(span, "table index is unavailable"))
}

fn set_index(
    table: &mut ArrayValue,
    indices: &[i32],
    value: Value,
    span: Span,
) -> Result<(), RuntimeError> {
    if indices.len() != table.bounds.len() {
        return Err(runtime_error(span, "wrong number of table indices"));
    }
    let mut flat = 0usize;
    let mut stride = 1usize;
    for (index, (lo, hi)) in indices.iter().zip(&table.bounds).rev() {
        if index < lo || index > hi {
            return Err(runtime_error(
                span,
                "table index is outside its declared bounds",
            ));
        }
        flat += (index - lo) as usize * stride;
        stride *= (hi - lo + 1) as usize;
    }
    let Some(slot) = table.values.get_mut(flat) else {
        return Err(runtime_error(span, "table index is unavailable"));
    };
    *slot = assign_value(value, &value_type(slot), span)?;
    Ok(())
}

fn statement_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Declare { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Input { span, .. }
        | Stmt::Output { span, .. }
        | Stmt::Newline { span }
        | Stmt::If { span, .. }
        | Stmt::Choose { span, .. }
        | Stmt::While { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::RepeatUntil { span, .. }
        | Stmt::Loop { span, .. }
        | Stmt::Exit { span }
        | Stmt::Assert { span, .. }
        | Stmt::Stop { span }
        | Stmt::Pause { span }
        | Stmt::Call { span, .. }
        | Stmt::ModuleCall { span, .. }
        | Stmt::File { span, .. } => *span,
    }
}

fn source_column(source: &str, char_index: usize) -> usize {
    source.chars().take(char_index).count() + 1
}
fn normalize(source: &str) -> String {
    source
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn negated_logical_name(name: &str) -> Option<String> {
    let name = normalize(name);
    let (before, after) = name.split_once(" не ")?;
    if before.is_empty() || after.is_empty() || after.contains(" не ") {
        return None;
    }
    Some(format!("{before} {after}"))
}
fn is_repeat_close(source: &str) -> bool {
    starts_keyword(source, "кц при") || starts_keyword(source, "кц_при")
}
fn repeat_close_condition(source: &str) -> Option<&str> {
    let source = source.trim();
    if starts_keyword(source, "кц при") {
        source.split_once(char::is_whitespace).map(|(_, tail)| {
            tail.trim_start()
                .strip_prefix("при")
                .unwrap_or(tail.trim_start())
                .trim_start()
        })
    } else if starts_keyword(source, "кц_при") {
        source["кц_при".len()..].trim_start().into()
    } else {
        None
    }
}
fn starts_keyword(source: &str, keyword: &str) -> bool {
    let text = normalize(source);
    let key = normalize(keyword);
    text == key
        || text
            .strip_prefix(&key)
            .is_some_and(|rest| rest.starts_with(' ') || rest.starts_with('('))
}
fn is_keyword(source: &str, keyword: &str) -> bool {
    normalize(source) == normalize(keyword)
}
fn is_terminator(source: &str, keyword: &str) -> bool {
    if normalize(keyword) == "при" {
        starts_keyword(source, keyword)
    } else {
        is_keyword(source, keyword)
    }
}
fn strip_suffix_keyword<'a>(source: &'a str, keyword: &str) -> Option<&'a str> {
    let source = source.trim_end();
    let key = keyword.to_lowercase();
    if source.to_lowercase().ends_with(&key) {
        Some(source[..source.len().checked_sub(keyword.len())?].trim_end())
    } else {
        None
    }
}
fn split_keyword<'a>(source: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let words = source
        .split_whitespace()
        .scan(0usize, |cursor, word| {
            let offset = source[*cursor..].find(word)? + *cursor;
            let end = offset + word.len();
            *cursor = end;
            Some((normalize(word), offset, end))
        })
        .collect::<Vec<_>>();
    let wanted = normalize(keyword)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if wanted.is_empty() {
        return None;
    }
    for start in 0..=words.len().saturating_sub(wanted.len()) {
        if words[start..start + wanted.len()]
            .iter()
            .map(|(word, _, _)| word)
            .eq(wanted.iter())
        {
            let left = words[start].1;
            let right = words[start + wanted.len() - 1].2;
            return Some((source[..left].trim_end(), source[right..].trim_start()));
        }
    }
    None
}
fn split_top_level(source: &str, delimiter: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quote = None;
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, ch) in source.char_indices() {
        if let Some(q) = quote {
            if ch == q {
                quote = None
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                out.push(&source[start..i]);
                start = i + ch.len_utf8()
            }
            _ => {}
        }
    }
    out.push(&source[start..]);
    out
}

fn split_top_level_once(source: &str, delimiter: char) -> Option<(&str, &str)> {
    let mut quote = None;
    let mut depth = 0usize;
    for (index, ch) in source.char_indices() {
        if let Some(quoted) = quote {
            if ch == quoted {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ if ch == delimiter && depth == 0 => {
                return Some((&source[..index], &source[index + ch.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}
fn split_names(source: &str) -> Vec<&str> {
    source
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect()
}
fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}
fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit() || ch == '@'
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
