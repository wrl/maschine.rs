// Copyright (c) 2015 William Light <wrl@illest.net>
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use libc::{
    c_uint,
    c_uchar
};

use midi::*;
use alsa_sys::*;

//
// extra seq.h constants that alsa-sys missed
//

const SND_SEQ_QUEUE_DIRECT: c_uchar = 253;

const SND_SEQ_ADDRESS_SUBSCRIBERS: c_uchar = 254;
const SND_SEQ_ADDRESS_UNKNOWN: c_uchar = 253;

const SND_SEQ_EVENT_LENGTH_MASK: c_uint = (3 << 2);
const SND_SEQ_EVENT_LENGTH_FIXED: c_uint = (0 << 2);

pub trait TransliteratedFromCMacros {
    fn set_fixed(&mut self);
    fn set_direct(&mut self);
    fn set_subs(&mut self);
    fn set_source(&mut self, port: c_uchar);

    fn set_note(&mut self, _type: c_uint, channel: Channel, note_number: u8, velocity: u8,
                duration: c_uint);
}

impl TransliteratedFromCMacros for snd_seq_event_t {
    #[inline]
    fn set_fixed(&mut self) {
        self.flags &= !(SND_SEQ_EVENT_LENGTH_MASK as u8);
        self.flags |= SND_SEQ_EVENT_LENGTH_FIXED as u8;
    }

    #[inline]
    fn set_direct(&mut self) {
        self.queue = SND_SEQ_QUEUE_DIRECT;
    }

    #[inline]
    fn set_subs(&mut self) {
        self.dest.client = SND_SEQ_ADDRESS_SUBSCRIBERS;
        self.dest.port = SND_SEQ_ADDRESS_UNKNOWN;
    }

    #[inline]
    fn set_source(&mut self, port: c_uchar) {
        self.source.port = port;
    }

    #[inline]
    fn set_note(&mut self, _type: c_uint, channel: Channel, note_number: u8, velocity: u8,
                duration: c_uint) {
        self._type = _type as snd_seq_event_type_t;
        self.set_fixed();

        let note = self.data.note();
        unsafe {
            (*note).channel = (channel as c_uchar) + 1;
            (*note).note = note_number;
            (*note).velocity = velocity;
            (*note).duration = duration;
        }
    }

}

pub trait ToSndSeqEvent {
    fn to_snd_seq_event(&self) -> Option<snd_seq_event_t>;
}

impl ToSndSeqEvent for Message {
    fn to_snd_seq_event(&self) -> Option<snd_seq_event_t> {
        let mut ev = snd_seq_event_t {
            _type: 0,
            flags: 0,
            tag: 0,
            queue: 0,

            time: snd_seq_timestamp_t { 
                data: [0; 2]
            },

            source: snd_seq_addr_t {
                client: 0,
                port: 0
            },

            dest: snd_seq_addr_t {
                client: 0,
                port: 0
            },

            data: Union_Unnamed10 {
                data: [0; 3]
            }
        };

        match *self {
            Message::NoteOn(channel, note_number, velocity) =>
                ev.set_note(SND_SEQ_EVENT_NOTEON, channel, note_number, velocity, 0),

            Message::NoteOff(channel, note_number, velocity) =>
                ev.set_note(SND_SEQ_EVENT_NOTEOFF, channel, note_number, velocity, 0),

            Message::PolyphonicPressure(channel, note_number, velocity) =>
                ev.set_note(SND_SEQ_EVENT_KEYPRESS, channel, note_number, velocity, 0),

            _ => return None
        }

        Some(ev)
    }
}
