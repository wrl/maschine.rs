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

use std::mem::transmute;
use std::error::Error;

extern crate mio;
use mio::{TryRead, TryWrite};

use base::{
    Maschine,
    MaschineHandler,
    MaschineButton,

    MaschinePad,
    MaschinePadStateTransition
};

const BUTTON_REPORT_TO_MIKROBUTTONS_MAP: [[Option<MaschineButton>; 8]; 4] = [
    [
        Some(MaschineButton::Restart),
        Some(MaschineButton::StepLeft),
        Some(MaschineButton::StepRight),
        Some(MaschineButton::Grid),
        Some(MaschineButton::Play),
        Some(MaschineButton::Rec),
        Some(MaschineButton::Erase),
        Some(MaschineButton::Shift),
    ],

    [
        Some(MaschineButton::Group),
        Some(MaschineButton::Browse),
        Some(MaschineButton::Sampling),
        Some(MaschineButton::NoteRepeat),
        Some(MaschineButton::Encoder),
        None,
        None,
        None,
    ],

    [
        Some(MaschineButton::F1),
        Some(MaschineButton::F2),
        Some(MaschineButton::F3),
        Some(MaschineButton::Control),
        Some(MaschineButton::Nav),
        Some(MaschineButton::NavLeft),
        Some(MaschineButton::NavRight),
        Some(MaschineButton::Main),
    ],

    [
        Some(MaschineButton::Scene),
        Some(MaschineButton::Pattern),
        Some(MaschineButton::PadMode),
        Some(MaschineButton::View),
        Some(MaschineButton::Duplicate),
        Some(MaschineButton::Select),
        Some(MaschineButton::Solo),
        Some(MaschineButton::Mute),
    ]
];

#[allow(dead_code)]
struct ButtonReport {
    pub buttons: u32,
    pub encoder: u8
}

pub struct Mikro {
    dev: mio::Io,
    light_buf: [u8; 79],

    pads: [MaschinePad; 16],
    buttons: [u8; 5]
}

impl Mikro {
    fn sixteen_maschine_pads() -> [MaschinePad; 16] {
        [
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default()
        ]
    }

    pub fn new(dev: mio::Io) -> Self {
        let mut _self = Mikro {
            dev: dev,
            light_buf: [0u8; 79],

            pads: Mikro::sixteen_maschine_pads(),
            buttons: [0, 0, 0, 0, 0x10]
        };

        _self.light_buf[0] = 0x80;
        return _self;
    }

    fn read_buttons(&mut self, handler: &mut MaschineHandler, buf: &[u8]) {
        for (idx, &byte) in buf[0..4].iter().enumerate() {
            let mut diff = (byte ^ self.buttons[idx]) as u32;

            while diff != 0 {
                let off = (diff.trailing_zeros() + 1) as usize;
                let btn = BUTTON_REPORT_TO_MIKROBUTTONS_MAP[idx][8 - off]
                    .expect("unknown button received from device");

                if (byte & (1 << (off - 1))) != 0 {
                    handler.button_down(self, btn);
                } else {
                    handler.button_up(self, btn);
                }

                diff >>= off;
            }

            self.buttons[idx] = byte;
        }

        if self.buttons[4] > 0xF {
            self.buttons[4] = buf[4];
            return
        } else if self.buttons[4] == buf[4] {
            return;
        }

        if ((self.buttons[4] + 1) & 0xF) == buf[4] {
            handler.encoder_step(self, 0, 1);
        } else {
            handler.encoder_step(self, 0, -1);
        }

        self.buttons[4] = buf[4];
    }

    fn read_pads(&mut self, handler: &mut MaschineHandler, buf: &[u8]) {
        let pads: &[u16] = unsafe { transmute(buf) };

        for i in 0..16 {
            let pressure = ((pads[i] & 0xFFF) as f32) / 4095.0;

            match self.pads[i].pressure_val(pressure) {
                MaschinePadStateTransition::Pressed =>
                    handler.pad_pressed(self, i, pressure),

                MaschinePadStateTransition::Aftertouch =>
                    handler.pad_aftertouch(self, i, pressure),

                MaschinePadStateTransition::Released =>
                    handler.pad_released(self, i),

                _ => {}
            }
        }
    }
}

