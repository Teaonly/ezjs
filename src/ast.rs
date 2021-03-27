use crate::token::*;

/* ast stuff */
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AstType {
	AST_NULL = -1,

    AST_LIST = 0,
	AST_FUNDEC,
	AST_IDENTIFIER,

	EXP_IDENTIFIER,
	EXP_NUMBER,
	EXP_STRING,

	/* literals */
	EXP_UNDEF, /* for array elisions */
	EXP_NULL,
	EXP_TRUE,
	EXP_FALSE,
	EXP_THIS,

	EXP_ARRAY,
	EXP_OBJECT,
	EXP_PROP_VAL,
	EXP_PROP_GET,
	EXP_PROP_SET,

	EXP_FUN,

	/* expressions */
	EXP_INDEX,
	EXP_MEMBER,
	EXP_CALL,
	EXP_NEW,

	EXP_POSTINC,
	EXP_POSTDEC,

	EXP_DELETE,
	EXP_VOID,
	EXP_TYPEOF,
	EXP_PREINC,
	EXP_PREDEC,
	EXP_POS,
	EXP_NEG,
	EXP_BITNOT,
	EXP_LOGNOT,

	EXP_MOD,
	EXP_DIV,
	EXP_MUL,
	EXP_SUB,
	EXP_ADD,
	EXP_USHR,
	EXP_SHR,
	EXP_SHL,
	EXP_IN,
	EXP_INSTANCEOF,
	EXP_GE,
	EXP_LE,
	EXP_GT,
	EXP_LT,
	EXP_STRICTNE,
	EXP_STRICTEQ,
	EXP_NE,
	EXP_EQ,
	EXP_BITAND,
	EXP_BITXOR,
	EXP_BITOR,
	EXP_LOGAND,
	EXP_LOGOR,

	EXP_COND,

	EXP_ASS,
	EXP_ASS_MUL,
	EXP_ASS_DIV,
	EXP_ASS_MOD,
	EXP_ASS_ADD,
	EXP_ASS_SUB,
	EXP_ASS_SHL,
	EXP_ASS_SHR,
	EXP_ASS_USHR,
	EXP_ASS_BITAND,
	EXP_ASS_BITXOR,
	EXP_ASS_BITOR,

	EXP_COMMA,

	EXP_VAR, /* var initializer */

	/* statements */
	STM_BLOCK,
	STM_EMPTY,
	STM_VAR,
	STM_IF,
	STM_DO,
	STM_WHILE,
	STM_FOR,
	STM_FOR_VAR,
	STM_FOR_IN,
	STM_FOR_IN_VAR,
	STM_CONTINUE,
	STM_BREAK,
	STM_RETURN,
	STM_SWITCH,
	STM_THROW,
	STM_TRY,

	STM_LABEL,
	STM_CASE,
	STM_DEFAULT,

	STM_DEBUG,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub ast_type:   AstType,
    pub src_line:   u32,
    pub num_value:  Option<f64>,
	pub str_value:  Option<String>,

    pub a:      Option<Box<AstNode>>,
    pub b:      Option<Box<AstNode>>,
    pub c:      Option<Box<AstNode>>,
    pub d:      Option<Box<AstNode>>,
}

/* Local help function */
impl AstNode {
    pub fn null() -> Self {
        AstNode {
            ast_type:  AstType::AST_NULL,
            src_line:  0,
            num_value: None,
            str_value: None,
            a: None,
            b: None,
            c: None,
            d: None
        }
    }

    fn new(ntype: AstType, line: u32) -> Self {
        AstNode {
            ast_type:  ntype,
            src_line:  line,
            num_value: None,
            str_value: None,
            a: None,
            b: None,
            c: None,
            d: None
        }
    }

