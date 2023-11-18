extern crate image;

use image::{DynamicImage, ImageBuffer, ImageError, Rgba};

use steganography::decoder::Decoder;

fn main() {
    let input_path = "hidden_message.png";

    // Open the image using the image crate
    let dynamic_image = image::open(input_path).expect("Failed to open image");

    // Convert the DynamicImage to an ImageBuffer with Rgba<u8> pixel format
    let encoded_image: ImageBuffer<Rgba<u8>, Vec<u8>> = dynamic_image.to_rgba();

    // Create a decoder and decode the image
    let dec = Decoder::new(encoded_image);

    //Decode the image by reading the alpha channel
    let out_buffer = dec.decode_alpha();

    //let decoded_image_result = image::load_from_memory(&out_buffer);
    let decoded_image_result: Result<DynamicImage, ImageError> =
        image::load_from_memory(&out_buffer);

    let decoded_image = decoded_image_result.unwrap();

    decoded_image
        .save("decoded_image.png")
        .expect("Failed to save decoded image");
}
