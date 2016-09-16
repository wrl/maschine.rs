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

use std::path::Path;
use std::os::unix::io::AsRawFd;
use std::env;

use std::net::{
    UdpSocket,
    SocketAddr,
    SocketAddrV4,
    Ipv4Addr
};

use std::time::{
    Duration,
    SystemTime
};

extern crate nix;
use nix::fcntl::{O_RDWR, O_NONBLOCK};
use nix::{fcntl,sys};
use nix::poll::*;

extern crate midi;
extern crate alsa_seq;
use midi::*;
use alsa_seq::*;

extern crate hsl;
use hsl::HSL;

#[macro_use(osc_args)]
extern crate tinyosc;
use tinyosc as osc;

mod devices;
mod base;

use base::{
    Maschine,
    MaschineHandler,
    MaschineButton
};

fn ev_loop<'a>(dev: &'a mut Maschine, mhandler: &'a mut MHandler<'a>) {
    let mut fds = [
        PollFd::new(dev.get_fd(), POLLIN, EventFlags::empty()),
        PollFd::new(mhandler.osc_socket.as_raw_fd(), POLLIN, EventFlags::empty())
    ];

    let mut now = SystemTime::now();
    let timer_interval = Duration::from_millis(16);

    loop {
        poll(&mut fds, 16).unwrap();

        if fds[0].revents().unwrap().contains(POLLIN) {
            dev.readable(mhandler);
        }

        if fds[1].revents().unwrap().contains(POLLIN) {
            mhandler.recv_osc_msg(dev);
        }

        if now.elapsed().unwrap() >= timer_interval {
            dev.write_lights();
            now = SystemTime::now();
        }
    }
}

fn usage(prog_name: &String) {
    println!("usage: {} <hidraw device>", prog_name);
}

const PAD_RELEASED_BRIGHTNESS: f32 = 0.015;

#[allow(dead_code)]
enum PressureShape {
    Linear,
    Exponential(f32),
    Constant(f32)
}

struct MHandler<'a> {
    color: HSL,

    seq_handle: &'a SequencerHandle,
    seq_port: &'a SequencerPort<'a>,

    pressure_shape: PressureShape,
    send_aftertouch: bool,

    osc_socket: &'a UdpSocket,
    osc_outgoing_addr: SocketAddr
}

fn osc_button_to_btn_map(osc_button: &str) -> Option<MaschineButton> {
    match osc_button {
        "restart" => Some(MaschineButton::Restart),
        "step_left" => Some(MaschineButton::StepLeft),
        "step_right" => Some(MaschineButton::StepRight),
        "grid" => Some(MaschineButton::Grid),
        "play" => Some(MaschineButton::Play),
        "rec" => Some(MaschineButton::Rec),
        "erase" => Some(MaschineButton::Erase),
        "shift" => Some(MaschineButton::Shift),

        "group" => Some(MaschineButton::Group),
        "browse" => Some(MaschineButton::Browse),
        "sampling" => Some(MaschineButton::Sampling),
        "note_repeat" => Some(MaschineButton::NoteRepeat),

        "encoder" => Some(MaschineButton::Encoder),

        "f1" => Some(MaschineButton::F1),
        "f2" => Some(MaschineButton::F2),
        "f3" => Some(MaschineButton::F3),
        "control" => Some(MaschineButton::Control),
        "nav" => Some(MaschineButton::Nav),
        "nav_left" => Some(MaschineButton::NavLeft),
        "nav_right" => Some(MaschineButton::NavRight),
        "main" => Some(MaschineButton::Main),

        "scene" => Some(MaschineButton::Scene),
        "pattern" => Some(MaschineButton::Pattern),
        "pad_mode" => Some(MaschineButton::PadMode),
        "view" => Some(MaschineButton::View),
        "duplicate" => Some(MaschineButton::Duplicate),
        "select" => Some(MaschineButton::Select),
        "solo" => Some(MaschineButton::Solo),
        "mute" => Some(MaschineButton::Mute),

        _ => None
    }
}

