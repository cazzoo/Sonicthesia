//! PLY Audio Integration Module
//! 
//! This module provides integration between Neothesia's audio system and the PLY engine.
//! It maps Neothesia audio events to work with PLY's architecture while maintaining
//! all existing audio functionality including MIDI output, synthesis, and gain controls.

use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

use midi_file::midly::{self, num::u4, MidiMessage};

/// PLY-compatible audio event that can be processed through PLY's event system
#[derive(Debug, Clone)]
pub enum PlyAudioEvent {
    /// Note on event
    NoteOn {
        channel: u4,
        key: u8,
        velocity: u8,
    },
    /// Note off event
    NoteOff {
        channel: u4,
        key: u8,
    },
    /// MIDI control change
    ControlChange {
        channel: u4,
        controller: u8,
        value: u8,
    },
    /// Program change
    ProgramChange {
        channel: u4,
        program: u8,
    },
    /// Pitch bend
    PitchBend {
        channel: u4,
        value: u16,
    },
    /// Channel aftertouch
    ChannelAftertouch {
        channel: u4,
        value: u8,
    },
    /// Polyphonic key pressure (aftertouch)
    PolyphonicPressure {
        channel: u4,
        key: u8,
        value: u8,
    },
    /// System exclusive message
    SysEx(Vec<u8>),
    /// Set audio gain
    SetGain(f32),
    /// Stop all notes
    StopAll,
}

/// Active note tracking for proper cleanup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ActiveNote {
    key: u8,
    channel: u8,
}

/// PLY audio output connection wrapper
/// This wraps the existing Neothesia audio backends to work with PLY's architecture
#[derive(Clone)]
pub enum PlyAudioConnection {
    /// MIDI output connection
    Midi(Rc<RefCell<MidiConnectionInner>>),
    /// Synth output connection (when synth feature is enabled)
    #[cfg(feature = "synth")]
    Synth(Rc<RefCell<SynthConnectionInner>>),
    /// Dummy output for testing
    Dummy,
}

struct MidiConnectionInner {
    active_notes: HashSet<ActiveNote>,
    midi_sender: Box<dyn MidiEventSender>,
    buffer: Vec<u8>,
}

#[cfg(feature = "synth")]
struct SynthConnectionInner {
    active_notes: HashSet<ActiveNote>,
    event_sender: std::sync::mpsc::Sender<crate::output_manager::synth_backend::SynthEvent>,
}

/// Trait for sending MIDI events
trait MidiEventSender: Send {
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>>;
}

/// Adapter for midi-io connections
struct MidiIoAdapter {
    conn: midi_io::MidiOutputConnection,
}

