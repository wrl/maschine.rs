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

use std::ptr::null_mut;
use std::ffi;

use alsa_sys::*;

use {
    SequencerHandle,
    SequencerPort,
    Error,
    PortType,
    PortCapabilities
};

#[repr(C)]
pub enum HandleOpenStreams {
    Output = 1,
    Input = 2,
    Duplex = 3
}

impl SequencerHandle {
    pub fn open(name: &str, streams: HandleOpenStreams) -> Result<Self, Error> {
        let cstr = match ffi::CString::new(name) {
            Ok(cstr) => cstr,
            Err(_) => return Err(Error::Unknown)
        };

        let mut inst = SequencerHandle {
            raw_handle: null_mut()
        };

        unsafe {
            let err = snd_seq_open(&mut inst.raw_handle, b"default\0".as_ptr() as *const i8,
                                   streams as i32, 0);

            if err != 0 {
                return Err(Error::Unknown);
            }

            if snd_seq_set_client_name(inst.raw_handle, cstr.as_ptr()) != 0 {
                return Err(Error::Unknown);
            } else {
                return Ok(inst);
            }
        }
    }

    pub fn create_port(&self, name: &str, capabilities: PortCapabilities, port_type: PortType) 
        -> Result<SequencerPort, Error> {
        let cstr = match ffi::CString::new(name) {
            Ok(cstr) => cstr,
            Err(_) => return Err(Error::Unknown)
        };

        let mut port = SequencerPort {
            raw_handle: -1,
            handle: self
        };

        unsafe {
            let port_nr = snd_seq_create_simple_port(self.raw_handle, cstr.as_ptr(),
                capabilities.bits(), port_type as u32);

            if port_nr < 0 {
                return Err(Error::Unknown);
            } else {
                port.raw_handle = port_nr;
                return Ok(port);
            }
        }
    }

    pub fn drain_output(&self) {
        unsafe {
            snd_seq_drain_output(self.raw_handle);
        }
    }
}

impl Drop for SequencerHandle {
    fn drop(&mut self) {
        if self.raw_handle.is_null() {
            return;
        }

        unsafe {
            snd_seq_close(self.raw_handle);
        }
    }
}
