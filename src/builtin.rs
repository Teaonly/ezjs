use std::collections::HashMap;

use crate::bytecode::*;
use crate::value::*;
use crate::runtime::*;

use crate::builtin_script::*;

impl<T:Hookable> JsBuiltinFunction<T> {
	pub fn new(f: fn(&mut JsRuntime<T>), argc: usize) -> Self {
		JsBuiltinFunction {
			f:		f,
			argc:	argc
		}
	}
}

// The Object class 
fn object_constructor<T: Hookable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_something() {        
        rt.push( value.duplicate() );
        return;
    }
    rt.push( SharedValue::new_vanilla(rt.prototypes.object_prototype.clone()) );
}

fn object_tostring<T: Hookable>(rt: &mut JsRuntime<T>)  {
    let thiz = rt.top(-1);
    rt.push_string( thiz.to_string());
}

fn object_proto<T: Hookable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-1);
    if !target.is_object() {
        rt.push_undefined();    
        return;
    }

    let target_object = target.get_object();

    if target_object.borrow().__proto__.is_some() {
        let proto = target_object.borrow().__proto__.as_ref().unwrap().clone();
        rt.push_object(proto);
        return;
    }
    rt.push_null();
}

fn object_proto_builtins<T: Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    builtins.insert("proto".to_string(), JsBuiltinFunction::new(object_proto, 0));
    return builtins;
}

fn object_preventextensions<T: Hookable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_object() {
        value.get_object().borrow_mut().extensible = false;
    }
    rt.push(value);
}

fn object_setprototypeof<T: Hookable>(rt: &mut JsRuntime<T>) {
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

fn object_defineproperty<T: Hookable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-3);
    if !target.is_object() {
        rt.push(target);
        return;
    }
    let target_object = target.get_object();

    let name = rt.top(-2);
    if !name.is_string() {
        rt.push(target);
        return;
    }
    let name = name.to_string();

    let desc = rt.top(-1);
    if !desc.is_object() {
        rt.push(target);
        return;
    }
    let desc_object = desc.get_object();

    let mut configurable = false;    
    let prop_r = desc_object.borrow().query_property("configurable");
    if let Some((prop,_)) = prop_r {
        if prop.value.is_boolean() {
            configurable = prop.value.to_boolean();
        }
    }

    let mut enumerable = false;    
    let prop_r = desc_object.borrow().query_property("enumerable");
    if let Some((prop,_)) = prop_r {
        if prop.value.is_boolean() {
            enumerable = prop.value.to_boolean();   
        }
    }

    let mut writable = false;
    let prop_r = desc_object.borrow().query_property("writable");
    if let Some((prop,_)) = prop_r {
        if prop.value.is_boolean() {
            writable = prop.value.to_boolean();
        }
    }

    let prop_attr : JsPropertyAttr = (writable, enumerable, configurable);
    let mut value = SharedValue::new_undefined();
    let mut getter = None;
    let mut setter = None;

    let prop_r = desc_object.borrow().query_property("value");
    if let Some((prop, _)) = prop_r {
        value = prop.value;                
    } else {
        let prop_r = desc_object.borrow().query_property("get");
        if let Some((prop, _)) = prop_r {
            //getter = Some(prop.value);
            if prop.value.is_object() {
                let obj = prop.value.get_object();
                if obj.borrow().callable() {
                    getter = Some(obj);
                }
            }
        }        
        let prop_r = desc_object.borrow().query_property("set");
        if let Some((prop, _)) = prop_r {
            //setter = Some(prop.value);
            if prop.value.is_object() {
                let obj = prop.value.get_object();
                if obj.borrow().callable() {
                    setter = Some(obj);
                }
            }
        }
    }

    let mut prop = JsProperty::new();
    prop.fill(value, prop_attr, getter, setter);
    target_object.borrow_mut().set_property(&name, prop);
    rt.push(target);
    return;
}

// Object.XXXX not in Object.prototype
fn object_builtins<T:Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();   
    builtins.insert("preventExtensions".to_string(), JsBuiltinFunction::new(object_preventextensions, 1));
    builtins.insert("getPrototypeOf".to_string(), JsBuiltinFunction::new(object_proto, 1));
    builtins.insert("setPrototypeOf".to_string(), JsBuiltinFunction::new(object_setprototypeof, 2));
    builtins.insert("defineProperty".to_string(), JsBuiltinFunction::new(object_defineproperty, 3));
    return builtins;
}

// The String class
fn string_constructor<T:Hookable>(rt: &mut JsRuntime<T>) {
    let value = rt.top(-1);
    if value.is_string() {
        rt.push(value.duplicate());
        return;
    }    
    rt.push_string("".to_string());
}

fn string_proto_builtins<T:Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    return builtins;
}

// The Array class
fn array_constructor<T:Hookable>(rt: &mut JsRuntime<T>) {    
    let obj = JsObject::new_array(rt.prototypes.array_prototype.clone());
    let jv = SharedValue::new_object(obj);
    rt.push(jv);
}

fn array_push<T:Hookable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-2);
    assert!(target.is_object());
    let sobj = target.get_object();
    let mut object = sobj.borrow_mut();
    assert!(object.is_array());
   
    let value = rt.top(-1).duplicate();
    object.get_mut_array().push(value);
    
    rt.push_number(object.get_array().len() as f64);
}

fn array_length<T:Hookable>(rt: &mut JsRuntime<T>) {
    let target = rt.top(-1);
    assert!(target.is_object());
    let sobj = target.get_object();
    let object = sobj.borrow_mut();
    assert!(object.is_array());
   
    rt.push_number(object.get_array().len() as f64);
}