fn btn_to_osc_button_map(btn: MaschineButton) -> &'static str {
    match btn {
        MaschineButton::Restart => "restart",
        MaschineButton::StepLeft => "step_left",
        MaschineButton::StepRight => "step_right",
        MaschineButton::Grid => "grid",
        MaschineButton::Play => "play",
        MaschineButton::Rec => "rec",
        MaschineButton::Erase => "erase",
        MaschineButton::Shift => "shift",

        MaschineButton::Group => "group",
        MaschineButton::Browse => "browse",
        MaschineButton::Sampling => "sampling",
        MaschineButton::NoteRepeat => "note_repeat",

        MaschineButton::Encoder => "encoder",

        MaschineButton::F1 => "f1",
        MaschineButton::F2 => "f2",
        MaschineButton::F3 => "f3",
        MaschineButton::Control => "control",
        MaschineButton::Nav => "nav",
        MaschineButton::NavLeft => "nav_left",
        MaschineButton::NavRight => "nav_right",
        MaschineButton::Main => "main",

        MaschineButton::Scene => "scene",
        MaschineButton::Pattern => "pattern",
        MaschineButton::PadMode => "pad_mode",
        MaschineButton::View => "view",
        MaschineButton::Duplicate => "duplicate",
        MaschineButton::Select => "select",
        MaschineButton::Solo => "solo",
        MaschineButton::Mute => "mute"
    }
}

impl<'a> MHandler<'a> {
    fn pad_color(&self) -> u32 {
        let (r, g, b) = self.color.to_rgb();

        (((r as u32) << 16)
         | ((g as u32) << 8)
         | (b as u32))
    }

    fn pressure_to_vel(&self, pressure: f32) -> U7 {
        (match self.pressure_shape {
            PressureShape::Linear => pressure,
            PressureShape::Exponential(power) => pressure.powf(power),
            PressureShape::Constant(c_pressure) => c_pressure
        } * 127.0) as U7
    }

    #[allow(dead_code)]
    fn update_pad_colors(&self, maschine: &mut Maschine) {
        for i in 0..16 {
            let brightness = match maschine.get_pad_pressure(i).unwrap() {
                0.0 => PAD_RELEASED_BRIGHTNESS,
                pressure @ _ => pressure.sqrt()
            };

            maschine.set_pad_light(i, self.pad_color(), brightness);
        }
    }

    fn recv_osc_msg(&self, maschine: &mut Maschine) {
        let mut buf = [0u8; 128];

        let nbytes = match self.osc_socket.recv_from(&mut buf) {
            Ok((nbytes, _)) => nbytes,
            Err(e) => {
                println!(" :: error in recv_from(): {}", e);
                return;
            }
        };

        let msg = match osc::Message::deserialize(&buf[.. nbytes]) {
            Ok(msg) => msg,
            Err(_) => {
                println!(" :: couldn't decode OSC message :c");
                return;
            }
        };

        self.handle_osc_messge(maschine, &msg);
    }

    fn handle_osc_messge(&self, maschine: &mut Maschine, msg: &osc::Message) {
        if msg.path.starts_with("/maschine/button") {
            let btn = match osc_button_to_btn_map(&msg.path[17 ..]) {
                Some(btn) => btn,
                None => return
            };

            match msg.arguments.len() {
                1 =>
                    maschine.set_button_light(btn, 0xFFFFFF, match msg.arguments[0] {
                        osc::Argument::i(val) => (val as f32),
                        osc::Argument::f(val) => val,
                        _ => return
                    }),

                2 => {
                    if let (&osc::Argument::i(color), &osc::Argument::f(brightness))
                        = (&msg.arguments[0], &msg.arguments[1]) {
                        maschine.set_button_light(btn, (color as u32) & 0xFFFFFF, brightness);
                    }
                }

                _ => return
            };
        }
        else if msg.path.starts_with("/maschine/pad") {
            match msg.arguments.len() {
                3 => {
                    if let (&osc::Argument::i(pad), &osc::Argument::i(color), &osc::Argument::f(brightness))
                        = (&msg.arguments[0], &msg.arguments[1], &msg.arguments[2]) {
                        maschine.set_pad_light( pad as usize, (color as u32) & 0xFFFFFF, brightness as f32);
                    }
                }

                _ => return
            }
        }
        else if msg.path.starts_with("/maschine/midi_note_base") {
            match msg.arguments.len() {
                1 => {
                  if let osc::Argument::i(base) = msg.arguments[0] {
                    maschine.set_midi_note_base(base as u8);
                  }
                }
                _ => return
            }
        }

    }

