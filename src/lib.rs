mod common;
mod token;
mod ast;
mod bytecode;

mod compile;
mod optimizer;

pub mod value;
pub mod runtime;
mod builtin;

use std::convert::TryFrom;
use std::collections::HashMap;
use crate::ast::*;
use crate::bytecode::*;
use crate::compile::*;

use crate::value::*;
use crate::runtime::*;
use crate::builtin::*;

pub fn build_function_from_code(script: &str) -> Result<SharedFunction, String> {
    let ast = build_ast_from_script(script).unwrap();

    let null = AstNode::null();
    let func = compile_func(&null, &null, &ast, true)?;
    return Ok(SharedFunction_new(func));
}

pub fn dump_function(f: &VMFunction) {
    println!("-------------------------------");
    println!("script: {}", f.script);
    println!("functions: {}", f.func_tab.len());
    println!("---num----");
    for n in &f.num_tab {
        println!("{}", n);
    }
    println!("---str----");
    for n in &f.str_tab {
        println!("{}", n);
    }
    println!("---code----");
    
    let mut addr = 0;
    for i in &f.code {
        if let Ok(op) = OpcodeType::try_from(*i) {
			println!("{}\t\tOP: {:?} V: {}", addr, op, i);
		} else {
            println!("{}\t\tV: {}", addr, i);
        }
        addr = addr + 1;
    }

    println!("---functions---");
    for i in &f.func_tab {
        dump_function( i );
    }
    println!("----------END-----------");
}

pub fn new_runtime<T: Hookable>(root: T) -> JsRuntime<T> {	
	let prototypes = JsPrototype {
		object_prototype:		SharedObject_new(JsObject::new()),
		string_prototype:		SharedObject_new(JsObject::new()),
		array_prototype:		SharedObject_new(JsObject::new()),
		function_prototype:		SharedObject_new(JsObject::new()),
		exception_prototype:	SharedObject_new(JsObject::new()),
	};

	let genv = JsEnvironment::new();
	let cenv = genv.clone();

	let mut runtime = JsRuntime {
		builtins:	Vec::new(),
		prototypes:	prototypes,
		genv:		genv,
		cenv:		cenv,
		stack:		Vec::new(),

		hooks:		HashMap::new(),
		hooks_id:	0,
		root:		root,
	};

	// init prototypes
	prototypes_init(&mut runtime);
	builtin_init(&mut runtime);
	
	return runtime;
}

pub fn run_script<T:Hookable>(rt: &mut JsRuntime<T>, vmf: SharedFunction) -> Result<SharedValue, String> {
	assert!( vmf.script == true);
	let fobj = SharedObject_new(JsObject::new_function(vmf, rt.genv.clone()));
	let thiz = rt.genv.borrow().target(); 

	rt.push_object(fobj);	// function object
	rt.push_object(thiz);	// this

	let result = jscall(rt, 0);
	if result.is_err() {
		let err_msg = format!("Exceptions: {:?}", result.err().unwrap());
		println!("{}", err_msg);
		rt.stack.clear();
		return Err(err_msg);
	}

	if rt.stack.len() != 1 {
		let err_msg = format!("stack len should be 1 but get {}", rt.stack.len());
		panic!(err_msg);
	}

	let value = rt.stack[0].clone();
	rt.stack.clear();
	return Ok(value);
}

