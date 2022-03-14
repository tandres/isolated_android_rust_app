use std::{
    io,
    os::{
        raw::c_char,
        unix::{net::UnixStream as StdUnixStream, prelude::FromRawFd},
    },
    thread, time,
};

use android_logger::{Config, FilterBuilder};
use log::{debug, error, info, trace, Level};

use tokio::{io::Interest, net::UnixStream as TokioUnixStream, runtime::Builder};

use jni::errors::Error;
use jni::objects::{JClass, JValue};
use jni::{objects::JObject, JNIEnv};

pub type Callback = unsafe extern "C" fn(*const c_char) -> ();

fn run_thread(name: String, socket_fd: i32) {
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
            match socket_handler(name, socket_fd).await {
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

async fn socket_handler(name: String, socket_fd: i32) -> std::io::Result<()> {
    let socket = unsafe { StdUnixStream::from_raw_fd(socket_fd) };
    socket.set_nonblocking(true)?;
    let stream = TokioUnixStream::from_std(socket)?;
    let mut message = "hello from ".to_string();
    message.push_str(&name);
    debug!("Set socket stream");
    loop {
        let tick = time::Duration::from_millis(1000);
        tokio::time::sleep(tick).await;
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;
        if ready.is_readable() {
            let mut data = vec![0; 1024];
            match stream.try_read(&mut data) {
                Ok(n) => {
                    trace!("read {} bytes", n);
                    trace!("{}", String::from_utf8_lossy(&data));
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
            match stream.try_write(message.as_bytes()) {
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
pub extern "C" fn Java_com_tandres_isolatedrustapp_IsolatedRustService_startChild(
    env: JNIEnv,
    _class: JClass,
    pfd: JObject,
) {
    let socket_fd: i32 = detach_fd(env, pfd).unwrap();

    let config = Config::default()
        .with_min_level(Level::Trace)
        .with_tag("Child Rust".to_string())
        .with_filter(FilterBuilder::new().parse("trace").build());
    android_logger::init_once(config);

    spawn_thread("Child".to_string(), socket_fd)
}

fn spawn_thread(name: String, socket_fd: i32) {
    thread::spawn(move || {
        run_thread(name, socket_fd);
    });
}

/// This will be called by android after we have connected to the IsolatedRustService.
/// The connection is initiated by the call to the Java method bindService in the rust
/// function bind_service.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainActivity_onServiceConnected(
    env: JNIEnv,
    _class: JClass,
    _class_name: JObject,
    service: JObject,
) {
    trace!("Service Connected");

    let (parent_fd, child_pfd) = match build_fds(env) {
        Ok((p, c)) => (p, c),
        Err(e) => {
            error!("Failed to build fds: {}", e);
            return;
        }
    };

    spawn_thread("Parent".to_string(), parent_fd);

    match start_child_process(env, service, child_pfd) {
        Ok(_) => trace!("Started child process!"),
        Err(e) => error!("Failed to start child process: {}", e),
    }
}

fn start_child_process(env: JNIEnv, service: JObject, pfd: JObject) -> Result<(), Error> {
    let service_stub_class =
        env.find_class("com/tandres/isolatedrustapp/IIsolatedRustInterface$Stub")?;
    let service_object = env
        .call_static_method(
            service_stub_class,
            "asInterface",
            "(Landroid/os/IBinder;)Lcom/tandres/isolatedrustapp/IIsolatedRustInterface;",
            &[JValue::Object(service)],
        )?
        .l()?;
    env.call_method(
        service_object,
        "start",
        "(Landroid/os/ParcelFileDescriptor;)V",
        &[JValue::Object(pfd)],
    )?;

    Ok(())
}

fn build_fds(env: JNIEnv) -> Result<(i32, JObject), Error> {
    let pfd_class = env.find_class("android/os/ParcelFileDescriptor")?;
    let pfd_pair = env
        .call_static_method(
            pfd_class,
            "createSocketPair",
            "()[Landroid/os/ParcelFileDescriptor;",
            &[],
        )?
        .l()?;
    let pfd_parent = env.get_object_array_element(*pfd_pair, 0)?;
    let pfd_child = env.get_object_array_element(*pfd_pair, 1)?;
    let fd_parent = detach_fd(env, pfd_parent)?;

    // for some reason we have to pass the pfd object to the child process
    Ok((fd_parent, pfd_child))
}

fn detach_fd(env: JNIEnv, pfd: JObject) -> Result<i32, Error> {
    env.call_method(pfd, "detachFd", "()I", &[])?.i()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainActivity_onServiceDisconnected(
    _env: JNIEnv,
    _class: JClass,
    _class_name: JObject,
) {
    error!("Service Disconnected!");
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainActivity_startParent(
    env: JNIEnv,
    _class: JClass,
    activity: JObject,
    intent: JObject,
) {
    let config = Config::default()
        .with_min_level(Level::Trace)
        .with_tag("Parent Rust".to_string())
        .with_filter(FilterBuilder::new().parse("trace").build());
    android_logger::init_once(config);

    match bind_service(env, activity, intent) {
        Ok(_) => trace!("Successfully called bindService!"),
        Err(e) => error!("Failed to call bindService: {}", e),
    }
}

fn bind_service(env: JNIEnv, activity: JObject, intent: JObject) -> Result<(), Error> {
    // intent here is currently passed in but could be created here
    // activity implements ServiceConnection as ServiceConnected ServiceDisconnected in this file
    // 1 here is android.content.Context.BIND_AUTO_CREATE
    let args = &[
        JValue::Object(intent),
        JValue::Object(activity),
        JValue::Int(1),
    ];
    let sig = "(Landroid/content/Intent;Landroid/content/ServiceConnection;I)Z";
    env.call_method(activity, "bindService", sig, args)?;

    Ok(())
}
