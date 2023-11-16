extern crate image;

use std::fs::File;
use std::io::Read;

use steganography::encoder::Encoder;
use steganography::util::save_image_buffer;
fn main() {
    // Open the original image
    let org_image = image::open("over.jpg").expect("Failed to open original image");

    let mut file = File::open("org.jpg").expect("Failed to open overlay image");

    // Read the image file into a Vec<u8>
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read image file");

    // Now 'buffer' contains the raw bytes of the image
    let overlay_image: &[u8] = &buffer;

    // Create an encoder and encode the alpha channel modified image
    let encoder = Encoder::new(overlay_image, org_image);
    let buffer = encoder.encode_alpha();

    save_image_buffer(buffer, "hidden_message.png".to_string());
}
