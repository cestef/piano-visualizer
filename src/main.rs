#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
mod api;
mod functions;
mod leds;
mod midi;
mod structs;

use cichlid::{prelude::*, ColorRGB};
use leds::functions::*;
use midi::functions::*;
use paris::{error, info, success};
use portmidi::{MidiEvent, PortMidi};
use rs_ws281x::{ChannelBuilder, ControllerBuilder, StripType};
use std::{
    fs,
    panic::set_hook,
    sync::{Arc, Mutex},
    thread, time,
};
use structs::*;
#[rocket::main]
async fn main() {
    set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<String>() {
            error!("{}", s);
        }
    }));

    let config_file = fs::read_to_string("config.toml").expect("Couldn't read config.toml");
    let config: Config = toml::from_str(&config_file).expect("Couldn't parse config.toml");
    let config_midi = config.clone();
    let config_leds = config.clone();

    let color_mode = Arc::new(Mutex::new(ColorMode::new(
        &config.leds.color_mode,
        &config.leds.num_leds,
    )));
    let color_mode_leds = color_mode.clone();

    let animator = Arc::new(Mutex::new(Animator::new(&config, &config.leds.animation)));
    let animator_leds = animator.clone();

    let (midi_tx, midi_rx) = std::sync::mpsc::channel::<(MidiEventType, MidiEvent, usize)>();

    thread::spawn(move || {
        let config = config_midi;
        info!("<blue>[MIDI]</> Starting the thread");

        let midi_context = PortMidi::new().expect("Couldn't create PortMidi context");
        let device_info = midi_context
            .device(config.midi.id)
            .expect(format!("Could not find device with id {}", config.midi.id).as_str());
        info!(
            "<blue>[MIDI]</> Using device {}) {}",
            device_info.id(),
            device_info.name()
        );
        let input_port = midi_context
            .input_port(device_info, config.midi.buffer_size)
            .expect("Could not create input port");

        watch_midi(&input_port, &midi_tx, &config)
    });

    thread::spawn(move || {
        let config = config_leds;
        let color_mode = color_mode_leds;
        let animator = animator_leds;
        info!("<blue>[WS2812]</> Starting the thread");
        let mut controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                config.leds.channel,
                ChannelBuilder::new()
                    .pin(config.leds.pin)
                    .count(config.leds.num_leds as i32)
                    .strip_type(StripType::Ws2812)
                    .brightness(config.leds.brightness)
                    .build(),
            )
            .build()
            .expect("Couldn't create controller");
        success!(
            "<green>[WS2812]</> Created controller on pin {}",
            config.leds.pin
        );
        let mut colors = vec![ColorRGB::Black; config.leds.num_leds];
        colors.rainbow_fill(0, (config.leds.num_leds * 4) as u16);

        loop {
            animate_strip(&animator, &mut controller, &midi_rx, &color_mode);
            thread::sleep(time::Duration::from_millis(config.midi.timeout));
        }
    });

    let _ = crate::api::main(&color_mode, &animator)
        .ignite()
        .await
        .expect("Couldn't ignite the API")
        .launch()
        .await
        .expect("Couldn't launch the API");
}
