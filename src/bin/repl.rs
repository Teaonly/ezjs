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

fn new_hook(rt: &mut ezjs::runtime::JsRuntime<MyHook>, argc: usize)  {
    if argc != 1 {
        panic!("new_hook argument count error! {}", line!());
    }
    let value = rt.top(-1).to_string();
    let new_hook = rt.new_hook(MyHook::new(value));
    rt.push_object( ezjs::value::SharedObject_new(new_hook));
}

fn print_hook(rt: &mut ezjs::runtime::JsRuntime<MyHook>, argc: usize) {
    if argc != 1 {
        panic!("print_hook argument count error! {}", line!());
    }
    let value = rt.top(-1);
    let hook = rt.get_hook( &value );
    println!("hook {:?} ", hook);
    rt.push_undefined();
}

fn show_hooks(rt: &mut ezjs::runtime::JsRuntime<MyHook>, _argc: usize)  {    
    println!("{:?}", rt.hooks);
    rt.push_number( rt.hooks.keys().len() as f64);
}

pub fn main() {
    let mut rt = ezjs::new_runtime::<MyHook>( MyHook::new("_".to_string()) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(new_hook));
    rt.genv.borrow_mut().init_var("new_hook", ezjs::value::SharedValue::new_object(fobj) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(print_hook));
    rt.genv.borrow_mut().init_var("print_hook", ezjs::value::SharedValue::new_object(fobj) );

    let fobj = rt.new_builtin(ezjs::runtime::JsBuiltinFunction::new(show_hooks));
    rt.genv.borrow_mut().init_var("show_hooks", ezjs::value::SharedValue::new_object(fobj) );

    println!("REPL of ezjs v0.1.0");
    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        let content = fs::read_to_string(&args[i]).unwrap();
        let vmf = ezjs::build_function_from_code(&content).unwrap();
        ezjs::dump_function(&vmf);
        let ret = ezjs::run_script(&mut rt, vmf);
        if ret.is_err() {
            println!("{}", ret.err().unwrap().to_string());
            break;
        }
    }

    loop {
        print!("=>");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_n) => {
                if line != "" {

                    let vmf = ezjs::build_function_from_code(&line).unwrap();
                    //ezjs::dump_function(&vmf);

                    let begin = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    let ret = ezjs::run_script(&mut rt, vmf);
                    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

                    if ret.is_ok() {
                        println!("<{}> {}", end - begin, ret.unwrap().to_string());
                    } else {
                        println!("{}", ret.err().unwrap().to_string());
                    }
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
