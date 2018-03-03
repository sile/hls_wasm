use url::Url;

pub use self::action::{Action, ActionFactory, ActionId};
pub use self::master_playlist_handler::MasterPlaylistHandler;
pub use self::media_playlist_handler::MediaPlaylistHandler;

mod action;
mod master_playlist_handler;
mod media_playlist_handler;

use Result;

pub type StreamId = u16;

#[derive(Debug)]
pub enum HlsPlayer {
    NotStarted,
    MasterPlaylist(MasterPlaylistHandler),
    MediaPlayilst(MediaPlaylistHandler),
}
impl HlsPlayer {
    pub fn new() -> Self {
        HlsPlayer::NotStarted
    }

    pub fn play_master_playlist(&mut self, url: Url, m3u8: &str) -> Result<()> {
        let handler = track!(MasterPlaylistHandler::new(url, m3u8))?;
        *self = HlsPlayer::MasterPlaylist(handler);
        Ok(())
    }

    pub fn play_media_playlist(&mut self, url: Url, m3u8: &str) -> Result<()> {
        let action_factory = ActionFactory::new(0);
        let handler = track!(MediaPlaylistHandler::with_m3u8(action_factory, url, m3u8))?;
        *self = HlsPlayer::MediaPlayilst(handler);
        Ok(())
    }

    pub fn next_action(&mut self) -> Option<Action> {
        match *self {
            HlsPlayer::NotStarted => None,
            HlsPlayer::MasterPlaylist(ref mut x) => x.next_action(),
            HlsPlayer::MediaPlayilst(ref mut x) => x.next_action(),
        }
    }

    pub fn next_segment(&mut self) -> Option<Vec<u8>> {
        match *self {
            HlsPlayer::NotStarted => None,
            HlsPlayer::MasterPlaylist(ref mut x) => x.next_segment(),
            HlsPlayer::MediaPlayilst(ref mut x) => x.next_segment(),
        }
    }

    pub fn handle_data(&mut self, action_id: ActionId, data: &[u8]) -> Result<()> {
        match *self {
            HlsPlayer::NotStarted => Ok(()),
            HlsPlayer::MasterPlaylist(ref mut x) => track!(x.handle_data(action_id, data)),
            HlsPlayer::MediaPlayilst(ref mut x) => track!(x.handle_data(action_id, data)),
        }
    }

    pub fn handle_timeout(&mut self, action_id: ActionId) -> Result<()> {
        match *self {
            HlsPlayer::NotStarted => Ok(()),
            HlsPlayer::MasterPlaylist(ref mut x) => track!(x.handle_timeout(action_id)),
            HlsPlayer::MediaPlayilst(ref mut x) => track!(x.handle_timeout(action_id)),
        }
    }
}
