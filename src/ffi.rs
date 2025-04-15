use std::ffi::{CStr, CString, c_char};
use std::os::raw::c_void;
use std::ptr;

// Import the main service
use crate::VoiceAssistantService;

// FFI function to process audio from Flutter
#[no_mangle]
pub extern "C" fn process_audio(
    api_key: *const c_char,
    audio_path: *const c_char,
    output_path: *const c_char,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) {
    // Convert C strings to Rust strings
    let api_key = unsafe {
        if api_key.is_null() {
            callback(
                CString::new("API key is null").unwrap().into_raw(),
                user_data,
            );
            return;
        }
        CStr::from_ptr(api_key).to_str().unwrap().to_owned()
    };

    let audio_path = unsafe {
        if audio_path.is_null() {
            callback(
                CString::new("Audio path is null").unwrap().into_raw(),
                user_data,
            );
            return;
        }
        CStr::from_ptr(audio_path).to_str().unwrap().to_owned()
    };

    let output_path = unsafe {
        if output_path.is_null() {
            callback(
                CString::new("Output path is null").unwrap().into_raw(),
                user_data,
            );
            return;
        }
        CStr::from_ptr(output_path).to_str().unwrap().to_owned()
    };

    // Process in a new thread to avoid blocking
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let assistant = VoiceAssistantService::new(api_key);

        let result = runtime.block_on(async {
            match assistant.process_audio(&audio_path, &output_path).await {
                Ok(()) => CString::new("Success").unwrap(),
                Err(e) => CString::new(format!("Error: {}", e)).unwrap(),
            }
        });

        callback(result.into_raw(), user_data);
    });
}

// FFI function to provide image recognition
#[no_mangle]
pub extern "C" fn recognize_image(
    api_key: *const c_char,
    image_path: *const c_char,
    callback: extern "C" fn(*const c_char, *mut c_void),
    user_data: *mut c_void,
) {
    // This would implement image recognition functionality
    // For demonstration, returning placeholder
    callback(
        CString::new("Image recognition not implemented yet")
            .unwrap()
            .into_raw(),
        user_data,
    );
}
