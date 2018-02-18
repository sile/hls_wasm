use hls_m3u8::MasterPlaylist;

use {Error, Result};

#[derive(Debug)]
pub struct HlsPlayer {
    inner: Option<HlsPlayerInner>,
}
impl HlsPlayer {
    pub fn new() -> Self {
        HlsPlayer { inner: None }
    }
    pub fn play(&mut self, master_m3u8: &str) -> Result<()> {
        let inner = track!(HlsPlayerInner::new(master_m3u8))?;
        self.inner = Some(inner);
        Ok(())
    }
}

#[derive(Debug)]
struct HlsPlayerInner {
    master_playlist: MasterPlaylist,
}
impl HlsPlayerInner {
    fn new(master_m3u8: &str) -> Result<Self> {
        let master_playlist = track!(master_m3u8.parse().map_err(Error::from))?;
        Ok(HlsPlayerInner { master_playlist })
    }
}
