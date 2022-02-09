use std::{
    io::{self, prelude::*},
    os::{
        raw::c_char,
        unix::{
            prelude::FromRawFd,
            net::UnixStream as StdUnixStream,
        },
    },
    thread, time,
};

use log::{debug, error, info, trace, Level};
use android_logger::{Config, FilterBuilder};

use tokio::{
    io::Interest,
    net::UnixStream as TokioUnixStream,
    runtime::Builder,
};

use jni::{JNIEnv, sys::jintArray};
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};

pub type Callback = unsafe extern "C" fn(*const c_char) -> ();

#[no_mangle]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_hello(env: JNIEnv, _class: JClass, input: JString) -> jstring {
    let input: String = env.get_string(input).expect("Couldn't get java string!").into();

    let output = env.new_string(format!("Hello, {}!", input)).expect("Couldn't create java string!");

    output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_readFileNative(env: JNIEnv, _class: JClass, input: jint) -> jstring {
    let input: i32 = input;
    let mut buf = String::new();
    let mut file = unsafe { std::fs::File::from_raw_fd(input) };
    file.read_to_string(&mut buf).expect("Failed to read from string!");

    env.new_string(buf).expect("Couldn't create java string!").into_inner()
}

fn run_thread(tag: String, socket_fd: i32) {
    let config = Config::default()
        .with_min_level(Level::Trace)
        .with_tag(tag)
        .with_filter(FilterBuilder::new().parse("trace").build());
    android_logger::init_once(config);
    info!("Running rust thread");
    
    let rt = Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(async {
        info!("Async runtime alive!");
        let tick_jh = tokio::spawn(async {
            let tick = time::Duration::from_millis(1000);
            loop {
                debug!("Tick");
                println!("Hello");
                tokio::time::sleep(tick).await;
            }
        });
        info!("Setting up socket handler");
        let sock_jh = tokio::spawn(async move {
            match socket_handler(socket_fd).await {
                Ok(_) => {
                    debug!("Socket handler returned ok");
                }
                Err(e) => {
                    error!("Socket handler returned error: {}", e);
                }
            }
        });

        let _ = tokio::join!(tick_jh, sock_jh);
    });
    debug!("End of rust thread");
}

async fn socket_handler(socket_fd: i32) -> std::io::Result<()> {
    let socket = unsafe { StdUnixStream::from_raw_fd(socket_fd) };
    socket.set_nonblocking(true)?;
    let stream = TokioUnixStream::from_std(socket)?;
    debug!("Set socket stream");
    loop {
        let tick = time::Duration::from_millis(1000);
        tokio::time::sleep(tick).await;
        let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;
        if ready.is_readable() {
            let mut data = vec![0; 1024];
            match stream.try_read(&mut data) {
                Ok(n) => {
                    trace!("read {} bytes", n);        
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if ready.is_writable() {
            match stream.try_write(b"hello world") {
                Ok(n) => {
                    trace!("write {} bytes", n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_RustHelloWorld_spawnThread(
    env: JNIEnv, 
    _class: JClass, 
    tag: JString,
    socket_fd: jint,
) {
    let tag: String = env.get_string(tag).expect("Couldn't get java string!").into();
    let socket_fd: i32 = socket_fd;
    thread::spawn(move || {
        run_thread(tag, socket_fd); 
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
