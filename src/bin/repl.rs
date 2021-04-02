use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use ezjs;

#[derive(Clone, Debug)]
struct MyHook {
    value: String,
}
impl MyHook {
    pub fn new(s: String) -> MyHook {        
        MyHook {
            value: s
        }
    }
}

impl Drop for MyHook {
    fn drop(&mut self) {
        println!(" ############## DROP HOOK ########### {:?}", self);
    }
}

impl ezjs::runtime::Hookable for MyHook {
    fn name(&self) -> String {
        return self.value.clone();
    }
}

fn new_hook(rt: &mut ezjs::runtime::JsRuntime<MyHook>)  {
    let value = rt.top(-1).to_string();
    let new_hook = rt.new_hook(MyHook::new(value));
    rt.push_object( ezjs::value::SharedObject_new(new_hook));             
}

fn print_hook(rt: &mut ezjs::runtime::JsRuntime<MyHook>) {
    let value = rt.top(-1);
    let hook = rt.get_hook( &value );
    println!("hook {:?} ", hook);
    rt.push_undefined();
}

fn show_hooks(rt: &mut ezjs::runtime::JsRuntime<MyHook>)  {
    println!("{:?}", rt.hooks);
    rt.push_number( rt.hooks.keys().len() as f64);
}

pub fn main() {
    let mut rt = ezjs::new_runtime::<MyHook>( MyHook::new("_".to_string()) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(new_hook, 1));
    rt.genv.borrow_mut().init_var("new_hook", ezjs::value::SharedValue::new_object(fobj) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(print_hook, 1));
    rt.genv.borrow_mut().init_var("print_hook", ezjs::value::SharedValue::new_object(fobj) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(show_hooks, 0));
    rt.genv.borrow_mut().init_var("show_hooks", ezjs::value::SharedValue::new_object(fobj) );

    println!("REPL of ezjs v0.1.0");
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        let content = fs::read_to_string(&args[i]).unwrap();
        let vmf = ezjs::build_function_from_code(&content).unwrap();
        ezjs::dump_function(&vmf);        
        ezjs::run_script(&mut rt, vmf).unwrap();
    }

    loop {
        print!("=>");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_n) => {
                if line != "" {
                    
                    let vmf = ezjs::build_function_from_code(&line).unwrap();
                    ezjs::dump_function(&vmf);

                    let begin = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    let _ret = ezjs::run_script(&mut rt, vmf).unwrap();
                    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

                    println!("<{}>", end - begin);
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    rt.push_undefined();
}