    fn send_osc_msg(&self, path: &str, arguments: Vec<osc::Argument>) {
        let msg = osc::Message {
            path: path,
            arguments: arguments
        };

        match self.osc_socket.send_to(&*msg.serialize().unwrap(), &self.osc_outgoing_addr) {
            Ok(_) => {},
            Err(e) => println!(" :: error in send_to: {}", e)
        }
    }

    fn send_osc_button_msg(&self, btn: MaschineButton, status: usize) {
        self.send_osc_msg(
            &*format!("/maschine/button/{}", btn_to_osc_button_map(btn)),
            osc_args![status as i32]);
    }

    fn send_osc_encoder_msg(&self, delta: i32) {
        self.send_osc_msg("/maschine/encoder", osc_args![delta]);
    }
}

const PAD_NOTE_MAP: [U7; 16] = [
    12, 13, 14, 15,
     8,  9, 10, 11,
     4,  5,  6,  7,
     0,  1,  2,  3
];

impl<'a> MaschineHandler for MHandler<'a> {
    fn pad_pressed(&mut self, maschine: &mut Maschine, pad_idx: usize, pressure: f32) {
        let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
        let msg = Message::NoteOn(Ch1, midi_note, self.pressure_to_vel(pressure));

        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
    }

    fn pad_aftertouch(&mut self, maschine: &mut Maschine, pad_idx: usize, pressure: f32) {
        match self.pressure_shape {
            PressureShape::Constant(_) => return,
            _ => {}
        }

        if !self.send_aftertouch {
            return
        }

        let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
        let msg = Message::PolyphonicPressure(Ch1, midi_note,
                                              self.pressure_to_vel(pressure));

        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), pressure.sqrt());
    }

    fn pad_released(&mut self, maschine: &mut Maschine, pad_idx: usize) {
        let midi_note = maschine.get_midi_note_base() + PAD_NOTE_MAP[pad_idx];
        let msg = Message::NoteOff(Ch1, midi_note, 0);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();

        maschine.set_pad_light(pad_idx, self.pad_color(), PAD_RELEASED_BRIGHTNESS);
    }

    fn encoder_step(&mut self, _: &mut Maschine, _: usize, delta: i32) {
        self.send_osc_encoder_msg(delta);
    }

    fn button_down(&mut self, _: &mut Maschine, btn: MaschineButton) {
        self.send_osc_button_msg(btn, 1);
    }

    fn button_up(&mut self, _: &mut Maschine, btn: MaschineButton) {
        self.send_osc_button_msg(btn, 0);
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

    let osc_socket = UdpSocket::bind("127.0.0.1:42434").unwrap();

    let seq_handle = SequencerHandle::open("maschine.rs", HandleOpenStreams::Output).unwrap();
    let seq_port = seq_handle.create_port(
        "Pads MIDI", PORT_CAPABILITY_READ | PORT_CAPABILITY_SUBS_READ, PortType::MidiGeneric)
            .unwrap();

    let mut dev = devices::mk2::Mikro::new(dev_fd);

    let mut handler = MHandler {
        color: HSL {
            h: 0.0,
            s: 1.0,
            l: 0.3
        },

        seq_port: &seq_port,
        seq_handle: &seq_handle,

        pressure_shape: PressureShape::Exponential(0.4),
        send_aftertouch: false,

        osc_socket: &osc_socket,
        osc_outgoing_addr: SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 42435))
    };

    dev.clear_screen();

    for i in 0..16 {
        dev.set_pad_light(i, handler.pad_color(), PAD_RELEASED_BRIGHTNESS);
    }

    ev_loop(&mut dev, &mut handler);
}
