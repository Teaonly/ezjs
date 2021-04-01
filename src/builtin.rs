use std::collections::HashMap;

use crate::bytecode::*;
use crate::value::*;
use crate::runtime::*;

impl<T:Expandable> JsBuiltinFunction<T> {
	pub fn new(f: fn(&mut JsRuntime<T>), argc: usize) -> Self {
		JsBuiltinFunction {
			f:		f,
			argc:	argc
		}
	}
}

// The Object class 
fn object_constructor<T: Expandable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_something() {        
        rt.push( value.duplicate() );
        return;
    }
    rt.push( SharedValue::new_vanilla(rt.prototypes.object_prototype.clone()) );
}

fn object_preventextensions<T: Expandable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_object() {
        value.get_object().borrow_mut().extensible = false;
    }
    rt.push(value);
}

fn object_tostring<T: Expandable>(rt: &mut JsRuntime<T>)  {
    rt.push_string( "[object]".to_string() );
}

fn object_setprototypeof<T: Expandable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-2);
    if !target.is_object() {
        rt.push_undefined();    
        return;
    }

    let proto = rt.top(-1);
    if !proto.is_object() {
        rt.push_undefined();
        return;    
    }

    target.get_object().borrow_mut().__proto__ = Some(proto.get_object());
    rt.push(target);
}

fn object_proto_builtins<T: Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    //builtins.insert("preventExtensions".to_string(), JsBuiltinFunction::new(object_preventextensions, 1));
    //builtins.insert("setPrototypeOf".to_string(), JsBuiltinFunction::new(object_setprototypeof, 2));
    return builtins;
}

fn object_builtins<T:Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();   
    builtins.insert("preventExtensions".to_string(), JsBuiltinFunction::new(object_preventextensions, 1));
    builtins.insert("setPrototypeOf".to_string(), JsBuiltinFunction::new(object_setprototypeof, 2));
    return builtins;
}

// The String class
fn string_constructor<T:Expandable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_string() {
        rt.push(value);
        return;
    }
    if value.is_something() {
        object_tostring(rt);
        return;
    }
    rt.push_string("".to_string());
}

fn string_tostring<T:Expandable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1).duplicate();     // this object
    assert!(value.is_string());
    rt.push(value);
}

fn string_proto_builtins<T:Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(string_tostring, 0));    
    return builtins;
}

// The Array class
fn array_constructor<T:Expandable>(rt: &mut JsRuntime<T>) {
    let a = JsClass::array(Vec::new());
    let obj = JsObject::new_with(rt.prototypes.array_prototype.clone(), a);
    let jv = SharedValue::new_object(obj);
    rt.push(jv);
}

fn array_tostring<T:Expandable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    assert!(value.is_object());
    let sobj = value.get_object();
    let object = sobj.borrow();
    assert!(object.is_array());

    let mut result = String::new();
    let v = object.get_array();
    for i in 0..v.len() {        
        result.push_str( &v[i].to_string() );
        if i != v.len() - 1 {
            result.push_str(", ");
        }
    }
    rt.push_string(result);
}

fn array_push<T:Expandable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-2);
    assert!(target.is_object());
    let sobj = target.get_object();
    let mut object = sobj.borrow_mut();
    assert!(object.is_array());
   
    let value = rt.top(-1).duplicate();
    object.get_mut_array().push(value);
    
    rt.push_number(object.get_array().len() as f64);
}

fn array_proto_builtins<T:Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(array_tostring, 0));
    builtins.insert("push".to_string(), JsBuiltinFunction::new(array_push, 1));
    return builtins;
}

// The Function class
fn function_constructor<T:Expandable>(rt: &mut JsRuntime<T>) {
    let vmf = SharedFunction_new(VMFunction::new_anonymous());
    let mut fobj = JsObject::new_function(vmf, rt.cenv.clone());
    fobj.__proto__ = Some(rt.prototypes.function_prototype.clone());
    rt.push(SharedValue::new_object(fobj));
}

fn function_tostring<T:Expandable>(rt: &mut JsRuntime<T>) {
    rt.push_string("function(...) {...}".to_string());
}

fn function_proto_builtins<T:Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(function_tostring, 0));    
    return builtins;
}

// The Exception class
fn exception_constructor<T:Expandable>(rt: &mut JsRuntime<T>) {
    
    let value = rt.top(-1);    
    let msg = value.to_string();

    let exp = JsException::new(msg);
    let value = SharedValue::new_object(JsObject::new_exception(rt.prototypes.exception_prototype.clone(), exp));
    rt.push(value);
}

fn exception_tostring<T:Expandable>(rt: &mut JsRuntime<T>) {
    rt.push_string("exception(...) {...}".to_string());
}

fn exception_message<T:Expandable>(rt: &mut JsRuntime<T>) {
    let exp_object = rt.top(-1).get_object();
    let exp = exp_object.borrow().get_exception();
    rt.push_string(exp.msg);
}

fn exception_proto_builtins<T:Expandable>() -> HashMap<String, JsBuiltinFunction<T>> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(exception_tostring, 0));
    builtins.insert("message".to_string(), JsBuiltinFunction::new(exception_message, 0));
    return builtins;
}

