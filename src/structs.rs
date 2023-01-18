use cichlid::{prelude::RainbowFillSingleCycle, ColorRGB};
use rand::prelude::*;
use rs_ws281x::Controller;
use serde_derive::Deserialize;

use crate::functions::hex_to_rgb;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub leds: LedsConfig,
    pub midi: MidiConfig,
    pub api: ApiConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LedsConfig {
    pub pin: i32,
    pub num_leds: usize,
    pub brightness: u8,
    pub offsets: Vec<Vec<u8>>,
    pub shift: u8,
    pub fade: i8,
    pub channel: usize,
    pub color_mode: String,
    pub animation: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MidiConfig {
    pub id: i32,
    pub buffer_size: usize,
    pub max_keys_processing: usize,
    pub timeout: u64,
    pub rtp: RtpConfig,
}
#[derive(Deserialize, Debug, Clone)]
pub struct RtpConfig {
    pub socket: String,
}

#[derive(Debug)]
pub enum MidiEventType {
    NoteOn,
    NoteOff,
    ControlChange,
}

pub enum AnimatorEnum {
    Fades(Fades),
    Ripples(Ripples),
    Static(StaticColor),
    Default(DefaultAnimator),
}

pub struct Animator {
    pub animation: String,
    pub animator: AnimatorEnum,
    config: Config,
}
impl Animator {
    pub fn new(config: &Config, animation: &String) -> Self {
        match animation.as_str() {
            "fade" => Self {
                config: config.clone(),
                animation: animation.to_string(),
                animator: AnimatorEnum::Fades(Fades::new(config)),
            },
            "ripple" => Self {
                config: config.clone(),
                animation: animation.to_string(),
                animator: AnimatorEnum::Ripples(Ripples::new(config)),
            },
            "static" => Self {
                config: config.clone(),
                animation: animation.to_string(),
                animator: AnimatorEnum::Static(StaticColor::new(config)),
            },
            _ => Self {
                config: config.clone(),
                animation: animation.to_string(),
                animator: AnimatorEnum::Default(DefaultAnimator::new(config)),
            },
        }
    }
    pub fn set_animation(&mut self, animation: String) {
        self.animation = animation.to_string();
        match animation.to_string().as_str() {
            "fade" => self.animator = AnimatorEnum::Fades(Fades::new(&self.config)),
            "ripple" => self.animator = AnimatorEnum::Ripples(Ripples::new(&self.config)),
            _ => self.animator = AnimatorEnum::Default(DefaultAnimator::new(&self.config)),
        }
    }
    pub fn update(&mut self) {
        match &mut self.animator {
            AnimatorEnum::Fades(fades) => fades.update(),
            AnimatorEnum::Ripples(ripples) => ripples.update(),
            AnimatorEnum::Default(_) => {}
            AnimatorEnum::Static(_) => {}
        }
    }
    pub fn draw(&mut self, controller: &mut Controller) {
        match &mut self.animator {
            AnimatorEnum::Fades(fades) => fades.draw(controller),
            AnimatorEnum::Ripples(ripples) => ripples.draw(controller),
            AnimatorEnum::Default(default) => default.draw(controller),
            AnimatorEnum::Static(static_color) => static_color.draw(controller),
        }
    }
    pub fn note_on(&mut self, led_index: usize, color: [u8; 4]) {
        match &mut self.animator {
            AnimatorEnum::Fades(fades) => fades.add_fade(led_index, color),
            AnimatorEnum::Ripples(ripples) => ripples.add_ripple(led_index, color),
            AnimatorEnum::Default(default) => default.note_on(led_index, color),
            AnimatorEnum::Static(_) => {}
        }
    }
    pub fn note_off(&mut self, led_index: usize, _color: [u8; 4]) {
        match &mut self.animator {
            AnimatorEnum::Fades(fades) => fades.start_fade(led_index),
            AnimatorEnum::Ripples(_) => {}
            AnimatorEnum::Default(default) => default.note_off(led_index),
            AnimatorEnum::Static(_) => {}
        }
    }
}

#[derive(Debug)]
pub struct Fade {
    pub position: usize,
    pub color: [u8; 4],
    pub fade: i8,
    pub started: bool,
}
#[derive(Debug)]
pub struct Fades {
    pub fades: Vec<Fade>,
    pub config: Config,
}
impl Fades {
    pub fn new(config: &Config) -> Fades {
        Fades {
            fades: Vec::new(),
            config: config.clone(),
        }
    }
    pub fn add_fade(&mut self, position: usize, color: [u8; 4]) {
        self.fades.push(Fade {
            position,
            color,
            fade: self.config.leds.fade,
            started: false,
        });
    }
    pub fn start_fade(&mut self, position: usize) {
        for fade in self.fades.iter_mut() {
            if fade.position == position {
                fade.started = true;
            }
        }
    }
    pub fn update(&mut self) {
        for fade in self.fades.iter_mut() {
            fade.color = [
                (fade.color[0] as f32 * (fade.fade as f32 / self.config.leds.fade as f32)) as u8,
                (fade.color[1] as f32 * (fade.fade as f32 / self.config.leds.fade as f32)) as u8,
                (fade.color[2] as f32 * (fade.fade as f32 / self.config.leds.fade as f32)) as u8,
                fade.color[3],
            ];
            if fade.started {
                fade.fade -= 1;
            }
        }
        self.fades.retain(|fade| fade.fade > 0);
    }
    pub fn draw(&self, controller: &mut Controller) {
        let leds = controller.leds_mut(self.config.leds.channel);
        for fade in self.fades.iter() {
            let led = leds.get_mut(fade.position).expect("Led not found");
            *led = fade.color;
        }
    }
}

pub struct Led {
    pub position: usize,
    pub color: [u8; 4],
}
pub struct DefaultAnimator {
    pub config: Config,
    pub leds: Vec<Led>,
}
impl DefaultAnimator {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            leds: Vec::new(),
        }
    }
    pub fn note_on(&mut self, position: usize, color: [u8; 4]) {
        self.leds.push(Led { position, color });
    }
    pub fn note_off(&mut self, position: usize) {
        self.leds
            .iter_mut()
            .find(|led| led.position == position)
            .map(|led| led.color = [0, 0, 0, 0]);
    }
    pub fn draw(&self, controller: &mut Controller) {
        let leds = controller.leds_mut(self.config.leds.channel);
        for self_led in self.leds.iter() {
            let led = leds.get_mut(self_led.position).expect("Led not found");
            *led = self_led.color;
        }
    }
}
pub struct StaticColor {
    pub config: Config,
}
impl StaticColor {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }
    pub fn draw(&self, controller: &mut Controller) {
        let leds = controller.leds_mut(self.config.leds.channel);
        for led in leds.iter_mut() {
            let rgb = hex_to_rgb(&self.config.leds.color_mode);
            *led = [rgb[2], rgb[1], rgb[0], 0];
        }
    }
}
#[derive(Debug, Clone)]
pub struct TrailPart {
    pub position: usize,
    pub last_position: usize,
    pub color: [u8; 4],
}
#[derive(Debug)]
pub struct Ripple {
    left_trail: Vec<TrailPart>,
    right_trail: Vec<TrailPart>,
}
#[derive(Debug)]
pub struct Ripples {
    pub config: Config,
    pub ripples: Vec<Ripple>,
}

