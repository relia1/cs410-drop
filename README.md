# CS410-drop
Robert Elia


## Description
Using a microbit v2, accelerometer, and the led display show 2 states for whether the microbit is falling or not.
When the microbit is falling generate a square wave and display an exclamation mark with the display.
When the microbit is stable the speaker should be off and the display should show a single dot

## What I did
Using the Rust crates for the microbit/nrf52833 and lsm303agr the accelerometer is configured
using i2c. We loop until we have a certain number of samples only taking samples from the 
accelerometer when there is new data. Depending on the what we calculate from the acceleration 
sample we either stay in the state we are in or we transition states.

## How it went
This was a fun project, but I think I ended up sinking too much time into trying to figure out
getting the lsm303agr interrupt setup. This led to a pretty big rework close to the deadline
with it not being as clean as I would like, and some sections not being ideal e.g. the speaker
turning on/off part being unsafe.
