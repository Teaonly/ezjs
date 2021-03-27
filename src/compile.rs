use std::rc::Rc;

use crate::ast::*;
use crate::bytecode::*;

/* Local help function and struct */
struct AstListIterator<'a> {
    cursor: Option<&'a AstNode>
}

impl<'a> AstListIterator<'a> {
    fn new(lst: &'a AstNode ) -> Self {
        assert!(lst.ast_type == AstType::AST_LIST);
        return AstListIterator {
            cursor: Some(lst),
        }
    }
}

impl<'a> Iterator for AstListIterator<'a> {
    type Item = &'a AstNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.is_none() {
            return None;
        }

        let node = self.cursor.take().unwrap();
        if !node.b.is_none() {
            self.cursor = Some( node.b.as_ref().unwrap() );
        }
        return Some(node.a.as_ref().unwrap());
    }
}

#[allow(dead_code)] 
impl AstNode {
    fn str(&self) -> &str {
        self.str_value.as_ref().unwrap()
    }

    fn a(&self) -> &AstNode {
        return self.a.as_ref().unwrap();
    }
    fn b(&self) -> &AstNode {
        return self.b.as_ref().unwrap();
    }
    fn c(&self) -> &AstNode {
        return self.c.as_ref().unwrap();
    }
    fn d(&self) -> &AstNode {
        return self.d.as_ref().unwrap();
    }
    
    fn has_a(&self) -> bool {
        self.a.is_some()
    }

    fn has_b(&self) -> bool {
        self.b.is_some()
    }
    
    fn has_c(&self) -> bool {
        self.c.is_some()
    }
    
    fn has_d(&self) -> bool {
        self.d.is_some()
    }

    fn is_null(&self) -> bool {
        if self.ast_type == AstType::AST_NULL {
            return true;
        }
        return false;
    }

    fn is_list(&self) -> bool {
        if self.ast_type == AstType::AST_LIST {
            return true;
        }
        return false;
    }

    /*
    fn is_loop(&self) -> bool {        
        let at = self.ast_type;
        if at == AstType::STM_DO || at == AstType::STM_WHILE || 
           at == AstType::STM_FOR || at == AstType::STM_FOR_VAR || 
           at == AstType::STM_FOR_IN ||  at == AstType::STM_FOR_IN_VAR {
            return true;
        }
        return false;
    }
    */

    fn is_func(&self) -> bool {
        let at = self.ast_type;
        if at == AstType::AST_FUNDEC || at == AstType::EXP_FUN || at == AstType::EXP_PROP_GET || at == AstType::EXP_PROP_SET {
            return true;
        }
        return false;
    }

    fn len(&self) -> usize {
        if self.is_list() == false {
            return 0;
        }
        let mut l:usize = 1;

        let mut thiz = self;
        while thiz.b.is_some() {
            l = l + 1;
            thiz = thiz.b.as_ref().unwrap();
        }
        return l;
    }

    fn iter<'a> (&'a self) -> AstListIterator<'a> {
        return AstListIterator::new(self);
    }
}

/* component stuff */
impl VMFunction {
    fn new(script: bool) -> Self {
        VMFunction {
            name:   None,
            script: script,
            numparams: 0,
            numvars: 0,
            code:       Vec::new(),
            num_tab:    Vec::new(),
            str_tab:    Vec::new(),           
            func_tab:   Vec::new(),

            jumps:      Vec::new(),
        }
    }

    fn emit(&mut self, value: u16) {
        self.code.push(value);
    }

    fn emitop(&mut self, op: OpcodeType) {
        self.code.push(op as u16);
    }

    fn emitjump(&mut self, op: OpcodeType) -> usize {
        self.code.push(op as u16);
        let addr = self.code.len();
        if addr > 0xFFFFFFFF {
            panic!("code length is out of 4G!");
        }
        self.code.push(0);
        self.code.push(0);
        return addr;
    }

    fn emitjumpto(&mut self, op: OpcodeType, dst: usize) {
        self.code.push(op as u16);
        self.code.push((dst & 0xFFFF) as u16);
        self.code.push(((dst >> 16) & 0xFFFF) as u16);
    }

    fn emitnumber(&mut self, value:f64) {
        self.emitop(OpcodeType::OP_NUMBER);
        let id = self.addnumber(value);
        self.emit(id);
    }

    fn emitstring(&mut self, op: OpcodeType, var: &str) {
        self.emitop(op);
        let id = self.addstring(var);
        self.emit(id);
    }

    fn emitfunction(&mut self, func: VMFunction) {
        self.emitop(OpcodeType::OP_CLOSURE);
        let id = self.addfunc(func);
        self.emit(id);
    }

    fn addnumber(&mut self, value: f64) -> u16 {
        for i in 0..self.num_tab.len() {
            if self.num_tab[i] == value {
                return i as u16;
            }
        }
        let r = self.num_tab.len();
        self.num_tab.push( value);

        return r as u16;
    }
    fn findstring(&mut self, var: &str) -> (bool, u16) {
        for i in 0..self.str_tab.len() {
            if self.str_tab[i].eq(var) {
                return (true, i as u16);
            }
        }
        return (false, 0);
    }    
    fn addstring(&mut self, var: &str) -> u16 {
        for i in 0..self.str_tab.len() {
            if self.str_tab[i].eq(var) {
                return i as u16;
            }
        }

        let r = self.str_tab.len();
        self.str_tab.push( var.to_string() );

        return r as u16;
    }    

