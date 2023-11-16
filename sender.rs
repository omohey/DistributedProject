use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080")?;
    let destination = "127.0.0.1:8081";

    let message =
        "This is a long message that needs to be segmented and reassembled at the receiver's end.";
    let segment_size = 10;

    let total_segments = (message.len() + segment_size - 1) / segment_size; // Calculate the total number of segments.

    let segments: Vec<String> = message
        .chars()
        .collect::<String>()
        .as_bytes()
        .chunks(segment_size)
        .map(|s| String::from_utf8(s.to_vec()).unwrap())
        .collect();

    for (seq, segment) in segments.iter().enumerate() {
        let seq_str = format!("{:04}", seq);
        let packet = format!("{:04}{:04}{}", seq, total_segments, segment);
        socket.send_to(packet.as_bytes(), destination)?;
    }

    Ok(())
}
