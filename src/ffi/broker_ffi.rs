use std::{ffi::CStr, ptr};

use libc::c_char;

use crate::{
    broker::{CallOptions, HandlerResult},
    registry::{
        discoverers::{ServiceBroker, ServiceBrokerMessage},
        Payload,
    },
    runtimemacro::runtime,
    RUNTIME,
};
use tokio::{sync::oneshot, task::JoinHandle};

pub extern "C" fn broker_create() -> *mut ServiceBroker {
    todo!()
}

pub extern "C" fn broker_destroy(broker: *mut ServiceBroker) {
    if !broker.is_null() {
        unsafe { drop(Box::from_raw(broker)) }
    }
}

pub unsafe extern "C" fn broker_start(broker: *mut ServiceBroker) -> *mut JoinHandle<()> {
    if broker.is_null() {
        //TODO:
    }
    let broker = &mut *broker;
    broker.start();
    let rt = RUNTIME.as_ref().unwrap();
    let join_handle = rt.spawn(async move {
        while let Some(message) = broker.reciever.recv().await {
            match message {
                ServiceBrokerMessage::AddLocalService(service) => broker.add_local_service(service),
                ServiceBrokerMessage::RegisterLocalService(service_spec) => {
                    broker.register_local_service(service_spec).await
                }
                ServiceBrokerMessage::Call {
                    action_name,
                    params,
                    opts,
                    result_channel,
                } => {
                    let result = broker
                        .call(&action_name, params, opts, result_channel)
                        .await;
                }
                ServiceBrokerMessage::Close => return,
                _ => {}
            }
        }
    });
    return Box::into_raw(Box::new(join_handle));
}
pub extern "C" fn broker_stop(broker: *mut ServiceBroker) {
    if broker.is_null() {}
    unsafe {
        let broker = &mut *broker;
        broker.stop();
    }
}
pub unsafe extern "C" fn broker_has_event_listener(
    broker: *const ServiceBroker,
    event_name: *const c_char,
) -> bool {
    if broker.is_null() {}
    if event_name.is_null() {}

    let raw_event = CStr::from_ptr(event_name);
    let event_name = match raw_event.to_str() {
        Ok(s) => s,
        Err(_) => todo!(),
    };
    let broker = &*broker;
    let result = broker.has_event_listener(event_name);
    todo!()
}

pub unsafe extern "C" fn broker_call(
    broker: *const ServiceBroker,
    action_name: *const c_char,
    params: *const Payload,
    opts: *const CallOptions,
) -> *mut HandlerResult {
    let (sender, recv) = oneshot::channel::<anyhow::Result<HandlerResult>>();

    let broker = &*broker;
    let raw_action = CStr::from_ptr(action_name);
    let action_name = match raw_action.to_str() {
        Ok(s) => s,
        Err(_) => todo!(),
    };
    let opts = &*opts;
    let params = &*params;
    let rt = RUNTIME.as_ref().unwrap();
    rt.spawn(async move {
        let _ = broker
            .call(action_name, params.clone(), opts.clone(), sender)
            .await;

        let result = recv.await.unwrap().unwrap();
        println!("{:?}", result);
        return result;
    });
    return ptr::null_mut();
}
