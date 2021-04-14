use std::char;
use std::collections::LinkedList;
use crate::common::*;

/* token stuff */
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
	TK_NEWLN = -2,
	TK_EOF = -1,

	/* immedial primitive */
	TK_IDENTIFIER = 0,
    TK_NUMBER,
	TK_STRING,

	/* keywords */
	TK_BREAK,
	TK_CASE,
	TK_CATCH,
	TK_CONTINUE,
	TK_DEFAULT,
	TK_DELETE,
	TK_DO,
	TK_ELSE,
	TK_FALSE,
	TK_FINALLY,
	TK_FOR,
	TK_FUNCTION,
	TK_IF,
	TK_IN,
	TK_INSTANCEOF,
	TK_NEW,
	TK_NULL,
	TK_RETURN,
	TK_UNDEF,
	TK_SWITCH,
	TK_THIS,
	TK_THROW,
	TK_TRUE,
	TK_TRY,
	TK_TYPEOF,
	TK_VAR,
	TK_VOID,
	TK_WHILE,
	TK_DEBUG,

	/* single-character punctuators */
    TK_BRACE_LEFT,		// {}
    TK_BRACE_RIGHT,
    TK_PAREN_LEFT,		// ()
    TK_PAREN_RIGHT,
    TK_BRACKET_LEFT,	// []
    TK_BRACKET_RIGHT,

    TK_SEMICOLON,
    TK_COMMA,
	TK_POINT,
	TK_QUEST,
	TK_COLON,

    TK_ASS,
    TK_ADD,
    TK_SUB,
    TK_MUL,
    TK_DIV,
    TK_MOD,
    TK_NOT,
    TK_AND,
    TK_OR,
	TK_XOR,
	TK_BITNOT,
    TK_LT,
	TK_GT,

	/* multi-character punctuators */
	TK_LE,
	TK_GE,
	TK_EQ,
	TK_NE,
	TK_STRICTEQ,
	TK_STRICTNE,
	TK_SHL,
	TK_SHR,
	TK_USHR,
	TK_AND_AND,
	TK_OR_OR,
	TK_ADD_ASS,
	TK_SUB_ASS,
	TK_MUL_ASS,
	TK_DIV_ASS,
	TK_MOD_ASS,
	TK_SHL_ASS,
	TK_SHR_ASS,
	TK_USHR_ASS,
	TK_AND_ASS,
	TK_OR_ASS,
	TK_XOR_ASS,
	TK_INC,
	TK_DEC
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tk_type:    TokenType,
    pub tk_value:   Option<String>,
    pub src_line:   u32,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
enum GeneralTokenType {
    TK_EOF_,
	TK_COMMENT_,
    TK_SYMBOL_,
    TK_STRING_,
    TK_PUNCT_,
}

#[derive(Clone, Debug)]
struct GeneralToken {
    pub tk_type:     GeneralTokenType,
    pub tk_value:    Option<String>,
}

impl GeneralToken {
    fn new(tt: GeneralTokenType) -> Self {
        GeneralToken {
            tk_type: tt,
            tk_value: None
        }
    }

    fn new_with(tt: GeneralTokenType, value: String) -> Self {
        GeneralToken {
            tk_type: tt,
            tk_value: Some(value)
        }
    }
}