fn array_proto_builtins<T:Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    builtins.insert("push".to_string(), JsBuiltinFunction::new(array_push, 1));
    builtins.insert("__len__".to_string(), JsBuiltinFunction::new(array_length, 0));
    return builtins;
}

// The Function class
fn function_constructor<T:Hookable>(rt: &mut JsRuntime<T>) {
    let vmf = SharedFunction_new(VMFunction::new_anonymous());
    let fobj = JsObject::new_function(vmf, rt.cenv.clone(), rt.prototypes.function_prototype.clone());
    rt.push(SharedValue::new_object(fobj));
}

fn function_apply<T: Hookable>(rt: &mut JsRuntime<T>)  {
    let func = rt.top(-3);
    let new_thiz = rt.top(-2);
    let arguments_object = rt.top(-1);

    let mut arguments: Vec<SharedValue> = Vec::new();
    if arguments_object.is_object() {
        let obj_ = arguments_object.get_object();
        let obj = obj_.borrow();

        let args = obj.get_array();
        for i in 0..args.len() {
            arguments.push( args[i].clone());
        }
    }

    let argc = arguments.len();
    rt.push(func);
    rt.push(new_thiz);
    for i in 0..arguments.len() {
        rt.push( arguments[i].clone() );
    }

    let ret = jscall(rt, argc);
    if ret.is_ok() {
        return;
    }
    let exp = ret.err().unwrap();

    panic!(" apply with exception happen! {:?}", exp);
}

fn function_proto_builtins<T:Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    let mut builtins = HashMap::new();
    builtins.insert("toString".to_string(), JsBuiltinFunction::new(object_tostring, 0));
    builtins.insert("apply".to_string(), JsBuiltinFunction::new(function_apply, 2));
    return builtins;
}

// The Exception class
fn exception_constructor<T:Hookable>(rt: &mut JsRuntime<T>) {
    
    let value = rt.top(-1);    
    let msg = value.to_string();

    let exp = JsException::new(msg);
    let value = SharedValue::new_object(JsObject::new_exception(rt.prototypes.exception_prototype.clone(), exp));
    rt.push(value);
}

fn exception_message<T:Hookable>(rt: &mut JsRuntime<T>) {
    let exp_object = rt.top(-1).get_object();
    let exp = exp_object.borrow().get_exception();
    rt.push_string(exp.msg);
}

fn exception_proto_builtins<T:Hookable>() -> HashMap<String, JsBuiltinFunction<T>> {
    // TODO
    let mut builtins = HashMap::new();
    builtins.insert("message".to_string(), JsBuiltinFunction::new(exception_message, 0));
    return builtins;
}

// build class's global functions
fn create_class_functions<T:Hookable>(rt: &mut JsRuntime<T>, target: SharedObject, properties: HashMap<String, JsBuiltinFunction<T>>) {
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
fn create_builtin_class<T:Hookable>(rt: &mut JsRuntime<T>, constructor: JsBuiltinFunction<T>, properties: HashMap<String, JsBuiltinFunction<T>>, top: Option<SharedObject>) -> (SharedObject, SharedObject) {
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
fn set_global_class<T:Hookable>(rt: &mut JsRuntime<T>, name: &str, class_obj: SharedObject) {
    let mut prop = JsProperty::new();
    prop.fill_attr(JS_READONLY_ATTR);
    prop.value = SharedValue::new_sobject(class_obj);
    rt.genv.borrow_mut().target().borrow_mut().set_property(name, prop);
}

pub fn prototypes_init<T:Hookable>(rt: &mut JsRuntime<T>) {
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
    set_global_class(rt, "Error", exp_classs_object.clone());
    rt.prototypes.exception_prototype = exp_prototype;
}

fn create_console_object<T:Hookable>(runtime: &mut JsRuntime<T>) {
    fn println<T:Hookable>(rt: &mut JsRuntime<T>) {
        let msg = rt.top(-1).to_string();
        println!("{}", msg);
        rt.push_undefined();        
    }

    let console_value = SharedValue::new_vanilla(runtime.prototypes.object_prototype.clone());

    let mut prop = JsProperty::new();    
    let fvalue =  SharedValue::new_object(runtime.new_builtin(JsBuiltinFunction::new(println, 1)));
    prop.fill(fvalue, JS_DEFAULT_ATTR, None, None);    
    
    console_value.get_object().borrow_mut().set_property("log", prop);
    runtime.genv.borrow_mut().init_var("console", console_value);
}

pub fn builtin_init<T:Hookable>(runtime: &mut JsRuntime<T>) {
    // global functions for runtime 
    fn assert<T:Hookable>(rt: &mut JsRuntime<T>) {    
        let b = rt.top(-2).to_boolean();
        if !b {
            let info = rt.top(-1).to_string();
            panic!("ASSERT: {}", info);
        }
        rt.push_undefined();
    }
    // TODO : isFinite() isNaN() parseFloat() parseInt()
    
    // register some basic builtin functions
    let fobj = runtime.new_builtin(JsBuiltinFunction::new(assert, 2));
    runtime.genv.borrow_mut().init_var("assert", SharedValue::new_object(fobj) );

    // register some basic runtime objects
    create_console_object(runtime);

    // executing builtin code before any code.
    let vmf = crate::build_function_from_code(BUILDIN_SCRIPT).unwrap();    
    crate::run_script(runtime, vmf).unwrap();
}