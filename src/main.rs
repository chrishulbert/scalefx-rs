mod png;

// ScaleFX options:
const THRESHOLD: f32 = 0.5; // Min 0.01; max: 1; step: 0.01
const IS_FILTER_AA_ENABLED: bool = true;
const FILTER_CORNERS: bool = true; // SFX_SCN in the shader.

#[derive(Debug)]
struct Image {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

// Make a sample pixel image.
fn sample() -> Image {
    let t = 0; // Transparent.
    let b = 0xff; // Black.
    let y = 0xffff00ff; // Yellow.
    let g = 0x00ff00ff; // Green.
    let l = 0x0000ffff; // bLue.
    let w = 0xffffffff; // White.
    let e = 0x888888ff; // grEy.

    Image {
        width: 12,
        height: 12,
        pixels: vec![
            t,t,t,b,b,b,b,b,b,t,t,t,
            t,t,b,y,g,g,w,g,y,b,t,t,
            t,b,y,y,y,y,g,w,g,y,b,t,
            b,g,e,y,y,y,g,w,g,y,y,b,
            b,y,y,y,y,y,g,w,g,y,y,b,
            b,y,g,y,e,e,e,e,e,e,y,b,
            b,e,g,y,w,l,w,w,l,w,y,b,
            b,b,y,y,w,w,e,e,w,w,y,b,
            b,y,y,e,w,w,w,w,w,w,e,b,
            t,b,e,e,e,w,w,w,w,e,b,t,
            t,t,b,b,e,e,e,e,b,b,t,t,
            t,t,t,t,b,e,e,b,t,t,t,t,
        ],
    }
}

const MAX_COLOUR_DISTANCE: f32 = 765.;

// Determine the human-perceived difference between two colours.
// For humanity's sake, r and g and b are weighted differently.
// https://www.compuphase.com/cmetric.htm
// Returns 0 for same colours; 764.83 for white-black; 765 if any are transparent.
fn colour_distance(a: u32, b: u32) -> f32 {
    let a_r = a >> 24;
    let a_g = (a >> 16) & 0xff;
    let a_b = (a >> 8) & 0xff;
    let a_a = a & 0xff;

    let b_r = b >> 24;
    let b_g = (b >> 16) & 0xff;
    let b_b = (b >> 8) & 0xff;
    let b_a = b & 0xff;

    if a_a < 0x80 || b_a < 0x80 { return MAX_COLOUR_DISTANCE } // Consider transparent very different.

    let r_mean = (a_r + b_r) / 2;
    let r = a_r.abs_diff(b_r);
    let g = a_g.abs_diff(b_g);
    let b = a_b.abs_diff(b_b);

    if r == 0 && g == 0 && b == 0 { return 0. } // Save the conplicated calculation below.

    (((((512 + r_mean)*r*r)>>8) + 4*g*g + (((767-r_mean)*b*b)>>8)) as f32).sqrt()
}

#[derive(Debug, Clone, Copy)]
struct PixelWithDistances {
    pixel: u32,
    colour_distance_up_left: f32, // X: Colour distance to the pixel to the up-left.
    colour_distance_up: f32, // Y in the shader.
    colour_distance_up_right: f32, // Z.
    colour_distance_right: f32, // W.
}
impl PixelWithDistances {
    fn offscreen() -> Self { // The representation for a transparent offscreen pixel.
        Self {
            pixel: 0,
            colour_distance_up_left: MAX_COLOUR_DISTANCE,
            colour_distance_up: MAX_COLOUR_DISTANCE,
            colour_distance_up_right: MAX_COLOUR_DISTANCE,
            colour_distance_right: MAX_COLOUR_DISTANCE,
        }
    }
}

#[derive(Debug)]
struct ImageWithDistances {
    width: usize,
    height: usize,
    pixels: Vec<PixelWithDistances>,
}

// Calculate the colour distances to neighbours.
// This implements pass 0 here:
// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass0.slang
fn calculate_distances(image: &Image) -> ImageWithDistances {
    let pixels_len = image.pixels.len();
    let mut pixels: Vec<PixelWithDistances> = Vec::with_capacity(pixels_len);

    for y in 0..image.height {
        for x in 0..image.width {
            let i = y * image.width + x;

            // Get the neighbouring pixels, returning transparent if they're out of bounds.
            let up_left: u32 = if y==0 || x==0 { 0 } else { image.pixels[i - image.width - 1] };
            let up: u32 = if y==0 { 0 } else { image.pixels[i - image.width] };
            let up_right: u32 = if y==0 || x==image.width-1 { 0 } else { image.pixels[i - image.width + 1] };
            let center = image.pixels[i];
            let right = if x==image.width-1 { 0 } else { image.pixels[i + 1] };

            pixels.push(PixelWithDistances {
                pixel: center,
                colour_distance_up_left: colour_distance(center, up_left),
                colour_distance_up: colour_distance(center, up),
                colour_distance_up_right: colour_distance(center, up_right),
                colour_distance_right: colour_distance(center, right),
            });
        }
    }
    ImageWithDistances {
        width: image.width,
        height: image.height,
        pixels,
    }
}

#[derive(Debug, Clone, Copy)]
struct PixelWithCornerStrengths {
    pixel: u32,
    colour_distance_up_left: f32, // Colour distance to the pixel to the up-left.
    colour_distance_up: f32,
    colour_distance_up_right: f32,
    colour_distance_right: f32,
    corner_strength_up_left: f32, // Corner strength. Called X in the shader.
    corner_strength_up_right: f32, // Y in the shader.
    corner_strength_down_right: f32, // Z in the shader.
    corner_strength_down_left: f32, // W in the shader.
}
impl PixelWithCornerStrengths {
    fn offscreen() -> Self { // The representation for a transparent offscreen pixel.
        Self {
            pixel: 0,
            colour_distance_up_left: MAX_COLOUR_DISTANCE,
            colour_distance_up: MAX_COLOUR_DISTANCE,
            colour_distance_up_right: MAX_COLOUR_DISTANCE,
            colour_distance_right: MAX_COLOUR_DISTANCE,
            corner_strength_up_left: 0.,
            corner_strength_up_right: 0.,
            corner_strength_down_right: 0.,
            corner_strength_down_left: 0.,
        }
    }

}


#[derive(Debug)]
struct ImageWithCornerStrengths {
    width: usize,
    height: usize,
    pixels: Vec<PixelWithCornerStrengths>,
}

// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass1.slang
fn corner_strength(d: f32, a_x: f32, a_y: f32, b_x: f32, b_y: f32) -> f32 {
	let diff = a_x - a_y;
	let weight_1 = (THRESHOLD - d).max(0.) / THRESHOLD;
    let is_x_g_y = a_x.min(b_x) + a_x  >  a_y.min(b_y) + a_y;
    let x_g_y_diff = if is_x_g_y { diff } else { -diff };
    let weight_2_raw = (1. - d) + x_g_y_diff;
	let weight_2 = weight_2_raw.clamp(0., 1.);
	if IS_FILTER_AA_ENABLED || 2. * d < a_x + a_y { weight_1 * weight_2 * a_x * a_y } else { 0. }
}

// Calculate all the corner strengths.
// Aka "calculate strength of interpolation candidates" according to the shader comment.
// This implements pass 1 here:
// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass1.slang
fn calculate_corner_strengths(image: &ImageWithDistances) -> ImageWithCornerStrengths {
    let pixels_len = image.pixels.len();
    let mut pixels: Vec<PixelWithCornerStrengths> = Vec::with_capacity(pixels_len);
    let offscreen = PixelWithDistances::offscreen();

    for y in 0..image.height {
        for x in 0..image.width {
            let i = y * image.width + x;

            // Get the neighbouring pixels, returning transparent if they're out of bounds.
            let up_left = if y==0 || x==0 { offscreen } else { image.pixels[i - image.width - 1] };
            let up = if y==0 { offscreen } else { image.pixels[i - image.width] };
            let left = if x==0 { offscreen } else { image.pixels[i - 1] };
            let center = image.pixels[i];
            let right = if x==image.width-1 { offscreen } else { image.pixels[i + 1] };
            let down_left = if x==0 || y==image.height-1 { offscreen } else { image.pixels[i + image.width - 1] };
            let down = if y==image.height-1 { offscreen } else { image.pixels[i + image.width] };
            let down_right = if x==image.width-1 || y==image.height-1 { offscreen } else { image.pixels[i + image.width + 1] };

            // Calculate the corner strengths:
            let up_left = corner_strength(left.colour_distance_up_right, left.colour_distance_right, center.colour_distance_up, up_left.colour_distance_right, left.colour_distance_up);
            let up_right = corner_strength(right.colour_distance_up_left, center.colour_distance_right, center.colour_distance_up, up.colour_distance_right, right.colour_distance_up);
            let down_right = corner_strength(down.colour_distance_up_right, center.colour_distance_right, down.colour_distance_up, down.colour_distance_right, down_right.colour_distance_up);
            let down_left = corner_strength(down.colour_distance_up_left, left.colour_distance_right, down.colour_distance_up, down_left.colour_distance_right, down_left.colour_distance_up);

            pixels.push(PixelWithCornerStrengths {
                pixel: center.pixel,
                colour_distance_up_left: center.colour_distance_up_left,
                colour_distance_up: center.colour_distance_up,
                colour_distance_up_right: center.colour_distance_up_right,
                colour_distance_right: center.colour_distance_right,
                corner_strength_up_left: up_left,
                corner_strength_up_right: up_right,
                corner_strength_down_right: down_right,
                corner_strength_down_left: down_left,
            });
        }
    }
    ImageWithCornerStrengths {
        width: image.width,
        height: image.height,
        pixels,
    }
}

#[derive(Debug, Copy, Clone)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone)]
struct BVec2 {
    x: bool,
    y: bool,
}

#[derive(Debug, Copy, Clone)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Copy, Clone)]
struct BVec4 {
    x: bool,
    y: bool,
    z: bool,
    w: bool,
}
impl BVec4 {
    fn zero() -> Self {
        Self { x: false, y: false, z: false, w: false }
    }
}

#[derive(Debug, Copy, Clone)]
struct U8Vec4 {
    x: u8,
    y: u8,
    z: u8,
    w: u8,
}
impl U8Vec4 {
    fn zero() -> Self {
        Self { x: 0, y: 0, z: 0, w: 0 }
    }
}

#[derive(Debug, Copy, Clone)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
impl Vec4 {
    fn zero() -> Self {
        Self { x: 0., y: 0., z: 0., w: 0. }
    }
    fn to_bvec(&self) -> BVec4 {
        BVec4 { x: self.x > 0.5, y: self.y > 0.5, z: self.z > 0.5, w: self.w > 0.5 }
    }
    // https://thebookofshaders.com/glossary/?search=step
    fn step(edge: Self, x: Self) -> Self {
        Self {
            x: if x.x < edge.x { 0. } else { 1. },
            y: if x.y < edge.y { 0. } else { 1. },
            z: if x.z < edge.z { 0. } else { 1. },
            w: if x.w < edge.w { 0. } else { 1. },
        }
    }
    fn le(a: Self, b: Self) -> Self {
        1.0f32 - Self::step(b, a)
    }
    fn ge(a: Self, b: Self) -> Self {
        1.0f32 - Self::step(a, b)
    }
    fn leq(a: Self, b: Self) -> Self {
        Self::step(a, b)
    }
    fn geq(a: Self, b: Self) -> Self {
        Self::step(b, a)
    }
    fn not(self) -> Self {
        1.0f32 - self
    }
    fn min(&self, a: f32) -> Self {
        Self {
            x: self.x.min(a),
            y: self.y.min(a),
            z: self.z.min(a),
            w: self.w.min(a),
        }
    }
    fn yzwx(&self) -> Self {
        Self { x: self.y, y: self.z, z: self.w, w: self.x }
    }
    fn wxyz(&self) -> Self {
        Self { x: self.w, y: self.x, z: self.y, w: self.z }
    }
    fn zwxy(&self) -> Self {
        Self { x: self.z, y: self.w, z: self.x, w: self.y }
    }
}
impl std::ops::Add for Vec4 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z, w: self.w + other.w }
    }
}
impl std::ops::Sub for Vec4 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z, w: self.w - other.w }
    }
}
impl std::ops::Mul for Vec4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self { // Shader * isn't a dot product, it simply multiplies components.
        Self { x: self.x * other.x, y: self.y * other.y, z: self.z * other.z, w: self.w * other.w }
    }
}
impl std::ops::Mul<Vec4> for f32 {
    type Output = Vec4;
    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self * rhs.x, y: self * rhs.y, z: self * rhs.z, w: self * rhs.w }
    }
}
impl std::ops::Sub<Vec4> for f32 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 { x: self - rhs.x, y: self - rhs.y, z: self - rhs.z, w: self - rhs.w }
    }
}