///
/// Parsing general token from code string, return next general token.
///
fn next_general_token (script: &str, cursor: usize) -> Result<(GeneralToken, usize), &'static str> {
    // local helper
	#[allow(non_camel_case_types)]
	enum ct {
		CT_LETTER,
		CT_SPACE,
        CT_NEWLN,
		CT_PUNCT,
		CT_EOF,
	}

    #[allow(non_camel_case_types)]
	#[derive(Clone, Debug, PartialEq)]
    enum ps {
		PS_NULL,
		PS_SYMBOL,
        PS_PUNCT,
        PS_STRING_SINGLE,
        PS_STRING_DOUBLE,
        PS_COMMENT_LINE,
		PS_COMMENT_BLOCK,
	}

    fn check_ct(chr: Option<char>) -> ct {
		if  chr == None  {
			return ct::CT_EOF;
		}
		let ch = chr.unwrap();

        if ch == ' ' || ch == '\t' || ch == '\r' {
			return ct::CT_SPACE;
		}
        if ch == '\n' {
            return ct::CT_NEWLN;
        }
        if ch == '+' || ch == '-' || ch == '*' || ch == '/' || ch == '%' || ch == '=' || ch == ';' || ch == '\\' || ch == ':'
            || ch == '&' || ch == '!' || ch == '|' || ch == '^' || ch == ',' || ch == '\'' || ch == '"' || ch == '~' || ch == '?'
            || ch == '<' || ch == '>' || ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == '{' || ch == '}' {
            return ct::CT_PUNCT;
        }
        return ct::CT_LETTER;
	}

    fn check_escape(c: char) -> char {
        if c == 't' {
            return '\t';
        }
        if c == 'n' {
            return '\n';
        }
        return c;
    }

    const VALID_PUNCTS:  [&'static str; 24] =
        [ "<=", ">=", "==", "!=", "===", "!==",
          "<<", ">>", ">>>", "&&", "||",
          "+=", "-=", "*=", "/=", "%=",
          "<<=", ">>=", ">>>=", "&=", "|=", "^=",
          "++", "--"];

    fn check_punct(value: &String) -> bool {
        for i in 0..VALID_PUNCTS.len() {
            if VALID_PUNCTS[i] == value {
                return true;
            }
        }
        return false;
    }

    //
    // main code starting here
    //
    if cursor >= script.len() {
        let eof = GeneralToken::new(GeneralTokenType::TK_EOF_);
        return Ok((eof, cursor));
    }
    let code = &script[cursor..];
	let mut chars = code.chars();
	let mut pos = cursor;

	let mut ps = ps::PS_NULL;
	let mut tkbuf: Vec<char> = Vec::new();

    // executing token parsing LSM
	loop {
        let chr = chars.next();
		let ct = check_ct(chr);
        pos = pos + 1;

        // state handler
        if ps == ps::PS_NULL {
            match ct {
                ct::CT_EOF => {
                    let eof = GeneralToken::new(GeneralTokenType::TK_EOF_);
                    return Ok((eof, pos));
                },
                ct::CT_SPACE => {
                    continue;
                },
                ct::CT_NEWLN => {
                    let ln = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, String::from("\n"));
                    return Ok((ln, pos));
                },
                ct::CT_LETTER => {
                    tkbuf.push( chr.unwrap());
                    ps = ps::PS_SYMBOL;
                    continue;
                },
                ct::CT_PUNCT => {
                    let ch = chr.unwrap();
                    if ch == '\'' {
                        ps = ps::PS_STRING_SINGLE;
                        continue;
                    }
                    if ch == '"' {
                        ps = ps::PS_STRING_DOUBLE;
                        continue;
                    }
                    if ch == ';' {
                        let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, String::from(";"));
                        return Ok((punct, pos));
                    }
                    tkbuf.push(ch);
                    ps = ps::PS_PUNCT;
                    continue;
                },
            }
        }

        // state handler
        if ps == ps::PS_SYMBOL {
            match ct {
                ct::CT_EOF | ct::CT_SPACE => {
                    let value = tkbuf.into_iter().collect();
                    let symbol = GeneralToken::new_with(GeneralTokenType::TK_SYMBOL_, value);
                    return Ok((symbol, pos));
                },
                ct::CT_NEWLN => {
                    let value = tkbuf.into_iter().collect();
                    let symbol = GeneralToken::new_with(GeneralTokenType::TK_SYMBOL_, value);
                    return Ok((symbol, pos - 1));
                },
                ct::CT_LETTER => {
                    tkbuf.push( chr.unwrap());
                    continue;
                },
                ct::CT_PUNCT => {
                    let value = tkbuf.into_iter().collect();
                    let symbol = GeneralToken::new_with(GeneralTokenType::TK_SYMBOL_, value);
                    return Ok((symbol, pos-1));
                }
            }
        }

        // state handler
        if ps == ps::PS_STRING_SINGLE || ps == ps::PS_STRING_DOUBLE {
            match ct {
                ct::CT_EOF => {
                    return Err("Parsing string get end of file!");
                },
                ct::CT_NEWLN | ct::CT_LETTER | ct::CT_SPACE => {
                    tkbuf.push( chr.unwrap());
                    continue;
                },
                ct::CT_PUNCT => {
                    let ch = chr.unwrap();
                    if tkbuf.len() > 0 && tkbuf[tkbuf.len() - 1] == '\0' {
                        let last = tkbuf.len() - 1;
                        tkbuf[last] = check_escape( ch );
                        continue;
                    }
                    if ch == '\'' && ps == ps::PS_STRING_SINGLE {
                        let value = tkbuf.into_iter().collect();
                        let string = GeneralToken::new_with(GeneralTokenType::TK_STRING_, value);
                        return Ok((string, pos));
                    }
                    if ch == '"' && ps == ps::PS_STRING_DOUBLE {
                        let value = tkbuf.into_iter().collect();
                        let string = GeneralToken::new_with(GeneralTokenType::TK_STRING_, value);
                        return Ok((string, pos));
                    }
                    if ch == '\\' {
                         tkbuf.push( '\0' );
                         continue;
                    }
                    tkbuf.push( chr.unwrap());
                    continue;
                }
            }
        }

        // state handler
        if ps == ps::PS_COMMENT_BLOCK {
            match ct {
                ct::CT_EOF => {
                    return Err("Parsing block comment get end of file!");
                },
                ct::CT_PUNCT | ct::CT_LETTER | ct::CT_SPACE | ct::CT_NEWLN => {
                    tkbuf.push( chr.unwrap());

                    if tkbuf.len() >= 2 && tkbuf[tkbuf.len() - 2] == '*' && tkbuf[tkbuf.len() - 1] == '/' {
                        tkbuf.pop();
                        tkbuf.pop();
                        let value = tkbuf.into_iter().collect();
                        let comment = GeneralToken::new_with(GeneralTokenType::TK_COMMENT_, value);
                        return Ok((comment, pos));
                    }
                    continue;
                },
            }
        }

        // state handler
        if ps == ps::PS_COMMENT_LINE {
            match ct {
                ct::CT_NEWLN => {
                    let value = tkbuf.into_iter().collect();
                    let comment = GeneralToken::new_with(GeneralTokenType::TK_COMMENT_, value);
                    return Ok((comment, pos-1));
                },
                ct::CT_EOF => {
                    let value = tkbuf.into_iter().collect();
                    let comment = GeneralToken::new_with(GeneralTokenType::TK_COMMENT_, value);
                    return Ok((comment, pos));
                },
                ct::CT_PUNCT | ct::CT_LETTER | ct::CT_SPACE => {
                    tkbuf.push( chr.unwrap());
                    continue;
                },
            }
        }

        // state handler
        if ps == ps::PS_PUNCT {
            match ct {
                ct::CT_EOF | ct::CT_SPACE => {
                    let value = tkbuf.into_iter().collect();
                    let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, value);
                    return Ok((punct, pos));
                },
                ct::CT_NEWLN => {
                    let value = tkbuf.into_iter().collect();
                    let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, value);
                    return Ok((punct, pos-1));
                },
                ct::CT_LETTER => {
                    let value = tkbuf.into_iter().collect();
                    let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, value);
                    return Ok((punct, pos-1));
                },
                ct::CT_PUNCT => {
                    let ch = chr.unwrap();
                    if ch == ';' {
                        let value = tkbuf.into_iter().collect();
                        let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, value);
                        return Ok((punct, pos-1));
                    }
                    {
                        // check is valid multiple punctuators
                        let mut value = String::new();
                        for i in 0..tkbuf.len() {
                            value.push(tkbuf[i]);
                        }
                        value.push(ch);
                        if value == "//" {
                            tkbuf.clear();
                            ps = ps::PS_COMMENT_LINE;
                            continue;
                        }
                        if value == "/*" {
                            tkbuf.clear();
                            ps = ps::PS_COMMENT_BLOCK;
                            continue;
                        }
                        if check_punct(&value) == true {
                            tkbuf.push(ch);
                            continue;
                        }
                    }
                    let value = tkbuf.into_iter().collect();
                    let punct = GeneralToken::new_with(GeneralTokenType::TK_PUNCT_, value);
                    return Ok((punct, pos-1));
                }
            }
        }
    }
}

