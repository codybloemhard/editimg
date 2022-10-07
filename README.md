# editimg

## What
Edit images.

## Why
I got tired of using Gimp. It's a fine program that can do lot's of things.
It's a GUI program with a workflow similar to many windows and mac software.
You'll have to click around in menu's and widgets.
This breaks the keyboard based workflow of tilling window managers, vim, etc.
I also tend to do the same tasks often such as cropping an image.
This can be done fine in Gimp but one could image a more efficient
workflow and user experience for specific tasks.

## How (just plans for now)
Combine CLI, TUI and GUI to get a better workflow.
Start up in CLI choosing input files and options such as a specific workflow.
Editimg turns into a TUI in the terminal you launched it and it creates a GUI window.
This window expects to be resized by your TWM.
Simple minimal inputs can be given in the GUI and more complex ones in the TUI.
The GUI shows the (intermediate) results.
The TUI should be a REPL and take in Rhai.
All functionality should be callable from Rhai.
Scripts can be build to automate editing images.
Scripts can be written to design workflows that can be launced.

## Goals

- Inputs
  - [ ] Cli interface with subcommands
  - [ ] Configurable keybindings
  - TUI
    - [ ] Number input
    - [ ] Text input
    - [ ] REPL
  - GUI
    - [ ] Point click
    - [ ] Rectangle select
    - [ ] Confirm
    - [ ] Return to REPL
    - [ ] Move around
    - [ ] Zoom
- Scripting
  - [ ] Rhai embedding
  - [ ] Wizards to take input
  - Functions
    - [ ] Export
    - [ ] New
    - [ ] Copy
    - [ ] Paste
    - [ ] Crop
    - [ ] Resize
    - [ ] Flip
    - [ ] Rotate
    - [ ] Tile
    - [ ] Blur
    - [ ] Greyscale
    - [ ] Unsharpen
    - [ ] Filter
    - [ ] Noise
- Show image in GUI
  - [x] TWM friendly window
  - [x] Resize image in window
  - [ ] Padding

## License

```
Copyright (C) 2022 Cody Bloemhard

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
