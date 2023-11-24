use std::{ffi::CStr, os::raw::c_char};

use ledger_transport::APDUCommand;
use ledger_transport_hid::{hidapi::HidApi, TransportNativeHID};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LedgerAnswer {
    data: Vec<u8>,
    status_word: u16,
}
pub struct LedgerHidTransport {
    transport: TransportNativeHID,
    _hid_api: HidApi,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LedgerDeviceInfo {
    pub name: String,
    pub path: String,
}

impl LedgerHidTransport {
    pub unsafe fn new(path: *const c_char) -> Result<Self, String> {
        let hid_api = HidApi::new();

        match hid_api {
            Ok(hid_api) => {
                let path = CStr::from_ptr(path);
                let transport = TransportNativeHID::open_path(&hid_api, path);
                match transport {
                    Ok(transport) => Ok(Self {
                        transport: transport,
                        _hid_api: hid_api,
                    }),
                    Err(err) => Err(err.to_string()),
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn exchange(
        &self,
        cla: u8,
        ins: u8,
        p1: u8,
        p2: u8,
        data: Vec<u8>,
    ) -> Result<LedgerAnswer, String> {
        let command = APDUCommand {
            cla: cla,
            ins: ins,
            p1: p1,
            p2: p2,
            data: data,
        };

        let result = self.transport.exchange(&command);
        match result {
            Ok(answer) => Ok(LedgerAnswer {
                data: answer.apdu_data().to_vec(),
                status_word: answer.retcode(),
            }),
            Err(err) => Err(err.to_string()),
        }
    }

    pub fn get_ledger_devices() -> Result<Vec<LedgerDeviceInfo>, String> {
        let hid_api = HidApi::new();

        match hid_api {
            Ok(hid_api) => {
                let devices = TransportNativeHID::list_ledgers(&hid_api);
                let hid_devices: &mut Vec<LedgerDeviceInfo> = &mut vec![];
                for device in devices {
                    hid_devices.push(LedgerDeviceInfo {
                        name: device.product_string().unwrap().to_string(),
                        path: device.path().to_str().unwrap().to_string(),
                    });
                }
                Ok(hid_devices.clone())
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
