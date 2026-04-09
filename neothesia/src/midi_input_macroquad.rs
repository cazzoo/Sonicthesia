use std::sync::mpsc::{self, Receiver, Sender};

use midi_file::midly::{self, live::LiveEvent, MidiMessage};

const PRESSURE_HISTORY_LEN: usize = 128;

pub struct MacroquadMidiInputManager {
    input: Option<midi_io::MidiInputManager>,
    current_connection: Option<midi_io::MidiInputConnection>,
    rx: Receiver<(u8, MidiMessage)>,
    tx: Sender<(u8, MidiMessage)>,
    pressure_history: Vec<f32>,
    active_notes: std::collections::HashMap<u8, f32>,
}

impl MacroquadMidiInputManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let input = midi_io::MidiInputManager::new()
            .map(|m| {
                let ports: Vec<String> = m.inputs().iter().map(|p| p.to_string()).collect();
                log::info!(
                    "[MIDI-IN] Initialized, {} input ports available: {:?}",
                    ports.len(),
                    ports
                );
                m
            })
            .map_err(|e| log::error!("[MIDI-IN] Failed to init: {}", e))
            .ok();

        Self {
            input,
            current_connection: None,
            rx,
            tx,
            pressure_history: vec![0.0; PRESSURE_HISTORY_LEN],
            active_notes: std::collections::HashMap::new(),
        }
    }

    pub fn inputs(&self) -> Vec<String> {
        self.input
            .as_ref()
            .map(|m| m.inputs().iter().map(|p| p.to_string()).collect())
            .unwrap_or_default()
    }

    pub fn connect_input(&mut self, port_name: &str) {
        let Some(ref mgr) = self.input else {
            log::warn!("[MIDI-IN] No input manager available");
            return;
        };

        let ports = mgr.inputs();
        log::info!(
            "[MIDI-IN] connect_input('{}'), available ports: {:?}",
            port_name,
            ports.iter().map(|p| p.to_string()).collect::<Vec<_>>()
        );

        let port = match ports.iter().find(|p| p.to_string() == port_name) {
            Some(p) => p.clone(),
            None => {
                log::warn!(
                    "[MIDI-IN] Port '{}' not found in available ports",
                    port_name
                );
                return;
            }
        };

        self.current_connection = None;
        log::info!("[MIDI-IN] Connecting to '{}'...", port_name);

        let tx = self.tx.clone();
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let Ok(event) = LiveEvent::parse(message) else {
                return;
            };

            if let LiveEvent::Midi { channel, message } = event {
                match message {
                    MidiMessage::NoteOn { key, vel } if vel == 0 => {
                        tx.send((channel.as_int(), MidiMessage::NoteOff { key, vel }))
                            .ok();
                    }
                    MidiMessage::NoteOn { key, vel } => {
                        tx.send((channel.as_int(), MidiMessage::NoteOn { key, vel }))
                            .ok();
                    }
                    MidiMessage::Aftertouch { key, vel } => {
                        tx.send((channel.as_int(), MidiMessage::Aftertouch { key, vel }))
                            .ok();
                    }
                    MidiMessage::ChannelAftertouch { vel } => {
                        tx.send((channel.as_int(), MidiMessage::ChannelAftertouch { vel }))
                            .ok();
                    }
                    msg => {
                        tx.send((channel.as_int(), msg)).ok();
                    }
                }
            }
        });

        if self.current_connection.is_some() {
            log::info!("[MIDI-IN] Successfully connected to '{}'", port_name);
        } else {
            log::error!("[MIDI-IN] Failed to connect to '{}'", port_name);
        }
    }

    pub fn poll_events(&mut self) -> Vec<(u8, MidiMessage)> {
        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            match &event.1 {
                MidiMessage::NoteOn { key, vel } => {
                    self.active_notes
                        .insert(key.as_int(), vel.as_int() as f32 / 127.0);
                }
                MidiMessage::NoteOff { key, .. } => {
                    self.active_notes.remove(&key.as_int());
                }
                MidiMessage::Aftertouch { key, vel } => {
                    self.active_notes
                        .insert(key.as_int(), vel.as_int() as f32 / 127.0);
                }
                MidiMessage::ChannelAftertouch { vel } => {
                    let pressure = vel.as_int() as f32 / 127.0;
                    for (_key, v) in self.active_notes.iter_mut() {
                        *v = pressure;
                    }
                }
                _ => {}
            }
            events.push(event);
        }

        let max_pressure = self.active_notes.values().copied().fold(0.0f32, f32::max);
        self.pressure_history.push(max_pressure);
        if self.pressure_history.len() > PRESSURE_HISTORY_LEN {
            self.pressure_history.remove(0);
        }

        events
    }

    pub fn pressure_history(&self) -> &[f32] {
        &self.pressure_history
    }

    pub fn active_note_pressure(&self) -> f32 {
        self.active_notes.values().copied().fold(0.0f32, f32::max)
    }
}