    fn current(& self) -> usize {
        return self.code.len();
    }

    fn label_current_to(&mut self, addr: usize) {
        self.labelto(addr, self.current());
    }

    fn labelto(&mut self, addr:usize,  target_addr: usize) {
        if target_addr > 0xFFFFFFFF {
            panic!("current address is out of 4G!");
        }
        self.code[addr] = (target_addr & 0xFFFF) as u16;
        self.code[addr+1] = ((target_addr >> 16) & 0xFFFF) as u16;
    }

    fn new_scope(&mut self, scope: VMJumpScope) {
        let jump = VMJumpTable{
            scope: scope,
            lst: Vec::new(),
        };
        self.jumps.push(jump);
    }

    fn add_jump(&mut self, scope: usize, jmp: VMJumpType) {
        let jmp_lst = &mut self.jumps[scope].lst;
        jmp_lst.push(jmp);
    }

    fn fill_jumps(&mut self, break_addr: usize, continue_addr: usize) {
        let jmp_lst = self.jumps.last().unwrap();
        for j in &jmp_lst.lst {
            match j {
                VMJumpType::BreakJump(pos) => {
                    self.code[*pos] = (break_addr & 0xFFFF) as u16;
                    self.code[*pos+1] = ((break_addr >> 16) & 0xFFFF) as u16;
                },
                VMJumpType::ContinueJump(pos) => {
                    self.code[*pos] = (continue_addr & 0xFFFF) as u16;
                    self.code[*pos+1] = ((continue_addr >> 16) & 0xFFFF) as u16;
                },
            }
        }
    }

    #[allow(non_camel_case_types)]
    fn target_scope_by_name(&self, name: &str) -> usize {
        let mut brk_index = 0;
        for i in (0..self.jumps.len()).rev() {
            match &self.jumps[i].scope {
                VMJumpScope::LabelSection(label) => {
                    if label.eq(name) {
                        brk_index = i + 1;
                        return brk_index;
                    }
                },
                _ => {}
            }
        }
        return brk_index;
    }

    fn target_break_scope(&self) -> usize {
        let mut brk_index = 0;
        for i in (0..self.jumps.len()).rev() {
            match &self.jumps[i].scope {
                VMJumpScope::ForLoop | VMJumpScope::ForInLoop | VMJumpScope::DoLoop | VMJumpScope::WhileLoop | VMJumpScope::SwitchScope => {
                    brk_index = i + 1;
                    break;
                },
                _ => {}
            }
        }
        return brk_index;
    }

    fn target_continue_scope(&self) -> usize {
        let mut brk_index = 0;
        for i in (0..self.jumps.len()).rev() {
            match &self.jumps[i].scope {
                VMJumpScope::ForLoop | VMJumpScope::ForInLoop | VMJumpScope::DoLoop | VMJumpScope::WhileLoop => {
                    brk_index = i + 1;
                    break;
                },
                _ => {}
            }
        }
        return brk_index;
    }

    fn delete_scope(&mut self) {
        self.jumps.pop();
    }

    fn addfunc(&mut self, func: VMFunction) -> u16 {
        let r = self.func_tab.len();
        self.func_tab.push(Rc::new(Box::new(func)));
        return r as u16;
    }

    fn parsing_vardec(&mut self, node: &AstNode) {
        if node.is_list() {
            let it = node.iter();
            for n in it {
                self.parsing_vardec(n);
            }
            return;
        }

        if node.is_func() {
            return; /* stop at inner functions */
        }

        if node.ast_type == AstType::EXP_VAR {
            let var_name = node.a().str();
            self.addstring(var_name);
        }

        if node.a.is_some() {
            self.parsing_vardec(node.a());
        }
        if node.b.is_some() {
            self.parsing_vardec(node.b());
        }

        if node.c.is_some() {
            self.parsing_vardec(node.c());
        }

        if node.d.is_some() {
            self.parsing_vardec(node.d());
        }
    }

    fn parsing_fundec(&mut self, lst: &AstNode) {
        if lst.is_list() {
            let it = lst.iter();
            for n in it {
                if n.ast_type == AstType::AST_FUNDEC {
                    let newfunc = compile_func( n.a(), n.b(), n.c(), false).unwrap();
                    let fid = self.addfunc(newfunc);
                    self.emitop(OpcodeType::OP_CLOSURE);
                    self.emit(fid);

                    let name = n.a().str();
                    let vid = self.addstring( &name );
                    self.emitop(OpcodeType::OP_SETVAR);
                    self.emit(vid);
                    self.emitop(OpcodeType::OP_POP);
                }
            }
            return;
        }
    }
}

