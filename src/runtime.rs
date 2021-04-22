use std::collections::HashMap;
use std::rc::Rc;
use std::cmp;

use crate::common::*;
use crate::bytecode::*;
use crate::value::*;

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

pub trait Hookable : Clone + Sized {
	fn name(&self) -> String;
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct JsBuiltinFunction<T> where T: Hookable {
	pub f:		fn(&mut JsRuntime<T>),
	pub argc:	usize,
}

#[allow(non_camel_case_types)]
pub struct JsRuntime<T> where T: Hookable  {
	pub builtins:		Vec<JsBuiltinFunction<T>>,
	pub prototypes:		JsPrototype,

	pub genv:			SharedScope,
	pub cenv:			SharedScope,

	pub stack:			Vec<SharedValue>,

	pub hooks:			HashMap<u64, T>,
	pub hooks_id:		u64,
	pub root:			T,
}


/* implementation for JsRuntime and jscall */
impl<T: Hookable> JsRuntime<T> {
	/* hooks */
	pub fn new_hook(&mut self, hook: T) -> JsObject {
		let hid = self.hooks_id;
		self.hooks_id = hid + 1;
		self.hooks.insert(hid, hook);

		JsObject {
			extensible:	false,
			__proto__: None,
			properties: HashMap::new(),
			value: JsClass::hook(hid),
		}
	}
	pub fn get_hook<'a>(&'a mut self, v: &SharedValue) -> &'a T {
		let obj = v.get_object();
		assert!(obj.borrow().is_hook() );
		let hid = obj.borrow().get_hook();