fn step_f32(edge: f32, x: f32) -> f32 {
    if x < edge { 0. } else { 1. }
}
fn ge_f32(x: f32, y: f32) -> f32 {
    1. - step_f32(x, y)
}


#[derive(Debug, Clone, Copy)]
struct PixelWithCornerConfiguration {
    pixel: u32,
    // colour_distance_up_left: f32, // X: Colour distance to the pixel to the up-left.
    // colour_distance_up: f32, // Y.
    // colour_distance_up_right: f32, // Z.
    // colour_distance_right: f32, // W.
    // corner_strength_up_left: f32, // Corner strength. Called X in the shader.
    // corner_strength_up_right: f32, // Y in the shader.
    // corner_strength_down_right: f32, // Z in the shader.
    // corner_strength_down_left: f32, // W in the shader.
    res: BVec4, // Resolution?
    horizontal_edges: BVec4,
    vertical_edges: BVec4,
    orientation: BVec4,
}
impl PixelWithCornerConfiguration {
    fn offscreen() -> Self {
        Self {
            pixel: 0,
            // colour_distance_up_left: MAX_COLOUR_DISTANCE,
            // colour_distance_up: MAX_COLOUR_DISTANCE,
            // colour_distance_up_right: MAX_COLOUR_DISTANCE,
            // colour_distance_right: MAX_COLOUR_DISTANCE,
            // corner_strength_up_left: 0.,
            // corner_strength_up_right: 0.,
            // corner_strength_down_right: 0.,
            // corner_strength_down_left: 0.,
            res: BVec4::zero(),
            horizontal_edges: BVec4::zero(),
            vertical_edges: BVec4::zero(),
            orientation: BVec4::zero(),
        }
    }
}

