function fetchAndInstantiate(url, importObject) {
    return fetch(url)
        .then(response => response.arrayBuffer())
        .then(bytes => WebAssembly.instantiate(bytes, importObject))
        .then(results => results.instance);
}

class HlsPlayer {
    constructor(hls_wasm, master_m3u8) {
        this.hls_wasm = hls_wasm;
        this.api = hls_wasm.exports;
        this.player = this.hls_wasm.exports.hls_player_new();
    }

    play(master_m3u8_url) {
        fetch(master_m3u8_url)
            .then(response => response.arrayBuffer())
            .then(m3u8 => {
                let actions = this.with_wasm_str(new Uint8Array(m3u8), wasm_m3u8 => {
                    console.log("[DEBUG] Starts playing master m3u8");
                    return this.api.hls_player_play_master_m3u8(this.player, wasm_m3u8);
                });
                console.log(actions);
            })
            .catch(error => alert(`Cannot fetch ${master_m3u8_url}\n\n[Reason]\n${error}`))

    }

    with_wasm_str(src_utf8, callback) {
        let wasm_buf = this.api.wasm_str_new(src_utf8.length);
        let wasm_ptr = this.api.wasm_str_ptr(wasm_buf);
        let buf = new Uint8Array(this.api.memory.buffer, wasm_ptr, src_utf8.length);
        buf.set(src_utf8);
        let result = callback(wasm_buf);
        this.api.wasm_str_free(wasm_buf);
        return result;
    }
}

var hls_wasm;
fetchAndInstantiate("../target/wasm32-unknown-unknown/debug/hls_wasm.wasm", {})
    .then(instance => {
        hls_wasm = instance;
    });

var hls = new Vue({
    el: '#hls-play',
    data: {
        master_m3u8_url: "https://bitdash-a.akamaihd.net/content/sintel/hls/playlist.m3u8"
    },
    methods: {
        hlsPlay: function () {
            let player = new HlsPlayer(hls_wasm);
            player.play(this.master_m3u8_url);
        }
    }
})