/* Expressions */
fn compile_object(f: &mut VMFunction, lst: &AstNode) {
    if lst.is_null() {
        return;
    }

    let it = lst.iter();
    for kv in it {
        let prop = kv.a();
        match prop.ast_type {
            AstType::AST_IDENTIFIER | AstType::EXP_STRING => {
                let prop_str = prop.str();
                f.emitstring(OpcodeType::OP_STRING, prop_str);
            },
            AstType::EXP_NUMBER => {
                let prop_number = prop.num_value.unwrap();
                f.emitnumber(prop_number);
            },
            _ => {
                panic!("invalid property name in object initializer");
            }
        }

        /*    
		if (F->strict)
            checkdup(J, F, head, kv);
        */
        match kv.ast_type {
            AstType::EXP_PROP_VAL => {
                compile_exp(f, kv.b());
                f.emitop(OpcodeType::OP_INITPROP);
            },
            AstType::EXP_PROP_GET => {
                let null = AstNode::null();
                let func = compile_func( &null, &null, kv.c(), false).unwrap();
                f.emitfunction(func);
                f.emitop(OpcodeType::OP_INITGETTER);                
            },
            AstType::EXP_PROP_SET => {
                let null = AstNode::null();
                let func = compile_func( &null, kv.b(), kv.c(), false).unwrap();
                f.emitfunction(func);
                f.emitop(OpcodeType::OP_INITSETTER);  
            },
            _ => {
                panic!("invalid value type in object initializer");
            }
        }
    }
}

fn compile_array(f: &mut VMFunction, lst: &AstNode) {
    if lst.is_null() {
        return;
    }
    let mut i:u16 = 0;
    let it = lst.iter();
    for n in it {
        if n.ast_type != AstType::EXP_UNDEF {
            f.emitop(OpcodeType::OP_INTEGER);
            f.emit(i);
            compile_exp(f, n);
            f.emitop(OpcodeType::OP_INITPROP);
        }
        i = i + 1;
    }
}

fn compile_delete(f: &mut VMFunction, exp: &AstNode) {
    let arg = exp.a();
    match arg.ast_type {

        AstType::EXP_INDEX => {
            compile_exp(f, arg.a());
            compile_exp(f, arg.b());
            f.emitop(OpcodeType::OP_DELPROP);
        },
        AstType::EXP_MEMBER => {
            compile_exp(f, arg.a());
            let member_str = arg.b().str();
            f.emitstring(OpcodeType::OP_DELPROP_S, member_str);
        },
        AstType::EXP_IDENTIFIER => {
            panic!("delete on an unqualified name is not allowed in strict mode");
        },
        _ => {
            panic!("invalid l-value in delete expression");
        }
    }
}

fn compile_typeof(f: &mut VMFunction, exp: &AstNode) {
    if exp.a().ast_type == AstType::EXP_IDENTIFIER {
        let var_str = exp.a().str();
        f.emitstring(OpcodeType::OP_HASVAR, var_str);
    } else {
        compile_exp(f, exp.a());
    }
    f.emitop(OpcodeType::OP_TYPEOF);
}

fn compile_unary(f: &mut VMFunction, exp: &AstNode, op: OpcodeType) {
    compile_exp(f, exp.a());
    f.emitop(op);
}

fn compile_binary(f: &mut VMFunction, exp: &AstNode, op: OpcodeType) {
    compile_exp(f, exp.a());
    compile_exp(f, exp.b());
    f.emitop(op);
}

fn compile_assignop(f: &mut VMFunction, var: &AstNode, op: OpcodeType, is_post: bool) {
    match var.ast_type {
        AstType::EXP_IDENTIFIER => {
            let id_str = var.str();
            f.emitstring(OpcodeType::OP_GETVAR, id_str);
            f.emitop(op);
            if is_post {
                f.emitop(OpcodeType::OP_ROT2);
            }
            f.emitstring(OpcodeType::OP_SETVAR, id_str);
            if is_post {
                f.emitop(OpcodeType::OP_POP);
            }
        },
        AstType::EXP_INDEX => {
            compile_exp(f, var.a());
            compile_exp(f, var.b());
            f.emitop(OpcodeType::OP_DUP2);
            f.emitop(OpcodeType::OP_GETPROP);
            f.emitop(op);
            if is_post {
                f.emitop(OpcodeType::OP_ROT4);
            }
            f.emitop(OpcodeType::OP_SETPROP);
            if is_post {
                f.emitop(OpcodeType::OP_POP);
            }
        },
        AstType::EXP_MEMBER => {
            compile_exp(f, var.a());
            f.emitop(OpcodeType::OP_DUP);
            let member_str = var.b().str();
            f.emitstring(OpcodeType::OP_GETPROP_S, member_str);
            f.emitop(op);
            if is_post {
                f.emitop(OpcodeType::OP_ROT3);
            }
            f.emitstring(OpcodeType::OP_SETPROP_S, member_str);
            if is_post {
                f.emitop(OpcodeType::OP_POP);
            }
        },
        _ => {
            panic!("invalid l-value in assignment");
        }
    }
}

fn compile_assignwith(f: &mut VMFunction, exp: &AstNode, op: OpcodeType) {
    let var = exp.a();
    let rhs = exp.b();

    match var.ast_type {
        AstType::EXP_IDENTIFIER => {
            let id_str = var.str();
            f.emitstring(OpcodeType::OP_GETVAR, id_str);
            compile_exp(f, rhs);
            f.emitop(op);
            f.emitstring(OpcodeType::OP_SETVAR, id_str);
        },
        AstType::EXP_INDEX => {
            compile_exp(f, var.a());
            compile_exp(f, var.b());
            f.emitop(OpcodeType::OP_DUP2);
            f.emitop(OpcodeType::OP_GETPROP);
            compile_exp(f, rhs);
            f.emitop(op);
            f.emitop(OpcodeType::OP_SETPROP);
        },
        AstType::EXP_MEMBER => {
            compile_exp(f, var.a());
            f.emitop(OpcodeType::OP_DUP);
            let member_str = var.b().str();
            f.emitstring(OpcodeType::OP_GETPROP_S, member_str);
            compile_exp(f, rhs);
            f.emitop(op);
            f.emitstring(OpcodeType::OP_SETPROP_S, member_str);
        },
        _ => {
            panic!("invalid l-value in assignment");
        }
    }
}