    fn new_number(ntype: AstType, line: u32, num: f64) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: Some(num),
            str_value: None,
            a: None,
            b: None,
            c: None,
            d: None
        }
    }

    fn new_string(ntype: AstType, line: u32, string: &str) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: None,
            str_value: Some(String::from(string)),
            a: None,
            b: None,
            c: None,
            d: None
        }
    }

    fn new_a(ntype: AstType, line: u32, a: Self) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: None,
            str_value: None,
            a: Some(Box::new(a)),
            b: None,
            c: None,
            d: None
        }
    }

    fn new_a_b(ntype: AstType, line: u32, a: Self, b: Self) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: None,
            str_value: None,
            a: Some(Box::new(a)),
            b: Some(Box::new(b)),
            c: None,
            d: None
        }
    }

    fn new_a_b_c(ntype: AstType, line: u32, a: Self, b: Self, c: Self) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: None,
            str_value: None,
            a: Some(Box::new(a)),
            b: Some(Box::new(b)),
            c: Some(Box::new(c)),
            d: None
        }
    }

    fn new_a_b_c_d(ntype: AstType, line: u32, a: Self, b: Self, c: Self, d: Self) -> Self {
        AstNode {
            ast_type: ntype,
            src_line: line,
            num_value: None,
            str_value: None,
            a: Some(Box::new(a)),
            b: Some(Box::new(b)),
            c: Some(Box::new(c)),
            d: Some(Box::new(d)),
        }
    }

    // linked list
    fn new_list(anode: AstNode) -> Self {
        let mut new_list_item = AstNode::new(AstType::AST_LIST, anode.src_line);
        new_list_item.a = Some(Box::new(anode));
        return new_list_item;
    }
    fn list_tail_push(&mut self, anode: AstNode) {
        assert!(self.ast_type == AstType::AST_LIST);
        assert!(self.b.is_none());
        let mut new_list_item = AstNode::new(AstType::AST_LIST, anode.src_line);
        new_list_item.a = Some(Box::new(anode));
        self.b = Some(Box::new( new_list_item ));
    }
}

fn tk_accept(tkr: &mut Tokenlizer, tkt: TokenType) -> Result<bool, String> {
    let ntk = tkr.forward()?;
    if ntk.tk_type != tkt {
        return Ok(false);
    }
    tkr.next()?;
    return Ok(true);
}

fn tk_expect(tkr: &mut Tokenlizer, tkt: TokenType) -> Result<Token, String> {
    let ntk = tkr.next()?;
    if ntk.tk_type != tkt {
        return Err(format!("AST error: except {:?} but got {:?} @ {}", tkt, ntk.tk_type, tkr.line()));
    }
    return Ok(ntk);
}

fn tk_lookahead(tkr: &mut Tokenlizer, tkt: TokenType) -> Result<bool, String> {
    let fwd = tkr.forward()?;
    if fwd.tk_type == tkt {
        return Ok(true);
    }
    return Ok(false);
}

fn ast_identifier(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let id = tk_expect(tkr, TokenType::TK_IDENTIFIER)?;
    let node = AstNode::new_string(AstType::AST_IDENTIFIER, tkr.line(), &id.tk_value.unwrap());
    return Ok(node);
}

fn ast_identifier_opt(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let ntk = tkr.forward()?;
    if ntk.tk_type == TokenType::TK_IDENTIFIER {
        tkr.next()?;
        let node = AstNode::new_string(AstType::AST_IDENTIFIER, tkr.line(), &ntk.tk_value.unwrap());
        return Ok(node);
    } else {
        return Ok( AstNode::new(AstType::AST_NULL, tkr.line()));
    }
}


fn ast_propname(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let lookahead = tkr.forward()?;
    if tk_accept(tkr, TokenType::TK_NUMBER)? {
        let value = lookahead.to_number();
        let a = AstNode::new_number(AstType::EXP_NUMBER, tkr.line(), value);
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_STRING)? {
        let a = AstNode::new_string(AstType::EXP_STRING, tkr.line(), &lookahead.tk_value.unwrap());
        return Ok(a);
    }
    return ast_identifier(tkr);
}

