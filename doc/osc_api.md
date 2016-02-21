Maschine.rs OSC API
===================
This document describes the OSC API that maschine.rs exposes. It is written
in the hope that anybody implementing integration with the maschine
hardware can do so quickly and easily.

Some devices in the Maschine family have more buttons and capabilities,
these are presented in sub-sections for each device.

For testing OSC commands, the `oscsend` (a CLI OSC message sender
tool) comes with the `liblo` package on most distros. It is a simple CLI
OSC sending program, and will be used throughout this document for
examples. As oscsend does *not* accept hex values, they are noted below in
decimal.

Setting On/Off and Brightness
-----------------------------
Most of the buttons on the Maschine are just one colour: white.
To turn these buttons on or off, we set the brightness.

For example, to turn on or off the "group" button, the following two
commands do just that:
```
oscsend localhost 42434 /maschine/button/group i 1
oscsend localhost 42434 /maschine/button/group i 0
```

To set the brightness count "backwards" from 0x7F (decimal 127):
```
min:  oscsend localhost 42434 /maschine/button/group i 127
low:  oscsend localhost 42434 /maschine/button/group i 100
mid:  oscsend localhost 42434 /maschine/button/group i 60
hi :  oscsend localhost 42434 /maschine/button/group i 30
max:  oscsend localhost 42434 /maschine/button/group i 1
```

RGB buttons and Pads
--------------------
Group button has RGB support and uses white if just turned on. Colours are
composed of 3 bytes: Red-Green-Blue:

Translating the above (easily bit-shifted) numbers, we get this to test:
```
* Blue  0x0000FF
oscsend localhost 42434 /maschine/button/group if 255       1

* Green 0x00FF00
oscsend localhost 42434 /maschine/button/group if 32768     1

* Red   0xFF0000
oscsend localhost 42434 /maschine/button/group if 8388608   1
```

Exception Buttons
-----------------
There are a few buttons that are only ever a specifc colour:
* Play button *always* green
* Rec button *always* red
