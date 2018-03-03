use std::str;
use std::time::Duration;
use url::Url;
use url_serde;

pub use self::master_playlist_handler::MasterPlaylistHandler;
pub use self::media_playlist_handler::MediaPlaylistHandler;

mod master_playlist_handler;
mod media_playlist_handler;

use {ErrorKind, Result};

#[derive(Debug)]
pub struct HlsPlayer {
    master_playlist_handler: Option<MasterPlaylistHandler>,
    media_playlist_handlers: Vec<MediaPlaylistHandler>,
}
impl HlsPlayer {
    pub fn new() -> Self {
        HlsPlayer {
            master_playlist_handler: None,
            media_playlist_handlers: Vec::new(),
        }
    }

    pub fn play_master_playlist(&mut self, url: Url) -> Result<()> {
        track_assert!(
            self.master_playlist_handler.is_none(),
            ErrorKind::InvalidInput
        );
        track_assert!(
            self.media_playlist_handlers.is_empty(),
            ErrorKind::InvalidInput
        );

        self.master_playlist_handler = Some(MasterPlaylistHandler::new(url));
        Ok(())
    }

    pub fn play_media_playlist(&mut self, url: Url) -> Result<()> {
        track_assert!(
            self.master_playlist_handler.is_none(),
            ErrorKind::InvalidInput
        );
        track_assert!(
            self.media_playlist_handlers.is_empty(),
            ErrorKind::InvalidInput
        );

        let action_id = ActionId::default().clone_for_new_playlist();
        self.media_playlist_handlers
            .push(MediaPlaylistHandler::new(action_id, url));
        Ok(())
    }

    pub fn next_action(&mut self) -> Option<HlsAction> {
        if let Some(action) = self.master_playlist_handler
            .as_mut()
            .and_then(|x| x.next_action())
        {
            Some(action)
        } else {
            for handler in &mut self.media_playlist_handlers {
                if let Some(action) = handler.next_action() {
                    return Some(action);
                }
            }
            None
        }
    }
    pub fn next_segment(&mut self) -> Option<Vec<u8>> {
        panic!()
    }
    pub fn handle_data(&mut self, action_id: ActionId, data: &[u8]) -> Result<()> {
        panic!()
    }
    pub fn handle_timeout(&mut self, action_id: ActionId) -> Result<()> {
        panic!()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActionId(u64);
impl ActionId {
    pub fn clone_for_new_playlist(&self) -> ActionId {
        let playlist_seqno = (self.0 >> 32) + 1;
        ActionId(playlist_seqno << 32)
    }

    pub fn next(&mut self) -> ActionId {
        let id = self.clone();
        self.0 += 1;
        id
    }
}
impl From<u64> for ActionId {
    fn from(f: u64) -> Self {
        ActionId(f)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HlsAction {
    FetchPlaylist {
        action_id: ActionId,
        #[serde(with = "url_serde")] url: Url,
    },
    FetchSegment {
        action_id: ActionId,
        #[serde(with = "url_serde")] url: Url,
    },
    Fetch {
        action_id: ActionId,
        #[serde(with = "url_serde")] url: Url,
    },
    SetTimeout {
        action_id: ActionId,
        duration: Duration,
    },
}