fn ast_propassign(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let name = ast_propname(tkr)?;
    let lookahead = tkr.forward()?;

    if lookahead.tk_type != TokenType::TK_COLON && name.ast_type == AstType::AST_IDENTIFIER {
        if name.str_value.as_ref().unwrap() == "get" {
            let null = AstNode::new(AstType::AST_NULL, tkr.line());
            let name = ast_propname(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            let body = ast_funbody(tkr)?;
            let exp = AstNode::new_a_b_c(AstType::EXP_PROP_GET, tkr.line(), name, null, body);
            return Ok(exp);
        }
        if name.str_value.as_ref().unwrap() == "set" {
            let name = ast_propname(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
            let arg = ast_identifier(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            let body = ast_funbody(tkr)?;
            let exp = AstNode::new_a_b_c(AstType::EXP_PROP_GET, tkr.line(), name, arg, body);
            return Ok(exp);
        }
    }
    tk_expect(tkr, TokenType::TK_COLON)?;
    let value = ast_assignment(tkr)?;
    let exp = AstNode::new_a_b(AstType::EXP_PROP_VAL, tkr.line(), name, value);
    return Ok(exp);
}

fn ast_objectliteral(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tkr.forward()?.tk_type == TokenType::TK_BRACE_RIGHT {
        let null = AstNode::new(AstType::AST_NULL, tkr.line());
        return Ok(null);
    }

    let node = ast_propassign(tkr)?;
    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;

    while tk_accept(tkr, TokenType::TK_COMMA)? {
        if tkr.forward()?.tk_type == TokenType::TK_BRACE_RIGHT {
            break;
        }
        AstNode::list_tail_push(tail, ast_propassign(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }
    return Ok(head);
}

fn ast_arrayelement(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tkr.forward()?.tk_type == TokenType::TK_COMMA {
        return Ok(AstNode::new(AstType::EXP_UNDEF, tkr.line()));
    }
    return ast_assignment(tkr);
}

fn ast_arrayliteral(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let node = ast_arrayelement(tkr)?;
    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;

    while tk_accept(tkr, TokenType::TK_COMMA)? {
        AstNode::list_tail_push(tail, ast_arrayelement(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

fn ast_primary(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let lookahead = tkr.forward()?;
    if tk_accept(tkr, TokenType::TK_IDENTIFIER)? {
        let a = AstNode::new_string(AstType::EXP_IDENTIFIER, tkr.line(), &lookahead.tk_value.unwrap());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_STRING)? {
        let a = AstNode::new_string(AstType::EXP_STRING, tkr.line(), &lookahead.tk_value.unwrap());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_NUMBER)? {
        let value = lookahead.to_number();
        let a = AstNode::new_number(AstType::EXP_NUMBER, tkr.line(), value);
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_THIS)? {
        let a = AstNode::new(AstType::EXP_THIS, tkr.line());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_NULL)? {
        let a = AstNode::new(AstType::EXP_NULL, tkr.line());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_UNDEF)? {
        let a = AstNode::new(AstType::EXP_UNDEF, tkr.line());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_TRUE)? {
        let a = AstNode::new(AstType::EXP_TRUE, tkr.line());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_FALSE)? {
        let a = AstNode::new(AstType::EXP_FALSE, tkr.line());
        return Ok(a);
    }
    if tk_accept(tkr, TokenType::TK_BRACE_LEFT)? {
        let a = ast_objectliteral(tkr)?;
        tk_expect(tkr, TokenType::TK_BRACE_RIGHT)?;
        let obj = AstNode::new_a(AstType::EXP_OBJECT, tkr.line(), a);
        return Ok(obj);
    }
    if tk_accept(tkr, TokenType::TK_BRACKET_LEFT)? {
        let a = if tkr.forward()?.tk_type == TokenType::TK_BRACKET_RIGHT {
            AstNode::new(AstType::AST_NULL, tkr.line())
        } else {
            ast_arrayliteral(tkr)?
        };

        tk_expect(tkr, TokenType::TK_BRACKET_RIGHT)?;
        let array = AstNode::new_a(AstType::EXP_ARRAY, tkr.line(), a);
        return Ok(array);
    }
    if tk_accept(tkr, TokenType::TK_PAREN_LEFT)? {
        let a = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        return Ok(a);
    }

    panic!(format!("unexpected token in expression: {:?} @ {}", lookahead, tkr.line()));
}

fn ast_arguments(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tkr.forward()?.tk_type == TokenType::TK_PAREN_RIGHT {
        return Ok(AstNode::new(AstType::AST_NULL, tkr.line()));
    }
    let node = ast_assignment(tkr)?;
    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;

    while tk_accept(tkr, TokenType::TK_COMMA)? {
        AstNode::list_tail_push(tail, ast_assignment(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

fn ast_formula_funexp(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let a = ast_identifier_opt(tkr)?;
    tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
    let b = ast_parameters(tkr)?;
    tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
    let c = ast_funbody(tkr)?;
    let node = AstNode::new_a_b_c(AstType::EXP_FUN, tkr.line(), a, b, c);
    return Ok(node);
}

fn ast_formula_memberexp(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_newexp(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_POINT)? {
            let b = ast_identifier(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_MEMBER, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_BRACKET_LEFT)? {
            let b = ast_expression(tkr)?;
            tk_expect(tkr, TokenType::TK_BRACKET_RIGHT)?;
            a = AstNode::new_a_b(AstType::EXP_INDEX, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_newexp(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tk_accept(tkr, TokenType::TK_NEW)? {
        let a = ast_formula_memberexp(tkr)?;
        if tk_accept(tkr, TokenType::TK_PAREN_LEFT)? {
            let b = ast_arguments(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            return Ok(AstNode::new_a_b(AstType::EXP_NEW, tkr.line(), a, b));
        }
        return Ok(AstNode::new_a(AstType::EXP_NEW, tkr.line(), a));
    }

    if tk_accept(tkr, TokenType::TK_FUNCTION)? {
        return ast_formula_funexp(tkr);
    }
    return ast_primary(tkr);
}

fn ast_formula_callexp(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_newexp(tkr)?;
    loop {
        if tk_accept(tkr, TokenType::TK_POINT)? {
            let b = ast_identifier(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_MEMBER, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_BRACKET_LEFT)? {
            let b = ast_expression(tkr)?;
            tk_expect(tkr, TokenType::TK_BRACKET_RIGHT)?;
            a = AstNode::new_a_b(AstType::EXP_INDEX, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_PAREN_LEFT)? {
            let b = ast_arguments(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            a = AstNode::new_a_b(AstType::EXP_CALL, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_postfix(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let a = ast_formula_callexp(tkr)?;
    if tkr.new_line()? == false {
        if tk_accept(tkr, TokenType::TK_INC)? {
            let node = AstNode::new_a(AstType::EXP_POSTINC, tkr.line(), a);
            return Ok(node);
        }
        if tk_accept(tkr, TokenType::TK_DEC)? {
            let node = AstNode::new_a(AstType::EXP_POSTDEC, tkr.line(), a);
            return Ok(node);
        }
    }
    return Ok(a);
}

fn ast_formula_unary(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tk_accept(tkr, TokenType::TK_DELETE)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_DELETE, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_VOID)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_VOID, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_TYPEOF)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_TYPEOF, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_INC)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_PREINC, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_DEC)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_PREDEC, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_BITNOT)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_BITNOT, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_NOT)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_LOGNOT, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_SUB)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_NEG, tkr.line(), a);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_ADD)? {
        let a = ast_formula_unary(tkr)?;
        let stm = AstNode::new_a(AstType::EXP_POS, tkr.line(), a);
        return Ok(stm);
    }
    return ast_formula_postfix(tkr);
}

fn ast_formula_multiplicative(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_unary(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_MUL)? {
            let b = ast_formula_unary(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_MUL, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_DIV)? {
            let b = ast_formula_unary(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_DIV, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_MOD)? {
            let b = ast_formula_unary(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_MOD, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_additive(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_multiplicative(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_ADD)? {
            let b = ast_formula_multiplicative(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_ADD, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_SUB)? {
            let b = ast_formula_multiplicative(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_SUB, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_shift(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_additive(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_SHL)? {
            let b = ast_formula_additive(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_SHL, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_SHR)? {
            let b = ast_formula_additive(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_SHR, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_USHR)? {
            let b = ast_formula_additive(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_USHR, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_relational(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_shift(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_LT)? {
            let b = ast_formula_shift(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_LT, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_GT)? {
            let b = ast_formula_shift(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_GT, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_LE)? {
            let b = ast_formula_shift(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_LE, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_GE)? {
            let b = ast_formula_shift(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_GE, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_INSTANCEOF)? {
            let b = ast_formula_shift(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_INSTANCEOF, tkr.line(), a, b);
            continue;
        }
        if !tkr.notin {
            if tk_accept(tkr, TokenType::TK_IN)? {
                let b = ast_formula_shift(tkr)?;
                a = AstNode::new_a_b(AstType::EXP_IN, tkr.line(), a, b);
                continue;
            }
        } 
        break;
    }
    return Ok(a);
}

fn ast_formula_eq(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_relational(tkr)?;

    loop {
        if tk_accept(tkr, TokenType::TK_EQ)? {
            let b = ast_formula_relational(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_EQ, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_NE)? {
            let b = ast_formula_relational(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_NE, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_STRICTEQ)? {
            let b = ast_formula_relational(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_STRICTEQ, tkr.line(), a, b);
            continue;
        }
        if tk_accept(tkr, TokenType::TK_STRICTNE)? {
            let b = ast_formula_relational(tkr)?;
            a = AstNode::new_a_b(AstType::EXP_STRICTNE, tkr.line(), a, b);
            continue;
        }
        break;
    }
    return Ok(a);
}

fn ast_formula_bitand(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_eq(tkr)?;
    while tk_accept(tkr, TokenType::TK_AND)? {
        let b = ast_formula_eq(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_BITAND, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_formula_bitxor(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_bitand(tkr)?;
    while tk_accept(tkr, TokenType::TK_XOR)? {
        let b = ast_formula_bitand(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_BITXOR, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_formula_bitor(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_bitxor(tkr)?;
    while tk_accept(tkr, TokenType::TK_OR)? {
        let b = ast_formula_bitxor(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_BITOR, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_formula_and(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_bitor(tkr)?;
    while tk_accept(tkr, TokenType::TK_AND_AND)? {
        let b = ast_formula_bitor(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_LOGAND, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_formula_or(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_and(tkr)?;
    while tk_accept(tkr, TokenType::TK_OR_OR)? {
        let b = ast_formula_and(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_LOGOR, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_formula(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_formula_or(tkr)?;
    if tk_accept(tkr, TokenType::TK_QUEST)? {
        let b = ast_assignment(tkr)?;
        tk_expect(tkr, TokenType::TK_COLON)?;
        let c = ast_assignment(tkr)?;
        a = AstNode::new_a_b_c(AstType::EXP_COND, tkr.line(), a, b, c);
    }
    return Ok(a);
}

fn ast_assignment(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let a = ast_formula(tkr)?;

    if tk_accept(tkr, TokenType::TK_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_MUL_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_MUL, tkr.line(), a, b);
        return Ok(node);
    }  else if tk_accept(tkr, TokenType::TK_DIV_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_DIV, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_MOD_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_MOD, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_ADD_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_ADD, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_SUB_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_SUB, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_SHL_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_SHL, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_SHR_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_SHR, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_USHR_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_USHR, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_AND_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_BITAND, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_XOR_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_BITXOR, tkr.line(), a, b);
        return Ok(node);
    } else if tk_accept(tkr, TokenType::TK_OR_ASS)? {
        let b = ast_assignment(tkr)?;
        let node = AstNode::new_a_b(AstType::EXP_ASS_BITOR, tkr.line(), a, b);
        return Ok(node);
    }
    return Ok(a);
}

fn ast_expression(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let mut a = ast_assignment(tkr)?;
    while tk_accept(tkr, TokenType::TK_COMMA)? {
        let b = ast_assignment(tkr)?;
        a = AstNode::new_a_b(AstType::EXP_COMMA, tkr.line(), a, b);
    }
    return Ok(a);
}

fn ast_vardec(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let a = ast_identifier(tkr)?;
    if tk_accept(tkr, TokenType::TK_ASS)? {
        let b = ast_assignment(tkr)?;
        let exp = AstNode::new_a_b(AstType::EXP_VAR, tkr.line(), a, b);
        return Ok(exp);
    }
    let exp = AstNode::new_a(AstType::EXP_VAR, tkr.line(), a);
    return Ok(exp);
}

fn ast_vardeclist(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let node = ast_vardec(tkr)?;
    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;
    while tk_accept(tkr, TokenType::TK_COMMA)? {
        AstNode::list_tail_push(tail, ast_vardec(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }
    return Ok(head);
}

fn ast_parameters(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let n = tkr.forward()?;
    if n.tk_type == TokenType::TK_PAREN_RIGHT {
        return Ok(AstNode::new(AstType::AST_NULL, tkr.line()));
    }

    let node = ast_identifier(tkr)?;

    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;
    while tk_accept(tkr, TokenType::TK_COMMA)?  {
        AstNode::list_tail_push(tail, ast_identifier(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

fn ast_caseclause(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tk_accept(tkr, TokenType::TK_CASE)? {
        let a = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_COLON)?;
        let b = ast_statement_list(tkr)?;
        let stm = AstNode::new_a_b(AstType::STM_CASE, tkr.line(), a, b);
        return Ok(stm);
    }
    if tk_accept(tkr, TokenType::TK_DEFAULT)? {
        tk_expect(tkr, TokenType::TK_COLON)?;
        let a = ast_statement_list(tkr)?;
        let stm = AstNode::new_a(AstType::STM_DEFAULT, tkr.line(), a);
        return Ok(stm);
    }

    return Err(format!("AST error: unexpected token in switch: {:?} @ {}  (expected 'case' or 'default')", tkr.forward(), tkr.line()));
}

fn ast_semicolon(tkr: &mut Tokenlizer) -> Result<(), String> {
    if tkr.new_line()? {
        return Ok(());
    }

    let lookahead = tkr.forward()?;
    if lookahead.tk_type == TokenType::TK_SEMICOLON {
        tkr.next()?;
        return Ok(());
    }

    if lookahead.tk_type == TokenType::TK_BRACE_RIGHT {
        return Ok(());
    }

    if lookahead.tk_type == TokenType::TK_EOF {
        return Ok(());
    }

    return Err(format!("unexpected token: {:?} (expected ';')", lookahead));
}

fn ast_forexpression(tkr: &mut Tokenlizer, stop: TokenType) -> Result<AstNode, String> {
    if tkr.forward()?.tk_type == stop {
        tk_expect(tkr, stop)?;
        return Ok(AstNode::new(AstType::AST_NULL, tkr.line()));
    }
    let a = ast_expression(tkr)?;
    tk_expect(tkr, stop)?;
    return Ok(a);
}

fn ast_forstatement(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;

    if tk_accept(tkr, TokenType::TK_VAR)? {
        let a = ast_vardeclist(tkr)?;
        if tk_accept(tkr, TokenType::TK_SEMICOLON)? {
            let b = ast_forexpression(tkr, TokenType::TK_SEMICOLON)?;
            let c = ast_forexpression(tkr, TokenType::TK_PAREN_RIGHT)?;
            let d = ast_statement(tkr)?;

            let stm = AstNode::new_a_b_c_d(AstType::STM_FOR_VAR, tkr.line(), a, b, c, d);
            return Ok(stm);
        }
        if tk_accept(tkr, TokenType::TK_IN)? {
            let b = ast_expression(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            let c = ast_statement(tkr)?;

            let stm = AstNode::new_a_b_c(AstType::STM_FOR_IN_VAR, tkr.line(), a, b, c);
            return Ok(stm);
        }
        return Err(format!("unexpected token in for-var-statement: {:?}", tkr.forward()));
    }

    let mut a = AstNode::new(AstType::AST_NULL, tkr.line());
    if tkr.forward()?.tk_type != TokenType::TK_SEMICOLON {
        // inside this expression, we don't accept in operator.
        let old = tkr.notin;
        tkr.notin = true;
        a = ast_expression(tkr)?;
        tkr.notin = old;
    }
    if tk_accept(tkr, TokenType::TK_SEMICOLON)? {
        let b = ast_forexpression(tkr, TokenType::TK_SEMICOLON)?;
        let c = ast_forexpression(tkr, TokenType::TK_PAREN_RIGHT)?;
        let d = ast_statement(tkr)?;
        let stm = AstNode::new_a_b_c_d(AstType::STM_FOR, tkr.line(), a, b, c, d);
        return Ok(stm);

    }
    if tk_accept(tkr, TokenType::TK_IN)? {
        let b = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        let c = ast_statement(tkr)?;

        let stm = AstNode::new_a_b_c(AstType::STM_FOR_IN, tkr.line(), a, b, c);
        return Ok(stm);
    }

    return Err(format!("unexpected token in for-statement: {:?}", tkr.forward()));
}

fn ast_caselist(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let node = ast_caseclause(tkr)?;

    let mut head = AstNode::new_list( node );
    let mut tail: &mut AstNode = &mut head;
    while tkr.forward()?.tk_type != TokenType::TK_BRACE_RIGHT  {
        AstNode::list_tail_push(tail, ast_caseclause(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }
    return Ok(head);
}

fn ast_statement_list(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let tk = tkr.forward()?;
    if tk.tk_type == TokenType::TK_BRACE_RIGHT || tk.tk_type == TokenType::TK_CASE || tk.tk_type == TokenType::TK_DEFAULT {
        return Ok(AstNode::new(AstType::AST_NULL, tkr.line()));
    }
    let mut head = AstNode::new_list( ast_statement(tkr)?);

    let mut tail: &mut AstNode = &mut head;
    loop {
        let tk = tkr.forward()?;
        if tk.tk_type == TokenType::TK_BRACE_RIGHT || tk.tk_type == TokenType::TK_CASE || tk.tk_type == TokenType::TK_DEFAULT {
            break;
        }
        AstNode::list_tail_push(tail, ast_statement(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

fn ast_block(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let _leftbrace = tk_expect(tkr, TokenType::TK_BRACE_LEFT)?;
    let a = ast_statement_list(tkr)?;
    tk_expect(tkr, TokenType::TK_BRACE_RIGHT)?;
    return Ok( AstNode::new_a(AstType::STM_BLOCK, tkr.line(), a) );
}

fn ast_statement(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tkr.forward()?.tk_type == TokenType::TK_BRACE_LEFT {
        return ast_block(tkr);

    } else if tk_accept(tkr, TokenType::TK_VAR)? {
        let a = ast_vardeclist(tkr)?;
        ast_semicolon(tkr)?;
        let stm = AstNode::new_a(AstType::STM_VAR, tkr.line(), a);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_SEMICOLON)? {
        return Ok( AstNode::new(AstType::STM_EMPTY, tkr.line()) );

    } else if tk_accept(tkr, TokenType::TK_IF)? {
        tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
        let a = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        let b = ast_statement(tkr)?;
        if tk_accept(tkr, TokenType::TK_ELSE)? {
            let c = ast_statement(tkr)?;
            return Ok(AstNode::new_a_b_c(AstType::STM_IF, tkr.line(), a, b, c));
        }
        return Ok(AstNode::new_a_b(AstType::STM_IF, tkr.line(), a, b));

    } else if tk_accept(tkr, TokenType::TK_DO)? {
        let a = ast_statement(tkr)?;
        tk_expect(tkr, TokenType::TK_WHILE)?;
        tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
        let b = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        ast_semicolon(tkr)?;

        let stm = AstNode::new_a_b(AstType::STM_DO, tkr.line(), a, b);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_WHILE)? {
        tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
        let a = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        let b = ast_statement(tkr)?;

        let stm = AstNode::new_a_b(AstType::STM_WHILE, tkr.line(), a, b);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_FOR)? {
        let stm = ast_forstatement(tkr)?;
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_CONTINUE)? {
        let id_opt = ast_identifier_opt(tkr)?;
        ast_semicolon(tkr)?;
        let stm = AstNode::new_a(AstType::STM_CONTINUE, tkr.line(), id_opt);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_BREAK)? {
        let id_opt = ast_identifier_opt(tkr)?;
        ast_semicolon(tkr)?;
        let stm = AstNode::new_a(AstType::STM_BREAK, tkr.line(), id_opt);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_RETURN)? {

        let ntk = tkr.forward()?;
        if ntk.tk_type != TokenType::TK_SEMICOLON && ntk.tk_type != TokenType::TK_BRACE_RIGHT {            
            let a = ast_expression(tkr)?;
            let stm = AstNode::new_a(AstType::STM_RETURN, tkr.line(), a);
            return Ok(stm);
        }

        ast_semicolon(tkr)?;
        let a = AstNode::new(AstType::AST_NULL, tkr.line());
        let stm = AstNode::new_a(AstType::STM_RETURN, tkr.line(), a);

        return Ok(stm);
        
    } else if tk_accept(tkr, TokenType::TK_SWITCH)? {
        tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
        let a = ast_expression(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        tk_expect(tkr, TokenType::TK_BRACE_LEFT)?;
        if tk_accept(tkr, TokenType::TK_BRACE_RIGHT)? {
            let stm = AstNode::new_a(AstType::STM_SWITCH, tkr.line(), a);
            return Ok(stm);
        }
        let b = ast_caselist(tkr)?;
        tk_expect(tkr, TokenType::TK_BRACE_RIGHT)?;
        let stm = AstNode::new_a_b(AstType::STM_SWITCH, tkr.line(), a, b);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_THROW)? {
        let a = ast_expression(tkr)?;
        ast_semicolon(tkr)?;

        let stm = AstNode::new_a(AstType::STM_THROW, tkr.line(), a);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_TRY)? {
        let a = ast_block(tkr)?;
        if tk_accept(tkr, TokenType::TK_CATCH)? {
            tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
            let b = ast_identifier(tkr)?;
            tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
            let c = ast_block(tkr)?;

            if tk_accept(tkr, TokenType::TK_FINALLY)? {
                let d = ast_block(tkr)?;
                let stm = AstNode::new_a_b_c_d(AstType::STM_TRY, tkr.line(), a, b, c, d);
                return Ok(stm);
            }
            let stm = AstNode::new_a_b_c(AstType::STM_TRY, tkr.line(), a, b, c);
            return Ok(stm);
        }
        if tk_accept(tkr, TokenType::TK_FINALLY)? {
            let b = ast_block(tkr)?;
            let stm = AstNode::new_a_b(AstType::STM_TRY, tkr.line(), a, b);
            return Ok(stm);
        }
        return Err(format!("unexpected token in try: {:?} (expected 'catch' or 'finally')", tkr.forward()? ));

    } else if tk_accept(tkr, TokenType::TK_FUNCTION)? {
        let a = ast_identifier(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
        let b = ast_parameters(tkr)?;
        tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
        let c = ast_funbody(tkr)?;

        /* rewrite function statement as "var X = function X() {}" */
        let aa = a.clone();
        let fun = AstNode::new_a_b_c(AstType::EXP_FUN, tkr.line(), a, b, c);
        let var = AstNode::new_a_b(AstType::EXP_VAR, tkr.line(), aa, fun);
        let lst = AstNode::new_list(var);
        let stm = AstNode::new_a(AstType::STM_VAR, tkr.line(), lst);
        return Ok(stm);

    } else if tk_accept(tkr, TokenType::TK_DEBUG)? {
        let a = AstNode::new(AstType::STM_DEBUG, tkr.line());
        ast_semicolon(tkr)?;
        return Ok(a);
                
    } else if tk_lookahead(tkr, TokenType::TK_IDENTIFIER)? {
        let mut a = ast_expression(tkr)?;
        if a.ast_type == AstType::EXP_IDENTIFIER {
            if tk_accept(tkr, TokenType::TK_COLON)? {
                a.ast_type = AstType::AST_IDENTIFIER;
                let b = ast_statement(tkr)?;
                let stm = AstNode::new_a_b(AstType::STM_LABEL, tkr.line(), a, b);
                return Ok(stm);
            }
        }
        ast_semicolon(tkr)?;
        return Ok(a);     
    }

    let stm = ast_expression(tkr)?;
    ast_semicolon(tkr)?;
    return Ok(stm);
}

fn ast_funbody(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    tk_expect(tkr, TokenType::TK_BRACE_LEFT)?;
    
    if tk_accept(tkr, TokenType::TK_BRACE_RIGHT)? == true {
        let empty = AstNode::new( AstType::AST_NULL, 0);
        return Ok(empty);
    }
    
    let mut head = AstNode::new_list( ast_element(tkr)?);

    let mut tail: &mut AstNode = &mut head;
    while tk_accept(tkr, TokenType::TK_BRACE_RIGHT)? == false {
        AstNode::list_tail_push(tail, ast_element(tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

fn ast_fundec(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    let a = ast_identifier(tkr)?;
    tk_expect(tkr, TokenType::TK_PAREN_LEFT)?;
    let b = ast_parameters(tkr)?;
    tk_expect(tkr, TokenType::TK_PAREN_RIGHT)?;
    let c = ast_funbody(tkr)?;

    let func = AstNode::new_a_b_c(AstType::AST_FUNDEC, tkr.line(), a, b, c);
    return Ok(func);
}

fn ast_element(tkr: &mut Tokenlizer) -> Result<AstNode, String> {
    if tk_accept(tkr, TokenType::TK_FUNCTION)? {
        return ast_fundec(tkr);
    }
    return ast_statement(tkr);
}

pub fn build_ast_from_script(script: &str) -> Result<AstNode, String> {
    let mut tkr = Tokenlizer::new(script);

    if tk_accept(&mut tkr, TokenType::TK_EOF)? {
        let empty = AstNode::new( AstType::AST_NULL, 0);
        return Ok(empty);
    }

    let mut head = AstNode::new_list( ast_element(&mut tkr)?);

    let mut tail: &mut AstNode = &mut head;
    while tk_accept(&mut tkr, TokenType::TK_EOF)? == false {
        AstNode::list_tail_push(tail, ast_element(&mut tkr)?);
        tail = tail.b.as_mut().unwrap();
    }

    return Ok(head);
}

