extern crate hls_m3u8;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use player::HlsPlayer;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

macro_rules! maybe_error {
    ($expr:expr) => {
        if let Err(e) = $expr {
            return WasmStr::from(e.to_json_string());
        }
    }
}
macro_rules! ok {
    () => { WasmStr(Ptr::null()) }
}

pub mod wasm_api;

mod error;
mod player;

pub type MaybeError = WasmStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Action {
    Fetch,
    Play,
    Wait,
}

#[no_mangle]
pub fn wasm_str_new(size: i32) -> WasmStr {
    assert!(size >= 0);
    WasmStr::new(size as usize)
}

#[no_mangle]
pub fn wasm_str_free(mut s: WasmStr) {
    unsafe {
        s.free();
    }
}

#[no_mangle]
pub fn wasm_str_ptr(s: WasmStr) -> i32 {
    s.as_ptr()
}

#[no_mangle]
pub fn wasm_str_len(s: WasmStr) -> i32 {
    s.len() as i32
}

#[no_mangle]
pub fn wasm_str_get(s: WasmStr, index: i32) -> i32 {
    s.as_bytes()[index as usize] as i32
}

#[derive(Debug)]
pub struct Ptr<T> {
    ptr: i32,
    _phantom: PhantomData<T>,
}
impl<T> Ptr<T> {
    pub fn new(t: T) -> Self {
        let ptr = Box::into_raw(Box::new(t)) as i32;
        Ptr {
            ptr,
            _phantom: PhantomData,
        }
    }
    pub fn null() -> Self {
        Ptr {
            ptr: 0,
            _phantom: PhantomData,
        }
    }
    pub unsafe fn free(&mut self) {
        if self.ptr != 0 {
            let _ = Box::from_raw(self.ptr as *mut T);
        }
    }
}
impl<T> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *const T) }
    }
}
impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

#[derive(Debug)]
pub struct WasmStr(Ptr<String>);
impl WasmStr {
    pub fn new(size: usize) -> Self {
        let s = unsafe { String::from_utf8_unchecked(vec![0; size]) };
        WasmStr(Ptr::new(s))
    }

    pub fn as_ptr(&self) -> i32 {
        self.deref().as_ptr() as i32
    }

    pub unsafe fn free(&mut self) {
        self.0.free();
    }
}
impl Deref for WasmStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl From<String> for WasmStr {
    fn from(f: String) -> Self {
        WasmStr(Ptr::new(f))
    }
}
