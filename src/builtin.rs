use std::collections::HashMap;

use crate::bytecode::*;
use crate::runtime::*;

// The Object class 
fn object_constructor(rt: &mut JsRuntime) {
    let value = rt.top(-1);
    if value.is_something() {        
        rt.push( value.duplicate() );
    }
    rt.push( SharedValue::new_vanilla(rt.prototypes.object_prototype.clone()) );
}

fn object_preventextensions(rt: &mut JsRuntime) {
    let value = rt.top(-1);
    if value.is_object() {
        value.get_object().borrow_mut().extensible = false;
    }
    rt.push(value);
}

fn object_tostring(rt: &mut JsRuntime)  {
    rt.push_string( "[object]".to_string() );
}

fn object_setprototypeof(rt: &mut JsRuntime) {
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

fn object_proto_builtins() -> HashMap<String, JsBuiltinFunction> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    //builtins.insert("preventExtensions".to_string(), JsBuiltinFunction::new(object_preventextensions, 1));
    //builtins.insert("setPrototypeOf".to_string(), JsBuiltinFunction::new(object_setprototypeof, 2));
    return builtins;
}

fn object_builtins() -> HashMap<String, JsBuiltinFunction> {
    let mut builtins = HashMap::new();   
    builtins.insert("preventExtensions".to_string(), JsBuiltinFunction::new(object_preventextensions, 1));
    builtins.insert("setPrototypeOf".to_string(), JsBuiltinFunction::new(object_setprototypeof, 2));
    return builtins;
}

// The String class
fn string_constructor(rt: &mut JsRuntime) {
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

fn string_tostring(rt: &mut JsRuntime) {
    let value = rt.top(-1).duplicate();     // this object
    assert!(value.is_string());
    rt.push(value);
}

fn string_proto_builtins() -> HashMap<String, JsBuiltinFunction> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(string_tostring, 0));    
    return builtins;
}

// The Array class
fn array_constructor(rt: &mut JsRuntime) {
    let a = JsClass::array(Vec::new());
    let obj = JsObject::new_with(rt.prototypes.array_prototype.clone(), a);
    let jv = SharedValue::new_object(obj);
    rt.push(jv);
}

fn array_tostring(rt: &mut JsRuntime) {
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

fn array_push(rt: &mut JsRuntime) {
    let target = rt.top(-2);
    assert!(target.is_object());
    let sobj = target.get_object();
    let mut object = sobj.borrow_mut();
    assert!(object.is_array());
   
    let value = rt.top(-1).duplicate();
    object.get_mut_array().push(value);
    
    rt.push_number(object.get_array().len() as f64);
}

fn array_proto_builtins() -> HashMap<String, JsBuiltinFunction> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(array_tostring, 0));
    builtins.insert("push".to_string(), JsBuiltinFunction::new(array_push, 1));
    return builtins;
}

// The Function class
fn function_constructor(rt: &mut JsRuntime) {
    let vmf = SharedFunction_new(VMFunction::new_anonymous());
    let mut fobj = JsObject::new_function(vmf, rt.cenv.clone());
    fobj.__proto__ = Some(rt.prototypes.function_prototype.clone());
    rt.push(SharedValue::new_object(fobj));
}

fn function_tostring(rt: &mut JsRuntime) {
    rt.push_string("function(...) {...}".to_string());
}

fn function_proto_builtins() -> HashMap<String, JsBuiltinFunction> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(function_tostring, 0));    
    return builtins;
}

// The Exception class
fn exception_constructor(rt: &mut JsRuntime) {
    
    let value = rt.top(-1);    
    let msg = value.to_string();

    let exp = JsException::new(msg);
    let value = SharedValue::new_object(JsObject::new_exception(rt.prototypes.exception_prototype.clone(), exp));
    rt.push(value);
}

fn exception_tostring(rt: &mut JsRuntime) {
    rt.push_string("exception(...) {...}".to_string());
}

