mod transport;

use std::{
    ffi::c_void,
    os::raw::{c_char, c_int, c_longlong},
    sync::Arc,
};

use allo_isolate::Isolate;

use crate::{
    models::{HandleError, MatchResult, ToCStringPtr, ToStringFromPtr},
    runtime, PostWithResult, ToPtrAddress, RUNTIME,
};

use self::transport::LedgerHidTransport;

#[no_mangle]
pub unsafe extern "C" fn ll_get_ledger_devices(result_port: c_longlong) {
    runtime!().spawn(async move {
        fn internal_fn() -> Result<serde_json::Value, String> {
            let devices = LedgerHidTransport::get_ledger_devices();
            match devices {
                Ok(devices) => {
                    let ptr = serde_json::to_string(&devices)
                        .handle_error()?
                        .to_cstring_ptr() as u64;

                    serde_json::to_value(ptr).handle_error()
                }

                Err(err) => Err(err),
            }
        }

        let result = internal_fn().match_result();

        Isolate::new(result_port)
            .post_with_result(result.to_ptr_address())
            .unwrap();
    });
}

#[no_mangle]
pub unsafe extern "C" fn ll_create_ledger_transport(path: *const c_char) -> *mut c_char {
    unsafe fn internal_fn(path: *const c_char) -> Result<serde_json::Value, String> {
        let transport = LedgerHidTransport::new(path);

        match transport {
            Ok(transport) => {
                let ptr = Box::into_raw(Box::new(Arc::new(transport)));

                serde_json::to_value(ptr.to_ptr_address()).handle_error()
            }
            Err(err) => Err(err),
        }
    }

    internal_fn(path).match_result()
}
#[no_mangle]
pub unsafe extern "C" fn ll_ledger_transport_free_ptr(ptr: *mut c_void) {
    let _ = Box::from_raw(ptr as *mut Arc<LedgerHidTransport>);
}

#[no_mangle]
pub unsafe extern "C" fn ll_ledger_exchange(
    result_port: c_longlong,
    transport: *mut c_void,
    cla: c_int,
    ins: c_int,
    p1: c_int,
    p2: c_int,
    data: *mut c_char,
) {
    let transport = (&*(transport as *mut Arc<LedgerHidTransport>)).clone();
    let data = data.to_string_from_ptr();

    runtime!().spawn(async move {
        fn internal_fn(
            transport: Arc<LedgerHidTransport>,
            cla: c_int,
            ins: c_int,
            p1: c_int,
            p2: c_int,
            data: String,
        ) -> Result<serde_json::Value, String> {
            let data = serde_json::from_str::<Vec<u8>>(&data)
                .handle_error()?
                .into_iter()
                .collect::<Vec<_>>();

            let result = transport.exchange(cla as u8, ins as u8, p1 as u8, p2 as u8, data);

            match result {
                Ok(result) => {
                    let result = serde_json::to_string(&result)
                        .handle_error()?
                        .to_cstring_ptr() as u64;

                    serde_json::to_value(result).handle_error()
                }
                Err(err) => Err(err),
            }
        }

        let result = internal_fn(transport, cla, ins, p1, p2, data).match_result();

        Isolate::new(result_port)
            .post_with_result(result.to_ptr_address())
            .unwrap();
    });
}
