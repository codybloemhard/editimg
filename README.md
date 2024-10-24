# editimg

## What

Edit images.

- for TWM users
- scriptable
- focus on (keyboard heavy) UX
- do simple/repetitive things quickly
- not for advanced stuff

## Why

I got tired of using Gimp. It's a fine program that can do lot's of things.
It's a GUI program with a workflow similar to many windows and mac software.
You'll have to click around in menu's and widgets.
This breaks the keyboard based workflow of tilling window managers, vim, etc.
I also tend to do the same tasks often such as cropping an image.
This can be done fine in Gimp but one could image a more efficient
workflow and user experience for specific tasks.

## How

Combine CLI, TUI and GUI to get a better workflow.
Start up in CLI choosing input files and options such as a specific workflow.
Editimg turns into a TUI in the terminal you launched it and it creates a GUI window.
This window expects to be resized by your TWM.
Simple minimal inputs can be given in the GUI and more complex ones in the TUI.
The GUI shows the (intermediate) results.
The TUI should be a REPL and take in Rhai.
All functionality should be callable from Rhai.
Scripts can be build to automate editing images.
Scripts can be written to design workflows that can be launched.

## Goals

- inputs
  - [x] CLI interface
  - GUI
    - [x] TWM friendly window
    - [x] Resize image in window
    - [x] show current buffer
    - [x] mouse input
    - [x] keyboard input
    - [x] draw rectangles
    - [ ] move around
    - [ ] zoom
- scripting
  - [x] rhai embedding
  - [x] Wizards to take input (in script)
  - [ ] headless mode

## Features

General functions:
- kill: kills session
- get_input_event: returns Input
- get_wh: returns WH
- clear_rects: clears all rectangles on the screen
- draw_rect_uv(px: f64, py: f64, qx: f64, qy: f64): draw rectangle with UV coordinates
- draw_rect_xy(px: f64, py: f64, qx: f64, qy: f64): draw rectangle with pixel coordinates
- show(img: i64): show image buffer on screen
- show_next: show next image buffer on screen
- show_prev: show previous image buffer on screen
- shown: return index of image buffer currently on screen
- repeat: repeat last image function

Image functions:
- crop(src: i64, dst: i64, px: i64, py: i64, qx: i64, qy: i64):
 take section defined by p and q from src to dst
- save(img: i64, filename: String): save image buffer as file
- fliph(src: i64, dst: i64): flip image horizontally
- flipv(src: i64, dst: i64): flip image vertically
- rotate90(src: i64, dst: i64): rotate image 90 degrees clockwise
- rotate180(src: i64, dst: i64): rotate image 180 degrees clockwise
- rotate270(src: i64, dst: i64): rotate image 270 degrees clockwise
- invert(src: i64, dst: i64): invert colours
- grayscale(src: i64, dst: i64): turn coloured image into grayscale image
- blur(src: i64, dst: i64, sigma: f64): gaussian blur
- unsharpen(src: i64, dst: i64, sigma: f64, threshold: i64): unsharpen filter
- filter3x3(src: i64, dst: i64, filter: [f64; 9]): apply general linear filter
- adjust_contrast(src: i64, dst: i64, dt: f64): change contrast
- brighten(src: i64, dst: i64, dt: i64): change brightness
- huerotate(src: i64, dst: i64, dt: i64): rotate hue (in degrees)
- resize(src: i64, dst: i64, w: i64, h: i64, method: String): resize image to maxium dimensions
- resize_exact(src: i64, dst: i64, w: i64, h: i64, method: String): resize image to exact dimensions
- resize_fill(src: i64, dst: i64, w: i64, h: i64, method: String): resize image to fill dimensions
- thumbnail(src: i64, dst: i64, w: i64, h: i64): like resize, but with quick algorithm
- thumbnail_exact(src: i64, dst: i64, w: i64, h: i64): like resize_exact, but with quick algorithm
- create(w: i64, h: i64): create new image buffer with given dimensions
- copy(src: i64, dst: i64, x: i64, y: i64): copy image to destination with coordinates

Datatypes:
- Input
  - is_click: bool, whether this is a mouse click
  - key: String, what key or button was pressed
  - u: f64, u component of UV coordinate of the mouse position ([0..1])
  - v: f64, v component of UV coordinate of the mouse position ([0..1])
  - x: i64, x component of the XY coordinate of the mouse position (in pixels)
  - y: i64, y component of the XY coordinate of the mouse position (in pxiels)
  - shift: bool, whether shift was held while pressing this key
  - control: bool, whether control was held while pressing this key
  - alt: bool, whether alt was held while pressing this key
  - nummod: bool, whether numlock was active while pressing this key
  - capsmod: bool, whether capslock was active while pressing this key
- WH
  - w: i64, the width of the current image buffer in pixels
  - h: i64, the height of the current image buffer in pixels

## Issues

- Keyboard input completely breaks when using latest sdl2 0.37.0?
- Cropping a selection of size 0 crashes the application

## License

```
Copyright (C) 2024 Cody Bloemhard

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
