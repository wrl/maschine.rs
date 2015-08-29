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

use std::collections::VecDeque;
use std::cmp::Ordering::Equal;

// XXX: need smarter debouncing
const THRESHOLD: f32 = 32.0 / 4096.0;
const MEDIAN_KERNEL_LENGTH: usize = 15;

#[derive(Copy, Clone, Debug)]
enum MaschinePadState {
    Unpressed = 0,
    PressedBelowThreshold,
    PressedAboveThreshold
}

#[derive(Copy, Clone, Debug)]
pub enum MaschinePadStateTransition {
    AtRest,
    Pressed,
    Aftertouch,
    Released
}

#[derive(Clone)]
pub struct MaschinePad {
    state: MaschinePadState,
    pressure: VecDeque<f32>
}

impl Default for MaschinePad {
    fn default() -> Self {
        let mut _self = MaschinePad {
            state: MaschinePadState::Unpressed,
            pressure: VecDeque::with_capacity(MEDIAN_KERNEL_LENGTH)
        };

        for _ in (0..MEDIAN_KERNEL_LENGTH) {
            _self.pressure.push_back(0.0);
        }

        _self
    }
}

impl MaschinePad {
    fn filtered_pressure(&self) -> f32 {
        let mut vals: Vec<_> = self.pressure.iter().take(MEDIAN_KERNEL_LENGTH).collect();
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));

        let middle = MEDIAN_KERNEL_LENGTH / 2;

        if (MEDIAN_KERNEL_LENGTH & 1) == 1 {
            // odd
            *(vals[middle])
        } else {
            // even
            (*(vals[middle]) + *(vals[middle + 1])) / 2.0
        }
    }

    pub fn pressure_val(&mut self, pressure: f32) -> MaschinePadStateTransition {
        self.pressure.pop_front();
        self.pressure.push_back(pressure);

        let pressure = self.filtered_pressure();

        match self.state {
            MaschinePadState::Unpressed =>
                if pressure > THRESHOLD {
                    self.state = MaschinePadState::PressedAboveThreshold;
                    return MaschinePadStateTransition::Pressed;
                } else if pressure > 0.0 {
                    self.state = MaschinePadState::PressedBelowThreshold;
                },

            MaschinePadState::PressedBelowThreshold =>
                if pressure == 0.0 {
                    self.state = MaschinePadState::Unpressed;
                },

            MaschinePadState::PressedAboveThreshold =>
                if pressure == 0.0 {
                    self.state = MaschinePadState::Unpressed;
                    return MaschinePadStateTransition::Released;
                } else {
                    return MaschinePadStateTransition::Aftertouch;
                },
        }

        return MaschinePadStateTransition::AtRest;
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self) -> bool {
        match self.state {
            MaschinePadState::PressedAboveThreshold => true,
            _ => false
        }
    }

    #[allow(dead_code)]
    pub fn get_pressure(&self) -> f32 {
        match self.state {
            MaschinePadState::PressedAboveThreshold => self.filtered_pressure(),
            _ => 0.0
        }
    }
}
