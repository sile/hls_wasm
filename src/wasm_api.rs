pub mod wasm_str {
    use WasmStr;

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

}
pub mod hls_player {
    use {HlsPlayer, MaybeError, Ptr, WasmStr};

    #[no_mangle]
    pub fn hls_player_new() -> Ptr<HlsPlayer> {
        Ptr::new(HlsPlayer::new())
    }

    #[no_mangle]
    pub fn hls_player_free(mut player: Ptr<HlsPlayer>) {
        unsafe {
            player.free();
        }
    }

    #[no_mangle]
    pub fn hls_player_play(mut player: Ptr<HlsPlayer>, master_m3u8: WasmStr) -> MaybeError {
        maybe_error!(player.play(&master_m3u8));
        ok!()
    }

    // #[no_mangle]
    // pub fn hls_player_poll(player: Ptr<HlsPlayer>) -> Ptr<Action> {
    //     panic!();
    // }
}
