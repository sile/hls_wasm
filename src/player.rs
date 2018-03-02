use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::str;
use std::time::Duration;
use hls_m3u8::{MasterPlaylist, MediaPlaylist};
use mpeg2ts::ts::TsPacketReader;
use mse_fmp4::mpeg2_ts;
use mse_fmp4::io::WriteTo;
use url::Url;
use url_serde;

use {Error, ErrorKind, Result};

type ActionId = i32;
type SeqNo = u64;

#[derive(Debug)]
pub struct HlsPlayer {
    master_m3u8_url: String,
    inner: Option<HlsPlayerInner>,
}
impl HlsPlayer {
    pub fn new(master_m3u8_url: &str) -> Self {
        HlsPlayer {
            master_m3u8_url: master_m3u8_url.to_owned(),
            inner: None,
        }
    }
    pub fn play(&mut self, master_m3u8: &str) -> Result<()> {
        let inner = track!(HlsPlayerInner::new(&self.master_m3u8_url, master_m3u8))?;
        self.inner = Some(inner);
        Ok(())
    }
    pub fn next_action(&mut self) -> Option<HlsAction> {
        if let Some(ref mut inner) = self.inner {
            inner.next_action()
        } else {
            None
        }
    }
    pub fn next_segment(&mut self) -> Option<Vec<u8>> {
        self.inner.as_mut().and_then(|inner| inner.next_segment())
    }
    pub fn handle_fetched_bytes(&mut self, action_id: i32, bytes: &[u8]) -> Result<()> {
        let inner = track_assert_some!(self.inner.as_mut(), ErrorKind::Other);
        track!(inner.handle_fetched_bytes(action_id, bytes))
    }
}

#[derive(Debug)]
struct HlsPlayerInner {
    master_m3u8_url: Url,
    master_playlist: MasterPlaylist,
    media_m3u8_url: Url,
    media_playlist: Option<MediaPlaylist>,
    next_action_id: ActionId,
    actions: VecDeque<HlsAction>,
    fetchings: HashMap<ActionId, Fetching>,
    segments: BinaryHeap<Reverse<(SeqNo, Vec<u8>)>>,
    is_init: bool,
}
impl HlsPlayerInner {
    fn new(master_m3u8_url: &str, master_m3u8: &str) -> Result<Self> {
        let master_m3u8_url = track!(Url::parse(master_m3u8_url).map_err(Error::from))?;
        let master_playlist: MasterPlaylist = track!(master_m3u8.parse().map_err(Error::from))?;
        track_assert!(
            !master_playlist.stream_inf_tags().is_empty(),
            ErrorKind::InvalidInput
        );
        let media_m3u8_url = {
            let media_m3u8_url = master_playlist.stream_inf_tags()[0].uri();
            track!(
                Url::options()
                    .base_url(Some(&master_m3u8_url))
                    .parse(media_m3u8_url)
                    .map_err(Error::from)
            )?
        };
        let mut actions = VecDeque::new();
        actions.push_back(HlsAction::Fetch {
            action_id: 0,
            url: media_m3u8_url.clone(),
        });
        Ok(HlsPlayerInner {
            master_m3u8_url,
            master_playlist,
            media_m3u8_url,
            media_playlist: None,
            next_action_id: 1,
            actions,
            fetchings: HashMap::new(),
            segments: BinaryHeap::new(),
            is_init: false,
        })
    }
    fn next_action(&mut self) -> Option<HlsAction> {
        self.actions.pop_front()
    }
    fn handle_fetched_bytes(&mut self, action_id: i32, bytes: &[u8]) -> Result<()> {
        if action_id == 0 {
            let s = track!(str::from_utf8(bytes).map_err(Error::from))?;
            self.media_playlist = Some(track!(s.parse().map_err(Error::from))?);

            let mut seq_no = self.media_playlist()
                .media_sequence_tag()
                .map_or(0, |t| t.seq_num());
            for segment in self.media_playlist().clone().segments().iter().take(3) {
                // TODO: remove clone
                let action_id = self.next_action_id();
                let url = track!(
                    Url::options()
                        .base_url(Some(&self.media_m3u8_url))
                        .parse(segment.uri())
                        .map_err(Error::from)
                )?;
                self.actions.push_back(HlsAction::Fetch { action_id, url });
                self.fetchings.insert(
                    action_id,
                    Fetching::MediaSegment(MediaSegmentHeader { seq_no }),
                );
                seq_no += 1;
            }
            let action_id = self.next_action_id();
            let duration = self.media_playlist().target_duration_tag().duration();
            self.actions.push_back(HlsAction::SetTimeout {
                action_id,
                duration,
            });
        } else {
            let fetching =
                track_assert_some!(self.fetchings.remove(&action_id), ErrorKind::InvalidInput);
            match fetching {
                Fetching::MediaSegment(h) => track!(self.handle_media_segment(h, bytes))?,
            }
        }
        Ok(())
    }
    fn handle_media_segment(
        &mut self,
        header: MediaSegmentHeader,
        ts_segment: &[u8],
    ) -> Result<()> {
        let fmp4_segments = track!(mpeg2_ts::to_fmp4(TsPacketReader::new(ts_segment)))?;

        let mut initialization_segment = Vec::new();
        let mut media_segment = Vec::new();
        track!(fmp4_segments.0.write_to(&mut initialization_segment))?;
        track!(fmp4_segments.1.write_to(&mut media_segment))?;

        if !self.is_init {
            self.segments
                .push(Reverse((header.seq_no * 2, initialization_segment)));
            self.is_init = true;
        }
        self.segments
            .push(Reverse((header.seq_no * 2 + 1, media_segment)));
        Ok(())
    }
    fn next_action_id(&mut self) -> i32 {
        let id = self.next_action_id;
        self.next_action_id += 1;
        id
    }
    fn next_segment(&mut self) -> Option<Vec<u8>> {
        self.segments.pop().map(|x| (x.0).1)
    }
    fn media_playlist(&self) -> &MediaPlaylist {
        self.media_playlist.as_ref().expect("Never fails")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HlsAction {
    Fetch {
        action_id: i32,
        #[serde(with = "url_serde")] url: Url,
    },
    SetTimeout {
        action_id: i32,
        duration: Duration,
    },
}

#[derive(Debug)]
enum Fetching {
    MediaSegment(MediaSegmentHeader),
}

#[derive(Debug)]
struct MediaSegmentHeader {
    seq_no: u64,
}
