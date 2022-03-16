use std::{
    io,
    os::{
        raw::c_char,
        unix::{net::UnixStream as StdUnixStream, prelude::FromRawFd},
    },
    thread, time::{self}, sync::{Mutex, Once}, borrow::BorrowMut,
};

use android_logger::{Config, FilterBuilder};
use log::{debug, error, info, trace, Level};

use tokio::{io::Interest, net::UnixStream as TokioUnixStream, runtime::Builder};

use jni::{errors::Error, objects::{GlobalRef, JString}};
use jni::objects::{JClass, JValue};
use jni::{objects::JObject, JNIEnv};

pub type Callback = unsafe extern "C" fn(*const c_char) -> ();

static mut GLOBAL_SERVICES: Option<[Mutex<Option<GlobalRef>>; 5]> = None;
static INIT: Once = Once::new();

fn global_services<'a>(index: usize) -> &'a Mutex<Option<GlobalRef>> {
    INIT.call_once(|| {
        unsafe {
            *GLOBAL_SERVICES.borrow_mut() = Some([Mutex::new(None), Mutex::new(None), Mutex::new(None), Mutex::new(None), Mutex::new(None)]);
        }
    });

    unsafe { &GLOBAL_SERVICES.as_ref().unwrap()[index] }
}

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

    let name = format!("Child");
    spawn_thread(name, socket_fd)
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
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainService_onServiceConnected(
    env: JNIEnv,
    _class: JClass,
    component: JObject,
    service: JObject,
) {
    trace!("Service Connected");

     match save_service_object(env, service, component) {
        Ok(index) => trace!("Saved service object {} successfully", index),
        Err(e) => error!("Failed to save service object: {}", e),
    };
}

fn save_service_object(env: JNIEnv, service: JObject, component: JObject) -> Result<u32, Error> {
    // parse class name for the index of this service, this breaks if we go double digit services
    let class_name_jstring  = JString::from(env.call_method(component, "getClassName", "()Ljava/lang/String;", &[])?.l()?);
    let class_name = String::from(env.get_string(class_name_jstring)?);
    let index = class_name.chars().nth(class_name.len() - 1).unwrap().to_digit(10).unwrap();

    // service interface
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
    let global_service_object = env.new_global_ref(service_object)?;
    
    // save to our list of global services
    *global_services(index as usize).lock().unwrap() = Some(global_service_object);

    Ok(index)
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
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainService_onServiceDisconnected(
    _env: JNIEnv,
    _class: JClass,
    _class_name: JObject,
) {
    error!("Service Disconnected!");
    // TODO - deallocate global ref
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn Java_com_tandres_isolatedrustapp_MainService_startParent(
    env: JNIEnv,
    _class: JClass,
    service: JObject,
) {
    let config = Config::default()
        .with_min_level(Level::Trace)
        .with_tag("Parent Rust".to_string())
        .with_filter(FilterBuilder::new().parse("trace").build());
    android_logger::init_once(config);

    // there should probably be a struct with this stuff in it 
    // env.find_class() only works on the main thread for our package classes
    let class_loader = get_class_loader(env).unwrap();
    // save service as global ref to move to new thread
    let mainservice = env.new_global_ref(service).unwrap();
    // save jvm to attach to new thread
    let jvm = env.get_java_vm().unwrap();

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap();
        rt.block_on(async {
            let mut futures = Vec::new();
            let tick_jh = tokio::spawn(async {
                let tick = time::Duration::from_millis(1000);
                loop {
                    debug!("Tick");
                    println!("Hello");
                    tokio::time::sleep(tick).await;
                }
            });
            let guard = jvm.attach_current_thread().unwrap();
            
            // bind all 5 of our services, this would be done as needed in real applications
            for i in 0..5 {
                let service_name = format!("com/tandres/isolatedrustapp/IsolatedRustService{}", i);
                let service_name_object = guard.new_string(service_name).unwrap();
                let service_class = guard.call_method(&class_loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(*service_name_object)]).unwrap().l().unwrap();
                match bind_service(*guard, mainservice.as_obj(), service_class) {
                    Ok(_) => trace!("Called to bindService for service id: {}", i),
                    Err(e) => error!("Failed to call bindService for service id {}: {}", i, e),
                };
            }

            for i in 0..5 {
                loop {
                    tokio::time::sleep(time::Duration::from_secs(1)).await;
                    let service_lock = global_services(i).lock().unwrap();
                    if let Some(service) = service_lock.as_ref() {
                        let (parent_fd, child_pfd) = build_fds(*guard).unwrap();

                        guard.call_method(
                            service,
                            "start",
                            "(Landroid/os/ParcelFileDescriptor;)V",
                            &[JValue::Object(child_pfd)],
                        ).unwrap();
                        let sock_jh = tokio::spawn(async move {
                            match socket_handler("Parent".to_string(), parent_fd).await {
                                Ok(_) => {
                                    debug!("Socket handler returned ok");
                                }
                                Err(e) => {
                                    error!("Socket handler returned error: {}", e);
                                }
                            }
                        });
                        futures.push(sock_jh);
                        break;
                    }
                }
            }
            futures.push(tick_jh);

            for future in futures {
                tokio::join!(future);
            }

        });
    });
}

fn bind_service(env: JNIEnv, mainservice: JObject, service_class: JObject) -> Result<(), Error> {
    let intent_class = env.find_class("android/content/Intent")?;
    let intent = env.new_object(intent_class, "(Landroid/content/Context;Ljava/lang/Class;)V", &[JValue::Object(mainservice), JValue::Object(service_class)])?;
    
    // mainservice implements ServiceConnection as ServiceConnected ServiceDisconnected in this file
    // 1 here is android.content.Context.BIND_AUTO_CREATE
    let args = &[
        JValue::Object(intent),
        JValue::Object(mainservice),
        JValue::Int(1),
    ];
    let sig = "(Landroid/content/Intent;Landroid/content/ServiceConnection;I)Z";
    env.call_method(mainservice, "bindService", sig, args)?;

    Ok(())
}

fn get_class_loader(env: JNIEnv) -> Result<GlobalRef, Error> {
    let thread_class = env.find_class("java/lang/Thread")?;
    let current_thread = env.call_static_method(thread_class, "currentThread", "()Ljava/lang/Thread;", &[])?.l()?;
    let class_loader = env.call_method(current_thread, "getContextClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
    env.new_global_ref(class_loader)
}