fn compile_assign(f: &mut VMFunction, exp: &AstNode) {
    let var = exp.a();
    let rhs = exp.b();

    match var.ast_type {
        AstType::EXP_IDENTIFIER => {
            let id_str = var.str();
            compile_exp(f, rhs);
            f.emitstring(OpcodeType::OP_SETVAR, id_str);
        },
        AstType::EXP_INDEX => {
            compile_exp(f, var.a());
            compile_exp(f, var.b());
            compile_exp(f, rhs);
            f.emitop(OpcodeType::OP_SETPROP);
        },
        AstType::EXP_MEMBER => {            
            let member_str = var.b().str();
            compile_exp(f, var.a());
            compile_exp(f, rhs);
            f.emitstring(OpcodeType::OP_SETPROP_S, member_str);
        },
        _ => {
            panic!("invalid l-value in assignment");
        }
    }
}

fn compile_args(f: &mut VMFunction, lst: &AstNode) -> u16 {
    if lst.is_null() {
        return 0;
    }
    let mut num:u16 = 0;
    let it = lst.iter();
    for n in it {
        compile_exp(f, n);
        num = num + 1;
    }
    return num;
}

fn compile_call(f: &mut VMFunction, exp: &AstNode) {
    let fun = exp.a();
    let args = exp.b();

    match fun.ast_type {
        AstType::EXP_INDEX => {
            compile_exp(f, fun.a());
            f.emitop(OpcodeType::OP_DUP);
            compile_exp(f, fun.b());
            f.emitop(OpcodeType::OP_GETPROP);
            f.emitop(OpcodeType::OP_ROT2);
        },
        AstType::EXP_MEMBER => {
            compile_exp(f, fun.a());
            f.emitop(OpcodeType::OP_DUP);
            let member = fun.b().str();
            f.emitstring(OpcodeType::OP_GETPROP_S, member);            
            f.emitop(OpcodeType::OP_ROT2);      // function object | this object
        },
        _ => {
            compile_exp(f, fun);
            f.emitop(OpcodeType::OP_UNDEF);     // this object is UNDEFINED
        }
    }

    let n = compile_args(f, args);
    f.emitop(OpcodeType::OP_CALL);
    f.emit(n);
}