fn set_rgb_light(rgb: &mut [u8], color: u32, brightness: f32) {
    let brightness = brightness * 0.5;

    rgb[0] = (brightness * (((color >> 16) & 0xFF) as f32)) as u8;
    rgb[1] = (brightness * (((color >>  8) & 0xFF) as f32)) as u8;
    rgb[2] = (brightness * (((color      ) & 0xFF) as f32)) as u8;
}

impl Maschine for Mikro {
    fn get_io(&mut self) -> &mut mio::Io {
        return &mut self.dev;
    }

    fn write_lights(&mut self) {
        self.dev.write(&mut mio::buf::SliceBuf::wrap(&self.light_buf))
            .unwrap();
    }

    fn set_pad_light(&mut self, pad: usize, color: u32, brightness: f32) {
        let offset = 31 + (pad * 3);
        let rgb = &mut self.light_buf[offset .. (offset + 3)];

        set_rgb_light(rgb, color, brightness);
    }

    fn set_button_light(&mut self, btn: MaschineButton, brightness: f32) {
        let idx = match btn {
            MaschineButton::F1 => 1,
            MaschineButton::F2 => 2,
            MaschineButton::F3 => 3,
            MaschineButton::Control => 4,
            MaschineButton::Nav => 5,
            MaschineButton::NavLeft => 6,
            MaschineButton::NavRight => 7,
            MaschineButton::Main => 8,

            MaschineButton::Group => 9, // 9, 10, 11 make up rgb pair
            MaschineButton::Browse => 12,
            MaschineButton::Sampling => 13,
            MaschineButton::NoteRepeat => 14,

            MaschineButton::Restart => 15,
            MaschineButton::StepLeft => 16,
            MaschineButton::StepRight => 17,
            MaschineButton::Grid => 18,
            MaschineButton::Play => 19,
            MaschineButton::Rec => 20,
            MaschineButton::Erase => 21,
            MaschineButton::Shift => 22,

            MaschineButton::Scene => 23,
            MaschineButton::Pattern => 24,
            MaschineButton::PadMode => 25,
            MaschineButton::View => 26,
            MaschineButton::Duplicate => 27,
            MaschineButton::Select => 28,
            MaschineButton::Solo => 29,
            MaschineButton::Mute => 30,

            _ => {
                // happens for buttons which don't have a light (such as the encoder).
                // could instead return a Result indicating when something such as this
                // happens, but whatever.

                return
            }
        };

        self.light_buf[idx] = (brightness * 255.0) as u8;
    }

    fn readable(&mut self, handler: &mut MaschineHandler) {
        let mut buf = [0u8; 256];

        let nbytes = match self.dev.read(&mut mio::buf::MutSliceBuf::wrap(&mut buf)) {
            Err(err) => panic!("read failed: {}", Error::description(&err)),
            Ok(nbytes) => nbytes.unwrap()
        };

        let report_nr = buf[0];
        let buf = &buf[1 .. nbytes];

        match report_nr {
            0x01 => self.read_buttons(handler, &buf),
            0x20 => self.read_pads(handler, &buf),
            _ => println!(" :: {:2X}: got {} bytes", report_nr, nbytes)
        }
    }

    fn get_pad_pressure(&mut self, pad_idx: usize) -> Result<f32, ()> {
        match pad_idx {
            0 ... 15 => Ok(self.pads[pad_idx].get_pressure()),
            _ => Err(())
        }
    }

    fn clear_screen(&mut self) {
        let mut screen_buf = [0u8; 1 + 8 + 256];

        screen_buf[0] = 0xE0;

        screen_buf[5] = 0x20;
        screen_buf[7] = 0x08;

        for i in 0..4 {
            screen_buf[1] = i * 32;
            self.dev.write(&mut mio::buf::SliceBuf::wrap(&screen_buf))
                .unwrap();
        }
    }
}
