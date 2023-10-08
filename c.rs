use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct Loc {
    file_path: String,
    row: usize,
    col: usize,
}

impl Loc {
    fn new(file_path: String, row: usize, col: usize) -> Self {
        Loc {
            file_path,
            row,
            col,
        }
    }

    fn display(&self) -> String {
        format!("{}:{}:{}", self.file_path, self.row + 1, self.col + 1)
    }
}

#[derive(Debug)]
enum TokenKind {
    Name(String),
    Oparen,
    Cparen,
    Ocbracket,
    Ccbracket,
    Comma,
    Semicolon,
    Number(i32),
    StringLiteral(String),
    Return,
}

#[derive(Debug)]
struct Token {
    loc: Loc,
    kind: TokenKind,
}

impl Token {
    fn new(loc: Loc, kind: TokenKind) -> Self {
        Token { loc, kind }
    }
}

struct Lexer {
    file_path: String,
    source: String,
    cur: usize,
    bol: usize,
    row: usize,
}

impl Lexer {
    fn new(file_path: String, source: String) -> Self {
        Lexer {
            file_path,
            source,
            cur: 0,
            bol: 0,
            row: 0,
        }
    }

    fn is_not_empty(&self) -> bool {
        self.cur < self.source.len()
    }

    fn is_empty(&self) -> bool {
        !self.is_not_empty()
    }

    fn chop_char(&mut self) {
        if self.is_not_empty() {
            let x = self.source.chars().nth(self.cur).unwrap();
            self.cur += 1;
            if x == '\n' {
                self.bol = self.cur;
                self.row += 1;
            }
        }
    }

    fn loc(&self) -> Loc {
        Loc::new(
            self.file_path.clone(),
            self.row,
            self.cur - self.bol,
        )
    }

    fn trim_left(&mut self) {
        while self.is_not_empty() && self.source.chars().nth(self.cur).unwrap().is_whitespace() {
            self.chop_char();
        }
    }