		return self.hooks.get(&hid).unwrap();
	}

	fn check_hook_replace(&mut self, v: &SharedValue) {
		if v.is_object() {
			if v.get_object().borrow().is_hook() {
				if SharedObject::strong_count(&v.get_object()) == 2 {
					let hid = v.get_object().borrow().get_hook();
					self.hooks.remove(&hid);
				}
			}
		}
	}

	/* builtins */
	pub fn new_builtin(&mut self, bf: JsBuiltinFunction<T>) -> JsObject {
		let fid = self.builtins.len();
		self.builtins.push(bf);
		JsObject {
			extensible:	false,
			__proto__: Some(self.prototypes.function_prototype.clone()),
			properties: HashMap::new(),
			value: JsClass::builtin(fid),
		}
	}

	/* environment's variables */
	fn delvariable(&mut self, name: &str) -> bool {
		let mut env: SharedScope = self.cenv.clone();
		loop {
			let r = env.borrow().query_variable(name);
			if r {
				env.borrow().drop_variable(name);
				return true;
			}

			if env.borrow().outer.is_none() {
				return false;
			}
			let r = env.borrow().fetch_outer();
			env = r;
		}
	}

	fn getvariable(&mut self, name: &str) -> Result<bool, JsException> {
		let mut env: SharedScope = self.cenv.clone();
		loop {
			let r = env.borrow().query_variable(name);
			if r {
				let prop = env.borrow().get_variable(name);
				self.push(prop.value.clone());
				return Ok(true);
			}
			if env.borrow().outer.is_none() {
				return Ok(false);
			}
			let r = env.borrow().fetch_outer();
			env = r;
		}
	}

	fn setvariable(&mut self, name: &str) -> Result<(), JsException> {
		let mut env: SharedScope = self.cenv.clone();
		loop {
			let r = env.borrow().query_variable(name);
			if r {
				let mut prop = env.borrow().get_variable(name);
				self.check_hook_replace(&prop.value);
				prop.value.replace(self.top(-1));
				return Ok(());
			}
			if env.borrow().outer.is_none() {
				break;
			}
			let r = env.borrow().fetch_outer();
			env = r;
		}

		let value = self.top(-1);
		self.cenv.borrow().put_variable(name);
		let mut prop = self.cenv.borrow().get_variable(name);
		self.check_hook_replace(&prop.value);
		prop.value.replace(value);
		self.cenv.borrow().set_variable(name, prop);

		return Ok(());
	}

	/* properties operation */
    // make a new  or replace proptery o for object, following reler of object's attr
    fn defproperty(&mut self, target_: SharedObject, name: &str, value: SharedValue,
		attr:JsPropertyAttr, getter: Option<SharedObject>, setter: Option<SharedObject>) -> Result<(), JsException> {
		let mut target = target_.borrow_mut();

		// value/setter/getter/attr can't be conflicted
		if getter.is_some() || setter.is_some() {
			assert!( value.is_undefined() );
		}
		if attr.0 == false && value.is_undefined() {
			assert!( setter.is_some() );
		}

		if target.put_property(name) {
			let mut prop = target.get_property(name);
			if prop.writeable() {
				prop.value = value;
			}
			if prop.configable() {
				prop.fill_attr(attr);
				if let Some(setter) = setter {
					if setter.borrow().callable() {
						prop.setter = Some(setter);
					} else {
						println!("setter should be callable");
					}
				}
				if let Some(getter) = getter {
					if getter.borrow().callable() {
						prop.getter = Some(getter);
					} else {
						println!("getter should be callable");
					}
				}
			}
			target.set_property(name, prop);
			return Ok(());
		} else {
			return Err(JsException::new(format!("runtime TODO: {}", line!())));
		}
	}

	// change value of the proptery for object
	fn setproperty(&mut self, target_: SharedObject, name: &str, value: SharedValue) -> Result<(), JsException> {

		if target_.borrow().is_array() {
			if let Some(number) = str_to_integer(name) {
				let number = number as usize;
				let mut obj = target_.borrow_mut();
				let array = obj.get_mut_array();
				if number == array.len() {
					array.push(value);
					return Ok(());
				} else if number < array.len() {
					array[number] = value;
					return Ok(());
				}
			}
		}

		let prop_r = target_.borrow().query_property(name);
		if let Some((mut prop, own)) = prop_r {
			if let Some(setter) = prop.setter {
				self.push_object(setter.clone());
				self.push_object(target_.clone());
				self.push(value);
				jscall(self, 1)?;
				self.pop(1);
				return Ok(());
			}
            if own {
                if prop.writeable() {
                    self.check_hook_replace(&prop.value);
                    prop.value.replace(value);
                    return Ok(());
                } else {
                    println!("Cant write property for specia object!");
                    return Err(JsException::new(format!("TODO: {}", line!())));
                }
            }
		}

		/* Property not found on this object, so create one with default attr*/
		self.defproperty(target_, name, value, JS_DEFAULT_ATTR, None, None)?;
		return Ok(());
	}

	// get value from the proptery of object
	fn getproperty(&mut self, target: SharedObject, name: &str) -> Result<bool, JsException> {

		// get value from index
		match target.borrow().value {
			JsClass::string(ref s) => {
				if let Ok(idx) = name.parse::<usize>() {
					if idx < s.len() {
						self.push_string( s[idx..idx+1].to_string() );
						return Ok(true);
					}
				}
			},
			JsClass::array(ref v) => {
				if let Ok(idx) = name.parse::<usize>() {
					if idx < v.len() {
						self.push( v[idx].clone() );
						return Ok(true);
					}
				}
			},
			_ => {}
		}
		let prop_r = target.borrow().query_property(name);
		if let Some((prop, _own)) = prop_r {
			if let Some(getter) = prop.getter {
				self.push_object(getter.clone());
				self.push_object(target);
				jscall(self, 0)?;
			} else {
				self.push(prop.value.clone());
			}
			return Ok(true);
		}
		self.push_undefined();
		return Ok(false);
	}

    fn delproperty(&mut self, target_: SharedObject, name: &str) -> bool {
		let mut target = target_.borrow_mut();

		match target.value {
			JsClass::object => {},
			_ => {
				println!("Cant delete property for specia object!");
				return false;
			}
		}

		let prop_r = target.query_property(name);
		if let Some((prop, own)) = prop_r {
			if own {
				if prop.configable() {
					target.drop_property(name);
					return true;
				}
			}
		}
		return false;
	}

	/* item + item */
	fn concat_add(&mut self) {
		let x = self.top(-2);
		let y = self.top(-1);
		self.pop(2);

		if x.is_number() {
			let x = x.to_number();
			let y = y.to_number();
			self.push_number(x+y);
			return;
		}

		let x = x.to_string();
		let y = y.to_string();

		self.push_string( x + &y);
	}

	/* item op item */
	fn equal(&mut self) -> bool {
		let x = self.top(-2);
		let y = self.top(-1);
		self.pop(2);

		// string with others
		if x.is_string() {
			let x_str = x.to_string();
			if y.is_string() {
				let y_str = y.to_string();
				if x_str == y_str {
					return true;
				} else {
					return false;
				}
			} else if y.is_number() {
				let y_str = y.to_number().to_string();
				if x_str == y_str {
					return true;
				} else {
					return false;
				}
			}
			return false;
		}

		// null with defineded
		if x.is_undefined() {
			if y.is_undefined() {
				return true;
			}
			if y.is_null() {
				return true;
			}
			return false;
		}

		if x.is_null() {
			if y.is_undefined() {
				return true;
			}
			if y.is_null() {
				return true;
			}
			return false;
		}

		// boolean with boolean
		if x.is_boolean()  {
			if y.is_boolean() {
				return x.to_boolean() == y.to_boolean();
			}
			return false;
		}

		// number with others
		if x.is_number() {
			let x_num = x.to_number();
			if y.is_number() {
				let y_num = y.to_number();
				if x_num == y_num {
					return true;
				} else {
					return false;
				}
			}
			if y.is_string() {
				let y_str = y.to_string();
				if let Ok(y_num) = y_str.parse::<f64>() {
					return x_num == y_num;
				}
			}
			return false;
		}

		// object with object
		let x_obj = x.get_object();
		if y.is_object() {
			let y_obj = y.get_object();
			return Rc::ptr_eq(&x_obj, &y_obj);
		}
		return false;

	}

	fn strict_equal(&mut self) -> bool {
		let x = self.top(-2);
		let y = self.top(-1);

		// string with others
		if x.is_string() {
			let x_str = x.to_string();
			if y.is_string() {
				let y_str = y.to_string();
				if x_str == y_str {
					return true;
				}
			}
			return false;
		}

		// null with defineded
		if x.is_undefined() {
			if y.is_undefined() {
				return true;
			}
			return false;
		}

		if x.is_null() {
			if y.is_null() {
				return true;
			}
			return false;
		}

		// boolean with boolean
		if x.is_boolean()  {
			if y.is_boolean() {
				return x.to_boolean() == y.to_boolean();
			}
			return false;
		}

		// number with others
		if x.is_number() {
			let x_num = x.to_number();
			if y.is_number() {
				let y_num = y.to_number();
				if x_num == y_num {
					return true;
				}
			}
			return false;
		}

		// object with object
		let x_obj = x.get_object();
		if y.is_object() {
			let y_obj = y.get_object();
			return Rc::ptr_eq(&x_obj, &y_obj);
		}
		return false;
	}

	fn compare_item(&mut self) -> Option<i32> {
		let x = self.top(-2);
		let y = self.top(-1);
		self.pop(2);

		if x.is_number() {
			let x = x.to_number();
			let y = y.to_number();
			if x == f64::NAN || y == f64::NAN {
				return None;
			}
			if x > y {
				return Some(1);
			} else if x == y {
				return Some(0);
			} else  {
				return Some(-1);
			}
		}
		if x.is_string() {
			let x = x.to_string();
			let y = y.to_string();
			if x > y {
				return Some(1);
			} else if x == y {
				return Some(0);
			} else  {
				return Some(-1);
			}
		}
		return None;
	}

	fn in_operator(&mut self) -> Result<(), JsException> {
		let x = self.top(-2);
		let y = self.top(-1);
		self.pop(2);

		if !y.is_object() {
			println!("in: invalid operand");
			self.push_boolean(false);
			return Ok(());
		}

		let propstr = x.to_string();
		if let Some((_prop, _own)) = y.get_object().borrow().query_property(&propstr) {
			self.push_boolean(true);
			return Ok(());
		}

		self.push_boolean(false);
		return Ok(());
	}

	fn instanceof(&mut self) -> Result<(), JsException> {
		let x = self.top(-2);
		let y = self.top(-1);
		self.pop(2);

		if !x.is_object() {
			self.push_boolean(false);
			return Ok(());
		}
		if !y.is_object() {
			println!("instanceof: invalid operand");
			self.push_boolean(false);
			return Ok(());
		}
		let mut x = x.get_object();
		let y = y.get_object();
		if !y.borrow().callable() {
			println!("instanceof: invalid operand");
			self.push_boolean(false);
			return Ok(());
		}

		self.getproperty(y, "prototype")?;
		let o = self.top(-1);
		self.pop(1);
		if !o.is_object() {
			println!("instanceof: 'prototype' property is not an object");
			self.push_boolean(false);
			return Ok(());
		}
		let o = o.get_object();

		loop {
			let proto = x.borrow().__proto__.clone();
			if let Some( proto ) = proto {
				x = proto;
				if o.as_ptr() == x.as_ptr() {
					self.push_boolean(true);
					return Ok(());
				}
			} else {
				break;
			}
		}

		self.push_boolean(false);
		return Ok(());
	}

	/* Exceptions */
	pub fn new_exception(&mut self, e: JsException) {
		let obj = JsObject::new_exception(self.prototypes.exception_prototype.clone(), e);
		let value = SharedValue::new_object(obj);
		self.push(value);
	}

	/* create new object */
	fn new_call(&mut self, argc: usize) -> Result<(), JsException> {
		let obj = self.top(-1 - argc as isize).get_object();

		/* built-in constructors create their own objects, give them a 'null' this */
		if obj.borrow().is_builtin() {
			self.push_null();
			if argc > 0 {
				self.rot(argc+1);
			}
			jscall_builtin(self, argc);
			return Ok(());
		}

		/* extract the function object's prototype property */
		self.getproperty(obj, "prototype")?;

		let proto = if self.top(-1).is_object() {
			self.top(-1).get_object()
		} else {
			self.prototypes.object_prototype.clone()
		};

		self.pop(1);

		/* create a new object with above prototype, and shift it into the 'this' slot */
		let mut nobj = JsObject::new();
		nobj.__proto__ = Some(proto);
		let nobj = SharedObject_new(nobj);
		self.push_object(nobj.clone());
		if argc > 0 {
			self.rot(argc+1);
		}

		/* call the function */
		jscall(self, argc)?;

		/* if result is not an object, return the original object we created */
		if !self.top(-1).is_object() {
			self.pop(1);
			self.push_object(nobj);
		}
		return Ok(());
	}

	pub fn new_closure(&mut self, f: SharedFunction) {
		let fobj = SharedObject_new(JsObject::new_function(f.clone(), self.cenv.clone(), self.prototypes.function_prototype.clone()));

		// prototype object self
		let mut prop = JsProperty::new();
		prop.fill_attr(JS_READONLY_ATTR);
		prop.value = SharedValue::new_sobject(fobj.clone());
		let mut prototype_obj = JsObject::new();
    	prototype_obj.extensible = true;
		prototype_obj.__proto__ = Some(self.prototypes.object_prototype.clone());
		prototype_obj.properties.insert("constructor".to_string(), prop );

		// binding prototype to function object
		let prototype_obj = SharedObject_new(prototype_obj);
		let mut prop = JsProperty::new();
		prop.value = SharedValue::new_sobject(prototype_obj.clone());
		fobj.borrow_mut().properties.insert("prototype".to_string(), prop);

		self.push(SharedValue::new_sobject(fobj));
	}

	/* stack operations */
	pub fn top(&self, offset: isize) -> SharedValue {
		if offset < 0 {
			let offset: usize = (self.stack.len() as isize + offset) as usize;
			return self.stack[offset].clone();
		}
		panic!("top access only support negtive offset!")
	}
	pub fn push(&mut self, jv: SharedValue) {
		self.stack.push(jv);
	}
	pub fn push_undefined(&mut self) {
		let jv = SharedValue::new_undefined();
		self.stack.push(jv);
	}
	pub fn push_null(&mut self) {
		let jv = SharedValue::new_null();
		self.stack.push(jv);
	}
	pub fn push_boolean(&mut self, v: bool) {
		let jv = SharedValue::new_boolean(v);
		self.stack.push(jv);
	}
	pub fn push_number(&mut self, v:f64) {
		let jv = SharedValue::new_number(v);
		self.stack.push(jv);
	}
	pub fn push_string(&mut self, v:String) {
		let jclass = JsClass::string(v);
		let jobj = JsObject::new_with(self.prototypes.string_prototype.clone(), jclass);
		let jv = SharedValue::new_object(jobj);
		self.stack.push(jv);
	}
	pub fn push_object(&mut self, target: SharedObject) {
		let jv = SharedValue::new_sobject(target);
		self.stack.push(jv);
	}
	fn push_from(&mut self, from: usize) {
		if from >= self.stack.len() {
			panic!("stack underflow! @ push_from");
		}
		let jv = SharedValue::clone( &self.stack[from] );
		self.stack.push(jv);
	}

	/* opcode helper*/
	fn pop(&mut self, mut n: usize) {
		if n > self.stack.len() {
			panic!("stack underflow! @ pop");
		}
		while n > 0 {
			self.stack.pop();
			n = n - 1;
		}
	}
	fn dup(&mut self) {
		if self.stack.len() < 1 {
			panic!("stack underflow! @ dup");
		}
		let nv = self.top(-1);
		self.stack.push(nv);
	}
	fn dup2(&mut self) {
		if self.stack.len() < 2 {
			panic!("stack underflow! @ dup2");
		}

		let nv1: SharedValue = self.top(-2);
		let nv2: SharedValue = self.top(-1);
		self.stack.push(nv1);
		self.stack.push(nv2);
	}
	fn rot(&mut self, n: usize) {
		if self.stack.len() < n {
			panic!("stack underflow! @ rot");
		}
		let top = self.stack.len();
		for i in 0..n-1 {
			self.stack.swap(top-1-i, top-2-i);
		}
	}
	fn rot2(&mut self) {
		if self.stack.len() < 2 {
			panic!("stack underflow! @ rot2");
		}
		/* A B -> B A */
		let top = self.stack.len();
		self.stack.swap(top-1, top-2);
	}
	fn rot3(&mut self) {
		if self.stack.len() < 3 {
			panic!("stack underflow! @ rot3");
		}
		/* A B C -> C A B */
		let top = self.stack.len();
		self.stack.swap(top-1, top-2);
		self.stack.swap(top-2, top-3);
	}
	fn rot4(&mut self) {
		if self.stack.len() < 4 {
			panic!("stack underflow! @ rot4");
		}
		/* A B C D -> D A B C */
		let top = self.stack.len();
		self.stack.swap(top-1, top-2);
		self.stack.swap(top-2, top-3);
		self.stack.swap(top-3, top-4);
	}
	fn rot3pop2(&mut self) {
		if self.stack.len() < 3 {
			panic!("stack underflow! @ rot3pop2");
		}
		/* A B C -> C */
		let top = self.stack.len();
		self.stack[top-3] = self.stack[top-1].clone();
		self.pop(2);
	}
	fn rot2pop1(&mut self) {
		if self.stack.len() < 2 {
			panic!("stack underflow! @ rot3pop2");
		}
		/* A B -> B */
		let top = self.stack.len();
		self.stack[top-2] = self.stack[top-1].clone();
		self.pop(1);
	}

	fn debugger(&mut self) {
		// runtime virtual machine debugger
		println!("=======>{}", self.stack.len());
	}

}

