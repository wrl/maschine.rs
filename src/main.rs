//  maschine.rs: user-space drivers for native instruments USB HIDs
//  Copyright (C) 2015 William Light <wrl@illest.net>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as
//  published by the Free Software Foundation, either version 3 of the
//  License, or (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with this program.  If not, see
//  <http://www.gnu.org/licenses/>.

use std::default::Default;
use std::path::Path;
use std::env;

extern crate nix;
use nix::fcntl::{O_RDWR, O_NONBLOCK};
use nix::{fcntl,sys};

extern crate mio;
extern crate glm;
extern crate glm_color;

extern crate alsa_seq;
use alsa_seq::*;

use glm::ext::*;
use glm_color::*;

extern crate midi;
use midi::*;

mod devices;
mod base;

use base::{
    Maschine,
    MaschineHandler
};

const DEVICE: mio::Token = mio::Token(0);

struct EvLoopHandler<'a> {
    dev: &'a mut Maschine,
    handler: &'a mut MaschineHandler
}

impl<'a> mio::Handler for EvLoopHandler<'a> {
    type Timeout = ();
    type Message = u32;

    fn readable(&mut self, _: &mut mio::EventLoop<Self>,
                token: mio::Token, _: mio::ReadHint) {
        match token {
            DEVICE => self.dev.readable(self.handler),
            _  => panic!("unexpected token")
        }
    }

    fn timeout(&mut self, ev_loop: &mut mio::EventLoop<Self>, _: ()) {
        self.dev.write_lights();
        self.set_timeout(ev_loop);
    }
}

impl<'a> EvLoopHandler<'a> {
    fn set_timeout(&self, ev_loop: &mut mio::EventLoop<Self>) {
        ev_loop.timeout_ms((), 1).unwrap();
    }
}

fn ev_loop(dev: &mut Maschine, handler: &mut MaschineHandler) {
    let mut config = mio::EventLoopConfig::default();
    config.timer_tick_ms = 20;

    let mut ev_loop = mio::EventLoop::configured(config).unwrap();

    let mut handler = EvLoopHandler {
        dev: dev,
        handler: handler
    };

    ev_loop.register(handler.dev.get_io(), DEVICE).unwrap();
    handler.set_timeout(&mut ev_loop);

    ev_loop.run(&mut handler).unwrap();
}

fn usage(prog_name: &String) {
    println!("usage: {} <hidraw device>", prog_name);
}

const PAD_RELEASED_BRIGHTNESS: f32 = 0.015;

const PAD_NOTE_MAP: [U7; 16] = [
    60, 61, 62, 63,
    56, 57, 58, 59,
    52, 53, 54, 55,
    48, 49, 50, 51
];

struct MHandler<'a> {
    color: Hsv,

    seq_handle: &'a SequencerHandle,
    seq_port: &'a SequencerPort<'a>
}

impl<'a> MHandler<'a> {
    pub fn pad_color(&self) -> u32 {
        let rgb = self.color.to_rgb();

        ((((rgb.red() * 255.0) as u32) << 16)
         | (((rgb.green() * 255.0) as u32) << 8)
         | ((rgb.blue() * 255.0) as u32))
    }
}

impl<'a> MaschineHandler for MHandler<'a> {
    fn pad_pressed(&mut self, maschine: &mut Maschine, pad_idx: usize, pressure: f32) {
        let msg = Message::NoteOn(Ch1, PAD_NOTE_MAP[pad_idx], (pressure * 127.0) as U7);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
    }

    fn pad_aftertouch(&mut self, maschine: &mut Maschine, pad_idx: usize, pressure: f32) {
        let msg = Message::PolyphonicPressure(Ch1, PAD_NOTE_MAP[pad_idx], (pressure * 127.0) as U7);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
    }

    fn pad_released(&mut self, maschine: &mut Maschine, pad_idx: usize) {
        let msg = Message::NoteOff(Ch1, PAD_NOTE_MAP[pad_idx], 0);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), PAD_RELEASED_BRIGHTNESS);
    }

    fn encoder_step(&mut self, maschine: &mut Maschine, _: usize, delta: i32) {
        if delta > 0 {
            println!(" :: encoder [>]");
        } else {
            println!(" :: encoder [<]");
        }

        let mut hue = self.color.hue() + ((delta as f32) * 0.2);
        while hue < 0.0 {
            hue += f32::tau();
        }

        self.color.set_hue(hue);

        for i in (0 .. 16) {
            let brightness = match maschine.get_pad_pressure(i).unwrap() {
                0.0 => PAD_RELEASED_BRIGHTNESS,
                pressure @ _ => pressure.sqrt()
            };

            maschine.set_pad_light(i, self.pad_color(), brightness);
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        usage(&args[0]);
        panic!("missing hidraw device path");
    }

    let dev_fd = match fcntl::open(Path::new(&args[1]), O_RDWR | O_NONBLOCK,
                                   sys::stat::Mode::empty()) {
        Err(err) => panic!("couldn't open {}: {}", args[1],
                           err.errno().desc()),
        Ok(file) => file
    };

    let seq_handle = SequencerHandle::open("maschine.rs", HandleOpenStreams::Output).unwrap();
    let seq_port = seq_handle.create_port(
        "Pads MIDI", PORT_CAPABILITY_READ | PORT_CAPABILITY_SUBS_READ, PortType::MidiGeneric)
            .unwrap();

    let mut handler = MHandler {
        color: Hsv::new(0.0, 1.0, 1.0),

        seq_port: &seq_port,
        seq_handle: &seq_handle
    };

    let mut dev = devices::mk2::Mikro::new(mio::Io::new(dev_fd));

    dev.clear_screen();

    for i in (0..16) {
        dev.set_pad_light(i, handler.pad_color(), PAD_RELEASED_BRIGHTNESS);
    }

    ev_loop(&mut dev, &mut handler);
}
