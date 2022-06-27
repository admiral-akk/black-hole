use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use png::chunk::IDAT;
fn main() {
    let path = Path::new(r"output/animated.apng");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, 800, 800);
    encoder.set_animated(360, 0);

    let mut writer = encoder.write_header().unwrap();
    for i in 0..360 {
        let file_name = format!(
            r"rendering/output/gif/black_hole_field_1_size_800_angle_{}.png",
            i
        );

        let read_file = File::open(file_name).unwrap();
        let decoder = png::Decoder::new(read_file);
        let mut reader = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; reader.output_buffer_size()];
        // Read the next frame. An APNG might contain multiple frames.
        let info = reader.next_frame(&mut buf).unwrap();
        // Grab the bytes of the image.
        let bytes = &buf[..info.buffer_size()];
        writer.write_chunk(IDAT, bytes);
    }
    writer.finish();
}
