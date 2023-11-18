use std::collections::BTreeMap;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8081")?;
    let mut received_segments: BTreeMap<usize, String> = BTreeMap::new();
    let mut total_segments = 0;

    loop {
        let mut buffer = [0; 2 * 1024];
        let (n, _) = socket.recv_from(&mut buffer)?;

        let data = &buffer[..n];
        let seq_str = String::from_utf8(data[..4].to_vec()).unwrap();
        let seq = seq_str.parse::<usize>().unwrap();
        let current_total_segments_str = String::from_utf8(data[4..8].to_vec()).unwrap();
        let current_total_segments = current_total_segments_str.parse::<usize>().unwrap();
        let segment = String::from_utf8(data[8..].to_vec()).unwrap();

        // Ensure total_segments is set once.
        if total_segments == 0 {
            total_segments = current_total_segments;
        }

        // Store received segments in a map for reordering.
        received_segments.insert(seq, segment);

        // Check if all segments have been received.
        if received_segments.len() == total_segments {
            let message = (0..total_segments)
                .map(|seq| received_segments[&seq].as_str())
                .collect::<String>();
            println!("Received Message: {}", message);
            break;
        }
    }

    Ok(())
}