fn jsrun<T: Hookable>(rt: &mut JsRuntime<T>, func: &VMFunction, pc: usize) -> Result<(), JsException> {
	assert!(rt.stack.len() > 0);
	let mut pc:usize = pc;
	let bot:usize = rt.stack.len() - 1;

	let mut with_exception = None;
	let mut catch_scopes: Vec<(usize, usize)> = Vec::new();

	macro_rules! handle_exception {
		($e:ident) => {
			if let Some((new_pc, new_top)) = catch_scopes.pop() {
				let dropped = rt.stack.len() - new_top;
				rt.pop(dropped);

				rt.new_exception($e);
				pc = new_pc;
				continue;
			} else {
				with_exception = Some($e);
				break;
			}
		}
	}

	loop {
		let opcode = func.opcode(&mut pc);
		match opcode {
			OpcodeType::OP_POP => {
				rt.pop(1);
			},
			OpcodeType::OP_DUP => {
				rt.dup();
			},
			OpcodeType::OP_DUP2 => {
				rt.dup2();
			},
			OpcodeType::OP_ROT2 => {
				rt.rot2();
			},
			OpcodeType::OP_ROT3 => {
				rt.rot3();
			},
			OpcodeType::OP_ROT4 => {
				rt.rot4();
			},

			OpcodeType::OP_UNDEF => {
				rt.push(SharedValue::new_undefined());
			},
			OpcodeType::OP_NULL => {
				rt.push(SharedValue::new_null());
			},
			OpcodeType::OP_FALSE => {
				rt.push_boolean(false);
			},
			OpcodeType::OP_TRUE => {
				rt.push_boolean(true);
			},

			OpcodeType::OP_INTEGER => {
				let v = func.int(&mut pc);
				rt.push_number(v);
			},
			OpcodeType::OP_NUMBER => {
				let v = func.number(&mut pc);
				rt.push_number(v);
			},
			OpcodeType::OP_STRING => {
				let v = func.string(&mut pc);
				rt.push_string(v.to_string());
			},

			/* Creating objects */
			OpcodeType::OP_CLOSURE => {
				let f = func.function(&mut pc);
				rt.new_closure(f);
			},
			OpcodeType::OP_NEWOBJECT => {
				let obj = SharedValue::new_vanilla(rt.prototypes.object_prototype.clone());
				rt.push(obj);
			},
			OpcodeType::OP_NEWARRAY => {
				let obj = JsObject::new_array(rt.prototypes.array_prototype.clone());
				let jv = SharedValue::new_object(obj);
				rt.push(jv);
			},

			OpcodeType::OP_THIS => {
				let thiz = rt.stack[bot].clone();
				if thiz.is_object() {
					rt.push_from(bot);
				} else {
					let global = rt.genv.borrow().target();
					rt.push_object(global);
				}
			},
			OpcodeType::OP_CURRENT => {
				rt.push_from(bot - 1);
			},

			OpcodeType::OP_GETVAR => {
				let s = func.string(&mut pc);
				let result = rt.getvariable(&s);
				let excp = match result {
					Ok(br) => {
						if br == true {
							continue;
						} else {
							println!("'{}' is not defined", s);
							JsException::new(format!("TODO: {}", line!()))
						}
					},
					Err(e) => {
						e
					},
				};
				handle_exception!(excp);
			},
			OpcodeType::OP_HASVAR => {
				let s = func.string(&mut pc);
				let result = rt.getvariable(&s);
				let excp = match result {
					Ok(br) => {
						if br == false {
							rt.push_undefined();
						}
						continue;
					},
					Err(e) => {
						e
					},
				};
				handle_exception!(excp);
			},
			OpcodeType::OP_SETVAR => {
				let s = func.string(&mut pc);
				let result = rt.setvariable(s);
				if let Err(e) = result {
					handle_exception!(e);
				}
			},
			OpcodeType::OP_DELVAR => {
				let s = func.string(&mut pc);
				let r = rt.delvariable(s);
				rt.push_boolean(r);
			},

			OpcodeType::OP_INITPROP => {
				let target = rt.top(-3).get_object();
				let name = rt.top(-2).to_string();
				let value = rt.top(-1);
				if let Err(e) = rt.setproperty(target, &name, value) {
					handle_exception!(e);
				}
				rt.pop(2);
			},
			OpcodeType::OP_INITGETTER => {
				let target = rt.top(-3).get_object();
				let name = rt.top(-2).to_string();
				let func = rt.top(-1);
				if func.is_object() {
					let result = rt.defproperty(target, &name, SharedValue::new_undefined(), JS_DEFAULT_ATTR, Some(func.get_object()), None);
					if let Err(e) = result {
						handle_exception!(e);
					}
				} else {
					println!("getter should be a object!");
				}
				rt.pop(2);
			},
			OpcodeType::OP_INITSETTER => {
				let target = rt.top(-3).get_object();
				let name = rt.top(-2).to_string();
				let func = rt.top(-1);
				if func.is_object() {
					let result = rt.defproperty(target, &name, SharedValue::new_undefined(), JS_DEFAULT_ATTR, None, Some(func.get_object()));
					if let Err(e) = result {
						handle_exception!(e);
					}
				} else {
					println!("setter should be a object!");
				}
				rt.pop(2);
			},

			OpcodeType::OP_GETPROP => {
				let target = rt.top(-2).get_object();
				let name = rt.top(-1).to_string();
				if let Err(e) = rt.getproperty(target, &name) {
					handle_exception!(e);
				}
				rt.rot3pop2();
			},
			OpcodeType::OP_GETPROP_S => {
				let target = rt.top(-1);
				if !target.is_object() {
					let e = JsException::new("Access none objects's property!".to_string());
					handle_exception!(e);
				}
				let target = target.get_object();
				let name = func.string(&mut pc);
				if let Err(e) = rt.getproperty(target, &name) {
					handle_exception!(e);
				}
				rt.rot2pop1();
			},
			OpcodeType::OP_SETPROP => {
				let target = rt.top(-3).get_object();
				let name = rt.top(-2).to_string();
				let value = rt.top(-1);
				if let Err(e) = rt.setproperty(target, &name, value) {
					handle_exception!(e);
				}
				rt.rot3pop2();
			},
			OpcodeType::OP_SETPROP_S => {
				let target = rt.top(-2).get_object();
				let value = rt.top(-1);
				let name = func.string(&mut pc);
				if let Err(e) = rt.setproperty(target, &name, value) {
					handle_exception!(e);
				}
				rt.rot2pop1();
			},
			OpcodeType::OP_DELPROP => {
				let target = rt.top(-2).get_object();
				let name = rt.top(-1).to_string();
				let b = rt.delproperty(target, &name);
				rt.pop(2);
				rt.push_boolean(b);
			},
			OpcodeType::OP_DELPROP_S => {
				let name = func.string(&mut pc);
				let target_value = rt.top(-1);
				if target_value.is_object() {
					let target = target_value.get_object();
					let b = rt.delproperty(target, &name);
					rt.pop(1);
					rt.push_boolean(b);
				} else {
					let e = JsException::new("Can't delete none object's proptery".to_string());
					handle_exception!(e);
				}
			},

			OpcodeType::OP_ITERATOR => {
				if rt.top(-1).is_object() {
					let target = rt.top(-1).get_object();
					if target.borrow().is_vanilla() {
						let iter = JsObject::new_iterator(target);
						rt.pop(1);
						rt.push( SharedValue::new_object(iter) );
					}
				}
			},
			OpcodeType::OP_NEXTITER => {
				if rt.top(-1).is_object() {
					let target = rt.top(-1).get_object();
					if target.borrow().is_iterator() {
						let mut target = target.borrow_mut();
						let it: &mut JsIterator = target.get_iterator();
						if let Some(s) = it.next() {
							rt.push_string(s);
							rt.push_boolean(true);
						} else {
							rt.pop(1);
							rt.push_boolean(false);
						}
						continue;
					}
				}
				rt.pop(1);
				rt.push_boolean(false);
			},

			/* Function calls */
			OpcodeType::OP_CALL => {
				let n = func.int(&mut pc) as usize;
				if let Err(e) = jscall(rt, n) {
					handle_exception!(e);
				}
			},
			OpcodeType::OP_NEW => {
				let n = func.int(&mut pc) as usize;
				if let Err(e) = rt.new_call(n) {
					handle_exception!(e);
				}
			},

			/* Unary operators */
			OpcodeType::OP_TYPEOF => {
				let target = rt.top(-1);
				let str = target.type_string();
				rt.pop(1);
				rt.push_string(str);
			},

			OpcodeType::OP_POS => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(n);
			},
			OpcodeType::OP_NEG => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(-n);
			},
			OpcodeType::OP_BITNOT => {
				let n = rt.top(-1).to_number() as i32;
				rt.pop(1);
				rt.push_number( (!n) as f64 );
			},
			OpcodeType::OP_LOGNOT => {
				let n = rt.top(-1).to_boolean();
				rt.pop(1);
				rt.push_boolean(!n);
			},
			OpcodeType::OP_INC => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(n+1.0);
			},
			OpcodeType::OP_DEC => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(n-1.0);
			},
			OpcodeType::OP_POSTINC => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(n+1.0);
				rt.push_number(n);
			},
			OpcodeType::OP_POSTDEC => {
				let n = rt.top(-1).to_number();
				rt.pop(1);
				rt.push_number(n-1.0);
				rt.push_number(n);
			},

			/* Multiplicative operators */
			OpcodeType::OP_MUL => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				rt.push_number(x * y);
			},
			OpcodeType::OP_DIV => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				rt.push_number(x / y);
			},
			OpcodeType::OP_MOD => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				rt.push_number(x % y);
			},

			/* Additive operators */
			OpcodeType::OP_ADD => {
				rt.concat_add();
			},
			OpcodeType::OP_SUB => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				rt.push_number(x - y);
			},

			/* Shift operators */
			OpcodeType::OP_SHL => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					let x = x as i64;
					let y = y as u64;
					rt.push_number( (x << (y&0x1F)) as f64);
				}
			},
			OpcodeType::OP_SHR => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					let x = x as i64;
					let y = y as u64;
					rt.push_number( (x >> (y&0x1F)) as f64);
				}
			},
			OpcodeType::OP_USHR => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					let x = x as u64;
					let y = y as u64;
					rt.push_number( (x >> (y&0x1F)) as f64);
				}
			},

			/* Relational operators */
			OpcodeType::OP_LT => {
				let r = rt.compare_item();
				if let Some(b) = r {
					rt.push_boolean( b < 0 );
				} else {
					rt.push_boolean(false);
				}
			},
			OpcodeType::OP_GT => {
				let r = rt.compare_item();
				if let Some(b) = r {
					rt.push_boolean( b > 0);
				} else {
					rt.push_boolean(false);
				}
			},
			OpcodeType::OP_LE => {
				let r = rt.compare_item();
				if let Some(b) = r {
					rt.push_boolean( b <= 0 );
				} else {
					rt.push_boolean(false);
				}
			},
			OpcodeType::OP_GE => {
				let r = rt.compare_item();
				if let Some(b) = r {
					rt.push_boolean( b >= 0);
				} else {
					rt.push_boolean(false);
				}
			},

			OpcodeType::OP_IN => {
				if let Err(e) = rt.in_operator() {
					handle_exception!(e);
				}
			},

			OpcodeType::OP_INSTANCEOF => {
				if let Err(e) = rt.instanceof() {
					handle_exception!(e);
				}
			},

			/* Equality */
			OpcodeType::OP_EQ => {
				let b = rt.equal();
				rt.push_boolean(b);
			},
			OpcodeType::OP_NE => {
				let b = rt.equal();
				rt.push_boolean(!b);
			},
			OpcodeType::OP_STRICTEQ => {
				let b = rt.strict_equal();
				rt.pop(2);
				rt.push_boolean(b);
			},
			OpcodeType::OP_STRICTNE => {
				let b = rt.strict_equal();
				rt.pop(2);
				rt.push_boolean(!b);
			},

			/* Binary bitwise operators */
			OpcodeType::OP_BITAND => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					rt.push_number( (x as i64 & y as i64) as f64);
				}
			},
			OpcodeType::OP_BITXOR => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					rt.push_number( (x as i64 ^ y as i64) as f64);
				}
			},
			OpcodeType::OP_BITOR => {
				let x = rt.top(-2).to_number();
				let y = rt.top(-1).to_number();
				rt.pop(2);
				if x == f64::NAN || y == f64::NAN {
					rt.push_number(0.0);
				} else if x == f64::INFINITY || y == f64::INFINITY {
					rt.push_number(0.0);
				} else if x == f64::NEG_INFINITY || y == f64::NEG_INFINITY {
					rt.push_number(0.0);
				} else {
					rt.push_number( (x as i64 | y as i64) as f64);
				}
			},

			/* Try and Catch */
			OpcodeType::OP_TRY => {
				let catch_block = func.address(&mut pc);
				catch_scopes.push((pc, rt.stack.len()));
				pc = catch_block;
			},
			OpcodeType::OP_ENDTRY => {
				catch_scopes.pop();
			},
			OpcodeType::OP_CATCH => {
				let str = func.string(&mut pc);
				let eobj = rt.top(-1);
				rt.pop(1);

				let new_env = JsEnvironment::new_from(rt.cenv.clone());
				new_env.borrow_mut().init_var(str, eobj);
				rt.cenv = new_env;
			},
			OpcodeType::OP_ENDCATCH => {
				let outer = rt.cenv.borrow().fetch_outer();
				rt.cenv = outer;
			},
			OpcodeType::OP_THROW => {
				let evalue = rt.top(-1);
				rt.pop(1);
				if evalue.is_exception() {
					let e = evalue.get_object().borrow().get_exception();
					handle_exception!(e);
				} else {
					panic!("Throw a none exception object!");
				}
			},

			/* Branching & Flow control */
			OpcodeType::OP_JCASE => {
				let offset = func.address(&mut pc);
				let b = rt.strict_equal();
				if b {
					rt.pop(2);
					pc = offset;
				} else {
					rt.pop(1);
				}
			},
			OpcodeType::OP_JUMP => {
				let addr = func.address(&mut pc);
				pc = addr;
			},
			OpcodeType::OP_JTRUE => {
				let addr = func.address(&mut pc);
				let b = rt.top(-1).to_boolean();
				rt.pop(1);
				if b {
					pc = addr;
				}
			},
			OpcodeType::OP_JFALSE => {
				let addr = func.address(&mut pc);
				let b = rt.top(-1).to_boolean();
				rt.pop(1);
				if !b {
					pc = addr;
				}
			},
			OpcodeType::OP_RETURN => {
				break;
			},

			OpcodeType::OP_DEBUG => {
				rt.debugger();
				panic!("Exiting with debug");
			},

			/* do nothing */
			OpcodeType::OP_EVAL => {},
			OpcodeType::OP_NOP => {},
			OpcodeType::OP_LAST => {},
		}
	}

	// breaked from loop, return to caller
	if with_exception.is_none() {
		return Ok(());
	}
	return Err( with_exception.unwrap() );
}

