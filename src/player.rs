use std::time::Duration;
use hls_m3u8::MasterPlaylist;
use url::Url;

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
    pub fn take_actions(&mut self) -> Vec<HlsAction> {
        Vec::new()
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

#[derive(Debug)]
pub enum HlsAction {
    Fetch(Url),
    // Play(MediaSegment),
    SetTimeout(Duration),
}