// build class's global functions
fn create_class_functions<T:Expandable>(rt: &mut JsRuntime<T>, target: SharedObject, properties: HashMap<String, JsBuiltinFunction<T>>) {
    let mut class_obj = target.borrow_mut();
    for (k, v) in properties {
        let func_obj = rt.new_builtin(v);
        
        let mut prop = JsProperty::new();
        prop.fill_attr(JS_READONLY_ATTR);
        prop.value = SharedValue::new_object(func_obj);

        class_obj.properties.insert(k, prop);
    }
}

// build prototypes chain
fn create_builtin_class<T:Expandable>(rt: &mut JsRuntime<T>, constructor: JsBuiltinFunction<T>, properties: HashMap<String, JsBuiltinFunction<T>>, top: Option<SharedObject>) -> (SharedObject, SharedObject) {
    let mut class_obj = rt.new_builtin(constructor);
    class_obj.extensible = false;
    let class_obj =  SharedObject_new(class_obj);
    
    let mut prototype_obj = JsObject::new();
    prototype_obj.extensible = false;
    for (k, v) in properties {
        let func_obj = rt.new_builtin(v);
        
        let mut prop = JsProperty::new();
        prop.fill_attr(JS_READONLY_ATTR);
        prop.value = SharedValue::new_object(func_obj);

        prototype_obj.properties.insert(k, prop);
    }
    let mut prop = JsProperty::new();
    prop.fill_attr(JS_READONLY_ATTR);
    prop.value = SharedValue::new_sobject(class_obj.clone());
    prototype_obj.properties.insert("constructor".to_string(), prop);
    prototype_obj.__proto__ = top;

    let prototype_obj = SharedObject_new(prototype_obj);

    let mut prop = JsProperty::new();
    prop.fill_attr(JS_READONLY_ATTR);
    prop.value = SharedValue::new_sobject(prototype_obj.clone());
    class_obj.borrow_mut().properties.insert("prototype".to_string(), prop);
    
    return (class_obj, prototype_obj);
}
fn set_global_class<T:Expandable>(rt: &mut JsRuntime<T>, name: &str, class_obj: SharedObject) {
    let mut prop = JsProperty::new();
    prop.fill_attr(JS_READONLY_ATTR);
    prop.value = SharedValue::new_sobject(class_obj);
    rt.genv.borrow_mut().target().borrow_mut().set_property(name, prop);
}

pub fn prototypes_init<T:Expandable>(rt: &mut JsRuntime<T>) {
    // Object
    let (top_class, top_prototype) = create_builtin_class(rt, JsBuiltinFunction::new(object_constructor, 1), object_proto_builtins(), None);
    create_class_functions(rt, top_class.clone(), object_builtins());
    set_global_class(rt, "Object", top_class.clone());
    rt.prototypes.object_prototype = top_prototype.clone();
    
    // String
    let (string_classs_object, string_prototype) = create_builtin_class(rt, JsBuiltinFunction::new(string_constructor, 1), string_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "String", string_classs_object.clone());
    rt.prototypes.string_prototype = string_prototype;

    // Array
    let (array_classs_object, array_prototype) = create_builtin_class(rt, JsBuiltinFunction::new(array_constructor, 0), array_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Array", array_classs_object.clone());
    rt.prototypes.array_prototype = array_prototype;

    // Function
    let (func_classs_object, func_prototype) = create_builtin_class(rt, JsBuiltinFunction::new(function_constructor, 0), function_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Function", func_classs_object.clone());
    rt.prototypes.function_prototype = func_prototype;
    
    // Exception
    let (exp_classs_object, exp_prototype) = create_builtin_class(rt, JsBuiltinFunction::new(exception_constructor, 1), exception_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Exception", exp_classs_object.clone());
    rt.prototypes.exception_prototype = exp_prototype;
}

pub fn builtin_init<T:Expandable>(runtime: &mut JsRuntime<T>) {
    // global functions for runtime 
    fn assert<T:Expandable>(rt: &mut JsRuntime<T>) {    
        let b = rt.top(-2).to_boolean();
        if !b {
            let info = rt.top(-1).to_string();
            panic!("ASSERT: {}", info);
        }
        rt.push_undefined();
    }

    fn println<T:Expandable>(rt: &mut JsRuntime<T>) {
        let info = rt.to_string( rt.top(-1) );
        if let Ok(msg) = info {
            println!("{}", msg);
            rt.push_undefined();
            return;
        } 
        if let Err(e) = info {
            rt.new_exception(e);
        }
    }
    // TODO : isFinite() isNaN() parseFloat() parseInt()

    // register some basic builtin functions
    let fobj = runtime.new_builtin(JsBuiltinFunction::new(assert, 2));
    runtime.genv.borrow_mut().init_var("assert", SharedValue::new_object(fobj) );

    let fobj = runtime.new_builtin(JsBuiltinFunction::new(println, 1));
    runtime.genv.borrow_mut().init_var("println", SharedValue::new_object(fobj));
}