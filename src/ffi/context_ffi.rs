use std::{ffi::CStr, ptr};

use crate::broker::BrokerOptions;
use crate::runtimemacro::runtime;
use crate::RUNTIME;
use crate::{broker::CallOptions, context::Context, registry::Payload, BrokerSender};
use libc::c_char;
use serde_json::Value;
#[no_mangle]
pub unsafe extern "C" fn context_create(
    broker_sender: *const BrokerSender,
    node_id: *const c_char,
    service: *const c_char,
) -> *mut Context {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn context_destroy(context: *mut Context) {
    if !context.is_null() {
        drop(Box::from_raw(context))
    }
}

#[no_mangle]
pub unsafe extern "C" fn context_call(
    context: *const Context,
    action_name: *const c_char,
    broker_options: *const BrokerOptions,
    params: *const Payload,
    opts: *mut CallOptions,
) {
    if context.is_null() {
        //TODO: Error handling
    }
    if params.is_null() {
        //TODO: Error handling
    }
    if opts.is_null() {
        //TODO: Error handling
    }
    if action_name.is_null() {
        //TODO: Error handling
    }
    if broker_options.is_null() {
        //TODO: Error handling
    }
    let raw_action = CStr::from_ptr(action_name);
    let action_as_str = match raw_action.to_str() {
        Ok(s) => s,
        Err(_) => {
            //TODO: error handling
            todo!()
        }
    };
    let context = &*context;
    let broker_options = &*broker_options;
    let params = (&*params).to_owned();
    let rt = runtime!();
    let mut opts = (&*opts).to_owned();
    rt.spawn(async move {
        let result = context
            .call(broker_options, action_as_str, params, opts)
            .await;
    });

    todo!();
}
#[no_mangle]
pub unsafe extern "C" fn context_emit(
    context: *const Context,
    event_name: *const c_char,
    data: *mut Value,
    opts: *mut Value,
) {
    if context.is_null() {}
    if event_name.is_null() {}
    let raw_event = CStr::from_ptr(event_name);
    let event_name = match raw_event.to_str() {
        Ok(s) => s,
        Err(_) => todo!(),
    };
    if data.is_null() {}
    let opts = match opts.is_null() {
        true => None,
        false => Some((&*opts).to_owned()),
    };
    let context = &*context;
    let data = (&*data).to_owned();

    let result = context.emit(event_name, data, opts);
}
#[no_mangle]
pub unsafe extern "C" fn context_broadcast(
    context: *const Context,
    event_name: *const c_char,
    data: *mut Value,
    opts: *mut Value,
) {
    if context.is_null() {}
    if event_name.is_null() {}
    let raw_event = CStr::from_ptr(event_name);
    let event_name = match raw_event.to_str() {
        Ok(s) => s,
        Err(_) => todo!(),
    };
    if data.is_null() {}
    let opts = match opts.is_null() {
        true => None,
        false => Some((&*opts).to_owned()),
    };
    let context = &*context;
    let data = (&*data).to_owned();
    let rt = runtime!();
    rt.spawn(async move {
        let _ = context.broadcast(event_name, data, opts).await;
    });
}