fn compile_exp(f: &mut VMFunction, exp: &AstNode) {
    match exp.ast_type {
        /* immediately value*/ 
        AstType::EXP_STRING => {
            let value = exp.str();
            f.emitstring(OpcodeType::OP_STRING, value);
        },
        AstType::EXP_NUMBER => {
            let value = exp.num_value.unwrap();
            f.emitnumber(value);
        },
        AstType::EXP_UNDEF => {
            f.emitop(OpcodeType::OP_UNDEF);
        },
        AstType::EXP_NULL => {
            f.emitop(OpcodeType::OP_NULL);
        },
        AstType::EXP_TRUE => {
            f.emitop(OpcodeType::OP_TRUE);
        },
        AstType::EXP_FALSE => {
            f.emitop(OpcodeType::OP_FALSE);
        },
        AstType::EXP_THIS => {
            f.emitop(OpcodeType::OP_THIS);
        },

        /* complex value*/
        AstType::EXP_OBJECT => {
            f.emitop(OpcodeType::OP_NEWOBJECT);
            compile_object(f, exp.a());
        },

        AstType::EXP_ARRAY => {
            f.emitop(OpcodeType::OP_NEWARRAY);
            compile_array(f, exp.a());
        },

        AstType::EXP_FUN => {
            let func = compile_func( exp.a(), exp.b(), exp.c(), false).unwrap();
            f.emitfunction(func);
        },

        AstType::EXP_VOID => {
            compile_exp(f, exp.a());
            f.emitop(OpcodeType::OP_POP);
            f.emitop(OpcodeType::OP_UNDEF);
        }

        AstType::EXP_IDENTIFIER => {
            let var_string = exp.str();
            f.emitstring(OpcodeType::OP_GETVAR, var_string);
        },

        AstType::EXP_INDEX => {
            compile_exp(f, exp.a());
            compile_exp(f, exp.b());
            f.emitop(OpcodeType::OP_GETPROP);
        },

        AstType::EXP_MEMBER => {
            compile_exp(f, exp.a());
            let prop_str = exp.b().str();
            f.emitstring(OpcodeType::OP_GETPROP_S, prop_str);
        },

        AstType::EXP_CALL => {
            compile_call(f, exp);
        },

        AstType::EXP_NEW => {
            compile_exp(f, exp.a());
            let n = compile_args(f, exp.b());
            f.emitop(OpcodeType::OP_NEW);
            f.emit(n);
        },
        
        // multiple exps 
        AstType::EXP_COMMA => {
            compile_exp(f, exp.a());
            f.emitop(OpcodeType::OP_POP);
            compile_exp(f, exp.b());
        },
        
        AstType::EXP_LOGOR => {
            compile_exp(f, exp.a());
            f.emitop(OpcodeType::OP_DUP);
            let end = f.emitjump(OpcodeType::OP_JTRUE);
            f.emitop(OpcodeType::OP_POP);
            compile_exp(f, exp.b());
            f.label_current_to(end);
        },

        AstType::EXP_LOGAND => {
            compile_exp(f, exp.a());
            f.emitop(OpcodeType::OP_DUP);
            let end = f.emitjump(OpcodeType::OP_JFALSE);
            f.emitop(OpcodeType::OP_POP);
            compile_exp(f, exp.b());
            f.label_current_to(end);
        },
        
        AstType::EXP_COND => {
            compile_exp(f, exp.a());
            let then = f.emitjump(OpcodeType::OP_JTRUE);
            compile_exp(f, exp.c());
            let end = f.emitjump(OpcodeType::OP_JUMP);
            f.label_current_to(then);
            compile_exp(f, exp.b());
            f.label_current_to(end);
        },

        // Unary operation
        AstType::EXP_DELETE => {
            compile_delete(f, exp);
        },
        AstType::EXP_PREINC => {
            compile_assignop(f, exp.a(), OpcodeType::OP_INC, false);
        },
        AstType::EXP_PREDEC => {
            compile_assignop(f, exp.a(), OpcodeType::OP_DEC, false);
        },
        AstType::EXP_POSTINC => {
            compile_assignop(f, exp.a(), OpcodeType::OP_POSTINC, true);
        },
        AstType::EXP_POSTDEC => {
            compile_assignop(f, exp.a(), OpcodeType::OP_POSTDEC, true);
        },
        AstType::EXP_TYPEOF => {
            compile_typeof(f, exp);
        },
        AstType::EXP_POS => {
            compile_unary(f, exp,  OpcodeType::OP_POS);
        },
        AstType::EXP_NEG => {
            compile_unary(f, exp,  OpcodeType::OP_NEG);
        },
        AstType::EXP_BITNOT => {
            compile_unary(f, exp,  OpcodeType::OP_BITNOT);
        },
        AstType::EXP_LOGNOT => {
            compile_unary(f, exp,  OpcodeType::OP_LOGNOT);
        },

        // Binary operation
        AstType::EXP_BITOR => {
            compile_binary(f, exp,  OpcodeType::OP_BITOR);
        },
        AstType::EXP_BITXOR => {
            compile_binary(f, exp,  OpcodeType::OP_BITXOR);
        },
        AstType::EXP_BITAND => {
            compile_binary(f, exp,  OpcodeType::OP_BITAND);
        },
        AstType::EXP_EQ => {
            compile_binary(f, exp,  OpcodeType::OP_EQ);
        },
        AstType::EXP_NE => {
            compile_binary(f, exp,  OpcodeType::OP_NE);
        },
        AstType::EXP_STRICTEQ => {
            compile_binary(f, exp,  OpcodeType::OP_STRICTEQ);
        },
        AstType::EXP_STRICTNE => {
            compile_binary(f, exp,  OpcodeType::OP_STRICTNE);
        },
        AstType::EXP_LT => {
            compile_binary(f, exp,  OpcodeType::OP_LT);
        },
        AstType::EXP_GT => {
            compile_binary(f, exp,  OpcodeType::OP_GT);
        },
        AstType::EXP_LE => {
            compile_binary(f, exp,  OpcodeType::OP_LE);
        },
        AstType::EXP_GE => {
            compile_binary(f, exp,  OpcodeType::OP_GE);
        },
        AstType::EXP_INSTANCEOF => {
            compile_binary(f, exp,  OpcodeType::OP_INSTANCEOF);
        },
        AstType::EXP_IN => {
            compile_binary(f, exp,  OpcodeType::OP_IN);
        },
        AstType::EXP_SHL => {
            compile_binary(f, exp,  OpcodeType::OP_SHL);
        },
        AstType::EXP_SHR => {
            compile_binary(f, exp,  OpcodeType::OP_SHR);
        },
        AstType::EXP_USHR => {
            compile_binary(f, exp,  OpcodeType::OP_USHR);
        },
        AstType::EXP_ADD => {
            compile_binary(f, exp,  OpcodeType::OP_ADD);
        },
        AstType::EXP_SUB => {
            compile_binary(f, exp,  OpcodeType::OP_SUB);
        },
        AstType::EXP_MUL => {
            compile_binary(f, exp,  OpcodeType::OP_MUL);
        },
        AstType::EXP_DIV => {
            compile_binary(f, exp,  OpcodeType::OP_DIV);
        },
        AstType::EXP_MOD => {
            compile_binary(f, exp,  OpcodeType::OP_MOD);
        },

        // assignments 
        AstType::EXP_ASS => {
            compile_assign(f, exp);
        },
        AstType::EXP_ASS_MUL => {
            compile_assignwith(f, exp, OpcodeType::OP_MUL);
        },
        AstType::EXP_ASS_DIV => {
            compile_assignwith(f, exp, OpcodeType::OP_DIV);
        },
        AstType::EXP_ASS_MOD => {
            compile_assignwith(f, exp, OpcodeType::OP_MOD);
        },
        AstType::EXP_ASS_ADD => {
            compile_assignwith(f, exp, OpcodeType::OP_ADD);
        },
        AstType::EXP_ASS_SUB => {
            compile_assignwith(f, exp, OpcodeType::OP_SUB);
        },
        AstType::EXP_ASS_SHL => {
            compile_assignwith(f, exp, OpcodeType::OP_SHL);
        },
        AstType::EXP_ASS_SHR => {
            compile_assignwith(f, exp, OpcodeType::OP_SHR);
        },
        AstType::EXP_ASS_USHR => {
            compile_assignwith(f, exp, OpcodeType::OP_USHR);
        },
        AstType::EXP_ASS_BITAND => {
            compile_assignwith(f, exp, OpcodeType::OP_BITAND);
        },
        AstType::EXP_ASS_BITXOR => {
            compile_assignwith(f, exp, OpcodeType::OP_BITXOR);
        },
        AstType::EXP_ASS_BITOR => {
            compile_assignwith(f, exp, OpcodeType::OP_BITOR);
        },

        _ => {
            panic!("unknown expression: ({:?})", exp);
        }
    }
}