#[derive(Debug)]
struct ImageWithCornerConfigurations {
    width: usize,
    height: usize,
    pixels: Vec<PixelWithCornerConfiguration>,
}

// Resolve ambiguous configurations of corner candidates at pixel junctions.
// This implements pass 2 here:
// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass2.slang
fn resolve_corner_configurations(image: &ImageWithCornerStrengths) -> ImageWithCornerConfigurations {

    // Calculate corner dominance at junctions:
    fn corner_dominance(x: &Vec3, y: &Vec3, z: &Vec3, w: &Vec3) -> Vec4 {
        2.0f32 * Vec4{x: x.y, y: y.y, z: z.y, w: w.y} - (Vec4{x: x.x, y: y.x, z: z.x, w: w.x} + Vec4{x: x.z, y: y.z, z: z.z, w: w.z})
    }

    // Necessary but not sufficient junction condition for orthogonal edges.
    fn clear(crn: Vec2, a: Vec2, b: Vec2) -> f32 {
        if crn.x >= a.x.min(a.y).max(b.x.min(b.y)) && crn.y >= a.x.min(b.y).max(b.x.min(a.y)) { 1. } else { 0. }
    }

    let pixels_len = image.pixels.len();
    let mut pixels: Vec<PixelWithCornerConfiguration> = Vec::with_capacity(pixels_len);
    let offscreen = PixelWithCornerStrengths::offscreen();
    let zero = Vec4::zero();

    for y in 0..image.height {
        for x in 0..image.width {
            let index = y * image.width + x;

            // Get the neighbouring pixels, returning transparent if they're out of bounds.
            // Grid: A B C
            //       D E F  (E is the current pixel)
            //       G H I
            let is_top = y==0;
            let is_left = x==0;
            let is_bottom = y>=image.height-1;
            let is_right = x>=image.width-1;
            let a = if is_top || is_left { offscreen } else { image.pixels[index - image.width - 1] };
            let b = if is_top { offscreen } else { image.pixels[index - image.width] };
            let c = if is_top || is_right { offscreen } else { image.pixels[index - image.width + 1] };
            let d = if is_left { offscreen } else { image.pixels[index - 1] };
            let e = image.pixels[index];
            let f = if is_right { offscreen } else { image.pixels[index + 1] };
            let g = if is_bottom || is_left { offscreen } else { image.pixels[index + image.width - 1] };
            let h = if is_bottom { offscreen } else { image.pixels[index + image.width] };
            let i = if is_bottom || is_right { offscreen } else { image.pixels[index + image.width + 1] };

            // Strength junctions:
            let jsx = Vec4{x: a.corner_strength_down_right, y: b.corner_strength_down_left, z: e.corner_strength_up_left, w: d.corner_strength_up_right};
            let jsy = Vec4{x: b.corner_strength_down_right, y: c.corner_strength_down_left, z: f.corner_strength_up_left, w: e.corner_strength_up_right};
            let jsz = Vec4{x: e.corner_strength_down_right, y: f.corner_strength_down_left, z: i.corner_strength_up_left, w: h.corner_strength_up_right};
            let jsw = Vec4{x: d.corner_strength_down_right, y: e.corner_strength_down_left, z: h.corner_strength_up_left, w: g.corner_strength_up_right};

            // Dominance junctions:
            let dominance_junction_x = corner_dominance(
                &Vec3 { x: a.corner_strength_up_right, y: a.corner_strength_down_right, z: a.corner_strength_down_left },
                &Vec3 { x: b.corner_strength_down_right, y: b.corner_strength_down_left, z: b.corner_strength_up_left},
                &Vec3 { x: e.corner_strength_down_left, y: e.corner_strength_up_left, z: e.corner_strength_up_right},
                &Vec3 { x: d.corner_strength_up_left, y: d.corner_strength_up_right, z: d.corner_strength_down_right});
            let dominance_junction_y = corner_dominance(
                &Vec3 { x: b.corner_strength_up_right, y: b.corner_strength_down_right, z: b.corner_strength_down_left },
                &Vec3 { x: c.corner_strength_down_right, y: c.corner_strength_down_left, z: c.corner_strength_up_left},
                &Vec3 { x: f.corner_strength_down_left, y: f.corner_strength_up_left, z: f.corner_strength_up_right},
                &Vec3 { x: e.corner_strength_up_left, y: e.corner_strength_up_right, z: e.corner_strength_down_right});
            let dominance_junction_z = corner_dominance(
                &Vec3 { x: e.corner_strength_up_right, y: e.corner_strength_down_right, z: e.corner_strength_down_left },
                &Vec3 { x: f.corner_strength_down_right, y: f.corner_strength_down_left, z: f.corner_strength_up_left},
                &Vec3 { x: i.corner_strength_down_left, y: i.corner_strength_up_left, z: i.corner_strength_up_right},
                &Vec3 { x: h.corner_strength_up_left, y: h.corner_strength_up_right, z: h.corner_strength_down_right});
            let dominance_junction_w = corner_dominance(
                &Vec3 { x: d.corner_strength_up_right, y: d.corner_strength_down_right, z: d.corner_strength_down_left },
                &Vec3 { x: e.corner_strength_down_right, y: e.corner_strength_down_left, z: e.corner_strength_up_left},
                &Vec3 { x: h.corner_strength_down_left, y: h.corner_strength_up_left, z: h.corner_strength_up_right},
                &Vec3 { x: g.corner_strength_up_left, y: g.corner_strength_up_right, z: g.corner_strength_down_right});

            // Majority vote for ambiguous dominance junctions:
            let zero4 = Vec4::zero();
            let jx = (Vec4::ge(dominance_junction_x, zero4) * (Vec4::leq(dominance_junction_x.yzwx(), zero4) * Vec4::leq(dominance_junction_x.wxyz(), zero4) + Vec4::ge(dominance_junction_x + dominance_junction_x.zwxy(), dominance_junction_x.yzwx() + dominance_junction_x.wxyz()))).min(1.);
            let jy = (Vec4::ge(dominance_junction_y, zero4) * (Vec4::leq(dominance_junction_y.yzwx(), zero4) * Vec4::leq(dominance_junction_y.wxyz(), zero4) + Vec4::ge(dominance_junction_y + dominance_junction_y.zwxy(), dominance_junction_y.yzwx() + dominance_junction_y.wxyz()))).min(1.);
            let jz = (Vec4::ge(dominance_junction_z, zero4) * (Vec4::leq(dominance_junction_z.yzwx(), zero4) * Vec4::leq(dominance_junction_z.wxyz(), zero4) + Vec4::ge(dominance_junction_z + dominance_junction_z.zwxy(), dominance_junction_z.yzwx() + dominance_junction_z.wxyz()))).min(1.);
            let jw = (Vec4::ge(dominance_junction_w, zero4) * (Vec4::leq(dominance_junction_w.yzwx(), zero4) * Vec4::leq(dominance_junction_w.wxyz(), zero4) + Vec4::ge(dominance_junction_w + dominance_junction_w.zwxy(), dominance_junction_w.yzwx() + dominance_junction_w.wxyz()))).min(1.);

            // Inject strength without creating new contradictions:
            let res_x = (jx.z + (1. - jx.y) * (1. - jx.w) * ge_f32(jsx.z, 0.) * (jx.x + ge_f32(jsx.x + jsx.z, jsx.y + jsx.w))).min(1.);
            let res_y = (jy.w + (1. - jy.z) * (1. - jy.x) * ge_f32(jsy.w, 0.) * (jy.y + ge_f32(jsy.y + jsy.w, jsy.x + jsy.z))).min(1.);
            let res_z = (jz.x + (1. - jz.w) * (1. - jz.y) * ge_f32(jsz.x, 0.) * (jz.z + ge_f32(jsz.x + jsz.z, jsz.y + jsz.w))).min(1.);
            let res_w = (jw.y + (1. - jw.x) * (1. - jw.z) * ge_f32(jsw.y, 0.) * (jw.w + ge_f32(jsw.y + jsw.w, jsw.x + jsw.z))).min(1.);
            let res_early = Vec4{ x: res_x, y: res_y, z: res_z, w: res_w };

            // Single pixel & end of line detection:
            let res = (res_early * (Vec4{x: jx.z, y: jy.w, z: jz.x, w: jw.y} + (res_early.wxyz() * res_early.yzwx()).not())).min(1.);

            // Output:
            let clr_x = clear(Vec2 { x: d.colour_distance_up_right, y: e.colour_distance_up_left}, Vec2 { x: d.colour_distance_right, y: e.colour_distance_up}, Vec2 { x: a.colour_distance_right, y: d.colour_distance_up});
            let clr_y = clear(Vec2 { x: f.colour_distance_up_left, y: e.colour_distance_up_right}, Vec2 { x: e.colour_distance_right, y: e.colour_distance_up}, Vec2 { x: b.colour_distance_right, y: f.colour_distance_up});
            let clr_z = clear(Vec2 { x: h.colour_distance_up_right, y: i.colour_distance_up_left}, Vec2 { x: e.colour_distance_right, y: h.colour_distance_up}, Vec2 { x: h.colour_distance_right, y: i.colour_distance_up});
            let clr_w = clear(Vec2 { x: h.colour_distance_up_left, y: g.colour_distance_up_right}, Vec2 { x: d.colour_distance_right, y: h.colour_distance_up}, Vec2 { x: g.colour_distance_right, y: g.colour_distance_up});
            let clr = Vec4{ x: clr_x, y: clr_y, z: clr_z, w: clr_w };

            let ho = Vec4 {
                x: d.colour_distance_right.min(a.colour_distance_right),
                y: e.colour_distance_right.min(b.colour_distance_right),
                z: e.colour_distance_right.min(h.colour_distance_right),
                w: d.colour_distance_right.min(g.colour_distance_right),
            };
            let v = Vec4 {
                x: e.colour_distance_up.min(d.colour_distance_up),
                y: e.colour_distance_up.min(f.colour_distance_up),
                z: h.colour_distance_up.min(i.colour_distance_up),
                w: h.colour_distance_up.min(g.colour_distance_up),
            };

            let orientation = Vec4::ge(
                ho + Vec4{x: d.colour_distance_right, y: e.colour_distance_right, z: e.colour_distance_right, w: d.colour_distance_right},
                v + Vec4{ x: e.colour_distance_up, y: e.colour_distance_up, z: h.colour_distance_up, w: h.colour_distance_up});
            let horizontal_edges = Vec4::le(ho, v) * clr;
            let vertical_edges = Vec4::ge(ho, v) * clr;

            pixels.push(PixelWithCornerConfiguration {
                pixel: e.pixel,
                // colour_distance_up_left: e.colour_distance_up_left,
                // colour_distance_up: e.colour_distance_up,
                // colour_distance_up_right: e.colour_distance_up_right,
                // colour_distance_right: e.colour_distance_right,
                // corner_strength_up_left: e.corner_strength_up_left,
                // corner_strength_up_right: e.corner_strength_up_right,
                // corner_strength_down_right: e.corner_strength_down_right,
                // corner_strength_down_left: e.corner_strength_down_left,
                res: res.to_bvec(),
                horizontal_edges: horizontal_edges.to_bvec(),
                vertical_edges: vertical_edges.to_bvec(),
                orientation: orientation.to_bvec(),
            });
        }
    }

    ImageWithCornerConfigurations {
        width: image.width,
        height: image.height,
        pixels,
    }
}

