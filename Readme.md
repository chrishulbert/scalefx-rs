# ScaleFX-rs

ScaleFX pixel art upcaler, on the CPU (not a shader!), in Rust.

![Blooguard](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/Blooguard.png)

![Blooguard](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/Blooguard.big.png)

To use: `cargo run in.png out.png`

Original shader algorithm thanks to Sp00kyFox, 2016.

## Why?

![Council](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/Council.png)

![Council](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/Council.big.png)

ScaleFX already exists as a far-faster GPU shader, so why this project? Fair question. My reasons:

* As a learning exercise.
* For offline upscaling of assets, as opposed to realtime.
* Looks better than XBRZ!

## How to use this in your code

![RoboRed](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/RoboRed.png)

![RoboRed](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/RoboRed.big.png)

Simply copy `scalefx.rs` into your project, and call `scalefx::scale3x(width, height, pixels)`, where pixels is a slice of u32, containing 0xRRGGBBAA data.

If anybody out there actually uses this, we can have a conversation about uplifting this into a proper crate :) 

## Examples

![SQ3](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/sq3.fixed.png)

![SQ3](https://github.com/chrishulbert/cpu-scalefx-rs/raw/main/readme/sq3.fixed.big.png)

## References

* https://github.com/libretro/slang-shaders/tree/master/edge-smoothing/scalefx/shaders
* http://www.compuphase.com/cmetric.htm
* https://docs.libretro.com/development/shader/slang-shaders/#pragma-parameter
