# ScaleFX-rs

ScaleFX pixel art upcaler, on the CPU (not a shader!), in Rust.

![Blooguard](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blooguard.png)

![Blooguard](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blooguard.big.png)

To use: `cargo run in.png out.png`

Original shader algorithm thanks to Sp00kyFox, 2016.

Check out my Typescript / Javascript port too: https://github.com/chrishulbert/scalefx-js

## Why?

![Council](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Council.png)

![Council](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Council.big.png)

ScaleFX already exists as a far-faster GPU shader, so why this project? Fair question. My reasons:

* As a learning exercise.
* For offline upscaling of assets, as opposed to realtime.
* Looks better than XBRZ!

## How to use this in your code

![RoboRed](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/RoboRed.png)

![RoboRed](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/RoboRed.big.png)

Simply copy `scalefx.rs` into your project, and call `scalefx::scale3x(width, height, pixels)`, where pixels is a slice of u32, containing 0xRRGGBBAA data.

If anybody out there actually uses this, we can have a conversation about uplifting this into a proper crate :) 

## Examples

![SQ3](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/sq3.fixed.png)

![SQ3](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/sq3.fixed.big.png)

![Arachnut](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Arachnut.png)

![Arachnut](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Arachnut.big.png)

![Berkeloid](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Berkeloid.png)

![Berkeloid](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Berkeloid.big.png)

![Bird](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bird.png)

![Bird](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bird.big.png)

![Bounder](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bounder.png)

![Bounder](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bounder.big.png)

![Cloud](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Cloud.png)

![Cloud](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Cloud.big.png)

![Diving](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Diving.png)

![Diving](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Diving.big.png)

![Dopefish](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Dopefish.png)

![Dopefish](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Dopefish.big.png)

![Fish](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Fish.png)

![Fish](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Fish.big.png)

![Keen](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Keen.png)

![Keen](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Keen.big.png)

![Lick](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Lick.png)

![Lick](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Lick.big.png)

![Mushroom](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Mushroom.png)

![Mushroom](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Mushroom.big.png)

![Princess](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Princess.png)

![Princess](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Princess.big.png)

![Rock](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Rock.png)

![Rock](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Rock.big.png)

![Skypest](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Skypest.png)

![Skypest](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Skypest.big.png)

![Slug](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Slug.png)

![Slug](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Slug.big.png)

![Sprite](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sprite.png)

![Sprite](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sprite.big.png)

![Thief](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Thief.png)

![Thief](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Thief.big.png)

![Worm](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Worm.png)

![Worm](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Worm.big.png)

![Wormouth](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Wormouth.png)

![Wormouth](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Wormouth.big.png)


![Ampton](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Ampton.png)

![Ampton](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Ampton.big.png)

![Korath](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Korath.png)

![Korath](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Korath.big.png)

![Mine](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Mine.png)

![Mine](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Mine.big.png)

![RoboRed](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/RoboRed.png)

![RoboRed](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/RoboRed.big.png)

![Shelley](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shelley.png)

![Shelley](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shelley.big.png)

![Shikadi](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shikadi.png)

![Shikadi](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shikadi.big.png)

![ShikadiMaster](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/ShikadiMaster.png)

![ShikadiMaster](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/ShikadiMaster.big.png)

![Shockshund](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shockshund.png)

![Shockshund](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Shockshund.big.png)

![Slicestar](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Slicestar.png)

![Slicestar](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Slicestar.big.png)

![Sparky](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sparky.png)

![Sparky](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sparky.big.png)

![Sphereful](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sphereful.png)

![Sphereful](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Sphereful.big.png)

![Spindred](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Spindred.png)

![Spindred](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Spindred.big.png)

![Spirogrip](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Spirogrip.png)

![Spirogrip](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Spirogrip.big.png)

![VolteFace](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/VolteFace.png)

![VolteFace](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/VolteFace.big.png)


![Babobba](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Babobba.png)

![Babobba.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Babobba.big.png)

![Bip](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bip.png)

![Bip.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bip.big.png)

![BipGuy](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BipGuy.png)

![BipGuy.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BipGuy.big.png)

![Bloog](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bloog.png)

![Bloog.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bloog.big.png)

![BloogletB](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletB.png)

![BloogletB.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletB.big.png)

![BloogletG](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletG.png)

![BloogletG.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletG.big.png)

![BloogletR](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletR.png)

![BloogletR.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletR.big.png)

![BloogletY](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletY.png)

![BloogletY.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/BloogletY.big.png)

![Blooguard](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blooguard.png)

![Blooguard.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blooguard.big.png)

![Blorb](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blorb.png)

![Blorb.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Blorb.big.png)

![Bobba](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bobba.png)

![Bobba.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Bobba.big.png)

![Ceilick](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Ceilick.png)

![Ceilick.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Ceilick.big.png)

![Flect](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Flect.png)

![Flect.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Flect.big.png)

![Fleex](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Fleex.png)

![Fleex.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Fleex.big.png)

![Gik](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Gik.png)

![Gik.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Gik.big.png)

![Grabbiter](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Grabbiter.png)

![Grabbiter.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Grabbiter.big.png)

![Molly](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Molly.png)

![Molly.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Molly.big.png)

![Nospike](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Nospike.png)

![Nospike.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Nospike.big.png)

![Orbatrix](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Orbatrix.png)

![Orbatrix.big](https://github.com/chrishulbert/scalefx-rs/raw/main/readme/Orbatrix.big.png)

## References

* https://github.com/libretro/slang-shaders/tree/master/edge-smoothing/scalefx/shaders
* http://www.compuphase.com/cmetric.htm
* https://docs.libretro.com/development/shader/slang-shaders/#pragma-parameter