impl Token {
    fn new(tt: TokenType, line:u32) -> Self {
        Token {
            tk_type: tt,
            tk_value: None,
            src_line: line,
        }
    }

    fn new_with(tt: TokenType, value: String, line:u32) -> Self {
        Token {
            tk_type: tt,
            tk_value: Some(value),
            src_line: line
        }
    }

    pub fn to_number(&self) -> f64 {
        let symbol: &str = &self.tk_value.as_ref().unwrap();
        return str_to_number(symbol).unwrap();
    }
}

/*
"'break'", "'case'", "'catch'", "'continue'", 
"'default'", "'delete'", "'do'", "'else'", "'false'", "'finally'", "'for'",
"'function'", "'if'", "'in'", "'instanceof'", "'new'", "'null'", "'return'",
"'switch'", "'this'", "'throw'", "'true'", "'try'", "'typeof'", "'var'",
"'void'", "'while'", "'with'",
*/

fn get_keyword(symbol: &str) -> Option<TokenType> {
    match symbol {
        "break" => Some(TokenType::TK_BREAK),
        "case" => Some(TokenType::TK_CASE),
        "catch" => Some(TokenType::TK_CATCH),
        "continue" => Some(TokenType::TK_CONTINUE),
        "default" => Some(TokenType::TK_DEFAULT),
        "delete" => Some(TokenType::TK_DELETE),
        "do" => Some(TokenType::TK_DO),
        "else" => Some(TokenType::TK_ELSE),
        "false" => Some(TokenType::TK_FALSE),
        "finally" => Some(TokenType::TK_FINALLY),
        "for" => Some(TokenType::TK_FOR),
        "function" => Some(TokenType::TK_FUNCTION),
        "if" => Some(TokenType::TK_IF),
        "in" => Some(TokenType::TK_IN),
        "instanceof" => Some(TokenType::TK_INSTANCEOF),
        "new" => Some(TokenType::TK_NEW),

        "null" => Some(TokenType::TK_NULL),
        "undefined" => Some(TokenType::TK_UNDEF),
        "return" => Some(TokenType::TK_RETURN),
        "switch" => Some(TokenType::TK_SWITCH),
        "this" => Some(TokenType::TK_THIS),
        "throw" => Some(TokenType::TK_THROW),
        "true" => Some(TokenType::TK_TRUE),

        "try" => Some(TokenType::TK_TRY),
        "typeof" => Some(TokenType::TK_TYPEOF),
        "var" => Some(TokenType::TK_VAR),
        "void" => Some(TokenType::TK_VOID),
        "while" => Some(TokenType::TK_WHILE),

        "debug" => Some(TokenType::TK_DEBUG),
        _ => None,
    }
}

