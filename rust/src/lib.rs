use jni::JNIEnv;

use jni::objects::{JClass, JString};

use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello(env: JNIEnv, class: JClass, input: JString) -> jstring {
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();

    let output = env.new_string(format!("Hello, {}!", input)).expect("Couldn't create java string!");

    output.into_inner()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
