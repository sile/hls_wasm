use std::collections::VecDeque;
use hls_m3u8::MasterPlaylist;
use url::Url;

use {Error, ErrorKind, Result};
use super::{ActionId, HlsAction, MediaPlaylistHandler};

#[derive(Debug)]
pub struct MasterPlaylistHandler {
    master_playlist_url: Url,
    next_action_id: ActionId,
    action_queue: VecDeque<HlsAction>,
}
impl MasterPlaylistHandler {
    pub fn new(master_playlist_url: Url) -> Self {
        let mut next_action_id = ActionId::default();
        let mut action_queue = VecDeque::new();
        action_queue.push_back(HlsAction::FetchPlaylist {
            action_id: next_action_id.next(),
            url: master_playlist_url.clone(),
        });
        MasterPlaylistHandler {
            master_playlist_url,
            next_action_id,
            action_queue,
        }
    }
    pub fn next_action(&mut self) -> Option<HlsAction> {
        self.action_queue.pop_front()
    }
    pub fn handle_playlist(
        &mut self,
        _action_id: ActionId,
        m3u8: &str,
    ) -> Result<MediaPlaylistHandler> {
        let master_playlist: MasterPlaylist = track!(m3u8.parse())?;

        let stream_inf_tag = track_assert_some!(
            master_playlist.stream_inf_tags().get(0),
            ErrorKind::InvalidInput
        );
        let media_playlist_url = track!(self.parse_url(stream_inf_tag.uri()))?;

        Ok(MediaPlaylistHandler::new(
            self.next_action_id.clone_for_new_playlist(),
            media_playlist_url,
        ))
    }

    fn parse_url(&self, url: &str) -> Result<Url> {
        track!(
            Url::options()
                .base_url(Some(&self.master_playlist_url))
                .parse(url)
                .map_err(Error::from)
        )
    }
}
