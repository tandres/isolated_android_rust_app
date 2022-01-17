use std::{
    io::Read,
    os::{
        raw::c_char,
        unix::prelude::FromRawFd,
    },
    thread, time,
};

use jni::{
    errors::Result,
    JNIEnv,
};
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jstring};

pub type Callback = unsafe extern "C" fn(*const c_char) -> ();

#[no_mangle]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello(env: JNIEnv, _class: JClass, input: JString) -> jstring {
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();

    let output = env.new_string(format!("Hello, {}!", input)).expect("Couldn't create java string!");

    output.into_inner()
}

/*
 * Hello, I don't know exactly what is happening here but something is reinterpreting _ as . in the 
 * toolchain. I was having trouble getting java to find the native version of read_file_native
 * (using snake case) so I created this thinking it was a type issue or something. This is also not
 * findable. Readelf shows unmodified symbol names.
#[no_mangle]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello_hello(env: JNIEnv, class: JClass, input: JString, num: jint) -> jstring {
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();

    let output = env.new_string(format!("Hello, {}!", input)).expect("Couldn't create java string!");

    output.into_inner()
}
*/

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_readFileNative(env: JNIEnv, _class: JClass, input: jint) -> jstring {
    let input: i32 = input;
    let mut buf = String::new();
    let mut file = unsafe { std::fs::File::from_raw_fd(input) };
    file.read_to_string(&mut buf).expect("Failed to read from string!");

    env.new_string(buf).expect("Couldn't create java string!").into_inner()
}

fn log_callback<'a, S, J>(env: &'a JNIEnv, msg: S, callback: J) -> Result<()>
where
    S: AsRef<str>,
    J: Into<JObject<'a>>,
{
    let s = format!("Rust: {}", msg.as_ref());
    let response = env.new_string(&s).expect("Couldn't create java string!");
    env.call_method(callback, 
                    "loggingCallback", 
                    "(Ljava/lang/String;)V", 
                    &[JValue::from(JObject::from(response))])
        .map(|_| ())
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_spawnThread(
    env: JNIEnv, 
    _class: JClass, 
    callback: JObject,
) {
    match log_callback(&env, "Starting thread", callback) {
        Ok(_) => (),
        Err(e) => {
            let _ = env.throw(format!("Exception: {}", e));
            return;
        }
    }
    let jvm = env.get_java_vm().unwrap();

    let callback = env.new_global_ref(callback).unwrap();
    thread::spawn(move || {
        let env = jvm.attach_current_thread().unwrap();
        let tick = time::Duration::from_millis(1000);
        loop {
            thread::sleep(tick);
            log_callback(&env, "Tick Tock", callback.as_obj());
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