impl MidiEventSender for MidiIoAdapter {
    fn send(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.send(data).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

impl PlyAudioConnection {
    /// Create a new MIDI connection from a midi-io connection
    pub fn from_midi_connection(conn: midi_io::MidiOutputConnection) -> Self {
        let inner = MidiConnectionInner {
            active_notes: HashSet::new(),
            midi_sender: Box::new(MidiIoAdapter { conn }),
            buffer: Vec::with_capacity(8),
        };
        PlyAudioConnection::Midi(Rc::new(RefCell::new(inner)))
    }

    /// Create a new synth connection (when synth feature is enabled)
    #[cfg(feature = "synth")]
    pub fn from_synth_sender(
        sender: std::sync::mpsc::Sender<crate::output_manager::synth_backend::SynthEvent>,
    ) -> Self {
        let inner = SynthConnectionInner {
            active_notes: HashSet::new(),
            event_sender: sender,
        };
        PlyAudioConnection::Synth(Rc::new(RefCell::new(inner)))
    }

    /// Create a dummy connection
    pub fn dummy() -> Self {
        PlyAudioConnection::Dummy
    }

    /// Process a PLY audio event
    pub fn process_event(&self, event: PlyAudioEvent) {
        match self {
            PlyAudioConnection::Midi(inner) => {
                let mut inner = inner.borrow_mut();
                Self::process_midi_event(&mut *inner, event);
            }
            #[cfg(feature = "synth")]
            PlyAudioConnection::Synth(inner) => {
                let mut inner = inner.borrow_mut();
                Self::process_synth_event(&mut *inner, event);
            }
            PlyAudioConnection::Dummy => {
                // Silently ignore events for dummy output
            }
        }
    }

    /// Process event for MIDI connection
    fn process_midi_event(inner: &mut MidiConnectionInner, event: PlyAudioEvent) {
        match event {
            PlyAudioEvent::NoteOn { channel, key, velocity } => {
                inner.active_notes.insert(ActiveNote { key, channel: channel.as_int() as u8 });
                
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::NoteOn {
                        key: midly::num::u7::new(key),
                        vel: midly::num::u7::new(velocity),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::NoteOff { channel, key } => {
                inner.active_notes.remove(&ActiveNote { key, channel: channel.as_int() as u8 });
                
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::NoteOff {
                        key: midly::num::u7::new(key),
                        vel: midly::num::u7::new(0),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::ControlChange { channel, controller, value } => {
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::Controller {
                        controller: midly::num::u7::new(controller),
                        value: midly::num::u7::new(value),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::ProgramChange { channel, program } => {
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::ProgramChange {
                        program: midly::num::u7::new(program),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::PitchBend { channel, value } => {
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: MidiMessage::PitchBend {
                        bend: midly::PitchBend(midly::num::u14::new(value)),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::ChannelAftertouch { channel, value } => {
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::ChannelAftertouch {
                        vel: midly::num::u7::new(value),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::PolyphonicPressure { channel, key, value } => {
                inner.buffer.clear();
                let msg = midly::live::LiveEvent::Midi {
                    channel,
                    message: midly::MidiMessage::Aftertouch {
                        key: midly::num::u7::new(key),
                        vel: midly::num::u7::new(value),
                    },
                };
                let _ = msg.write(&mut inner.buffer);
                let _ = inner.midi_sender.send(&inner.buffer);
            }
            PlyAudioEvent::SysEx(data) => {
                log::debug!("Sending MIDI SysEx: {:02X?} ({} bytes)", data, data.len());
                let _ = inner.midi_sender.send(&data);
            }
            PlyAudioEvent::SetGain(_) => {
                // Gain control not applicable for MIDI output
            }
            PlyAudioEvent::StopAll => {
                // Send note off for all active notes
                for note in std::mem::take(&mut inner.active_notes) {
                    inner.buffer.clear();
                    let msg = midly::live::LiveEvent::Midi {
                        channel: midly::num::u4::new(note.channel),
                        message: midly::MidiMessage::NoteOff {
                            key: midly::num::u7::new(note.key),
                            vel: midly::num::u7::new(0),
                        },
                    };
                    let _ = msg.write(&mut inner.buffer);
                    let _ = inner.midi_sender.send(&inner.buffer);
                }
            }
        }
    }

    /// Process event for synth connection (when synth feature is enabled)
    #[cfg(feature = "synth")]
    fn process_synth_event(inner: &mut SynthConnectionInner, event: PlyAudioEvent) {
        use crate::output_manager::synth_backend::SynthEvent;
        
        match event {
            PlyAudioEvent::NoteOn { channel, key, velocity } => {
                inner.active_notes.insert(ActiveNote { key, channel: channel.as_int() as u8 });
                
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::NoteOn {
                    key: midly::num::u7::new(key),
                    vel: midly::num::u7::new(velocity),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::NoteOff { channel, key } => {
                inner.active_notes.remove(&ActiveNote { key, channel: channel.as_int() as u8 });
                
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::NoteOff {
                    key: midly::num::u7::new(key),
                    vel: midly::num::u7::new(0),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::ControlChange { channel, controller, value } => {
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::Controller {
                    controller: midly::num::u7::new(controller),
                    value: midly::num::u7::new(value),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::ProgramChange { channel, program } => {
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::ProgramChange {
                    program: midly::num::u7::new(program),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::PitchBend { channel, value } => {
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::PitchBend {
                    bend: midly::PitchBend(midly::num::u14::new(value)),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::ChannelAftertouch { channel, value } => {
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::ChannelAftertouch {
                    vel: midly::num::u7::new(value),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::PolyphonicPressure { channel, key, value } => {
                let event = libmidi_to_oxisynth_event(channel, midly::MidiMessage::Aftertouch {
                    key: midly::num::u7::new(key),
                    vel: midly::num::u7::new(value),
                });
                let _ = inner.event_sender.send(SynthEvent::Midi(event));
            }
            PlyAudioEvent::SysEx(_) => {
                // SysEx not supported for synth output
            }
            PlyAudioEvent::SetGain(gain) => {
                let _ = inner.event_sender.send(SynthEvent::SetGain(gain));
            }
            PlyAudioEvent::StopAll => {
                // Send all notes off and all sound off for all channels
                for channel in 0..16 {
                    let _ = inner.event_sender.send(SynthEvent::Midi(oxisynth::MidiEvent::AllNotesOff { channel }));
                    let _ = inner.event_sender.send(SynthEvent::Midi(oxisynth::MidiEvent::AllSoundOff { channel }));
                }
                inner.active_notes.clear();
            }
        }
    }

    /// Send a MIDI event using the existing Neothesia interface
    pub fn midi_event(&self, channel: u4, msg: MidiMessage) {
        let event = match msg {
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
        };
        self.process_event(event);
    }

    /// Send a System Exclusive message
    pub fn send_sysex(&self, message: &[u8]) {
        let event = PlyAudioEvent::SysEx(message.to_vec());
        self.process_event(event);
    }

    /// Set the audio gain (only applicable for synth output)
    pub fn set_gain(&self, gain: f32) {
        let event = PlyAudioEvent::SetGain(gain);
        self.process_event(event);
    }

    /// Stop all currently sounding notes
    pub fn stop_all(&self) {
        let event = PlyAudioEvent::StopAll;
        self.process_event(event);
    }
}

/// Convert midly MIDI message to oxisynth event
#[cfg(feature = "synth")]
fn libmidi_to_oxisynth_event(channel: u4, message: midly::MidiMessage) -> oxisynth::MidiEvent {
    let channel = channel.as_int();
    match message {
        midly::MidiMessage::NoteOff { key, .. } => oxisynth::MidiEvent::NoteOff {
            channel,
            key: key.as_int(),
        },
        midly::MidiMessage::NoteOn { key, vel } => oxisynth::MidiEvent::NoteOn {
            channel,
            key: key.as_int(),
            vel: vel.as_int(),
        },
        midly::MidiMessage::Aftertouch { key, vel } => oxisynth::MidiEvent::PolyphonicKeyPressure {
            channel,
            key: key.as_int(),
            value: vel.as_int(),
        },
        midly::MidiMessage::Controller { controller, value } => {
            oxisynth::MidiEvent::ControlChange {
                channel,
                ctrl: controller.as_int(),
                value: value.as_int(),
            }
        }
        midly::MidiMessage::ProgramChange { program } => oxisynth::MidiEvent::ProgramChange {
            channel,
            program_id: program.as_int(),
        },
        midly::MidiMessage::ChannelAftertouch { vel } => oxisynth::MidiEvent::ChannelPressure {
            channel,
            value: vel.as_int(),
        },
        midly::MidiMessage::PitchBend { bend } => oxisynth::MidiEvent::PitchBend {
            channel,
            value: bend.0.as_int(),
        },
    }
}

/// PLY audio manager - handles audio routing and gain control
pub struct PlyAudioManager {
    /// Main audio output connection
    main_output: Option<PlyAudioConnection>,
    
    /// Keyboard input connection (separate for independent gain control)
    keyboard_output: Option<PlyAudioConnection>,
    
    /// LUMI-specific connection (for SysEx and LED control)
    lumi_output: Option<PlyAudioConnection>,
    
    /// Runtime gain multiplier (applied on top of config gain)
    runtime_gain: f32,
}

impl Default for PlyAudioManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PlyAudioManager {
    /// Create a new PLY audio manager
    pub fn new() -> Self {
        Self {
            main_output: None,
            keyboard_output: None,
            lumi_output: None,
            runtime_gain: 1.0,
        }
    }

    /// Set the main audio output connection
    pub fn set_main_output(&mut self, connection: PlyAudioConnection) {
        self.main_output = Some(connection);
    }

    /// Set the keyboard output connection
    pub fn set_keyboard_output(&mut self, connection: PlyAudioConnection) {
        self.keyboard_output = Some(connection);
    }

    /// Set the LUMI output connection
    pub fn set_lumi_output(&mut self, connection: PlyAudioConnection) {
        self.lumi_output = Some(connection);
    }

    /// Get the main output connection
    pub fn main_output(&self) -> Option<&PlyAudioConnection> {
        self.main_output.as_ref()
    }

    /// Get the keyboard output connection
    pub fn keyboard_output(&self) -> &PlyAudioConnection {
        self.keyboard_output
            .as_ref()
            .unwrap_or_else(|| {
                log::warn!("Keyboard connection not initialized, falling back to main connection");
                self.main_output.as_ref().unwrap()
            })
    }

    /// Get the LUMI output connection
    pub fn lumi_output(&self) -> Option<&PlyAudioConnection> {
        self.lumi_output.as_ref()
    }

    /// Set the runtime gain multiplier
    pub fn set_runtime_gain(&mut self, gain: f32) {
        self.runtime_gain = gain.clamp(0.0, 2.0);
    }

    /// Get the runtime gain multiplier
    pub fn runtime_gain(&self) -> f32 {
        self.runtime_gain
    }

    /// Send a MIDI event to the main output
    pub fn midi_event(&self, channel: u4, msg: MidiMessage) {
        if let Some(output) = &self.main_output {
            output.midi_event(channel, msg);
        }
    }

    /// Send a MIDI event to the keyboard output
    pub fn keyboard_midi_event(&self, channel: u4, msg: MidiMessage) {
        self.keyboard_output().midi_event(channel, msg);
    }

    /// Send a SysEx message to the LUMI output
    pub fn lumi_sysex(&self, message: &[u8]) {
        if let Some(output) = &self.lumi_output {
            output.send_sysex(message);
        } else if let Some(main) = &self.main_output {
            log::warn!("No dedicated LUMI connection, sending to main output");
            main.send_sysex(message);
        }
    }

    /// Stop all notes on all outputs
    pub fn stop_all(&self) {
        if let Some(output) = &self.main_output {
            output.stop_all();
        }
        if let Some(output) = &self.keyboard_output {
            output.stop_all();
        }
    }

    /// Set gain on synth outputs
    pub fn set_gain(&self, gain: f32) {
        let adjusted_gain = gain * self.runtime_gain;
        if let Some(output) = &self.main_output {
            output.set_gain(adjusted_gain);
        }
        if let Some(output) = &self.keyboard_output {
            output.set_gain(adjusted_gain);
        }
    }

    /// Check if LUMI connection is active
    pub fn has_lumi_connection(&self) -> bool {
        self.lumi_output.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_event_creation() {
        let event = PlyAudioEvent::NoteOn {
            channel: midly::num::u4::new(0),
            key: 60,
            velocity: 100,
        };
        
        match event {
            PlyAudioEvent::NoteOn { channel, key, velocity } => {
                assert_eq!(channel.as_int(), 0);
                assert_eq!(key, 60);
                assert_eq!(velocity, 100);
            }
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_audio_manager_creation() {
        let manager = PlyAudioManager::new();
        assert!(manager.main_output().is_none());
        assert_eq!(manager.runtime_gain(), 1.0);
    }

    #[test]
    fn test_runtime_gain_clamping() {
        let mut manager = PlyAudioManager::new();
        
        manager.set_runtime_gain(3.0); // Should clamp to 2.0
        assert_eq!(manager.runtime_gain(), 2.0);
        
        manager.set_runtime_gain(-0.5); // Should clamp to 0.0
        assert_eq!(manager.runtime_gain(), 0.0);
        
        manager.set_runtime_gain(0.5); // Should stay at 0.5
        assert_eq!(manager.runtime_gain(), 0.5);
    }

    #[test]
    fn test_dummy_connection() {
        let conn = PlyAudioConnection::dummy();
        
        // Should not panic on any operations
        conn.midi_event(
            midly::num::u4::new(0),
            midly::MidiMessage::NoteOn {
                key: midly::num::u7::new(60),
                vel: midly::num::u7::new(100),
            },
        );
        conn.send_sysex(&[0xF0, 0x01, 0xF7]);
        conn.set_gain(0.8);
        conn.stop_all();
    }
}
