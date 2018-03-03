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
    use url::Url;

    use {Error, HlsPlayer, MaybeError, MaybeJson, Ptr, WasmBytes, WasmStr};
    use player::{Action, ActionId};

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
    pub fn hls_player_play(player: Ptr<HlsPlayer>, url: WasmStr, m3u8: WasmStr) -> MaybeError {
        if m3u8.contains("#EXT-X-TARGETDURATION") {
            hls_player_play_media_playlist(player, url, m3u8)
        } else {
            hls_player_play_master_playlist(player, url, m3u8)
        }
    }

    #[no_mangle]
    pub fn hls_player_play_master_playlist(
        mut player: Ptr<HlsPlayer>,
        master_playlist_url: WasmStr,
        m3u8: WasmStr,
    ) -> MaybeError {
        let url = maybe_error!(Url::parse(&master_playlist_url).map_err(Error::from));
        maybe_error!(player.play_master_playlist(url, &m3u8));
        ok!()
    }

    #[no_mangle]
    pub fn hls_player_play_media_playlist(
        mut player: Ptr<HlsPlayer>,
        media_playlist_url: WasmStr,
        m3u8: WasmStr,
    ) -> MaybeError {
        let url = maybe_error!(Url::parse(&media_playlist_url).map_err(Error::from));
        maybe_error!(player.play_media_playlist(url, &m3u8));
        ok!()
    }

    #[no_mangle]
    pub fn hls_player_handle_data(
        mut player: Ptr<HlsPlayer>,
        action_id: u32,
        data: WasmBytes,
    ) -> MaybeError {
        let action_id = ActionId::from(action_id);
        maybe_error!(player.handle_data(action_id, &data));
        ok!()
    }

    #[no_mangle]
    pub fn hls_player_handle_timeout(mut player: Ptr<HlsPlayer>, action_id: u32) -> MaybeError {
        let action_id = ActionId::from(action_id);
        maybe_error!(player.handle_timeout(action_id));
        ok!()
    }

    #[no_mangle]
    pub fn hls_player_next_action(mut player: Ptr<HlsPlayer>) -> MaybeJson<Action> {
        if let Some(action) = player.next_action() {
            MaybeJson::new(&action)
        } else {
            MaybeJson::null()
        }
    }

    #[no_mangle]
    pub fn hls_player_next_segment(mut player: Ptr<HlsPlayer>) -> WasmBytes {
        if let Some(segment) = player.next_segment() {
            WasmBytes::from(segment)
        } else {
            WasmBytes::null()
        }
    }
}
