use std::time::Duration;
use url::Url;
use url_serde;

use super::StreamId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    FetchData {
        action_id: ActionId,
        #[serde(with = "url_serde")] url: Url,
    },
    SetTimeout {
        action_id: ActionId,
        duration: Duration,
    },
}
impl Action {
    pub fn id(&self) -> ActionId {
        match *self {
            Action::FetchData { action_id, .. } => action_id,
            Action::SetTimeout { action_id, .. } => action_id,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActionId(u64);
impl ActionId {
    pub fn media_playlist_id(&self) -> StreamId {
        (self.0 >> 48) as u16
    }

    fn next(&mut self) -> ActionId {
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

#[derive(Debug)]
pub struct ActionFactory {
    next_action_id: ActionId,
}
impl ActionFactory {
    pub fn new(stream_id: StreamId) -> Self {
        ActionFactory {
            next_action_id: ActionId(u64::from(stream_id) << 48),
        }
    }

    pub fn media_playlist_id(&self) -> StreamId {
        self.next_action_id.media_playlist_id()
    }

    pub fn fetch_data(&mut self, url: Url) -> Action {
        let action_id = self.next_action_id.next();
        Action::FetchData { action_id, url }
    }

    pub fn set_timeout(&mut self, duration: Duration) -> Action {
        let action_id = self.next_action_id.next();
        Action::SetTimeout {
            action_id,
            duration,
        }
    }
}