    fn drop_line(&mut self) {
        while self.is_not_empty() && self.source.chars().nth(self.cur).unwrap() != '\n' {
            self.chop_char();
        }
        if self.is_not_empty() {
            self.chop_char();
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.trim_left();
        while self.is_not_empty() {
            let s = &self.source[self.cur..];
            if !s.starts_with("#") && !s.starts_with("//") {
                break;
            }
            self.drop_line();
            self.trim_left();
        }

        if self.is_empty() {
            return None;
        }

        let loc = self.loc();
        let first = self.source.chars().nth(self.cur).unwrap();

        if first.is_alphabetic() {
            let index = self.cur;
            while self.is_not_empty() && self.source.chars().nth(self.cur).unwrap().is_alphanumeric() {
                self.chop_char();
            }

            let value = &self.source[index..self.cur];
            return Some(Token::new(loc, TokenKind::Name(value.to_string())));
        }

        let literal_tokens = vec![
            ('(', TokenKind::Oparen),
            (')', TokenKind::Cparen),
            ('{', TokenKind::Ocbracket),
            ('}', TokenKind::Ccbracket),
            (',', TokenKind::Comma),
            (';', TokenKind::Semicolon),
        ];

        if let Some(&(_, kind)) = literal_tokens.iter().find(|&&(c, _)| c == first) {
            self.chop_char();
            return Some(Token::new(loc, kind));
        }

        if first == '"' {
            self.chop_char();
            let start = self.cur;
            let mut literal = String::new();
            while self.is_not_empty() {
                let ch = self.source.chars().nth(self.cur).unwrap();
                match ch {
                    '"' => break,
                    '\\' => {
                        self.chop_char();
                        if self.is_empty() {
                            eprintln!("{}: ERROR: unfinished escape sequence", loc.display());
                            std::process::exit(69);
                        }

                        let escape = self.source.chars().nth(self.cur).unwrap();
                        match escape {
                            'n' => {
                                literal.push('\n');
                                self.chop_char();
                            }
                            '"' => {
                                literal.push('"');
                                self.chop_char();
                            }
                            _ => {
                                eprintln!("{}: ERROR: unknown escape sequence starts with {}", loc.display(), escape);
                                std::process::exit(69);
                            }
                        }
                    }
                    _ => {
                        literal.push(ch);
                        self.chop_char();
                    }
                }
            }

            if self.is_not_empty() {
                self.chop_char();
                return Some(Token::new(loc, TokenKind::StringLiteral(literal)));
            }

            eprintln!("{}: ERROR: unclosed string literal", loc.display());
            std::process::exit(69);
        }

        if first.is_digit(10) {
            let start = self.cur;
            while self.is_not_empty() && self.source.chars().nth(self.cur).unwrap().is_digit(10) {
                self.chop_char();
            }

            let value = self.source[start..self.cur].parse::<i32>().unwrap();
            return Some(Token::new(loc, TokenKind::Number(value)));
        }

        eprintln!("{}: ERROR: unknown token starts with {}", loc.display(), first);
        std::process::exit(69);
    }
}

const TYPE_INT: &str = "TYPE_INT";

#[derive(Debug)]
struct FuncallStmt {
    name: Token,
    args: Vec<Token>,
}

#[derive(Debug)]
struct RetStmt {
    expr: Token,
}

#[derive(Debug)]
struct Func {
    name: Token,
    body: Vec<Box<dyn Stmt>>,
}

trait Stmt: fmt::Debug {}
impl Stmt for FuncallStmt {}
impl Stmt for RetStmt {}

fn expect_token(lexer: &mut Lexer, types: &[TokenKind]) -> Option<Token> {
    if let Some(token) = lexer.next_token() {
        if types.contains(&token.kind) {
            return Some(token);
        } else {
            eprintln!(
                "{}: ERROR: expected {:?} but got {:?}",
                token.loc.display(),
                types,
                token.kind
            );
        }
    } else {
        eprintln!("{}: ERROR: expected {:?} but got end of file", lexer.loc().display(), types);
    }
    std::process::exit(69);
}

fn parse_type(lexer: &mut Lexer) -> Option<String> {
    if let Some(return_type) = expect_token(lexer, &[TokenKind::Name("TYPE_INT".to_string())]) {
        return Some(return_type.kind.to_string());
    }
    None
}

fn parse_arglist(lexer: &mut Lexer) -> Option<Vec<Token>> {
    if let Some(oparen) = expect_token(lexer, &[TokenKind::Oparen]) {
        let mut arglist = Vec::new();
        loop {
            if let Some(cparen) = expect_token(lexer, &[TokenKind::Cparen]) {
                if arglist.is_empty() {
                    // Call with no arguments.
                    return Some(arglist);
                } else {
                    return Some(arglist);
                }
            }
            if let Some(arg) = expect_token(lexer, &[TokenKind::StringLiteral, TokenKind::Number]) {
                arglist.push(arg);
            } else {
                return None;
            }
        }
    }
    None
}

fn parse_block(lexer: &mut Lexer) -> Option<Vec<Box<dyn Stmt>>> {
    if let Some(ocbracket) = expect_token(lexer, &[TokenKind::Ocbracket]) {
        let mut block = Vec::new();
        loop {
            if let Some(ccbracket) = expect_token(lexer, &[TokenKind::Ccbracket]) {
                return Some(block);
            }
            if let Some(name) = expect_token(lexer, &[TokenKind::Name("return".to_string())]) {
                if let Some(expr) = expect_token(lexer, &[TokenKind::Number, TokenKind::StringLiteral]) {
                    block.push(Box::new(RetStmt { expr }));
                } else {
                    return None;
                }
            } else if let Some(name) = expect_token(lexer, &[TokenKind::Name]) {
                if let Some(arglist) = parse_arglist(lexer) {
                    block.push(Box::new(FuncallStmt { name, args: arglist }));
                } else {
                    return None;
                }
            } else {
                return None;
            }
            if let Some(semicolon) = expect_token(lexer, &[TokenKind::Semicolon]) {
                // Continue parsing the next statement.
            } else {
                return None;
            }
        }
    }
    None
}

fn parse_function(lexer: &mut Lexer) -> Option<Func> {
    if let Some(return_type) = parse_type(lexer) {
        if let Some(name) = expect_token(lexer, &[TokenKind::Name]) {
            if let Some(oparen) = expect_token(lexer, &[TokenKind::Oparen]) {
                if let Some(cparen) = expect_token(lexer, &[TokenKind::Cparen]) {
                    if let Some(body) = parse_block(lexer) {
                        return Some(Func { name, body });
                    }
                }
            }
        }
    }
    None
}

fn generate_python3(func: Func) {
    fn literal_to_py(value: &Token) -> String {
        match &value.kind {
            TokenKind::StringLiteral(s) => format!("\"{}\"", s.replace("\n", "\\n")),
            TokenKind::Number(n) => n.to_string(),
            _ => "".to_string(),
        }
    }

    for stmt in &func.body {
        match &**stmt {
            FuncallStmt { name, args } if name.kind.to_string() == "printf" => {
                if args.len() <= 1 {
                    if let Some(format) = args.get(0) {
                        if let TokenKind::StringLiteral(s) = &format.kind {
                            if s.ends_with("\\n") {
                                println!("print({})", literal_to_py(format));
                            } else {
                                println!("print({}, end=\"\")", literal_to_py(format));
                            }
                        }
                    }
                } else {
                    let mut substitutions = " % (".to_string();
                    for (i, arg) in args.iter().enumerate() {
                        if i == 0 {
                            continue; // Skip format string.
                        }
                        substitutions += &format!("{},", literal_to_py(arg));
                    }
                    substitutions += ")";
                    if let Some(format) = args.get(0) {
                        if let TokenKind::StringLiteral(s) = &format.kind {
                            println!("print({}{}, end=\"\")", literal_to_py(format), substitutions);
                        }
                    }
                }
            }
            _ => {
                eprintln!(
                    "{}: ERROR: unknown function {}",
                    stmt.loc.display(),
                    name.kind.to_string()
                );
                std::process::exit(69);
            }
        }
    }
}

fn generate_fasm_x86_64_linux(func: Func) {
    println!("format ELF64 executable 3");
    println!("segment readable executable");
    println!("entry start");
    println!("start:");

    let mut strings = Vec::new();
    for stmt in &func.body {
        match &**stmt {
            RetStmt { expr } => {
                if let TokenKind::Number(n) = &expr.kind {
                    println!("    mov rax, 60");
                    println!("    mov rdi, {}", n);
                    println!("    syscall");
                }
            }
            FuncallStmt { name, args } if name.kind.to_string() == "printf" => {
                let arity = args.len();
                if arity != 1 {
                    eprintln!("{}: ERROR: expected 1 argument but got {}", name.loc.display(), arity);
                    std::process::exit(69);
                }
                if let TokenKind::StringLiteral(format) = &args[0].kind {
                    let m = format.len();
                    println!("    mov rax, 1");
                    println!("    mov rdi, 1");
                    if !strings.contains(format) {
                        strings.push(format.clone());
                    }
                    let n = strings.iter().position(|s| *s == *format).unwrap();
                    println!("    mov rsi, str_{}", n);
                    println!("    mov rdx, {}", m);
                    println!("    syscall");
                }
            }
            _ => {
                eprintln!(
                    "{}: ERROR: unknown function {}",
                    stmt.loc.display(),
                    name.kind.to_string()
                );
                std::process::exit(69);
            }
        }
    }

    println!("segment readable writable");
    for (n, string) in strings.iter().enumerate() {
        print!("str_{} db ", n);
        for (i, c) in string.chars().enumerate() {
            if i > 0 {
                print!(",");
            }
            print!("{}", c as u8);
        }
        println!();
    }
}

fn usage(program: &str) {
    println!("Usage: {} [OPTIONS] <input.c>", program);
    println!("OPTIONS:");
    println!("    -target <target>    Compilation target. Provide `list` to get the list of targets. (default: python3)");
    println!("    -help               Print this message");
}

fn main() {
    let mut args = std::env::args().collect::<Vec<String>>();
    let program = args.remove(0);
    let mut input = String::new();
    let mut platform = "python3".to_string();

    while !args.is_empty() {
        let flag = args.remove(0);
        match flag.as_str() {
            "-help" => {
                usage(&program);
                std::process::exit(0);
            }
            "-target" => {
                if let Some(arg) = args.pop() {
                    if arg == "list" {
                        println!("Available targets:");
                        for p in &["python3", "fasm-x86_64-linux"] {
                            println!("    {}", p);
                        }
                        std::process::exit(69);
                    } else if ["python3", "fasm-x86_64-linux"].contains(&arg.as_str()) {
                        platform = arg;
                    } else {
                        usage(&program);
                        eprintln!("ERROR: unknown target {}", arg);
                        std::process::exit(69);
                    }
                } else {
                    usage(&program);
                    eprintln!("ERROR: no value was provided for flag -target");
                    std::process::exit(69);
                }
            }
            _ => input = flag,
        }
    }

    if input.is_empty() {
        usage(&program);
        eprintln!("ERROR: no input is provided");
        std::process::exit(69);
    }

    let file_path = input.clone();
    let source = std::fs::read_to_string(&file_path).unwrap();
    let mut lexer = Lexer::new(file_path, source);
    if let Some(func) = parse_function(&mut lexer) {
        match platform.as_str() {
            "python3" => generate_python3(func),
            "fasm-x86_64-linux" => generate_fasm_x86_64_linux(func),
            _ => todo!("unreachable"),
        }
    }
}

fn contains(strings: &Vec<String>, s: &str) -> bool {
    for string in strings.iter() {
        if *string == *s {
            return true;
        }
    }
    false
}

fn println(s: &str) {
    println!("{}", s);
}
