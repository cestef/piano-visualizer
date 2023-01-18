use core::time;
use std::sync::mpsc::Sender;
use std::{fs, thread};

use crate::leds::functions::get_note_position;
use crate::structs::{Config, MidiEventType};
use paris::{info, log};
use pm::MidiMessage;
use portmidi as pm;
use portmidi::MidiEvent;

pub fn get_midi_event_type(status: u8, velocity: u8) -> MidiEventType {
    if status == 144 && velocity > 0 {
        MidiEventType::NoteOn
    } else if status == 128 || (status == 144 && velocity == 0) {
        MidiEventType::NoteOff
    } else {
        MidiEventType::ControlChange
    }
}
pub fn play_midi_file(
    file: String,
    midi_context: &pm::PortMidi,
    tx: Sender<(MidiEventType, pm::MidiEvent, usize)>,
    config: &Config,
) {
    let mut out_port = midi_context
        .device(config.midi.id - 1)
        .and_then(|dev| midi_context.output_port(dev, 1024))
        .unwrap();
    let midi_data = fs::read(file).unwrap();
    let smf = midly::Smf::parse(&midi_data).unwrap();
    info!("<blue>[MIDI]</> Parsed SMF");
    info!("<blue>[MIDI]</> {} tracks", smf.tracks.len());
    let mut ticks_per_beat: u64 = 0;
    let mut beat_duration_us: u64 = 0;
    match smf.header.timing {
        midly::Timing::Metrical(ref timing) => {
            ticks_per_beat = timing.as_int().into();
            println!("[MIDI] Ticks per beat: {:?}/beat", ticks_per_beat);
        }
        midly::Timing::Timecode(fps, sub) => {
            println!("[MIDI] TimeCode timing: fps {:?} | sub {:?}", fps, sub);
        }
    }
    match smf.header.format {
        midly::Format::SingleTrack => {
            info!("<blue>[MIDI]</> Single track");
        }
        midly::Format::Parallel => {
            info!("<blue>[MIDI]</> Parallel track");
        }
        midly::Format::Sequential => {
            info!("<blue>[MIDI]</> Sequential track");
        }
    }
    // Tempo track
    for &event in smf.tracks[0].iter() {
        match event.kind {
            midly::TrackEventKind::Meta(ref meta) => match meta {
                midly::MetaMessage::Tempo(tempo) => {
                    beat_duration_us = tempo.as_int() as u64;
                    info!(
                        "<blue>[MIDI]</> Microseconds per beat: {}",
                        beat_duration_us
                    );
                }

                _ => {}
            },
            _ => {}
        }
    }
    for (i, track) in smf.tracks.iter().enumerate() {
        println!("Track {} has {} events", i, track.len());
        for &event in track.iter() {
            match event.kind {
                midly::TrackEventKind::Meta(ref meta) => match meta {
                    midly::MetaMessage::InstrumentName(ref name) => {
                        // convert from [u8] to string
                        let name = String::from_utf8(name.to_vec()).unwrap();
                        log!("<gray>[MIDI]</> Instrument name: {}", name);
                    }
                    midly::MetaMessage::TrackName(ref name) => {
                        // convert from [u8] to string
                        let name = String::from_utf8(name.to_vec()).unwrap();
                        log!("<gray>[MIDI]</> Track name: {}", name);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
    let track = &smf.tracks[1];
    info!(
        "<blue>[MIDI]</> Processing track with {} events",
        track.len()
    );
    for &event in track.iter() {
        let ticks_to_wait: u64 = event.delta.as_int().into();
        thread::sleep(time::Duration::from_micros(
            ticks_to_wait * beat_duration_us / ticks_per_beat,
        ));
        match event.kind {
            midly::TrackEventKind::Midi { message, .. } => match message {
                midly::MidiMessage::NoteOn { key, vel } => {
                    let note_on = MidiMessage {
                        status: 0x90 + 0,
                        data1: key.into(),
                        data2: vel.into(),
                        data3: 0,
                    };
                    out_port.write_message(note_on).unwrap();
                    if vel > 0 {
                        tx.send((
                            MidiEventType::NoteOn,
                            MidiEvent {
                                message: note_on,
                                timestamp: 0,
                            },
                            get_note_position(key.into(), config),
                        ))
                        .unwrap();
                    } else {
                        tx.send((
                            MidiEventType::NoteOff,
                            MidiEvent {
                                message: note_on,
                                timestamp: 0,
                            },
                            get_note_position(key.into(), config),
                        ))
                        .unwrap();
                    }
                }
                midly::MidiMessage::NoteOff { key, vel } => {
                    let note_off = MidiMessage {
                        status: 0x80 + 0,
                        data1: key.into(),
                        data2: vel.into(),
                        data3: 0,
                    };

                    out_port.write_message(note_off).unwrap();
                    tx.send((
                        MidiEventType::NoteOff,
                        MidiEvent {
                            message: note_off,
                            timestamp: 0,
                        },
                        get_note_position(key.into(), config),
                    ))
                    .unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
pub fn watch_midi(
    input_port: &pm::InputPort,
    tx: &Sender<(MidiEventType, pm::MidiEvent, usize)>,
    config: &Config,
) {
    loop {
        if let Ok(_) = input_port.poll() {
            if let Ok(Some(events)) = input_port.read_n(config.midi.max_keys_processing) {
                for event in events {
                    let event_type = get_midi_event_type(event.message.status, event.message.data2);
                    match event_type {
                        MidiEventType::NoteOn => {
                            tx.send((
                                event_type,
                                event,
                                get_note_position(event.message.data1, &config),
                            ))
                            .expect("Failed to send MIDI event");
                        }
                        MidiEventType::NoteOff => {
                            tx.send((
                                event_type,
                                event,
                                get_note_position(event.message.data1, &config),
                            ))
                            .expect("Failed to send MIDI event");
                        }
                        _ => {}
                    }
                }
            }
        }
        thread::sleep(time::Duration::from_millis(config.midi.timeout));
    }
}
