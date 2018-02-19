extern crate hls_m3u8;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate trackable;
extern crate url;
extern crate url_serde;

pub use error::{Error, ErrorKind};
pub use player::{HlsAction, HlsPlayer};

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

pub mod player;
pub mod wasm_api;

mod error;

pub type MaybeError = WasmStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct MaybeJson<T> {
    json_str: WasmStr,
    _phantom: PhantomData<T>,
}
impl<T> MaybeJson<T>
where
    T: serde::Serialize,
{
    pub fn new(t: &T) -> Self {
        let s = serde_json::ser::to_string(t).expect("TODO");
        MaybeJson {
            json_str: WasmStr::from(s),
            _phantom: PhantomData,
        }
    }
    pub fn null() -> Self {
        MaybeJson {
            json_str: WasmStr(Ptr::null()),
            _phantom: PhantomData,
        }
    }
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

#[derive(Debug)]
pub struct WasmBytes(Ptr<Vec<u8>>);
impl WasmBytes {
    pub fn new(size: usize) -> Self {
        WasmBytes(Ptr::new(vec![0; size]))
    }

    pub fn as_ptr(&self) -> i32 {
        self.deref().as_ptr() as i32
    }

    pub unsafe fn free(&mut self) {
        self.0.free();
    }
}
impl Deref for WasmBytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl From<Vec<u8>> for WasmBytes {
    fn from(f: Vec<u8>) -> Self {
        WasmBytes(Ptr::new(f))
    }
}
