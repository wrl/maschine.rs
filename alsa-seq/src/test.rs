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

#[allow(unused_imports)]
use super::*;

#[test]
fn test_seq_open() {
    let handle = SequencerHandle::open("alsa-seq.rs", HandleOpenStreams::Output).unwrap();
}

#[test]
fn test_seq_create_port() {
    let handle = SequencerHandle::open("alsa-seq.rs", HandleOpenStreams::Output).unwrap();

    let port = handle.create_port("test output",
                                  PORT_CAPABILITY_READ | PORT_CAPABILITY_SUBS_READ,
                                  PortType::MidiGeneric).unwrap();
    let port2 = handle.create_port("test output",
                                  PORT_CAPABILITY_READ | PORT_CAPABILITY_SUBS_READ,
                                  PortType::MidiGeneric).unwrap();
}
