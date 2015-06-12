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

extern crate libc;

#[macro_use]
extern crate bitflags;

extern crate midi;
extern crate alsa_sys;

use libc::*;
use alsa_sys::*;

pub struct SequencerHandle {
    raw_handle: *mut snd_seq_t
}

pub struct SequencerPort<'handle> {
    raw_handle: c_int,
    handle: &'handle SequencerHandle
}

#[derive(Debug)]
pub enum Error {
    Unknown
}

mod handle;
mod port;

pub use handle::{
    HandleOpenStreams,
};

pub use port::{
    PortType,
    PortCapabilities,

    PORT_CAPABILITY_DUPLEX,
    PORT_CAPABILITY_NO_EXPORT,
    PORT_CAPABILITY_READ,
    PORT_CAPABILITY_SUBS_READ,
    PORT_CAPABILITY_SUBS_WRITE,
    PORT_CAPABILITY_SYNC_READ,
    PORT_CAPABILITY_SYNC_WRITE
};

mod event;

mod test;
