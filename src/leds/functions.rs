use std::sync::{
    mpsc::{Receiver, TryRecvError},
    Arc, Mutex,
};

use paris::error;
use portmidi::MidiEvent;
use rs_ws281x::Controller;

use crate::structs::{Animator, ColorMode, MidiEventType};

pub fn get_note_position(note: u8, config: &crate::structs::Config) -> usize {
    if (note < 20) || (note > 108) {
        return 0;
    }
    let mut note_offset = 0;
    for i in 0..config.leds.offsets.len() {
        if note > config.leds.offsets[i][0] {
            note_offset = config.leds.offsets[i][1];
            break;
        }
    }
    note_offset -= config.leds.shift;
    let note_pos_raw = 2 * (note - 20) - note_offset;
    config.leds.num_leds - (note_pos_raw as usize)
}
pub fn animate_strip(
    animator: &Arc<Mutex<Animator>>,
    controller: &mut Controller,
    midi_rx: &Receiver<(MidiEventType, MidiEvent, usize)>,
    color_mode: &Arc<Mutex<ColorMode>>,
) {
    match midi_rx.try_recv() {
        Ok(msg) => {
            let (event_type, _event, led_index) = msg;
            match event_type {
                MidiEventType::NoteOn => animator
                    .lock()
                    .expect("Couldn't lock the animator")
                    .note_on(
                        led_index,
                        color_mode
                            .lock()
                            .expect("Couldn't lock the color_mode")
                            .get_color(led_index),
                    ),
                MidiEventType::NoteOff => animator
                    .lock()
                    .expect("Couldn't lock the animator")
                    .note_off(
                        led_index,
                        color_mode
                            .lock()
                            .expect("Couldn't lock the color_mode")
                            .get_color(led_index),
                    ),
                _ => {}
            }
        }
        Err(TryRecvError::Disconnected) => {
            error!("<red>[WS2812]</> MIDI Channel disconnected");
            return;
        }
        _ => {}
    }

    animator
        .lock()
        .expect("Couldn't lock the animator")
        .update();
    animator
        .lock()
        .expect("Couldn't lock the animator")
        .draw(controller);
    controller.render().expect("Couldn't render");
}