impl Ripples {
    pub fn new(config: &Config) -> Ripples {
        Ripples {
            config: config.clone(),
            ripples: Vec::new(),
        }
    }
    pub fn add_ripple(&mut self, position: usize, color: [u8; 4]) {
        // push a new ripple and fill left_trail and right_trail with the first trail parts
        self.ripples.push(Ripple {
            left_trail: vec![TrailPart {
                position,
                color,
                last_position: position,
            }],
            right_trail: vec![TrailPart {
                position,
                color,
                last_position: position,
            }],
        });
    }
    pub fn update(&mut self) {
        for ripple in self.ripples.iter_mut() {
            for trail_part in ripple.left_trail.iter_mut() {
                if (trail_part.position as i32 - 1) >= 0 {
                    trail_part.last_position = trail_part.position;
                    trail_part.position -= 1;
                } else {
                    trail_part.color = [0, 0, 0, 0];
                }
            }
            for trail_part in ripple.right_trail.iter_mut() {
                if (trail_part.position as i32 + 1) < self.config.leds.num_leds as i32 {
                    trail_part.last_position = trail_part.position;
                    trail_part.position += 1;
                } else {
                    trail_part.color = [0, 0, 0, 0];
                }
            }
        }
    }
    pub fn draw(&mut self, controller: &mut Controller) {
        let leds = controller.leds_mut(self.config.leds.channel);
        for ripple in self.ripples.iter_mut() {
            for trail_part in ripple.left_trail.iter_mut() {
                if let Some(led) = leds.get_mut(trail_part.position) {
                    *led = trail_part.color;
                }
                if let Some(led) = leds.get_mut(trail_part.last_position) {
                    *led = [0, 0, 0, 0];
                }
            }
            for trail_part in ripple.right_trail.iter_mut() {
                if let Some(led) = leds.get_mut(trail_part.position) {
                    *led = trail_part.color;
                }
                if let Some(led) = leds.get_mut(trail_part.last_position) {
                    *led = [0, 0, 0, 0];
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorMode {
    pub mode: String,
    pub num_leds: usize,
}
impl ColorMode {
    pub fn new(mode: &String, num_leds: &usize) -> ColorMode {
        ColorMode {
            mode: mode.to_string(),
            num_leds: *num_leds,
        }
    }
    pub fn set_color_mode(&mut self, mode: String) {
        self.mode = mode;
    }
    pub fn get_color(&self, position: usize) -> [u8; 4] {
        match self.mode.as_str() {
            "rainbow" => self.get_rainbow_color(position),
            "random" => self.get_random_color(),
            _ => self.get_solid_color(&self.mode),
        }
    }
    pub fn get_rainbow_color(&self, position: usize) -> [u8; 4] {
        let mut colors = vec![ColorRGB::Black; self.num_leds];
        colors.rainbow_fill_single_cycle(0);
        [
            colors[position][0],
            colors[position][1],
            colors[position][2],
            0,
        ]
    }
    pub fn get_random_color(&self) -> [u8; 4] {
        let color = [
            rand::thread_rng().gen_range(0..255),
            rand::thread_rng().gen_range(0..255),
            rand::thread_rng().gen_range(0..255),
            0,
        ];
        color
    }
    pub fn get_solid_color(&self, color: &String) -> [u8; 4] {
        let rgb = hex_to_rgb(color);
        [rgb[2], rgb[1], rgb[0], 0]
    }
}
