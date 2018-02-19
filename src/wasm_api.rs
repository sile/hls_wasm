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
pub mod wasm_bytes {
    use WasmBytes;

    #[no_mangle]
    pub fn wasm_bytes_new(size: i32) -> WasmBytes {
        assert!(size >= 0);
        WasmBytes::new(size as usize)
    }

    #[no_mangle]
    pub fn wasm_bytes_free(mut s: WasmBytes) {
        unsafe {
            s.free();
        }
    }

    #[no_mangle]
    pub fn wasm_bytes_ptr(s: WasmBytes) -> i32 {
        s.as_ptr()
    }

    #[no_mangle]
    pub fn wasm_bytes_len(s: WasmBytes) -> i32 {
        s.len() as i32
    }

}
pub mod hls_player {
    use {HlsPlayer, MaybeError, MaybeJson, Ptr, WasmBytes, WasmStr};
    use player::HlsAction;

    #[no_mangle]
    pub fn hls_player_new(master_m3u8_url: WasmStr) -> Ptr<HlsPlayer> {
        Ptr::new(HlsPlayer::new(&master_m3u8_url))
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

    #[no_mangle]
    pub fn hls_player_handle_fetched_bytes(
        mut player: Ptr<HlsPlayer>,
        action_id: i32,
        bytes: WasmBytes,
    ) -> MaybeError {
        maybe_error!(player.handle_fetched_bytes(action_id, &bytes));
        ok!()
    }

    #[no_mangle]
    pub fn hls_player_next_action(mut player: Ptr<HlsPlayer>) -> MaybeJson<HlsAction> {
        if let Some(action) = player.next_action() {
            MaybeJson::new(&action)
        } else {
            MaybeJson::null()
        }
    }
}
