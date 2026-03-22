use std::rc::Rc;

use midi_file::{MidiNote, MidiTrack};

/// Track configuration for filtering notes
#[derive(Clone, Debug)]
pub struct TrackChannelConfig {
    pub track_id: usize,
    pub hidden_channels: Vec<u8>,
}

impl TrackChannelConfig {
    pub fn is_channel_hidden(&self, channel: u8) -> bool {
        self.hidden_channels.contains(&channel)
    }
}

/// List of MIDI notes for rendering
#[derive(Clone)]
pub struct NoteList {
    pub inner: Rc<[MidiNote]>,
}

impl NoteList {
    pub fn empty() -> Self {
        Self { inner: Rc::new([]) }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn inner(&self) -> &[MidiNote] {
        &self.inner
    }

    pub fn new(
        tracks: &[MidiTrack],
        hidden_tracks: &[usize],
        track_channel_configs: &[TrackChannelConfig],
    ) -> Self {
        let mut notes: Vec<_> = tracks
            .iter()
            .filter(|track| !hidden_tracks.contains(&track.track_id))
            .flat_map(|track| {
                // Get channel config for this track
                let track_config = track_channel_configs
                    .iter()
                    .find(|tc| tc.track_id == track.track_id);

                track.notes.iter().cloned().filter(move |note| {
                    // Filter by channel if config exists for this track
                    if let Some(config) = track_config {
                        !config.is_channel_hidden(note.channel)
                    } else {
                        true
                    }
                })
            })
            .collect();

        // We want to render newer notes on top of old notes
        notes.sort_unstable_by_key(|note| note.start);

        Self {
            inner: notes.into(),
        }
    }
}
