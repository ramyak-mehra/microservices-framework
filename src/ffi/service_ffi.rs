use std::{ffi::CStr, ptr};

use libc::c_char;

use crate::{
    broker::HandlerResult,
    context::Context,
    service::{Schema, SchemaActions, Service},
    BrokerSender, RUNTIME,
};

#[no_mangle]
pub unsafe extern "C" fn service_create(
    schema: *const Schema,
    broker_sender: *const BrokerSender,
) -> *mut Service {
    if schema.is_null() {
        return ptr::null_mut();
    }
    if broker_sender.is_null() {
        return ptr::null_mut();
    }
    let schema = (&*schema).to_owned();
    let broker_sender = (&*broker_sender).to_owned();
    let service = Service::from_schema(schema, broker_sender);
    Box::into_raw(Box::new(service))
}

#[no_mangle]
pub unsafe extern "C" fn service_destroy(svc: *mut Service) {
    if !svc.is_null() {
        drop(Box::from_raw(svc))
    }
}

#[no_mangle]
pub unsafe extern "C" fn service_start(svc: *const Service) {
    if !svc.is_null() {
        let svc = &*svc;
        let rt = RUNTIME.as_ref().unwrap();
        rt.spawn(async move {
            svc.init().await;
            svc.start().await;
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn schema_create(
    name: *const c_char,
    version: *const c_char,
    action: *mut SchemaActions,
) -> *mut Schema {
    if name.is_null() {
        return ptr::null_mut();
    }
    let raw_name = CStr::from_ptr(name);
    let name_as_str = match raw_name.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut schema_version = None;
    if !version.is_null() {
        let raw_version = CStr::from_ptr(version);
        let version_as_str = match raw_version.to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        schema_version = Some(String::from(version_as_str));
    }
    let action = (&*action).to_owned();
    let schema = Schema::new(name_as_str.to_string(), schema_version, action);
    Box::into_raw(Box::new(schema))
}
#[no_mangle]
pub unsafe extern "C" fn schema_destroy(schema: *mut Schema) {
    if !schema.is_null() {
        drop(Box::from_raw(schema))
    }
}
#[no_mangle]
pub unsafe extern "C" fn schema_action_create(
    name: *const c_char,
    handler: fn(Context) -> HandlerResult,
) -> *mut SchemaActions {
    if name.is_null() {
        return ptr::null_mut();
    }
    let raw = CStr::from_ptr(name);
    let name_as_str = match raw.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let action = SchemaActions {
        name: name_as_str.to_string(),
        handler,
    };
    Box::into_raw(Box::new(action))
}
#[no_mangle]
pub unsafe extern "C" fn schema_action_destroy(action: *mut SchemaActions) {
    if !action.is_null() {
        drop(Box::from_raw(action))
    }
}
