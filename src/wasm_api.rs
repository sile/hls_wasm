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
