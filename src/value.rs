use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::common::*;
use crate::bytecode::*;

/* definment for VMFunction/SharedValue/JsValue/JsObject */
pub type SharedFunction = Rc<Box<VMFunction>>;
pub type SharedScope = Rc<RefCell<JsEnvironment>>;
pub type SharedObject = Rc<RefCell<JsObject>>;

#[allow(non_snake_case)]
pub fn SharedScope_new(scope: JsEnvironment) -> SharedScope {
	Rc::new(RefCell::new(scope))
}

#[allow(non_snake_case)]
pub fn SharedObject_new(obj: JsObject) -> SharedObject {
	Rc::new(RefCell::new(obj))
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

/* implementation for VMFunction/SharedValue/JsValue/JsObject */

impl VMFunction {
	pub fn new_anonymous() -> Self {
		let mut anonymous = VMFunction {
            name:   None,
            script: false,
            numparams: 0,
            numvars: 0,
            code:       Vec::new(),
            num_tab:    Vec::new(),
            str_tab:    Vec::new(),           
            func_tab:   Vec::new(),

            jumps:      Vec::new(),
        };
		anonymous.code.push( OpcodeType::OP_UNDEF as u16);
		anonymous.code.push( OpcodeType::OP_RETURN as u16);
		return anonymous;
	}
	pub fn opcode(&self, pc:&mut usize) -> OpcodeType {
		if *pc >= self.code.len() {
			panic!("fetch opcode out of code");
		}
		if let Ok(op) = OpcodeType::try_from(self.code[*pc]) {
			*pc = *pc + 1;
			return op;
		}
		panic!("fetch opcode error!");
	}
	pub fn int(&self, pc:&mut usize) -> f64 {
		if *pc >= self.code.len() {
			panic!("fetch raw out of code");
		}
		let value = self.code[*pc] as f64;
		*pc = *pc + 1;
		return value;
	}
	pub fn number(&self, pc:&mut usize) -> f64 {
		if *pc >= self.code.len() {
			panic!("fetch raw out of code");
		}
		let id = self.code[*pc] as usize;
		if id > self.num_tab.len() {
			panic!("number out of vm");
		}
		let value = self.num_tab[id];

		*pc = *pc + 1;
		return value;
	}
	pub fn string(&self, pc:&mut usize) -> &str {
		if *pc >= self.code.len() {
			panic!("fetch raw out of code");
		}
		let id = self.code[*pc] as usize;
		if id > self.str_tab.len() {
			panic!("string out of vm");
		}

		*pc = *pc + 1;
		return &self.str_tab[id];
	}
	pub fn function(&self, pc:&mut usize) -> SharedFunction {
		if *pc >= self.code.len() {
			panic!("fetch function out of code");			
		}
		let id = self.code[*pc] as usize;
		if id > self.func_tab.len() {
			panic!("function out of vm");
		}
		*pc = *pc + 1;
		return self.func_tab[id].clone();
	}
	pub fn address(&self, pc:&mut usize) -> usize {		
		let addr = self.code[*pc] as usize + (self.code[*pc+1] as usize) * 65536;
		*pc = *pc + 2;
		return addr;
	}
}

impl Clone for JsValue {
	fn clone(&self) -> JsValue {
        match self {
			JsValue::JSUndefined => JsValue::JSUndefined,
			JsValue::JSNULL => JsValue::JSNULL,
			JsValue::JSBoolean(b) => JsValue::JSBoolean(*b),
			JsValue::JSNumber(n) => JsValue::JSNumber(*n),
			JsValue::JSObject(obj) => {
				// only string is primitive
				if obj.borrow().is_string() {					
					JsValue::JSObject(SharedObject_new(obj.borrow().clone_string()))
				} else {
					JsValue::JSObject(obj.clone())
				}
			}
		}
    }
}

impl JsValue {
	pub fn copyfrom(&mut self, other: &Self) {
		*self = other.clone();
	}
}

impl SharedValue {
	pub fn replace(&mut self, other: SharedValue) {
		if self.v.as_ptr() != other.v.as_ptr() {	
			self.v.borrow_mut().copyfrom( &other.v.borrow());
		}
	}
	pub fn duplicate(&self) -> SharedValue {
		let sv = SharedValue::new_null();
		sv.v.borrow_mut().copyfrom( &self.v.borrow() );
		return sv;
	}

	pub fn new_null() -> Self {
		let v = JsValue::JSNULL;		
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn new_undefined() -> Self {
		let v = JsValue::JSUndefined;
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn new_boolean(v:bool) -> Self {
		let v = JsValue::JSBoolean(v);
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn new_number(v:f64) -> Self {
		let v = JsValue::JSNumber(v);
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}	
	pub fn new_vanilla(proto: SharedObject) -> Self {
		let shared_obj = SharedObject_new(JsObject::new_with(proto, JsClass::object));
		let v = JsValue::JSObject(shared_obj);
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn new_object(obj:JsObject) -> Self {
		let shared_obj = SharedObject_new(obj);
		let v = JsValue::JSObject(shared_obj);
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn new_sobject(obj:SharedObject) -> Self {
		let v = JsValue::JSObject(obj);
		SharedValue {
			v: Rc::new(RefCell::new(v))
		}
	}
	pub fn is_null(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSNULL = *v {
			return true;
		}
		return false;
	}
	pub fn is_undefined(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSUndefined = *v {
			return true;
		}
		return false;
	}
	pub fn is_something(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSUndefined = *v {
			return false;
		}
		if let JsValue::JSNULL = *v {
			return false;
		}
		return true;
	}
	pub fn is_object(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSObject(ref _obj) = *v {
			return true;
		}
		return false;
	}
	pub fn get_object(&self) -> SharedObject {
		let v = self.v.borrow();
		if let JsValue::JSObject(ref obj) = *v {
			return obj.clone();
		}
		panic!("JsValue is not an object!");
	}
	pub fn is_boolean(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSBoolean(ref _v) = *v {
			return true;
		}
		return false;
	}
	pub fn to_boolean(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSBoolean(ref v) = *v {
			return *v;
		}
		if self.is_null() {
			return false;
		}
		if self.is_undefined() {
			return false;
		}
		if self.is_number() {
			let v = self.to_number();
			if v != 0.0 {
				return true;
			}
			return false;
		}
		return true;
	}
	pub fn is_number(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSNumber(ref _v) = *v {
			return true;
		}
		return false;
	}
	pub fn to_number(&self) -> f64 {
		let v = self.v.borrow();
		if let JsValue::JSNumber(ref v) = *v {
			return *v;
		}
		if self.is_string() {
			let s = self.to_string();
			if let Some(v) = str_to_number(&s) {
				return v;
			}			
		}
		if self.is_boolean() {
			if self.to_boolean() {
				return 1.0;
			} else {
				return 0.0;
			}
		}
		return std::f64::NAN;
	}
	pub fn is_exception(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSObject(obj) = &*v {
			return obj.borrow().is_exception();
		}
		return false;
	}
	pub fn type_string(&self) -> String {
		let v = self.v.borrow();
		match &*v {
			JsValue::JSUndefined => {
				return "undefined".to_string();
			},
			JsValue::JSNULL => {
				return "object".to_string();
			},
			JsValue::JSBoolean(_b) => {
				return "boolean".to_string();
			},
			JsValue::JSNumber(_num) => {
				return "number".to_string();
			},
			JsValue::JSObject(obj) => {
				return obj.borrow().type_string();
			}
		}
	}
	pub fn is_string(&self) -> bool {
		let v = self.v.borrow();
		if let JsValue::JSObject(obj) = &*v {
			return obj.borrow().is_string();
		}
		return false;
	}
	pub fn to_string(&self) -> String {
		let v = self.v.borrow();
		match &*v {
			JsValue::JSUndefined => {
				return "undefined".to_string();
			},
			JsValue::JSNULL => {
				return "null".to_string();
			},
			JsValue::JSBoolean(b) => {
				if *b {
					return "true".to_string();
				} else {
					return "false".to_string();
				}
			},
			JsValue::JSNumber(num) => {
				return num.to_string();
			},
			JsValue::JSObject(obj) => {
				if obj.borrow().is_string() {
					return obj.borrow().get_string();
				} else {
					return format!("[object:_{}_]", obj.borrow().type_string());
				}
			}
		}
	}
}


impl JsProperty {
	pub fn new() -> Self {
		JsProperty {
			value: SharedValue::new_undefined(),
			attr_writable: true,
			attr_configurable: true,
			attr_enumerable: false,
			getter: None,
			setter: None,
		}
	}
	
	pub fn writeable(&self) -> bool {
		if self.setter.is_none() {
			return self.attr_writable;
		}
		return true;
	}
	pub fn enumerable(&self) -> bool {
		return self.attr_enumerable;
	}
	pub fn configable(&self) -> bool {
		return self.attr_configurable;
	}	
	pub fn fill_attr(&mut self, attr: JsPropertyAttr) {
		if self.attr_configurable {
			self.attr_writable = attr.0;
			self.attr_enumerable = attr.1;
			self.attr_configurable = attr.2;
		}
	}	
	pub fn fill(&mut self, jv: SharedValue, attr: JsPropertyAttr, getter:Option<SharedObject>, setter: Option<SharedObject>) {		
		if self.writeable() {
			self.value = jv;
		}		
		if self.configable() {
			self.getter = getter;
			self.setter = setter;
		}
		self.fill_attr(attr);
	}
}

impl JsException {
	pub fn new(msg: String) -> JsException {
		JsException{
			msg: msg
		}
	}
}

impl JsIterator {
	pub fn new(target_: SharedObject) -> Self {
		let target = target_.borrow();
	
		let mut keys: Vec<String> = Vec::new();
		for x in (*target).properties.keys() {
			if target.properties.get(x).unwrap().enumerable() {
				keys.push(x.to_string());
			}
		}
		JsIterator {
			keys: keys,
			index: 0,
		}
	}
	pub fn next(&mut self) -> Option<String> {
		if self.index >=  self.keys.len() {
			return None;
		}
		let s = self.keys[self.index].clone();
		self.index = self.index + 1;
		return Some(s);
	}
}

impl JsObject {
    pub fn new() -> JsObject {
        JsObject {
			extensible:	true,
            __proto__: None,
            properties: HashMap::new(),
            value: JsClass::object,
        }
	}
	pub fn new_with(prototype: SharedObject, value: JsClass) -> JsObject {
        JsObject {
			extensible:	true,
            __proto__: Some(prototype),
            properties: HashMap::new(),
            value: value
        }
	}
	
	pub fn new_expand(ptr: u64) -> JsObject {
		JsObject {
			extensible:	false,
			__proto__: None,
			properties: HashMap::new(),
			value: JsClass::expand(JsExpander{ptr: ptr}),
		}
	}

	pub fn new_exception(prototype: SharedObject, e: JsException) -> JsObject {		
		JsObject {
			extensible:	false,
			__proto__: Some(prototype),
			properties: HashMap::new(),
			value: JsClass::exception(e),
		}
	}

	pub fn new_iterator(target_: SharedObject) -> JsObject {
		let it = JsIterator::new(target_);
		JsObject {
			extensible:	false,
			__proto__: None,
			properties: HashMap::new(),
			value: JsClass::iterator(it),
		}
	}
	
	pub fn new_function(f: SharedFunction, scope: SharedScope) -> JsObject {
		let fvalue = JsClass::function(JsFunction {
			vmf: f,
			scope: scope,
		});
		JsObject {
			extensible:	false,
			__proto__: None,
			properties: HashMap::new(),
			value: fvalue,
		}
	}

	pub fn clone_string(&self) -> JsObject {
		assert!( self.is_string() );

		let str = self.get_string();
		let new_cls = JsClass::string(str);
		let proto = self.__proto__.as_ref().unwrap().clone();
		JsObject::new_with(proto, new_cls)
	}

	pub fn type_string(&self) -> String {
		match &self.value {
			JsClass::string(_) => {
				"string".to_string()
			},
			JsClass::builtin(_) => {
				"builtin".to_string()
			},
			JsClass::function(_) => {
				"function".to_string()
			},
			JsClass::expand(_) => {
				"expander".to_string()
			},
			_ => {
				"object".to_string()
			}
		}
	}

	pub fn is_vanilla(&self) -> bool {
		if let JsClass::object = self.value {
			return true;
		}
		return false;
	}
	pub fn is_expand(&self) -> bool {
		if let JsClass::expand(_) = self.value {
			return true;
		}
		return false;
	}
	pub fn get_expand(&self) -> JsExpander {
		if let JsClass::expand(ref expa) = self.value {
			return expa.clone();
		}
		panic!("Object can't be a expand!")
	}
	pub fn is_exception(&self) -> bool {
		if let JsClass::exception(_e) = &self.value {
			return true;
		}
		return false;
	}
	pub fn get_exception(&self) -> JsException {
		if let JsClass::exception(e) = &self.value {
			return e.clone();
		}
		panic!("Object can't be a exception!")
	}
	pub fn is_iterator(&self) -> bool {
		if let JsClass::iterator(_) = self.value {
			return true;
		}
		return false;
	}
	pub fn get_iterator(&mut self) -> &mut JsIterator {
		if let JsClass::iterator(ref mut it) = self.value {
			return it;
		}
		panic!("Object can't be a iterator!")
	}		
	pub fn is_builtin(&self) -> bool {
		if let JsClass::builtin(_) = self.value {
			return true;
		}
		return false;
	}	
	pub fn get_builtin(&self) -> usize {
		if let JsClass::builtin(fid) = self.value {
			return fid;
		}
		panic!("Object can't be a builtin!")
	}
	pub fn is_function(&self) -> bool {
		if let JsClass::function(ref _func) = self.value {
			return true;
		}
		return false;
	}
	pub fn get_func(&self) -> &JsFunction {
		if let JsClass::function(ref func) = self.value {
			return func;
		}
		panic!("Object can't be a func!")
	}
	pub fn is_array(&self) -> bool {
		if let JsClass::array(_) = self.value {
			return true;
		}
		return false;
	}
	pub fn get_array(&self) -> &Vec<SharedValue> {
		if let JsClass::array(ref v) = self.value {
			return v;
		}
		panic!("Object can't be a array!")
	}
	pub fn get_mut_array(&mut self) -> &mut Vec<SharedValue> {
		if let JsClass::array(ref mut v) = self.value {
			return v;
		}
		panic!("Object can't be a array!")
	}
	pub fn is_string(&self) -> bool {
		if let JsClass::string(ref _func) = self.value {
			return true;
		}
		return false;
	}
	pub fn get_string(&self) -> String {
		if let JsClass::string(ref s) = self.value {
			return s.to_string();
		}
		panic!("Object can't be a string!")
	}
	pub fn callable(&self) -> bool {
		if self.is_function() || self.is_builtin() {
			return true;
		}
		return false;
	}

	/* property's help functions */
	pub fn query_property(&self, name: &str) -> Option<(JsProperty, bool)> {
		let r = self.properties.get(name);
		if r.is_some() {
			return Some((r.unwrap().clone(), true));
		}

		if self.__proto__.is_some() {
			let proto = self.__proto__.as_ref().unwrap().borrow();
			let result = proto.query_property(name);
			if result.is_some() {
				return Some((result.unwrap().0, false));
			}
			return None;
		}
		return None;
	}
	pub fn get_property(&self, name: &str) -> JsProperty {
		return self.properties.get(name).unwrap().clone();
	}
	pub fn set_property(&mut self, name: &str, prop: JsProperty) {
		self.properties.insert(name.to_string(), prop);
	}
	pub fn put_property(&mut self, name: &str) -> bool {		
		let result = self.properties.get(name);
		if result.is_some() {
			return true;
		}
		if self.extensible == false {
			return false;
		}
		self.properties.insert(name.to_string(), JsProperty::new());
		return true;
	}
	pub fn drop_property(&mut self, name: &str) {
		self.properties.remove(name);
	}
}


impl JsEnvironment {
	pub fn new()  -> SharedScope {
		let env = JsEnvironment {
			variables: SharedObject_new(JsObject::new()),
			outer: None,
		};
		SharedScope_new(env)
	}
	pub fn new_from(outer: SharedScope) -> SharedScope {
		let env = JsEnvironment {
			variables: SharedObject_new(JsObject::new()),
			outer: Some(outer),
		};
		SharedScope_new(env)
	}
	pub fn target(&self) -> SharedObject {
		self.variables.clone()
	}

	pub fn init_var(&mut self, name: &str, jv: SharedValue) {
		let mut prop = JsProperty::new();
		prop.fill(jv, JS_DEFAULT_ATTR, None, None);
		
		if self.variables.borrow_mut().put_property(name) {
			self.variables.borrow_mut().set_property(name, prop);
		}
	}

	pub fn fetch_outer(&self) -> SharedScope {
		if let Some(scope) = &self.outer {
			return scope.clone();
		}
		panic!("Can't fetch outer from env!")
	}
	
	pub fn query_variable(&self, name: &str) -> bool {
		if let Some((_rprop, own)) = self.variables.borrow().query_property(name) {
			if own {
				return true;
			}
		}
		return false;
	}

	pub fn get_variable(&self, name: &str) -> JsProperty {
		self.variables.borrow().get_property(name)
	}

	pub fn put_variable(&self, name: &str) {
		self.variables.borrow_mut().put_property(name);
	}

	pub fn set_variable(&self, name: &str, prop: JsProperty) {
		self.variables.borrow_mut().set_property(name, prop);
	}

	pub fn drop_variable(&self, name: &str) {
		self.variables.borrow_mut().drop_property(name);
	}
}