fn jscall_script<T:Hookable>(rt: &mut JsRuntime<T>, argc: usize) -> Result<(), JsException> {
	let bot = rt.stack.len() - 1 - argc;

	let fobj = rt.stack[bot-1].get_object();
	let rfobj = fobj.borrow();
	let vmf = &rfobj.get_func().vmf;

	/* init var in current env*/
	for i in 0..vmf.numvars {
		let jv = SharedValue::new_undefined();
		let var = &vmf.str_tab[i];
		rt.cenv.borrow_mut().init_var(var, jv);
	}

	/* scripts take no arguments */
	rt.pop(argc);
	jsrun(rt, vmf, 0)?;

	/* clear stack */
	let jv = rt.stack.pop().unwrap();
	rt.pop(2);
	rt.push(jv);

	return Ok(())
}

fn jscall_function<T: Hookable>(rt: &mut JsRuntime<T>, argc: usize) -> Result<(), JsException> {
	let bot = rt.stack.len() - 1 - argc;

	let fobj = rt.stack[bot-1].get_object();
	let rfobj = fobj.borrow();
	let vmf = &rfobj.get_func().vmf;

	/* create new scope */
	let new_env = JsEnvironment::new_from(rfobj.get_func().scope.clone());
	let old_env = rt.cenv.clone();
	rt.cenv = new_env;

	/* create arguments */
	{
		let arg_obj = JsObject::new_with( rt.prototypes.object_prototype.clone(), JsClass::object);
		let arg_value = SharedValue::new_object(arg_obj);

		let jv = SharedValue::new_number(argc as f64);
		rt.defproperty(arg_value.get_object(), "length", jv,  JS_READONLY_ATTR, None, None)?;

		for i in 0..argc {
			let name = i.to_string();
			let jv = rt.stack[bot+1+i].clone();
			rt.defproperty(arg_value.get_object(), &name, jv, JS_DEFAULT_ATTR, None, None)?;
		}

		arg_value.get_object().borrow_mut().extensible = false;
		rt.cenv.borrow_mut().init_var("arguments", arg_value);
	}

	/* setup remained arguments*/
	let min_argc = cmp::min(argc, vmf.numparams);
	for i in 0..min_argc {
		let argv = rt.stack[i + 1 + bot].clone();
		rt.cenv.borrow_mut().init_var(&vmf.str_tab[i], argv);
	}
	rt.pop(argc);

	/* init var in current env*/
	for i in min_argc..(vmf.numvars + vmf.numparams) {
		let jv = SharedValue::new_undefined();
		rt.cenv.borrow_mut().init_var(&vmf.str_tab[i], jv);
	}

	/* for recurrent call function self, init a local variable into this */
	if let Some(ref name) = vmf.name {
		rt.cenv.borrow_mut().init_var(name, rt.stack[bot-1].clone());
	}


	jsrun(rt, vmf, 0)?;

	/* clear stack */
	let jv = rt.stack.pop().unwrap();
	rt.pop(2);
	rt.push(jv);

	/* restore old env */
	rt.cenv = old_env;

	return Ok(());
}

