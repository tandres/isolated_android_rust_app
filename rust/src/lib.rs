use std::{
    io::Read,
    os::unix::prelude::FromRawFd,
};

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};

#[no_mangle]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello(env: JNIEnv, _class: JClass, input: JString, num: jint) -> jstring {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
