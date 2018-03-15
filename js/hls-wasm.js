function fetchAndInstantiate(url, importObject) {
    return fetch(url)
        .then(response => response.arrayBuffer())
        .then(bytes => WebAssembly.instantiate(bytes, importObject))
        .then(results => results.instance);
}

class HlsPlayer {
    constructor(hls_wasm) {
        this.hls_wasm = hls_wasm;
        this.api = hls_wasm.exports;
        this.player = this.api.hls_player_new();

        let media_source = new MediaSource();
        media_source.addEventListener('sourceopen', () => {
            console.log("[DEBUG] Event.sourceopen");
            const mimeCodec = 'video/mp4; codecs="avc1.4dc00d,mp4a.40.2"'; // TODO
            this.sb = media_source.addSourceBuffer(mimeCodec);
            this.sb.mode = 'sequence';
        }, false);
        this.media_source = media_source;

        this.video = document.getElementsByTagName('video')[0];
        this.video.src = URL.createObjectURL(media_source);
        this.video.play();
    }

    play(m3u8_url) {
        fetch(m3u8_url)
            .then(response => response.arrayBuffer())
            .then(m3u8 => {
                let error =
                    this.with_wasm_str((new TextEncoder).encode(m3u8_url), url => {
                        return this.with_wasm_str(new Uint8Array(m3u8), m3u8 => {
                            console.log("[DEBUG] Starts playing m3u8");
                            return this.api.hls_player_play(this.player, url, m3u8);
                        })
                    });
                if (error != 0) {
                    let json = this.wasm_str_into_json(error);
                    console.log(json);
                    return Promise.reject(JSON.stringify(json));
                }

                this.poll();
            })
            .catch(error => alert(`Cannot fetch ${m3u8_url}\n\n[Reason]\n${error}`))
    }

    fetch_url(action_id, url) {
        console.log(`[DEBUG] Starts fetching url: [${action_id}] ${url}`);
        const start_time = new Date();
        fetch(url)
            .then(response => response.arrayBuffer())
            .then(body => {
                const end_time = new Date();
                const fetch_duration_ms = end_time - start_time;
                let error = this.with_wasm_bytes(new Uint8Array(body), bytes => {
                    return this.api.hls_player_handle_data(this.player, action_id, bytes, fetch_duration_ms);
                });
                if (error != 0) {
                    let json = this.wasm_str_into_json(error);
                    console.log(json);
                    return Promise.reject(JSON.stringify(json));
                }
                console.log(`[DEBUG] Handled: [${action_id}] ${url} (delay:${fetch_duration_ms})`);
                this.poll();
            })
            .catch(error => alert(`Cannot fetch url: ${url}: ${error}`))
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

    with_wasm_bytes(src, callback) {
        let wasm_buf = this.api.wasm_bytes_new(src.length);
        let wasm_ptr = this.api.wasm_bytes_ptr(wasm_buf);
        let buf = new Uint8Array(this.api.memory.buffer, wasm_ptr, src.length);
        buf.set(src);
        let result = callback(wasm_buf);
        this.api.wasm_bytes_free(wasm_buf);
        return result;
    }

    wasm_str_into_json(s) {
        let buf = new Uint8Array(this.api.memory.buffer, this.api.wasm_str_ptr(s), this.api.wasm_str_len(s));
        let json = JSON.parse(new TextDecoder("utf-8").decode(buf));
        this.api.wasm_str_free(s);
        return json;
    }
    wasm_bytes_into_uint8array(b) {
        let array = new Uint8Array(this.api.memory.buffer, this.api.wasm_bytes_ptr(b), this.api.wasm_bytes_len(b));
        this.api.wasm_bytes_free(b);
        return array;
    }
    poll_segment() {
        if (this.sb.updating) {
            return;
        }

        let wasm_bytes = this.api.hls_player_next_segment(this.player);
        if (wasm_bytes == 0) {
            return;
        }

        let segment =
            new Uint8Array(this.api.memory.buffer,
                           this.api.wasm_bytes_ptr(wasm_bytes),
                           this.api.wasm_bytes_len(wasm_bytes));
        console.log(`[DEBUG] segment: ${segment.length} bytes (ptr:${this.api.wasm_bytes_ptr(wasm_bytes)})`);
        this.sb.appendBuffer(segment);
        this.api.wasm_bytes_free(wasm_bytes);

        this.sb.addEventListener('updateend', () => {
            this.poll_segment();
        });
    }
    poll() {
        this.poll_segment();

        while (true) {
            let json = this.api.hls_player_next_action(this.player);
            if (json == 0) {
                break;
            }

            let action = this.wasm_str_into_json(json);
            console.log(`[DEBUG] Next Action: ${JSON.stringify(action)}`);
            if (action["type"] == "FetchData") {
                this.fetch_url(action["action_id"], action["url"]);
            } else if (action["type"] == "SetTimeout") {
                setTimeout(() => {
                    let error = this.api.hls_player_handle_timeout(this.player, action["action_id"]);
                    if (error != 0) {
                        let json = this.wasm_str_into_json(error);
                        console.log(json);
                    };
                    this.poll();
                }, action["duration"]);
            } else {
                console.log("[WARN] Unknown action");
            }
        }
    }
}

var hls_wasm;
fetchAndInstantiate("../target/wasm32-unknown-unknown/release/hls_wasm.wasm", {})
    .then(instance => {
        hls_wasm = instance;
    });

var hls = new Vue({
    el: '#hls-play',
    data: {
        m3u8_url: "http://localhost:8080/hls/foo.m3u8"
    },
    methods: {
        hlsPlay: function () {
            let player = new HlsPlayer(hls_wasm);
            player.play(this.m3u8_url);
        }
    }
})
