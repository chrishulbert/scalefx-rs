
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

const THRESHOLD: f32 = 0.5; // Min 0.01; max: 1; step: 0.01
const IS_FILTER_AA_ENABLED: bool = true;

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
            let down_left = if x==0 || y==image.height-1 { offscreen } else { image.pixels[i + image.height - 1] };
            let down = if y==image.height-1 { offscreen } else { image.pixels[i + image.height] };
            let down_right = if x==image.width-1 || y==image.height-1 { offscreen } else { image.pixels[i + image.height + 1] };

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
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
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
    colour_distance_up_left: f32, // X: Colour distance to the pixel to the up-left.
    colour_distance_up: f32, // Y.
    colour_distance_up_right: f32, // Z.
    colour_distance_right: f32, // W.
    corner_strength_up_left: f32, // Corner strength. Called X in the shader.
    corner_strength_up_right: f32, // Y in the shader.
    corner_strength_down_right: f32, // Z in the shader.
    corner_strength_down_left: f32, // W in the shader.
    res: Vec4, // Resolution?
    horizontal_edges: Vec4,
    vertical_edges: Vec4,
    orientation: Vec4,
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
            let g = if is_bottom || is_left { offscreen } else { image.pixels[index + image.height - 1] };
            let h = if is_bottom { offscreen } else { image.pixels[index + image.height] };
            let i = if is_bottom || is_right { offscreen } else { image.pixels[index + image.height + 1] };

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
                colour_distance_up_left: e.colour_distance_up_left,
                colour_distance_up: e.colour_distance_up,
                colour_distance_up_right: e.colour_distance_up_right,
                colour_distance_right: e.colour_distance_right,
                corner_strength_up_left: e.corner_strength_up_left,
                corner_strength_up_right: e.corner_strength_up_right,
                corner_strength_down_right: e.corner_strength_down_right,
                corner_strength_down_left: e.corner_strength_down_left,
                res,
                horizontal_edges,
                vertical_edges,
                orientation,
            });
        }
    }

    ImageWithCornerConfigurations {
        width: image.width,
        height: image.height,
        pixels,
    }
}

fn main() {
    println!("Hello, world!");
    println!("{}", colour_distance(0xffffffff, 0xff));
    println!("{}", colour_distance(0xff0000ff, 0xff0000ff));
    println!("{}", colour_distance(0xff0000ff, 0x00ff00ff));
    println!("{}", colour_distance(0xff0000ff, 0x0000ffff));
    println!("{}", colour_distance(0x00ff00ff, 0xff0000ff));
    println!("{}", colour_distance(0x00ff00ff, 0x00ff00ff));
    println!("{}", colour_distance(0x00ff00ff, 0x0000ffff));
    println!("{}", colour_distance(0x0000ffff, 0xff0000ff));
    println!("{}", colour_distance(0x0000ffff, 0x00ff00ff));
    println!("{}", colour_distance(0x0000ffff, 0x0000ffff));
    println!("{}", colour_distance(0x0000ffff, 0));

    let sample = sample();
    let distances = calculate_distances(&sample);
    let corners = calculate_corner_strengths(&distances);
    let configurations = resolve_corner_configurations(&corners);
    println!("{:#?}", configurations);
}
