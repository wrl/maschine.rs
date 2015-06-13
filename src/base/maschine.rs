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

extern crate mio;

#[derive(Copy,Clone,Debug)]
pub enum MaschineButton {
    Restart,
    StepLeft,
    StepRight,
    Grid,
    Play,
    Rec,
    Erase,
    Shift,

    Group,
    Browse,
    Sampling,
    NoteRepeat,

    Encoder,

    F1,
    F2,
    F3,
    Control,
    Nav,
    NavLeft,
    NavRight,
    Main,

    Scene,
    Pattern,
    PadMode,
    View,
    Duplicate,
    Select,
    Solo,
    Mute
}

pub trait Maschine {
    fn get_io(&mut self) -> &mut mio::Io;

    fn get_pad_pressure(&mut self, pad_idx: usize) -> Result<f32, ()>;

    fn set_pad_light(&mut self, pad_idx: usize, color: u32, brightness: f32);
    fn set_button_light(&mut self, btn: MaschineButton, color: u32, brightness: f32);

    fn readable(&mut self, &mut MaschineHandler);

    fn clear_screen(&mut self);
    fn write_lights(&mut self);
}

#[allow(unused_variables)]
pub trait MaschineHandler {
    fn pad_pressed(&mut self, &mut Maschine, pad_idx: usize, pressure: f32) {}
    fn pad_aftertouch(&mut self, &mut Maschine, pad_idx: usize, pressure: f32) {}
    fn pad_released(&mut self, &mut Maschine, pad_idx: usize) {}

    fn encoder_step(&mut self, &mut Maschine, encoder_idx: usize, delta: i32) {}

    fn button_down(&mut self, &mut Maschine, button: MaschineButton) {}
    fn button_up(&mut self, &mut Maschine, button: MaschineButton) {}
}