///
/// Parsing script to tokens
///
fn get_next_token(script: &str,  cursor: usize, line: u32) -> Result<(Token, (usize, u32)), String> {
    fn count_line(comment: &str) -> u32 {
        let mut chars = comment.chars();
        let mut line_count: u32 = 0;
        loop {
            let chr = chars.next();
            if chr.is_some() {
                if chr.unwrap() == '\n' {
                    line_count = line_count + 1;
                }
                continue;
            } else {
                break;
            }
        }
        line_count
    }



    fn get_token_type(punct: &str) -> Option<TokenType> {
        match punct {
            "(" => Some(TokenType::TK_PAREN_LEFT),
            ")" => Some(TokenType::TK_PAREN_RIGHT),
            "[" => Some(TokenType::TK_BRACKET_LEFT),
            "]" => Some(TokenType::TK_BRACKET_RIGHT),
            "{" => Some(TokenType::TK_BRACE_LEFT),
            "}" => Some(TokenType::TK_BRACE_RIGHT),

            "\n" => Some(TokenType::TK_NEWLN),
            ";" => Some(TokenType::TK_SEMICOLON),
            "," => Some(TokenType::TK_COMMA),
            "." => Some(TokenType::TK_POINT),
            "?" => Some(TokenType::TK_QUEST),
            ":" => Some(TokenType::TK_COLON),

            "=" => Some(TokenType::TK_ASS),
            "<" => Some(TokenType::TK_LT),
            ">" => Some(TokenType::TK_GT),
            "!" => Some(TokenType::TK_NOT),
            "&" => Some(TokenType::TK_AND),
            "|" => Some(TokenType::TK_OR),
            "^" => Some(TokenType::TK_XOR),
            "+" => Some(TokenType::TK_ADD),
            "-" => Some(TokenType::TK_SUB),
            "*" => Some(TokenType::TK_MUL),
            "/" => Some(TokenType::TK_DIV),
            "%" => Some(TokenType::TK_MOD),
            "~" => Some(TokenType::TK_BITNOT),

            "<=" => Some(TokenType::TK_LE),
            ">=" => Some(TokenType::TK_GE),
            "==" => Some(TokenType::TK_EQ),
            "!=" => Some(TokenType::TK_NE),
            "<<" => Some(TokenType::TK_SHL),
            ">>" => Some(TokenType::TK_SHR),
            "&&" => Some(TokenType::TK_AND_AND),
            "||" => Some(TokenType::TK_OR_OR),
            "++" => Some(TokenType::TK_INC),
            "--" => Some(TokenType::TK_DEC),
            "+=" => Some(TokenType::TK_ADD_ASS),
            "-=" => Some(TokenType::TK_SUB_ASS),
            "*=" => Some(TokenType::TK_MUL_ASS),
            "/=" => Some(TokenType::TK_DIV_ASS),
            "%=" => Some(TokenType::TK_MOD_ASS),
            "&=" => Some(TokenType::TK_AND_ASS),
            "|=" => Some(TokenType::TK_OR_ASS),
            "^=" => Some(TokenType::TK_XOR_ASS),

            "===" => Some(TokenType::TK_STRICTEQ),
            "!==" => Some(TokenType::TK_STRICTNE),
            ">>>" => Some(TokenType::TK_USHR),
            "<<=" => Some(TokenType::TK_SHL_ASS),
            ">>=" => Some(TokenType::TK_SHR_ASS),
            ">>>>=" => Some(TokenType::TK_USHR_ASS),
            _ => None
        }
    }

    fn check_number(symbol: &str) -> i32 {
        let mut symbol0:String = String::from(symbol);
        symbol0.push_str("+0");
        if symbol0.parse::<f64>().is_ok() {
            return 0;
        }
        if symbol.parse::<f64>().is_ok() {
            return 1;
        }
        if symbol.starts_with("0x") {
            let symbol: &str = &symbol[2..];
            if u64::from_str_radix(&symbol, 16).is_ok() {
                return 1;
            }
        }
        if symbol.starts_with("0b") {
            let symbol: &str = &symbol[2..];
            if u64::from_str_radix(&symbol, 2).is_ok() {
                return 1;
            }
        }
        if symbol == "NaN" {
            return 1;
        }
        if symbol == "Infinity" {
            return 1;
        }
        return -1;
    }

    let mut line = line;
    let mut cursor = cursor;

    // handling general token
    loop {
        let next = next_general_token(&script, cursor);
        if let Err(msg) = next {
            let err_msg = format!("Parsing error @ {} : {}", line, msg);
            return Err(err_msg);
        }

        let (tk, pos) = next.unwrap();
        cursor = pos;
        match tk.tk_type {
            GeneralTokenType::TK_EOF_ => {
                let eof = Token::new(TokenType::TK_EOF, line);
                return Ok((eof, (cursor, line)));
            },
            GeneralTokenType::TK_PUNCT_  => {
                let value = tk.tk_value.unwrap();
                let tkt = get_token_type(&value).unwrap();
                if tkt == TokenType::TK_NEWLN {
                    line = line + 1;
                }
                let ntk = Token::new(tkt, line);
                return Ok((ntk, (cursor, line)));
            },
            GeneralTokenType::TK_STRING_ => {
                let value = tk.tk_value.unwrap();
                line = line + count_line(&value);

                let ntk = Token::new_with(TokenType::TK_STRING, value, line);
                return Ok((ntk, (cursor, line)));
            },
            GeneralTokenType::TK_COMMENT_ => {
                let value = tk.tk_value.unwrap();
                line = line + count_line(&value);
                continue;
            },
            GeneralTokenType::TK_SYMBOL_ => {
                // handler primitive & keyword
                let value = tk.tk_value.unwrap();
                let isnum = check_number(&value);
                if isnum == -1 {
                    if let Some(tkt) = get_keyword(&value) {
                        let ntk = Token::new(tkt, line);
                        return Ok((ntk, (cursor, line)));
                    } else {
                        let ntk = Token::new_with(TokenType::TK_IDENTIFIER, value, line);
                        return Ok((ntk, (cursor, line)));
                    }
                }
                if isnum == 1 {
                    let ntk = Token::new_with(TokenType::TK_NUMBER, value, line);
                    return Ok((ntk, (cursor, line)));
                }

                // isnum == 0
                // concat ieee float string
                if let Ok((tk2, pos2)) = next_general_token(&script, cursor){
                    if tk2.tk_type == GeneralTokenType::TK_PUNCT_ {
                        let value2 = tk2.tk_value.unwrap();
                        if value2 == "+" || value2 == "-" {
                            if let Ok((tk3, pos3)) = next_general_token(&script, pos2) {
                                if tk3.tk_type == GeneralTokenType::TK_SYMBOL_ {
                                    let value3 = tk3.tk_value.unwrap();
                                    let value_all = format!("{}{}{}", value, value2, value3);
                                    if value_all.parse::<f64>().is_ok() {
                                        let ntk = Token::new_with(TokenType::TK_NUMBER, value_all, line);
                                        cursor = pos3;
                                        return Ok((ntk, (cursor, line)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct Tokenlizer<'a> {
    script : &'a str,
    cursor : usize,
    line : u32,
    forward_: LinkedList<(Token, bool)>,

    // help state variable for AST
    pub notin: bool, 
}

impl<'a> Tokenlizer<'a> {
    pub fn new(script:&'a str) -> Self {
        return Tokenlizer {
            script: script,
            cursor: 0,
            line: 1,
            forward_: LinkedList::new(),

            notin: false,
        }
    }

    pub fn next(&mut self) -> Result<Token, String> {
        if self.forward_.len() > 0 {
            let n = self.forward_.pop_front().unwrap().0;
            return Ok(n);
        }

        self.fetch_next()?;

        let n = self.forward_.pop_front().unwrap().0;
        return Ok(n);
    }

    pub fn forward(&mut self) -> Result<Token, String> {
        if self.forward_.len() > 0 {
            let n = self.forward_.front().unwrap().0.clone();
            return Ok(n);
        }

        self.fetch_next()?;

        let n = self.forward_.front().unwrap().0.clone();
        return Ok(n);
    }

    pub fn new_line(&mut self) -> Result<bool, String> {
        if self.forward_.len() > 0 {
            let n = self.forward_.front().unwrap().1;
            return Ok(n);
        }

        self.fetch_next()?;

        let n = self.forward_.front().unwrap().1;
        return Ok(n);
    }

    pub fn line(&self) -> u32 {
        return self.line;
    }

    fn split_identifier(&mut self, token: Token, new_line: bool) {
        assert!(token.tk_type == TokenType::TK_IDENTIFIER);

        let src_line = token.src_line;
        let ident = token.tk_value.unwrap();

        let ids : Vec<String> = ident.replace(".", " . ").split_whitespace().map(|x| x.to_string()).collect();
        for i in 0..ids.len() {
            let id = &ids[i];
            if id != "." {
                let tk = if let Some(tkt) = get_keyword(id) {
                    Token::new(tkt, src_line)
                } else {                        
                    Token {
                        tk_type: TokenType::TK_IDENTIFIER,
                        tk_value: Some(id.to_string()),
                        src_line: src_line,
                    }
                };

                self.forward_.push_back((tk, new_line && i == 0));
            } else {
                let tk = Token {
                    tk_type: TokenType::TK_POINT,
                    tk_value: None,
                    src_line: src_line,
                };
                self.forward_.push_back((tk, new_line && i == 0));
            }
        }
    }

    fn fetch_next(&mut self) -> Result<(), String> {
        let mut new_line = false;
        loop {            
            let result = get_next_token(self.script, self.cursor, self.line);
            if result.is_ok() {
                let (token, (cursor, line)) = result.unwrap();
                if token.tk_type != TokenType::TK_EOF {
                    self.cursor = cursor;
                    self.line = line;
                }

                // skiped new line                
                if token.tk_type == TokenType::TK_NEWLN {
                    new_line = true;
                    continue;
                }

                if token.tk_type != TokenType::TK_IDENTIFIER {
                    self.forward_.push_back((token, new_line));
                    return Ok(());
                }
                self.split_identifier(token, new_line);

                return Ok(());
            }

            let msg = result.err().unwrap();
            return Err(msg);
        }
    }
}

