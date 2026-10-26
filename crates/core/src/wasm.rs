use std::cell::RefCell;

use crate::scheduling::schedule_json;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

// Synchronous ABI: write exactly len bytes before execute; pointers expire on the next request.
#[unsafe(no_mangle)]
pub extern "C" fn schedule_request(len: u32) -> *mut u8 {
    INPUT.with_borrow_mut(|input| {
        input.clear();
        if len > 16_384 {
            return std::ptr::null_mut();
        }
        input.resize(len as usize, 0);
        input.as_mut_ptr()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn schedule_execute() -> u32 {
    let result = INPUT.with_borrow(|input| match std::str::from_utf8(input) {
        Ok(request) => schedule_json(request),
        Err(_) => "{\"Err\":\"request is not UTF-8\"}".into(),
    });
    OUTPUT.with_borrow_mut(|output| {
        *output = result.into_bytes();
        output.len() as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn schedule_result() -> *const u8 {
    OUTPUT.with_borrow(|output| output.as_ptr())
}
