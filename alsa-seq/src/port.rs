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

use libc::*;

use midi::*;
use alsa_sys::*;

use {
    SequencerPort,
    Error,
};

use event::{
    ToSndSeqEvent,
    TransliteratedFromCMacros
};

bitflags! {
    pub flags PortCapabilities: u32 {
        const PORT_CAPABILITY_DUPLEX = (1 << 4),
        const PORT_CAPABILITY_NO_EXPORT = (1 << 7),
        const PORT_CAPABILITY_READ = (1 << 0),
        const PORT_CAPABILITY_SUBS_READ = (1 << 5),
        const PORT_CAPABILITY_SUBS_WRITE = (1 << 6),
        const PORT_CAPABILITY_SYNC_READ = (1 << 2),
        const PORT_CAPABILITY_SYNC_WRITE = (1 << 3)
    }
}

#[repr(C)]
pub enum PortType {
    Application = (1 << 20),
    DirectSample = (1 << 11),
    Hardware = (1 << 16),
    MidiGeneric = (1 << 1),
    MidiGM = (1 << 2),
    MidiGM2 = (1 << 6),
    MidiGS = (1 << 3),
    MidiMT32 = (1 << 5),
    MidiXG = (1 << 4),
    Port = (1 << 19),
    Sample = (1 << 12),
    Software = (1 << 17),
    Specific = (1 << 0),
    Synth = (1 << 10),
    Synthesizer = (1 << 18)
}

impl<'handle> SequencerPort<'handle> {
    pub fn send_message(&self, msg: &Message) -> Result<(), Error> {
        let mut ev = match msg.to_snd_seq_event() {
            Some(ev) => ev,
            None => return Err(Error::Unknown)
        };

        ev.set_direct();
        ev.set_subs();

        ev.set_source(self.raw_handle as c_uchar);

        unsafe {
            match snd_seq_event_output(self.handle.raw_handle, &mut ev) {
                err_code @ _ if err_code < 0 => return Err(Error::Unknown),
                _ => {}
            }
        }

        Ok(())
    }
}

impl<'handle> Drop for SequencerPort<'handle> {
    fn drop(&mut self) {
        if self.raw_handle < 0 {
            return;
        }

        unsafe {
            snd_seq_delete_simple_port(self.handle.raw_handle, self.raw_handle);
        }
    }
}