/* Emit code to rebalance stack and scopes during an abrupt exit */
fn compile_exit(f: &mut VMFunction, scope_index: usize, jump_type: AstType) {
    if f.jumps.len() == 0 {
        return;
    }
    for i in (scope_index .. f.jumps.len()).rev() {
        let scope_type = f.jumps[i].scope.clone();
        match scope_type {
            VMJumpScope::TryScope(stm_d) => {
                f.emitop(OpcodeType::OP_ENDTRY);
                if stm_d.is_some() {
                    compile_stm(f, stm_d.as_ref().unwrap());
                }
            },
            VMJumpScope::CatchScope => {
                f.emitop(OpcodeType::OP_ENDCATCH);
            },
            VMJumpScope::ForInLoop => {
                if jump_type == AstType::STM_BREAK {
                    /* pop the iterator */
                    f.emitop(OpcodeType::OP_POP);
                } else if jump_type == AstType::STM_CONTINUE {
                    if scope_index != i {
                        /* pop the iterator */
                        f.emitop(OpcodeType::OP_POP);
                    }
                } else if jump_type == AstType::STM_RETURN {
                    /* pop the iterator, save the return value */
                    f.emitop(OpcodeType::OP_ROT2);
                    f.emitop(OpcodeType::OP_POP);
                } else {
                    panic!("compile_exit error: only break/continue/return supported!");
                }
            },
            _ => {
                
            }
        }
    }
}

/* Try/catch/finally */
fn compile_trycatchfinally(f: &mut VMFunction, try_block: &AstNode, catch_var: &AstNode, catch_block: &AstNode, finally_block: &AstNode) {
    let l1:usize;
    let l2:usize;
    let l3:usize;

    f.new_scope(VMJumpScope::TryScope(Some(finally_block.clone())));
    l1 = f.emitjump(OpcodeType::OP_TRY);
    {
        /* if we get here, we have caught an exception in the try block */
        l2 = f.emitjump(OpcodeType::OP_TRY);
        {
            /* if we get here, we have caught an exception in the catch block */
            compile_stm(f, finally_block);  /* inline finally block */
            f.emitop(OpcodeType::OP_THROW);
        }
        f.label_current_to(l2);

        let catchvar = catch_var.str();
        f.new_scope(VMJumpScope::CatchScope);
        {
            f.emitstring(OpcodeType::OP_CATCH, catchvar);
            compile_stm(f, catch_block);
            f.emitop(OpcodeType::OP_ENDCATCH);
        }
        f.delete_scope();

        f.emitop(OpcodeType::OP_ENDTRY);
        l3 = f.emitjump(OpcodeType::OP_JUMP);
    }
    f.label_current_to(l1);
    compile_stm(f, try_block);
    f.emitop(OpcodeType::OP_ENDTRY);
    f.delete_scope();

    f.label_current_to(l3);
    compile_stm(f, finally_block);
} 

fn compile_trycatch(f: &mut VMFunction, a: &AstNode, b: &AstNode, c: &AstNode) {
    let l1:usize;
    let l2:usize;

    f.new_scope(VMJumpScope::TryScope(None));
    l1 = f.emitjump(OpcodeType::OP_TRY);
    {
        /* if we get here, we have caught an exception in the try block */
        let catchvar = b.str();
        f.new_scope(VMJumpScope::CatchScope);
        {
            f.emitstring(OpcodeType::OP_CATCH, catchvar);
            compile_stm(f, c);
            f.emitop(OpcodeType::OP_ENDCATCH);
        }
        f.delete_scope();
        l2 = f.emitjump(OpcodeType::OP_JUMP);
    }
    f.label_current_to(l1);
    compile_stm(f, a);
    f.emitop(OpcodeType::OP_ENDTRY);
    f.delete_scope();    
    f.label_current_to(l2);
}

fn compile_finally(f: &mut VMFunction, a: &AstNode, b: &AstNode) {
    let l1:usize;

    l1 = f.emitjump(OpcodeType::OP_TRY);
    f.new_scope(VMJumpScope::TryScope(Some(b.clone())));
    {
        /* if we get here, we have caught an exception in the try block */
        compile_stm(f, b);
        f.emitop(OpcodeType::OP_THROW);
    }
    f.label_current_to(l1);
    compile_stm(f, a);
    f.emitop(OpcodeType::OP_ENDTRY);
    f.delete_scope();

    compile_stm(f, b);
} 