fn exception_message(rt: &mut JsRuntime) {
    let exp_object = rt.top(-1).get_object();
    let exp = exp_object.borrow().get_exception();
    rt.push_string(exp.msg);
}

fn exception_proto_builtins() -> HashMap<String, JsBuiltinFunction> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(exception_tostring, 0));
    builtins.insert("message".to_string(), JsBuiltinFunction::new(exception_message, 0));
    return builtins;
}

// build class's global functions
fn create_class_functions( target: SharedObject, properties: HashMap<String, JsBuiltinFunction>) {
    let mut class_obj = target.borrow_mut();
    for (k, v) in properties {
        let f = v.f;
        let argc = v.argc; 
        let func_obj = JsObject::new_builtin(f, argc);
        
        let mut prop = JsProperty::new();
        prop.fill_attr(JS_READONLY_ATTR);
        prop.value = SharedValue::new_object(func_obj);

        class_obj.properties.insert(k, prop);
    }
}

// build prototypes chian
fn create_builtin_class(constructor: JsBuiltinFunction, properties: HashMap<String, JsBuiltinFunction>, top: Option<SharedObject>) -> (SharedObject, SharedObject) {
    let mut class_obj = JsObject::new();
    class_obj.extensible = false;
    class_obj.value = JsClass::builtin(constructor);
    let class_obj =  SharedObject_new(class_obj);
    
    let mut prototype_obj = JsObject::new();
    prototype_obj.extensible = false;
    for (k, v) in properties {
        let f = v.f;
        let argc = v.argc; 
        let func_obj = JsObject::new_builtin(f, argc);
        
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
fn set_global_class(rt: &mut JsRuntime, name: &str, class_obj: SharedObject) {
    let mut prop = JsProperty::new();
    prop.fill_attr(JS_READONLY_ATTR);
    prop.value = SharedValue::new_sobject(class_obj);
    rt.genv.borrow_mut().target().borrow_mut().set_property(name, prop);
}

pub fn prototypes_init(rt: &mut JsRuntime) {
    // Object
    let (top_class, top_prototype) = create_builtin_class(JsBuiltinFunction::new(object_constructor, 1), object_proto_builtins(), None);
    create_class_functions(top_class.clone(), object_builtins());
    set_global_class(rt, "Object", top_class.clone());
    rt.prototypes.object_prototype = top_prototype.clone();
    
    // String
    let (string_classs_object, string_prototype) = create_builtin_class( JsBuiltinFunction::new(string_constructor, 1), string_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "String", string_classs_object.clone());
    rt.prototypes.string_prototype = string_prototype;

    // Array
    let (array_classs_object, array_prototype) = create_builtin_class( JsBuiltinFunction::new(array_constructor, 0), array_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Array", array_classs_object.clone());
    rt.prototypes.array_prototype = array_prototype;

    // Function
    let (func_classs_object, func_prototype) = create_builtin_class( JsBuiltinFunction::new(function_constructor, 0), function_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Function", func_classs_object.clone());
    rt.prototypes.function_prototype = func_prototype;
    
    // Exception
    let (exp_classs_object, exp_prototype) = create_builtin_class( JsBuiltinFunction::new(exception_constructor, 1), exception_proto_builtins(), Some(top_prototype.clone()));
    set_global_class(rt, "Exception", exp_classs_object.clone());
    rt.prototypes.exception_prototype = exp_prototype;
}

pub fn builtin_init(runtime: &mut JsRuntime) {
    // global functions for runtime 
    fn assert(rt: &mut JsRuntime) {    
        let b = rt.top(-2).to_boolean();
        if !b {
            let info = rt.top(-1).to_string();
            panic!("ASSERT: {}", info);
        }
        rt.push_undefined();
    }

    fn println(rt: &mut JsRuntime) {
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
    runtime.genv.borrow_mut().init_var("assert", SharedValue::new_object(JsObject::new_builtin(assert, 2)) );
    runtime.genv.borrow_mut().init_var("println", SharedValue::new_object(JsObject::new_builtin(println, 1)) );
}