#[derive(Debug, Clone, Copy)]
struct PixelWithEdgeLevel {
    pixel: u32,
    corners: U8Vec4,
    mids: U8Vec4,
}

#[derive(Debug)]
struct ImageWithEdgeLevels {
    width: usize,
    height: usize,
    pixels: Vec<PixelWithEdgeLevel>,
}
// Determines which edge level is present and prepares tags for subpixel output in the final pass.
// This implements pass 3 here:
// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass3.slang
fn determine_edge_levels(image: &ImageWithCornerConfigurations) -> ImageWithEdgeLevels {
    let pixels_len = image.pixels.len();
    let mut pixels: Vec<PixelWithEdgeLevel> = Vec::with_capacity(pixels_len);
    let offscreen = PixelWithCornerConfiguration::offscreen();
    let zero = Vec4::zero();

    for y in 0..image.height {
        for x in 0..image.width {
            // Get the neighbouring pixels, returning transparent if they're out of bounds.
            // Grid:
            //         B1
            //         B0
            //         B
            // D1 D0 D E F F0 F1  (E is the current pixel)
            //         H
            //         H0
            //         H1
            let index = y * image.width + x;
            let b1 = if y<=2 { offscreen } else { image.pixels[index - image.width * 3] };
            let b0 = if y<=1 { offscreen } else { image.pixels[index - image.width * 2] };
            let b = if y<=0 { offscreen } else { image.pixels[index - image.width] };
            let d = if x<=0 { offscreen } else { image.pixels[index - 1] };
            let d0 = if x<=1 { offscreen } else { image.pixels[index - 2] };
            let d1 = if x<=2 { offscreen } else { image.pixels[index - 3] };
            let e = image.pixels[index];
            let f = if x+1 >= image.width { offscreen } else { image.pixels[index + 1] };
            let f0 = if x+2 >= image.width { offscreen } else { image.pixels[index + 2] };
            let f1 = if x+3 >= image.width { offscreen } else { image.pixels[index + 3] };
            let h = if y+1 >= image.height { offscreen } else { image.pixels[index + image.width] };
            let h0 = if y+2 >= image.height { offscreen } else { image.pixels[index + image.width*2] };
            let h1 = if y+3 >= image.height { offscreen } else { image.pixels[index + image.width*3] };

            // Extract data:            
            let ec = e.res; let eh = e.horizontal_edges; let ev = e.vertical_edges; let eo = e.orientation;
            let dc = d.res; let dh = d.horizontal_edges; let dr = d.orientation; let d0c = d0.res; let d0h = d0.horizontal_edges; let d1h = d1.horizontal_edges;
            let fc = f.res; let fh = f.horizontal_edges; let fo = f.orientation; let f0c = f0.res; let f0h = f0.horizontal_edges; let f1h = f1.horizontal_edges;
            let bc = b.res; let bv = b.vertical_edges; let bo = b.orientation; let b0c = b0.res; let b0v = b0.vertical_edges; let b1v = b1.vertical_edges;
            let hc = h.res; let hv = h.vertical_edges; let ho = h.orientation; let h0c = h0.res; let h0v = h0.vertical_edges; let h1v = h1.vertical_edges;

            // Level 1 corners (horizontal, vertical):
            let lvl1x = ec.x && (dc.z || bc.z || FILTER_CORNERS);
            let lvl1y = ec.y && (fc.w || bc.w || FILTER_CORNERS);
            let lvl1z = ec.z && (fc.x || hc.x || FILTER_CORNERS);
            let lvl1w = ec.w && (dc.y || hc.y || FILTER_CORNERS);

            // Level 2 mid (left, right / up, down):
            let lvl2x = BVec2{ x: (ec.x && eh.y) && dc.z, y: (ec.y && eh.x) && fc.w };
            let lvl2y = BVec2{ x: (ec.y && ev.z) && bc.w, y: (ec.z && ev.y) && hc.x };
            let lvl2z = BVec2{ x: (ec.w && eh.z) && dc.y, y: (ec.z && eh.w) && fc.x };
            let lvl2w = BVec2{ x: (ec.x && ev.w) && bc.z, y: (ec.w && ev.x) && hc.y };

            // Level 3 corners (horizontal, vertical):
            let lvl3x = BVec2{ x: lvl2x.y && (dh.y && dh.x) && fh.z, y: lvl2w.y && (bv.w && bv.x) && hv.z };
            let lvl3y = BVec2{ x: lvl2x.x && (fh.x && fh.y) && dh.w, y: lvl2y.y && (bv.z && bv.y) && hv.w };
            let lvl3z = BVec2{ x: lvl2z.x && (fh.w && fh.z) && dh.x, y: lvl2y.x && (hv.y && hv.z) && bv.x };
            let lvl3w = BVec2{ x: lvl2z.y && (dh.z && dh.w) && fh.y, y: lvl2w.x && (hv.x && hv.w) && bv.y };

            // Level 4 corners (horizontal, vertical):
            let lvl4x = BVec2{ x: (dc.x && dh.y && eh.x && eh.y && fh.x && fh.y) && (d0c.z && d0h.w), y: (bc.x && bv.w && ev.x && ev.w && hv.x && hv.w) && (b0c.z && b0v.y) };
            let lvl4y = BVec2{ x: (fc.y && fh.x && eh.y && eh.x && dh.y && dh.x) && (f0c.w && f0h.z), y: (bc.y && bv.z && ev.y && ev.z && hv.y && hv.z) && (b0c.w && b0v.x) };
            let lvl4z = BVec2{ x: (fc.z && fh.w && eh.z && eh.w && dh.z && dh.w) && (f0c.x && f0h.y), y: (hc.z && hv.y && ev.z && ev.y && bv.z && bv.y) && (h0c.x && h0v.w) };
            let lvl4w = BVec2{ x: (dc.w && dh.z && eh.w && eh.z && fh.w && fh.z) && (d0c.y && d0h.x), y: (hc.w && hv.x && ev.w && ev.x && bv.w && bv.x) && (h0c.y && h0v.z) };

            // Level 5 mid (left, right / up, down):
            let lvl5x = BVec2{ x: lvl4x.x && (f0h.x && f0h.y) && (d1h.z && d1h.w), y: lvl4y.x && (d0h.y && d0h.x) && (f1h.w && f1h.z) };
            let lvl5y = BVec2{ x: lvl4y.y && (h0v.y && h0v.z) && (b1v.w && b1v.x), y: lvl4z.y && (b0v.z && b0v.y) && (h1v.x && h1v.w) };
            let lvl5z = BVec2{ x: lvl4w.x && (f0h.w && f0h.z) && (d1h.y && d1h.x), y: lvl4z.x && (d0h.z && d0h.w) && (f1h.x && f1h.y) };
            let lvl5w = BVec2{ x: lvl4x.y && (h0v.x && h0v.w) && (b1v.z && b1v.y), y: lvl4w.y && (b0v.w && b0v.x) && (h1v.y && h1v.z) };

            // Level 6 corners (horizontal, vertical):
            let lvl6x = BVec2{ x: lvl5x.y && (d1h.y && d1h.x), y: lvl5w.y && (b1v.w && b1v.x) };
            let lvl6y = BVec2{ x: lvl5x.x && (f1h.x && f1h.y), y: lvl5y.y && (b1v.z && b1v.y) };
            let lvl6z = BVec2{ x: lvl5z.x && (f1h.w && f1h.z), y: lvl5y.x && (h1v.y && h1v.z) };
            let lvl6w = BVec2{ x: lvl5z.y && (d1h.z && d1h.w), y: lvl5w.x && (h1v.x && h1v.w) };

            // Subpixels - 0 = E, 1 = D, 2 = D0, 3 = F, 4 = F0, 5 = B, 6 = B0, 7 = H, 8 = H0
	        let crn_x = if lvl1x && eo.x || lvl3x.x && eo.y || lvl4x.x && dr.x || lvl6x.x && fo.y { 5 } else { if lvl1x || lvl3x.y && !eo.w || lvl4x.y && !bo.x || lvl6x.y && !ho.w { 1 } else { if lvl3x.x { 3 } else { if lvl3x.y { 7 } else { if lvl4x.x { 2 } else { if lvl4x.y { 6 } else { if lvl6x.x { 4 } else { if lvl6x.y { 8 } else { 0 }}}}}}}};
	        let crn_y = if lvl1y && eo.y || lvl3y.x && eo.x || lvl4y.x && fo.y || lvl6y.x && dr.x { 5 } else { if lvl1y || lvl3y.y && !eo.z || lvl4y.y && !bo.y || lvl6y.y && !ho.z { 3 } else { if lvl3y.x { 1 } else { if lvl3y.y { 7 } else { if lvl4y.x { 4 } else { if lvl4y.y { 6 } else { if lvl6y.x { 2 } else { if lvl6y.y { 8 } else { 0 }}}}}}}};
	        let crn_z = if lvl1z && eo.z || lvl3z.x && eo.w || lvl4z.x && fo.z || lvl6z.x && dr.w { 7 } else { if lvl1z || lvl3z.y && !eo.y || lvl4z.y && !ho.z || lvl6z.y && !bo.y { 3 } else { if lvl3z.x { 1 } else { if lvl3z.y { 5 } else { if lvl4z.x { 4 } else { if lvl4z.y { 8 } else { if lvl6z.x { 2 } else { if lvl6z.y { 6 } else { 0 }}}}}}}};
	        let crn_w = if lvl1w && eo.w || lvl3w.x && eo.z || lvl4w.x && dr.w || lvl6w.x && fo.z { 7 } else { if lvl1w || lvl3w.y && !eo.x || lvl4w.y && !ho.w || lvl6w.y && !bo.x { 1 } else { if lvl3w.x { 3 } else { if lvl3w.y { 5 } else { if lvl4w.x { 2 } else { if lvl4w.y { 8 } else { if lvl6w.x { 4 } else { if lvl6w.y { 6 } else { 0 }}}}}}}};
            let corners = U8Vec4 { x: crn_x, y: crn_y, z: crn_z, w: crn_w };

            // TODO hope the precedence of the no-brackets nested ?: was resolved correctly below:
            let mid_x = if lvl2x.x &&  eo.x || lvl2x.y &&  eo.y || lvl5x.x &&  dr.x || lvl5x.y &&  fo.y { 5 } else { if lvl2x.x { 1 } else { if lvl2x.y { 3 } else { if lvl5x.x { 2 } else { if lvl5x.y { 4 } else { if ec.x && dc.z && ec.y && fc.w { if  eo.x { if  eo.y { 5 } else { 3 }} else { 1 }} else {0}}}}}};
            let mid_y = if lvl2y.x && !eo.y || lvl2y.y && !eo.z || lvl5y.x && !bo.y || lvl5y.y && !ho.z { 3 } else { if lvl2y.x { 5 } else { if lvl2y.y { 7 } else { if lvl5y.x { 6 } else { if lvl5y.y { 8 } else { if ec.y && bc.w && ec.z && hc.x { if !eo.y { if !eo.z { 3 } else { 7 }} else { 5 }} else {0}}}}}};
            let mid_z = if lvl2z.x &&  eo.w || lvl2z.y &&  eo.z || lvl5z.x &&  dr.w || lvl5z.y &&  fo.z { 7 } else { if lvl2z.x { 1 } else { if lvl2z.y { 3 } else { if lvl5z.x { 2 } else { if lvl5z.y { 4 } else { if ec.z && fc.x && ec.w && dc.y { if  eo.z { if  eo.w { 7 } else { 1 }} else { 3 }} else {0}}}}}};
            let mid_w = if lvl2w.x && !eo.x || lvl2w.y && !eo.w || lvl5w.x && !bo.x || lvl5w.y && !ho.w { 1 } else { if lvl2w.x { 5 } else { if lvl2w.y { 7 } else { if lvl5w.x { 6 } else { if lvl5w.y { 8 } else { if ec.w && hc.y && ec.x && bc.z { if !eo.w { if !eo.x { 1 } else { 5 }} else { 7 }} else {0}}}}}};
            let mids = U8Vec4 { x: mid_x, y: mid_y, z: mid_z, w: mid_w };

            pixels.push(PixelWithEdgeLevel {
                pixel: e.pixel,
                corners,
                mids,
            })
        }
    }
    ImageWithEdgeLevels {
        width: image.width,
        height: image.height,
        pixels,
    }
}