/* Switch */
fn compile_switch(f: &mut VMFunction, stm: &AstNode) {
    let mut def = None;

    compile_exp(f, stm.a());

    let mut case_jumps = Vec::new();

    if stm.has_b() {
        let it = stm.b().iter();
        for clause in it {            
            if clause.ast_type == AstType::STM_CASE {
                compile_exp(f, clause.a());                
                let addr = f.emitjump(OpcodeType::OP_JCASE);
                case_jumps.push(addr);
            } else if clause.ast_type == AstType::STM_DEFAULT {
                if !def.is_none() {
                    panic!("more than one default label in switch");
                }
                def = Some(clause);
            } else {
                panic!("Case list only support STM_CASE and STM_DEFAULT!");

            }
        }
    }
    
    f.emitop(OpcodeType::OP_POP);
    let last_jump = f.emitjump(OpcodeType::OP_JUMP);

    if stm.has_b() {
        let mut i:usize = 0;

        let it = stm.b().iter();
        for clause in it {
            if clause.ast_type == AstType::STM_CASE {
                let addr = case_jumps[i];
                f.label_current_to(addr);
                compile_stmlist(f, clause.b());
                i = i + 1;
            } else if clause.ast_type == AstType::STM_DEFAULT {
                f.label_current_to(last_jump);
                compile_stmlist(f, clause.a());
            }
        }
    }

    if def.is_none() {
        f.label_current_to(last_jump);
    }
}

/* Statements */
fn compile_varinit(f: &mut VMFunction, lst: &AstNode) {
    let it = lst.iter();
    for n in it {
        if n.has_b() {
            compile_exp(f, n.b());
            let var_str = n.a().str();
            f.emitstring(OpcodeType::OP_SETVAR, var_str); 
            f.emitop(OpcodeType::OP_POP);
        }
    }
}

fn compile_assignforin(f: &mut VMFunction, stm: &AstNode) {
    let lhs = stm.a();
    if stm.ast_type == AstType::STM_FOR_IN_VAR {
        if !lhs.is_list() {
            panic!("for var in statement must include an var list!");
        }
        if lhs.has_b() {
            panic!("more than one loop variable in for-in statement");
        }
        let var = lhs.a().a().str();    /* list(var-init(ident)) */
        f.emitstring(OpcodeType::OP_SETVAR, var);
        f.emitop(OpcodeType::OP_POP);
        return;
    }

    if lhs.ast_type != AstType::EXP_IDENTIFIER {
        panic!("invalid l-value in for-in loop assignment");
    }

    let var = lhs.str();
    f.emitstring(OpcodeType::OP_SETVAR, var);
    f.emitop(OpcodeType::OP_POP);
    return;
}