fn jscall_builtin<T: Hookable>(rt: &mut JsRuntime<T>, argc: usize) {
	let bot = rt.stack.len() - 1 - argc;
	let fobj = rt.stack[bot-1].get_object();
	let builtin = rt.builtins[fobj.borrow().get_builtin()].clone();

	if argc > builtin.argc {
		for _i in builtin.argc .. argc {
			rt.pop(1);
		}
	} else if argc < builtin.argc {
		for _i in argc .. builtin.argc {
			rt.push_undefined();
		}
	}

	(builtin.f)(rt);

	let jv = rt.stack.pop().unwrap();
	rt.pop(builtin.argc + 2);
	rt.push(jv);
}

pub fn jscall<T: Hookable>(rt: &mut JsRuntime<T>, argc: usize) -> Result<(), JsException> {
	assert!(rt.stack.len() >= argc + 2);
	let bot = rt.stack.len() - 1 - argc;

	if !rt.stack[bot-1].is_object() {
		return Err( JsException::new("Can't call on none function value".to_string()));
	}

	let fobj = rt.stack[bot-1].get_object();
	if fobj.borrow().is_function() == true {

		if fobj.borrow().get_func().vmf.script {
			jscall_script(rt, argc)?;
		} else {
			jscall_function(rt, argc)?;
		};

	} else if fobj.borrow().is_builtin() == true {
		jscall_builtin(rt, argc);
	} else {
        panic!("Can't call none function object");
	}

	return Ok(());
}
