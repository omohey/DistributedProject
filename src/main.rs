// use image::{DynamicImage, GenericImageView, Rgba};
// use std::convert::TryInto;

// fn overlay_images(original: &DynamicImage, overlay: &DynamicImage) -> Vec<u8> {
//     // Ensure the images have the same dimensions
//     assert_eq!(original.dimensions(), overlay.dimensions());

//     let encrypted_image: Vec<_> = original
//         .pixels()
//         .zip(overlay.pixels())
//         .map(|(original_pixel, overlay_pixel)| {
//             let Rgba([or, og, ob, oa]) = original_pixel.2;
//             let Rgba([ur, ug, ub, ua]) = overlay_pixel.2;

//             let blended_pixel = Rgba([
//                 ((oa as f32 / 255.0) * or as f32 + (1.0 - oa as f32 / 255.0) * ur as f32) as u8,
//                 ((oa as f32 / 255.0) * og as f32 + (1.0 - oa as f32 / 255.0) * ug as f32) as u8,
//                 ((oa as f32 / 255.0) * ob as f32 + (1.0 - oa as f32 / 255.0) * ub as f32) as u8,
//                 oa, // Use the alpha value from the overlay image
//             ]);

//             [
//                 blended_pixel[0],
//                 blended_pixel[1],
//                 blended_pixel[2],
//                 blended_pixel[3],
//             ]
//         })
//         .flatten()
//         .collect();

//     encrypted_image
// }

// fn remove_overlay(encrypted: &[u8], overlay: &DynamicImage) -> Vec<u8> {
//     // Ensure the images have the same dimensions
//     assert_eq!(
//         encrypted.len(),
//         overlay.width() as usize * overlay.height() as usize * 4
//     );

//     let decrypted_image: Vec<_> = encrypted
//         .chunks_exact(4)
//         .zip(overlay.pixels())
//         .map(|(encrypted_pixel, overlay_pixel)| {
//             let [er, eg, eb, ea]: [u8; 4] = encrypted_pixel.try_into().unwrap();
//             let Rgba([_, _, _, ua]) = overlay_pixel.2;

//             let original_pixel = Rgba([
//                 ((ua as f32 / 255.0) * er as f32 + (1.0 - ua as f32 / 255.0) * 255.0) as u8,
//                 ((ua as f32 / 255.0) * eg as f32 + (1.0 - ua as f32 / 255.0) * 255.0) as u8,
//                 ((ua as f32 / 255.0) * eb as f32 + (1.0 - ua as f32 / 255.0) * 255.0) as u8,
//                 255, // Fully opaque
//             ]);

//             [
//                 original_pixel[0],
//                 original_pixel[1],
//                 original_pixel[2],
//                 original_pixel[3],
//             ]
//         })
//         .flatten()
//         .collect();

//     decrypted_image
// }

// fn main() {
//     // Load the original and overlay images
//     let original_image_path =
//         "D:/Uni/fall 2023/distributed/tests/test 5 encrypt/image_encryption/org.jpg";
//     let overlay_image_path =
//         "D:/Uni/fall 2023/distributed/tests/test 5 encrypt/image_encryption/over.jpg";

//     let original_image = image::open(original_image_path).expect("Failed to open original image");
//     let overlay_image = image::open(overlay_image_path).expect("Failed to open overlay image");

//     // Print the dimensions of both images
//     println!("Original Dimensions: {:?}", original_image.dimensions());
//     println!("Overlay Dimensions: {:?}", overlay_image.dimensions());

//     // Encrypt the original image by overlaying it with the overlay image
//     let encrypted_image: Vec<_> = overlay_images(&original_image, &overlay_image);

//     // Save the encrypted image
//     image::save_buffer(
//         "D:/Uni/fall 2023/distributed/tests/test 5 encrypt/image_encryption/encrypted.jpg",
//         &encrypted_image,
//         original_image.width(),
//         original_image.height(),
//         image::ColorType::Rgba8,
//     )
//     .expect("Failed to save encrypted image");

//     // Decrypt the encrypted image by removing the overlay
//     // Decrypt the encrypted image by removing the overlay
//     // Decrypt the encrypted image by removing the overlay
//     let decrypted_image = remove_overlay(&encrypted_image, &overlay_image);

//     // Save the decrypted image
//     image::save_buffer(
//         "D:/Uni/fall 2023/distributed/tests/test 5 encrypt/image_encryption/decrypted.jpg",
//         &decrypted_image,
//         original_image.width(),
//         original_image.height(),
//         image::ColorType::Rgba8,
//     )
//     .expect("Failed to save decrypted image");
// }

// use aes_gcm::aead::{Aead, NewAead};
// use aes_gcm::Aes256Gcm;
// use generic_array::GenericArray;
// use image::{DynamicImage, GenericImageView, Rgb};
// use rand::{thread_rng, Rng};

// fn encrypt_decrypt_image(image_path: &str) {
//     // Load the image
//     let img = image::open(image_path).expect("Failed to open image");

//     // Convert the image to a Vec<u8>
//     let mut image_data: Vec<u8> = img
//         .pixels()
//         .flat_map(|pixel| vec![pixel.0[0], pixel.0[1], pixel.0[2]])
//         .collect();

//     // Generate a random key and IV
//     let mut key = [0u8; 32];
//     let mut nonce = [0u8; 12];
//     thread_rng().fill(&mut key[..]);
//     thread_rng().fill(&mut nonce[..]);

//     // Encrypt the image data
//     let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
//     let ciphertext = cipher
//         .encrypt(nonce.into(), image_data.as_mut_slice())
//         .expect("Encryption failed");

//     // Decrypt the image data
//     let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
//     let decrypted_data = cipher
//         .decrypt(nonce.into(), ciphertext.as_slice())
//         .expect("Decryption failed");

//     // Replace the original image data with decrypted data
//     image_data.copy_from_slice(decrypted_data);

//     // Save the decrypted image
//     let mut decrypted_image = DynamicImage::new_rgb8(img.width(), img.height());
//     for (x, y, pixel) in decrypted_image.enumerate_pixels_mut() {
//         let r = image_data.pop().unwrap();
//         let g = image_data.pop().unwrap();
//         let b = image_data.pop().unwrap();
//         *pixel = Rgb([r, g, b]);
//     }

//     decrypted_image
//         .save("decrypted_image.png")
//         .expect("Failed to save decrypted image");
// }

// fn main() {
//     encrypt_decrypt_image("path/to/your/image.png");
// }

//

//

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
