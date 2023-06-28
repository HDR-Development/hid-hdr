use std::{
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};

pub enum Status {
    Ok,
    NoRecenterFound,
    NoMapSticks1Found,
    NoMapSticks2Found,
    NotConnected,
    NotPresentOnSD,
    Unknown(u8),
}

impl From<u8> for Status {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::NoRecenterFound,
            2 => Self::NoMapSticks1Found,
            3 => Self::NoMapSticks2Found,
            other => Self::Unknown(other),
        }
    }
}

extern "C" {
    #[link_name = "_ZN2nn2sm16GetServiceHandleEPNS_3svc6HandleEPKcm"]
    fn get_service_handle(handle: *mut u32, name: *const u8, len: usize) -> u32;

    #[link_name = "_ZN2nn2sm15RegisterServiceEPNS_3svc6HandleEPKcmib"]
    fn register_service(
        handle: *mut u32,
        name: *const u8,
        len: usize,
        max: i32,
        is_light: bool,
    ) -> u32;

    #[link_name = "_ZN2nn2sm17UnregisterServiceEPKcm"]
    fn unregister_service(name: *const u8, len: usize) -> u32;

    fn svcSendSyncRequest(handle: u32) -> u32;
}

pub fn does_hid_hdr_exist() -> bool {
    unsafe {
        let mut handle = 0;
        let result = register_service(&mut handle, b"hid:hdr\0".as_ptr(), 7, 100, false);
        if result == 0 {
            unregister_service(b"hid:hdr\0".as_ptr(), 7);
        }
        result == 0x815
    }
}

static CONNECTED_HANDLE: AtomicU32 = AtomicU32::new(0);

pub fn connect_to_hid_hdr() -> bool {
    if CONNECTED_HANDLE.load(Ordering::Acquire) != 0 {
        return true;
    }

    let mut handle = 0u32;
    let result = unsafe { get_service_handle(&mut handle, b"hid:hdr\0".as_ptr(), 7) };

    if result == 0 {
        CONNECTED_HANDLE.store(handle, Ordering::Release);
    }

    result == 0
}

unsafe fn get_tls_ptr() -> *mut u8 {
    let tls_ptr: *mut u8;
    std::arch::asm!("mrs {}, tpidrro_el0", out(reg) tls_ptr);
    tls_ptr
}

std::arch::global_asm!(
    r#"
.section .text.svcSendSyncRequest, "ax", %progbits
.global svcSendSyncRequest
.type svcSendSyncRequest, %function
.hidden svcSendSyncRequest
.align 2
.cfi_startproc
svcSendSyncRequest:
    svc 0x21
    ret
.cfi_endproc
"#
);

pub fn get_hid_hdr_status() -> Result<Status, u32> {
    const PATHS: &[&'static str] = &[
        "sd:/atmosphere/contents/0100000000000013/exefs/main.npdm",
        "sd:/atmosphere/contents/0100000000000013/exefs/rtld",
    ];

    for path in PATHS {
        if !Path::new(path).exists() {
            return Ok(Status::NotPresentOnSD);
        }
    }

    let handle = CONNECTED_HANDLE.load(Ordering::Acquire);
    if handle == 0 {
        return Ok(Status::NotConnected);
    }

    unsafe {
        let tls = get_tls_ptr() as *mut u32;
        *tls.add(0) = 0x4; // Request
        *tls.add(1) = 0x8; // No extra info, raw data size of 8
        *tls.add(2) = 0; // padding 0
        *tls.add(3) = 0; // padding 1
        *tls.add(4) = 0x49434653; // SFCI magic
        *tls.add(5) = 1; // version 1
        *tls.add(6) = 1; // command id 0
        *tls.add(7) = 0; // raw header padding
        *tls.add(8) = 0; // turn on stick control
        *tls.add(9) = 0; // padding 2

        let result = svcSendSyncRequest(handle);
        if result != 0 {
            return Err(result);
        }

        let tls_ptr = get_tls_ptr() as *const u32;
        Ok(Status::from(*tls_ptr.add(8) as u8))
    }
}

pub fn configure_stick_gate_changes(enable: bool) -> Result<bool, u32> {
    let handle = CONNECTED_HANDLE.load(Ordering::Acquire);
    if handle == 0 {
        return Ok(false);
    }

    unsafe {
        let tls = get_tls_ptr() as *mut u32;
        *tls.add(0) = 0x4; // Request
        *tls.add(1) = 0x9; // No extra info, raw data size of 8
        *tls.add(2) = 0; // padding 0
        *tls.add(3) = 0; // padding 1
        *tls.add(4) = 0x49434653; // SFCI magic
        *tls.add(5) = 1; // version 1
        *tls.add(6) = 0; // command id 0
        *tls.add(7) = 0; // raw header padding
        *tls.add(8) = enable as u32; // turn on stick control
        *tls.add(9) = 0; // padding 2
        *tls.add(10) = 0; // padding 3

        let result = svcSendSyncRequest(handle);
        if result != 0 {
            return Err(result);
        }

        Ok(true)
    }
}

#[cfg(feature = "warnings")]
pub fn warn_unable_to_connect(discord_channel: &str, mod_name: &str, invite: &str) {
    skyline_web::DialogOk::new(
        format!(
            r#"{mod_name} is unable to connect to the HID system module.
            Usually this can be fixed by restarting your console (rebooting to payload
            is good enough). If this error continues, screenshot this and send
            it in the #{discord_channel} channel in the {mod_name} discord @ {invite}"#
        ),
        "Ok",
    );
}

#[cfg(feature = "warnings")]
pub fn warn_status(status: Status, discord_channel: &str, mod_name: &str, invite: &str) {
    match status {
        Status::Ok => {}
        Status::NotConnected => {
            skyline_web::DialogOk::new(
                format!(
                    r#"{mod_name} is not connected with the HID system module.
                    This is a dev error, screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
        Status::NoRecenterFound => {
            skyline_web::DialogOk::new(
                format!(
                    r#"The HID system module was unable to find the stick recentering code.
                    Please screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
        Status::NoMapSticks1Found => {
            skyline_web::DialogOk::new(
                format!(
                    r#"The HID system module was unable to find the first stick mapping function call.
                    Please screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
        Status::NoMapSticks2Found => {
            skyline_web::DialogOk::new(
                format!(
                    r#"The HID system module was unable to find the second stick mapping function call.
                    Please screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
        Status::NotPresentOnSD => {
            skyline_web::DialogOk::new(
                format!(
                    r#"The HID system module change is not present on the SD card.
                    Please screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
        Status::Unknown(error) => {
            skyline_web::DialogOk::new(
                format!(
                    r#"The HID system module had some kind of unknown error. Code: {error:#}
                    Please screenshot this and send it in the #{discord_channel}
                    channel in the {mod_name} discord @ {invite}"#
                ),
                "Ok",
            );
        }
    }
}
