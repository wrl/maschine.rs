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

    MaschinePad,
    MaschinePadStateTransition
};

#[allow(dead_code)]
enum MikroButtons {
    Restart = 0,
    StepLeft,
    StepRight,
    Grid,
    Play,
    Rec,
    Erase,
    Shift,

    Browse,
    Sampling,
    Group,
    NoteRepeat,
    Encoder,

    F1,
    F2,
    F3,
    Main,
    Nav,
    NavLeft,
    NavRight,
    Enter,

    Scene,
    Pattern,
    PadMode,
    View,
    Duplicate,
    Select,
    Solo,
    Mute
}

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
        if buf[4] > 0xF {
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

        let brightness = brightness * 0.5;

        rgb[0] = (brightness * (((color >> 16) & 0xFF) as f32)) as u8;
        rgb[1] = (brightness * (((color >>  8) & 0xFF) as f32)) as u8;
        rgb[2] = (brightness * (((color      ) & 0xFF) as f32)) as u8;
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