fn compile_stm(f: &mut VMFunction, stm: &AstNode) {
    match stm.ast_type {
        AstType::STM_BLOCK => {
            let block = stm.a.as_ref().unwrap();
            compile_stmlist(f, block);
        },
        AstType::STM_EMPTY => {
            // do nothing
        },
        AstType::STM_VAR => {            
            assert!( stm.a().ast_type == AstType::AST_LIST);
            compile_varinit(f, stm.a());
        },
        AstType::STM_IF => {
            if stm.c.is_some() {
                compile_exp(f, stm.a.as_ref().unwrap());
                let then = f.emitjump(OpcodeType::OP_JTRUE);
                compile_stm(f, stm.c.as_ref().unwrap());
                let end = f.emitjump(OpcodeType::OP_JUMP);
                f.label_current_to(then);
                compile_stm(f, stm.b.as_ref().unwrap());
                f.label_current_to(end);
            } else {
                compile_exp(f, stm.a.as_ref().unwrap());
                let end = f.emitjump(OpcodeType::OP_JFALSE);
                compile_stm(f, stm.b.as_ref().unwrap());
                f.label_current_to(end);
            }
        },
        AstType::STM_DO => {
            f.new_scope(VMJumpScope::DoLoop);
    
            let lop = f.current();
            compile_stm(f, stm.a.as_ref().unwrap());
            let cont = f.current();
            compile_exp(f, stm.b.as_ref().unwrap());
            f.emitjumpto(OpcodeType::OP_JTRUE, lop);
            
            f.fill_jumps(f.current(), cont);
            f.delete_scope();
        },

        AstType::STM_WHILE => {
            f.new_scope(VMJumpScope::WhileLoop);

            let lop = f.current();
            compile_exp(f, stm.a());
            let end = f.emitjump(OpcodeType::OP_JFALSE);
            compile_stm(f, stm.b());
            f.emitjumpto(OpcodeType::OP_JUMP, lop);
            f.label_current_to(end);

            f.fill_jumps(f.current(), lop);
            f.delete_scope();
        },

        AstType::STM_FOR |  AstType::STM_FOR_VAR => {
            f.new_scope(VMJumpScope::ForLoop);

            if stm.ast_type == AstType::STM_FOR_VAR {
                compile_varinit(f, stm.a());
            } else {       
                let a = stm.a();
                if ! a.is_null() {
                    compile_exp(f, a);
                    f.emitop(OpcodeType::OP_POP);
                }
            }

            let lop = f.current();
            let b = stm.b();
            let end = if ! b.is_null() {
                compile_exp(f, b);
                f.emitjump(OpcodeType::OP_JFALSE)
            } else {
                0
            };

            compile_stm(f, stm.d.as_ref().unwrap());

            let cont = f.current();
            let c = stm.c();
            if !c.is_null() {
                compile_exp(f, c);
                f.emitop(OpcodeType::OP_POP);
            }
            f.emitjumpto(OpcodeType::OP_JUMP, lop);

            if end > 0 {
                f.label_current_to(end);
            } 

            f.fill_jumps(f.current(), cont);
            f.delete_scope();
        },
        
        AstType::STM_FOR_IN |  AstType::STM_FOR_IN_VAR => {
            f.new_scope(VMJumpScope::ForInLoop);

            compile_exp(f, stm.b());
            f.emitop(OpcodeType::OP_ITERATOR);
            let lop = f.current();
            
            f.emitop(OpcodeType::OP_NEXTITER);
            let end = f.emitjump(OpcodeType::OP_JFALSE);
            compile_assignforin(f, stm);

            compile_stm(f, stm.c.as_ref().unwrap());
            
            f.emitjumpto(OpcodeType::OP_JUMP, lop);
            f.label_current_to(end);

            f.fill_jumps(f.current(), lop);
            f.delete_scope();
        },
        
        AstType::STM_SWITCH => {
            f.new_scope(VMJumpScope::SwitchScope);
            compile_switch(f, stm);
            f.fill_jumps(f.current(), f.current());
            f.delete_scope();
        },

        AstType::STM_LABEL => {
            let a = stm.a.as_ref().unwrap();
            f.new_scope(VMJumpScope::LabelSection(a.str().to_string()));           
            
            compile_stm(f, stm.b.as_ref().unwrap());
            
            f.fill_jumps(f.current(), f.current());
            f.delete_scope();
        },

        AstType::STM_BREAK => {
            let a = stm.a();
            let break_scope: usize;

            if !a.is_null() {
                let break_target = a.str();                
                break_scope = f.target_scope_by_name(break_target);
            } else {
                break_scope = f.target_break_scope();
            }
            if break_scope == 0 {
                panic!("Can't find break target!");
            }
            
            compile_exit(f, break_scope - 1, AstType::STM_BREAK);
            let from = f.emitjump(OpcodeType::OP_JUMP);
            let jump = VMJumpType::BreakJump(from);
            f.add_jump(break_scope - 1, jump);
        },
        
        AstType::STM_CONTINUE => {
            let a = stm.a();
            let continue_scope: usize;

            if !a.is_null() {
                let continue_target = a.str();
                continue_scope = f.target_scope_by_name(continue_target);
            } else {
                continue_scope = f.target_continue_scope();
            }
            if continue_scope == 0 {
                panic!("Can't find continue target!");                
            }

            compile_exit(f, continue_scope - 1, AstType::STM_CONTINUE);
            let from = f.emitjump(OpcodeType::OP_JUMP);
            let jump = VMJumpType::ContinueJump(from);
            f.add_jump(continue_scope - 1, jump);
        },
        
        AstType::STM_RETURN => {
            if f.script {
                panic!("Find return in script code!");
            }

            let a = stm.a.as_ref().unwrap();
            if a.is_null() {
                f.emitop(OpcodeType::OP_UNDEF);
            } else {
                compile_exp(f, a);
            }
            
            compile_exit(f, 0, AstType::STM_RETURN);
            f.emitop(OpcodeType::OP_RETURN);
        },

        AstType::STM_THROW => {
            compile_exp(f, stm.a.as_ref().unwrap());
            f.emitop(OpcodeType::OP_THROW);
        },

        AstType::STM_TRY => {
            if stm.has_b() && stm.has_c() {
                if stm.has_d() {
                    compile_trycatchfinally(f, stm.a(), stm.b(), stm.c(), stm.d());
                } else {
                    compile_trycatch(f, stm.a(), stm.b(), stm.c());
                }
            } else {
                compile_finally(f, stm.a(), stm.b()); 
            }
        },

        AstType::STM_DEBUG => {
            f.emitop(OpcodeType::OP_DEBUG);
        },

        AstType::AST_FUNDEC => {
            // just skip
        },

        _ => {
            compile_exp(f, stm);
            f.emitop(OpcodeType::OP_POP);
        }    
    }
}

fn compile_stmlist(f: &mut VMFunction, lst: &AstNode) {
    if lst.is_null() {
        return;
    }
    for stm in lst.iter() {
        compile_stm(f, stm);
    }
}

pub fn compile_func(name: &AstNode, params: &AstNode, body: &AstNode, script: bool) -> Result<VMFunction, String> {
    let mut f = VMFunction::new(script);

    // parsing params
    if !params.is_null() {
        f.numparams = params.len();
        let it = params.iter();
        for node in it {
            let name = node.str();
            f.addstring(name);
        }
    }

    if !body.is_null() {
		f.parsing_vardec(body);
        f.numvars = f.str_tab.len() - f.numparams;
		f.parsing_fundec(body);
    }

    if !name.is_null() {
        let name_str = name.str();        
        let (found, _) = f.findstring( name_str );
        if !found {
            f.name = Some(name_str.to_string());
        }
    }

    if f.script {
        f.emitop(OpcodeType::OP_UNDEF);
        compile_stmlist(&mut f, body);
        f.emitop(OpcodeType::OP_RETURN);
    } else {
        compile_stmlist(&mut f, body);
        f.emitop(OpcodeType::OP_UNDEF);
        f.emitop(OpcodeType::OP_RETURN);
    }

    return Ok(f);
}
