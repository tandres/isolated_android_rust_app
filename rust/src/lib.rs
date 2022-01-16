use std::{
    io::Read,
    os::unix::prelude::FromRawFd,
};

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};

#[no_mangle]
pub extern "system" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello(env: JNIEnv, _class: JClass, input: JString) -> jstring {
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();

    let output = env.new_string(format!("Hello, {}!", input)).expect("Couldn't create java string!");

    output.into_inner()
}

#[no_mangle]
pub extern "system" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_read_file(env: JNIEnv, _class: JClass, input: jint) -> jstring {
    let input: i32 = input;
    let mut buf = String::new();
    let mut file = unsafe { std::fs::File::from_raw_fd(input) };
    file.read_to_string(&mut buf).expect("Failed to read from string!");

    env.new_string(buf).expect("Couldn't create java string!").into_inner()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
