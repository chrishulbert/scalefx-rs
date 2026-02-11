mod scalefx;

use png::{self, BitDepth, ColorType, Transformations};

fn main() {
    println!("-=[ ScaleFX-rs Pixel Art Upscaler ]=-");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage:");
        println!("scalefx in.png out.png");
    } else {
        upscale(&args[1], &args[2]);
    }

    // let (width, height, pixels) = sample();
    // std::fs::write("sample.png", png::png_data(width, height, &pixels)).unwrap();

    // let (width, height, pixels) = scalefx::scale3x(width, height, &pixels);
    // std::fs::write("out.big.png", png::png_data(width, height, &pixels)).unwrap();

    // let (width, height, pixels) = scalefx::scale3x(width, height, &pixels);
    // std::fs::write("out.bigger.png", png::png_data(width, height, &pixels)).unwrap();
}

fn upscale(in_path: &str, out: &str) {
    let (width, height, pixels) = load_png(in_path);
    println!("Scaling...");
    let (width, height, pixels) = scalefx::scale9x(width, height, &pixels);
    println!("Scaled to: {} x {}", width, height);
    save_png(width, height, &pixels, out);
}

fn load_png(path: &str) -> (usize, usize, Vec<u32>) {
    println!("Loading: {}", path);
    let in_file = std::fs::File::open(path).expect("Failed to open file!");
    let in_reader = std::io::BufReader::new(in_file);
    let mut decoder = png::Decoder::new(in_reader);
    decoder.set_transformations(Transformations::ALPHA); // Auto-converts to 8-bit RGBA.
    let mut reader = decoder.read_info().expect("Failed to read header");
    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).expect("Failed to decode");
    let bytes = &buf[..info.buffer_size()];
    assert_eq!(info.bit_depth, BitDepth::Eight);
    assert_eq!(info.color_type, ColorType::Rgba);
    // Convert to RGBA
    let mut rgbas: Vec<u32> = Vec::with_capacity(info.width as usize * info.height as usize);
    for chunk in bytes.chunks_exact(4) {
        rgbas.push(u32::from_be_bytes(chunk.try_into().unwrap()));
    }
    println!("Loaded: {} x {} px", info.width, info.height);
    (info.width as usize, info.height as usize, rgbas)
}

fn save_png(width: usize, height: usize, pixels: &[u32], path: &str) {
    println!("Saving: {}", path);
    let file = std::fs::File::create(path).unwrap();
    let ref mut buf_writer = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(buf_writer, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let mut data: Vec<u8> = Vec::with_capacity(width * height * 4);
    for p in pixels {
        data.push((p >> 24) as u8);
        data.push((p >> 16) as u8);
        data.push((p >> 8) as u8);
        data.push(*p as u8);
    }
    writer.write_image_data(&data).unwrap(); // Save
}
