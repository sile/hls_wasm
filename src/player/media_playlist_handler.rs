use std::cmp;
use std::collections::VecDeque;
use hls_m3u8::MediaPlaylist;
use mpeg2ts::ts::TsPacketReader;
use mse_fmp4::mpeg2_ts;
use mse_fmp4::io::WriteTo;
use url::Url;

use {Error, Result};
use super::{Action, ActionFactory, ActionId};

type SequenceNumber = u64;

#[derive(Debug)]
pub struct MediaPlaylistHandler {
    media_playlist_url: Url,
    action_factory: ActionFactory,
    action_queue: VecDeque<Action>,
    segment_queue: VecDeque<(SequenceNumber, bool, Url)>,
    buffered_segments: VecDeque<Vec<u8>>,
    last_media_sequence: SequenceNumber,
    is_initialized: bool,
    fetch_playlist_action_id: ActionId,
}
impl MediaPlaylistHandler {
    pub fn new(mut action_factory: ActionFactory, media_playlist_url: Url) -> Self {
        let mut action_queue = VecDeque::new();
        let action = action_factory.fetch_data(media_playlist_url.clone());
        let action_id = action.id();
        action_queue.push_back(action);
        MediaPlaylistHandler {
            media_playlist_url,
            action_factory,
            action_queue,
            segment_queue: VecDeque::new(),
            buffered_segments: VecDeque::new(),
            last_media_sequence: 0,
            is_initialized: false,
            fetch_playlist_action_id: action_id,
        }
    }

    pub fn with_m3u8(
        action_factory: ActionFactory,
        media_playlist_url: Url,
        m3u8: &str,
    ) -> Result<Self> {
        let mut this = Self::new(action_factory, media_playlist_url);
        let _ = this.next_action();
        track!(this.handle_playlist(m3u8))?;
        Ok(this)
    }

    pub fn next_action(&mut self) -> Option<Action> {
        self.action_queue.pop_front()
    }

    pub fn next_segment(&mut self) -> Option<Vec<u8>> {
        self.buffered_segments.pop_front()
    }

    pub fn handle_timeout(&mut self, _action_id: ActionId) -> Result<()> {
        let action = self.action_factory
            .fetch_data(self.media_playlist_url.clone());
        let action_id = action.id();
        self.action_queue.push_back(action);
        self.fetch_playlist_action_id = action_id;
        Ok(())
    }

    pub fn handle_data(&mut self, action_id: ActionId, data: &[u8]) -> Result<()> {
        if action_id == self.fetch_playlist_action_id {
            use std::str;

            let m3u8 = track!(str::from_utf8(data).map_err(Error::from))?;
            track!(self.handle_playlist(m3u8))?;
        } else {
            track!(self.handle_segment(data))?;
        }
        Ok(())
    }

    fn handle_playlist(&mut self, m3u8: &str) -> Result<()> {
        let playlist: MediaPlaylist = track!(m3u8.parse())?;
        let media_sequence = playlist.media_sequence_tag().map_or(0, |t| t.seq_num());
        while self.segment_queue
            .front()
            .map_or(false, |x| x.0 < media_sequence)
        {
            self.segment_queue.pop_front();
        }

        let mut polling_interval = playlist.target_duration_tag().duration();
        for (i, segment) in playlist.segments().iter().enumerate() {
            let seq = media_sequence + i as u64;
            if seq <= self.last_media_sequence {
                continue;
            }
            self.last_media_sequence = seq;

            let segment_url = track!(self.parse_segment_url(segment.uri()))?;
            let ongoing = if self.segment_queue.is_empty() {
                self.action_queue
                    .push_back(self.action_factory.fetch_data(segment_url.clone()));
                true
            } else {
                false
            };
            self.segment_queue.push_back((seq, ongoing, segment_url));
            polling_interval = cmp::min(polling_interval, segment.inf_tag().duration());
        }

        self.action_queue
            .push_back(self.action_factory.set_timeout(polling_interval));
        Ok(())
    }

    fn handle_segment(&mut self, ts_segment: &[u8]) -> Result<()> {
        if let Some(x) = self.segment_queue.pop_front() {
            if !x.1 {
                self.segment_queue.push_front(x);
            }
        }
        if let Some((_, _, url)) = self.segment_queue.pop_front() {
            self.action_queue
                .push_back(self.action_factory.fetch_data(url));
        }

        let fmp4_segments = track!(mpeg2_ts::to_fmp4(TsPacketReader::new(ts_segment)))?;

        if !self.is_initialized {
            let mut initialization_segment = Vec::new();
            track!(fmp4_segments.0.write_to(&mut initialization_segment))?;
            self.buffered_segments.push_back(initialization_segment);

            self.is_initialized = true;
        }

        let mut media_segment = Vec::new();
        track!(fmp4_segments.1.write_to(&mut media_segment))?;
        self.buffered_segments.push_back(media_segment);

        Ok(())
    }

    fn parse_segment_url(&self, segment_url: &str) -> Result<Url> {
        track!(
            Url::options()
                .base_url(Some(&self.media_playlist_url))
                .parse(segment_url)
                .map_err(Error::from)
        )
    }
}