// Outputs subpixels based on previously calculated tags.
// This implements pass 4 from here:
// https://github.com/libretro/slang-shaders/blob/master/edge-smoothing/scalefx/shaders/scalefx-pass4.slang
fn scale_subpixels(image: &ImageWithEdgeLevels) -> Image {
    let pixels_len = image.pixels.len();
    let mut pixels: Vec<u32> = Vec::with_capacity(pixels_len * 9);
    let mut row0: Vec<u32> = Vec::with_capacity(image.width * 3);
    let mut row1: Vec<u32> = Vec::with_capacity(image.width * 3);
    let mut row2: Vec<u32> = Vec::with_capacity(image.width * 3);
    for y in 0..image.height {
        row0.clear();
        row1.clear();
        row2.clear();
        for x in 0..image.width {
            let source = image.pixels[y * image.width + x];
            let mid = source.mids;
            let crn = source.corners;
            for spy in 0..3 { // Loop the subpixels.
                for spx in 0..3 {
                    // Figure out which tag to use for each subpixel:
                    let sp: u8 = match (spx, spy) {
                        (0, 0) => crn.x,
                        (1, 0) => mid.x,
                        (2, 0) => crn.y,
                        (0, 1) => mid.w,
                        (1, 1) => 0,
                        (2, 1) => mid.y,
                        (0, 2) => crn.w,
                        (1, 2) => mid.z,
                        (2, 2) => crn.z,
                        _ => 0,                    
                    };

                    // Convert from a tag to an output coordinate:
                    // Tag 0 = coordinate E, 1 = D, 2 = D0, 3 = F, 4 = F0, 5 = B, 6 = B0, 7 = H, 8 = H0.
                    // Grid:
                    //      B0
                    //      B
                    // D0 D E F F0
                    //      H
                    //      H0
                    let (offset_x, offset_y) = match sp {
                        0 => (0,0),
                        1 => (-1,0),
                        2 => (-2,0),
                        3 => (1,0),
                        4 => (2,0),
                        5 => (0,-1),
                        6 => (0,-2),
                        7 => (0,1),
                        8 => (0,2),
                        _ => (0,0),
                    };

                    // Get the colour from that coordinate.
                    let x: isize = (x as isize) + offset_x;
                    let y: isize = (y as isize) + offset_y;
                    let in_bounds = 0<=x && x<(image.width as isize) && 0<=y && y<(image.height as isize); 
                    let colour = if in_bounds { image.pixels[(y as usize) * image.width + (x as usize)].pixel } else { 0 };

                    // Push it to the correct row.
                    match spy {
                        0 => row0.push(colour),
                        1 => row1.push(colour),
                        2 => row2.push(colour),
                        _ => {},
                    }
                }
            }
        }
        pixels.extend_from_slice(&row0);
        pixels.extend_from_slice(&row1);
        pixels.extend_from_slice(&row2);
    }
    Image {
        width: image.width * 3,
        height: image.height * 3,
        pixels,
    }

}

fn scalefx(image: &Image) -> Image {
    let distances = calculate_distances(&image);
    let corners = calculate_corner_strengths(&distances);
    let configurations = resolve_corner_configurations(&corners);
    let edges = determine_edge_levels(&configurations);
    let big = scale_subpixels(&edges);
    big
}

fn main() {
    let sample = sample();
    let big = scalefx(&sample);
    let bigger = scalefx(&big);

    let png_sample = png::png_data(sample.width, sample.height, &sample.pixels);
    std::fs::write("sample.png", &png_sample).unwrap();

    let png_big = png::png_data(big.width, big.height, &big.pixels);
    std::fs::write("out.big.png", &png_big).unwrap();

    let png_bigger = png::png_data(bigger.width, bigger.height, &bigger.pixels);
    std::fs::write("out.bigger.png", &png_bigger).unwrap();
}
