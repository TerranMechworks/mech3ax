pub struct CallbackBuffer(Option<Vec<u8>>);

impl CallbackBuffer {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn inner(self) -> Option<Vec<u8>> {
        self.0
    }
}

#[no_mangle]
pub extern "C" fn buffer_set_data(buffer: *mut CallbackBuffer, pointer: *const u8, length: usize) {
    if buffer.is_null() || pointer.is_null() {
        return;
    }
    let data = unsafe { std::slice::from_raw_parts(pointer, length) };
    let buffer = unsafe { buffer.as_mut().unwrap() };
    // importantly, to_vec copies the values into a new vec
    buffer.0 = Some(data.to_vec());
}
