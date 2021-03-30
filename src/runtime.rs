use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::bytecode::*;

// runtime stuff
pub type SharedObject = Rc<RefCell<JsObject>>;
pub type SharedScope = Rc<RefCell<JsEnvironment>>;
pub type SharedFunction = Rc<Box<VMFunction>>;

#[allow(non_snake_case)]
pub fn SharedObject_new(obj: JsObject) -> SharedObject {
	Rc::new(RefCell::new(obj))
}
#[allow(non_snake_case)]
pub fn SharedScope_new(scope: JsEnvironment) -> SharedScope {
	Rc::new(RefCell::new(scope))
}
#[allow(non_snake_case)]
pub fn SharedFunction_new(vmf: VMFunction) -> SharedFunction {
	Rc::new(Box::new(vmf))
}

// JsValue for access fast and memory effective 
// to simpilify implementation remvoed prototype for boolean/number
#[allow(non_camel_case_types)]
pub enum JsValue {
	JSUndefined,
	JSNULL,
	JSBoolean(bool),
	JSNumber(f64),	
	JSObject(SharedObject),
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct SharedValue {
	pub v:	Rc<RefCell<JsValue>>,
}

#[allow(non_camel_case_types)]
pub struct JsFunction {	
	pub vmf:	SharedFunction, 
	pub scope:	SharedScope,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct JsIterator {
	pub keys:	Vec<String>,
	pub index:	usize,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct JsException {
	pub msg:	String,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct JsExpander {
	pub ptr: u64,
}

#[allow(non_camel_case_types)]
pub enum JsClass {
	object,
	expand(JsExpander),
	exception(JsException),
	iterator(JsIterator),
	string(String),
	array(Vec<SharedValue>),
	function(JsFunction),
	builtin(usize),
}

#[allow(non_camel_case_types)]
pub struct JsObject {
	pub __proto__:	Option<SharedObject>,
	pub extensible:	bool,
	pub properties: HashMap<String, JsProperty>,
	pub value:	JsClass,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct JsProperty {
	pub value:			SharedValue,
	pub getter:	Option<SharedObject>,
	pub setter:	Option<SharedObject>,

	// attribute flags
	pub attr_writable:		bool,
	pub attr_enumerable: 	bool,
	pub attr_configurable:	bool,
}

pub type JsPropertyAttr = (bool, bool, bool);	//writeable, enumerable, configurable 
pub const JS_DEFAULT_ATTR: JsPropertyAttr = (true, true, true);
pub const JS_READONLY_ATTR: JsPropertyAttr = (false, false, false);

#[allow(non_camel_case_types)]
pub struct JsEnvironment {
	pub variables: SharedObject,		// variables stored in properties 
	pub outer: Option<SharedScope>,
}

#[allow(non_camel_case_types)]
pub struct JsPrototype {
	/* prototype for different objects */
	pub object_prototype:	SharedObject,
	pub string_prototype:	SharedObject,
	pub array_prototype:	SharedObject,
	pub function_prototype: SharedObject,

	/* prototype for exceptions */
	pub exception_prototype: SharedObject,
}

pub trait Expandable : Sized + Clone {
	fn hash(&self) -> u64;
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct JsBuiltinFunction<T> where T: Expandable {
	pub f:		fn(&mut JsRuntime<T>),
	pub argc:	usize,
}

#[allow(non_camel_case_types)]
pub struct JsRuntime<T> where T: Expandable  {
	pub builtins:		Vec<JsBuiltinFunction<T>>,
	pub prototypes:		JsPrototype,

	pub genv:			SharedScope,	
	pub cenv:			SharedScope,

	pub stack:			Vec<SharedValue>,

	pub container:		HashMap<u64, T>,
}


