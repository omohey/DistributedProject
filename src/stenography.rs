use std::fs::File;
use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
    // let mut image_bytes = Vec::new();
    // let mut f = File::open("./src/image.jpg")?;
    // f.read_to_end(&mut image_bytes)?;

    // let mut image_bytes2 = Vec::new();
    // let mut f = File::open("./src/image2.jpg")?;
    // f.read_to_end(&mut image_bytes2)?;

    // let mut f = File::create("./src/encoded.jpg")?;
    // f.write_all(&image_bytes)?;
    // f.write_all(&image_bytes2)?;

    let mut f = File::open("./src/encoded.jpg")?;
    let mut image_bytes = Vec::new();
    f.read_to_end(&mut image_bytes)?;

    // search array for first instance of 0xFFD9
    let mut prevByte = 0;
    let mut indx = 0;
    for i in 0..image_bytes.len() {
        if prevByte == 0xFF && image_bytes[i] == 0xD9 {
            println!("Found 0xFFD9 at index {}", i);
            indx = i + 1;
            break;
        }
        prevByte = image_bytes[i];
    }

    // image_bytes.truncate(indx);
    let imageWrite = &image_bytes[indx..];


    let mut f = File::create("./src/decoded.jpg")?;
    f.write_all(&imageWrite)?;


    Ok(())
}
