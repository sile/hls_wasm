use std::cmp;
use std::collections::VecDeque;
use hls_m3u8::MediaPlaylist;
use mpeg2ts::ts::TsPacketReader;
use mse_fmp4::mpeg2_ts;
use mse_fmp4::io::WriteTo;
use url::Url;

use {Error, Result};
use super::{ActionId, HlsAction};

type SequenceNumber = u64;

#[derive(Debug)]
pub struct MediaPlaylistHandler {
    media_playlist_url: Url,
    next_action_id: ActionId,
    action_queue: VecDeque<HlsAction>,
    segment_queue: VecDeque<(SequenceNumber, ActionId, Url)>,
    last_media_sequence: SequenceNumber,
    is_initialized: bool,
}
impl MediaPlaylistHandler {
    pub fn new(mut next_action_id: ActionId, media_playlist_url: Url) -> Self {
        let mut action_queue = VecDeque::new();
        action_queue.push_back(HlsAction::FetchPlaylist {
            action_id: next_action_id.next(),
            url: media_playlist_url.clone(),
        });
        MediaPlaylistHandler {
            media_playlist_url,
            next_action_id,
            action_queue,
            segment_queue: VecDeque::new(),
            last_media_sequence: 0,
            is_initialized: false,
        }
    }
    pub fn next_action(&mut self) -> Option<HlsAction> {
        self.action_queue.pop_front()
    }
    pub fn handle_timeout(&mut self, _action_id: ActionId) -> Result<()> {
        self.action_queue.push_back(HlsAction::FetchPlaylist {
            action_id: self.next_action_id.next(),
            url: self.media_playlist_url.clone(),
        });
        Ok(())
    }
    pub fn handle_playlist(&mut self, _action_id: ActionId, m3u8: &str) -> Result<()> {
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
            let action_id = self.next_action_id.clone();
            if self.segment_queue.is_empty() {
                self.action_queue.push_back(HlsAction::FetchSegment {
                    action_id,
                    url: segment_url.clone(),
                });
            }
            self.segment_queue.push_back((seq, action_id, segment_url));
            polling_interval = cmp::min(polling_interval, segment.inf_tag().duration());
        }

        self.action_queue.push_back(HlsAction::SetTimeout {
            action_id: self.next_action_id.next(),
            duration: polling_interval,
        });
        Ok(())
    }
    pub fn handle_segment(
        &mut self,
        action_id: ActionId,
        ts_segment: &[u8],
    ) -> Result<(Option<Vec<u8>>, Vec<u8>)> {
        if let Some(x) = self.segment_queue.pop_front() {
            if x.1 != action_id {
                self.segment_queue.push_front(x);
            }
        }
        if let Some((_, action_id, url)) = self.segment_queue.pop_front() {
            self.action_queue
                .push_back(HlsAction::FetchSegment { action_id, url });
        }

        let fmp4_segments = track!(mpeg2_ts::to_fmp4(TsPacketReader::new(ts_segment)))?;
        let mut initialization_segment = Vec::new();
        let mut media_segment = Vec::new();
        track!(fmp4_segments.0.write_to(&mut initialization_segment))?;
        track!(fmp4_segments.1.write_to(&mut media_segment))?;

        if !self.is_initialized {
            self.is_initialized = true;
            Ok((Some(initialization_segment), media_segment))
        } else {
            Ok((None, media_segment))
        }
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
