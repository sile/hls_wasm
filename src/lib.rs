extern crate hls_m3u8;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[no_mangle]
pub fn hls_player_new() -> Ptr<HlsPlayer> {
    Ptr::new(HlsPlayer { foo: 0, bar: 2 })
}

#[no_mangle]
pub fn hls_player_free(mut player: Ptr<HlsPlayer>) {
    unsafe {
        player.free();
    }
}

#[no_mangle]
pub fn hls_player_play_master_m3u8(player: Ptr<HlsPlayer>, master_m3u8: WasmStr) {}

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
    pub unsafe fn free(&mut self) {
        let _ = Box::from_raw(self.ptr as *mut T);
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

#[derive(Debug)]
pub struct HlsPlayer {
    foo: usize,
    bar: usize,
}

// #[repr(C)]
// #[derive(Debug)]
// pub struct WasmStr(Memory);

// #[repr(C)]
// pub struct Memory {
//     pub ptr: i32,
//     pub len: i32,
// }
// impl Memory {
//     pub fn allocate(size: i32) -> i32 {
//         let mem = vec![0; size as usize];
//         mem.as_ptr() as i32
//     }

//     pub fn release(&self) {
//         let ptr = self.ptr as usize;
//         let len = self.len as usize;
//         unsafe {
//             Vec::from_raw_parts(ptr as *mut u8, len, len);
//         }
//     }
// }

// #[no_mangle]
// pub fn mem_alloc(size: i32) -> i32 {
//     Memory::allocate(size)
// }

// #[no_mangle]
// pub fn mem_free(mem: Memory) {
//     mem.release();
// }
