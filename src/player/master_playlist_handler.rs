use hls_m3u8::MasterPlaylist;
use url::Url;

use {Error, ErrorKind, Result};
use super::{Action, ActionFactory, ActionId, MediaPlaylistHandler};

#[derive(Debug)]
pub struct MasterPlaylistHandler {
    media_playlist_handler: MediaPlaylistHandler,
}
impl MasterPlaylistHandler {
    pub fn new(url: Url, m3u8: &str) -> Result<Self> {
        let master_playlist: MasterPlaylist = track!(m3u8.parse())?;

        let stream_inf_tag = track_assert_some!(
            master_playlist.stream_inf_tags().get(0),
            ErrorKind::InvalidInput
        );
        let media_playlist_url = track!(
            Url::options()
                .base_url(Some(&url))
                .parse(stream_inf_tag.uri())
                .map_err(Error::from)
        )?;

        let action_factory = ActionFactory::new(0);
        let media_playlist_handler = MediaPlaylistHandler::new(action_factory, media_playlist_url);
        Ok(MasterPlaylistHandler {
            media_playlist_handler,
        })
    }

    pub fn next_action(&mut self) -> Option<Action> {
        self.media_playlist_handler.next_action()
    }

    pub fn next_segment(&mut self) -> Option<Vec<u8>> {
        self.media_playlist_handler.next_segment()
    }

    pub fn handle_data(
        &mut self,
        action_id: ActionId,
        data: &[u8],
        fetch_duration_ms: u32,
    ) -> Result<()> {
        track!(
            self.media_playlist_handler
                .handle_data(action_id, data, fetch_duration_ms)
        )
    }

    pub fn handle_timeout(&mut self, action_id: ActionId) -> Result<()> {
        track!(self.media_playlist_handler.handle_timeout(action_id))
    }
}
