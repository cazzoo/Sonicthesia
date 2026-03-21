//! PLY Audio Bridge for OutputManager
//!
//! This module provides a bridge between the existing OutputManager and the PLY audio system.
//! It allows the OutputManager to use PLY audio connections while maintaining backward compatibility.

use std::path::Path;

use midi_file::midly::{num::u4, MidiMessage};

use crate::output_manager::{OutputConnection, OutputDescriptor};
use crate::ply_integration::audio::{PlyAudioConnection, PlyAudioEvent, PlyAudioManager};

/// Extension trait for OutputManager to integrate with PLY audio
pub trait PlyAudioBridge {
    /// Create a PLY audio connection from an OutputConnection
    fn to_ply_connection(&self) -> Option<PlyAudioConnection>;

    /// Create a PLY audio manager from the current output manager state
    fn to_ply_manager(&self) -> PlyAudioManager;
}

impl PlyAudioBridge for OutputConnection {
    fn to_ply_connection(&self) -> Option<PlyAudioConnection> {
        match self {
            OutputConnection::Midi(conn) => {
                // Extract the underlying midi-io connection
                // We need to access the inner connection through the public API
                // Since MidiOutputConnection doesn't expose the inner connection directly,
                // we'll need to work with what's available

                // For now, we'll return None and handle this differently
                // The OutputManager will need to be updated to support PLY integration
                log::warn!("Direct MIDI to PLY connection conversion not yet implemented");
                None
            }
            #[cfg(feature = "synth")]
            OutputConnection::Synth(conn) => {
                // Extract the synth event sender
                // Similar to MIDI, we need access to the inner sender
                log::warn!("Direct Synth to PLY connection conversion not yet implemented");
                None
            }
            OutputConnection::DummyOutput => Some(PlyAudioConnection::dummy()),
        }
    }

    fn to_ply_manager(&self) -> PlyAudioManager {
        let manager = PlyAudioManager::new();

        // Try to convert the connection to a PLY connection
        if let Some(ply_conn) = self.to_ply_connection() {
            let mut manager_with_conn = manager;
            manager_with_conn.set_main_output(ply_conn);
            manager_with_conn
        } else {
            manager
        }
    }
}

/// Wrapper that integrates PLY audio with the existing OutputManager interface
pub struct PlyOutputWrapper {
    /// The PLY audio connection
    connection: PlyAudioConnection,
}

impl PlyOutputWrapper {
    /// Create a new wrapper from a PLY audio connection
    pub fn new(connection: PlyAudioConnection) -> Self {
        Self { connection }
    }

    /// Create a wrapper from a MIDI output connection
    pub fn from_midi_connection(conn: midi_io::MidiOutputConnection) -> Self {
        Self {
            connection: PlyAudioConnection::from_midi_connection(conn),
        }
    }

    /// Create a wrapper from a synth event sender
    #[cfg(feature = "synth")]
    pub fn from_synth_sender(
        sender: std::sync::mpsc::Sender<crate::output_manager::synth_backend::SynthEvent>,
    ) -> Self {
        Self {
            connection: PlyAudioConnection::from_synth_sender(sender),
        }
    }

    /// Create a dummy wrapper
    pub fn dummy() -> Self {
        Self {
            connection: PlyAudioConnection::dummy(),
        }
    }

    /// Send a MIDI event
    pub fn midi_event(&self, channel: u4, msg: MidiMessage) {
        self.connection.midi_event(channel, msg);
    }

    /// Send a SysEx message
    pub fn send_sysex(&self, message: &[u8]) {
        self.connection.send_sysex(message);
    }

    /// Set the audio gain
    pub fn set_gain(&self, gain: f32) {
        self.connection.set_gain(gain);
    }

    /// Stop all notes
    pub fn stop_all(&self) {
        self.connection.stop_all();
    }

    /// Get the underlying PLY audio connection
    pub fn into_inner(self) -> PlyAudioConnection {
        self.connection
    }
}

/// Helper functions for integrating PLY audio with Neothesia
pub struct PlyAudioIntegration;

impl PlyAudioIntegration {
    /// Create a PLY audio manager configured for Neothesia
    pub fn create_manager() -> PlyAudioManager {
        PlyAudioManager::new()
    }

    /// Map a Neothesia OutputDescriptor to a PLY audio event
    pub fn map_output_event(
        descriptor: &OutputDescriptor,
        event: PlyAudioEvent,
    ) -> Option<PlyAudioEvent> {
        // For now, just pass through the event
        // In the future, this could handle output-specific transformations
        Some(event)
    }

    /// Convert a Neothesia MIDI message to a PLY audio event
    pub fn midi_to_ply_event(channel: u4, msg: MidiMessage) -> PlyAudioEvent {
        match msg {
            MidiMessage::NoteOn { key, vel } => PlyAudioEvent::NoteOn {
                channel,
                key: key.as_int() as u8,
                velocity: vel.as_int() as u8,
            },
            MidiMessage::NoteOff { key, .. } => PlyAudioEvent::NoteOff {
                channel,
                key: key.as_int() as u8,
            },
            MidiMessage::Controller { controller, value } => PlyAudioEvent::ControlChange {
                channel,
                controller: controller.as_int() as u8,
                value: value.as_int() as u8,
            },
            MidiMessage::ProgramChange { program } => PlyAudioEvent::ProgramChange {
                channel,
                program: program.as_int() as u8,
            },
            MidiMessage::PitchBend { bend } => PlyAudioEvent::PitchBend {
                channel,
                value: bend.0.as_int(),
            },
            MidiMessage::ChannelAftertouch { vel } => PlyAudioEvent::ChannelAftertouch {
                channel,
                value: vel.as_int() as u8,
            },
            MidiMessage::Aftertouch { key, vel } => PlyAudioEvent::PolyphonicPressure {
                channel,
                key: key.as_int() as u8,
                value: vel.as_int() as u8,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_file::midly::num::{u4, u7};

    #[test]
    fn test_dummy_wrapper() {
        let wrapper = PlyOutputWrapper::dummy();

        // Should not panic on any operations
        wrapper.midi_event(
            u4::new(0),
            MidiMessage::NoteOn {
                key: u7::new(60),
                vel: u7::new(100),
            },
        );
        wrapper.send_sysex(&[0xF0, 0x01, 0xF7]);
        wrapper.set_gain(0.8);
        wrapper.stop_all();
    }

    #[test]
    fn test_midi_to_ply_event_conversion() {
        let event = PlyAudioIntegration::midi_to_ply_event(
            u4::new(0),
            MidiMessage::NoteOn {
                key: u7::new(60),
                vel: u7::new(100),
            },
        );

        match event {
            PlyAudioEvent::NoteOn {
                channel,
                key,
                velocity,
            } => {
                assert_eq!(channel.as_int(), 0);
                assert_eq!(key, 60);
                assert_eq!(velocity, 100);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_create_manager() {
        let manager = PlyAudioIntegration::create_manager();
        assert!(manager.main_output().is_none());
        assert_eq!(manager.runtime_gain(), 1.0);
    }
}
