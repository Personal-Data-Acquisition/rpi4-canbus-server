use std::io;
use std::time::{Duration, Instant};
use socketcan::{CanSocket, Socket};

const CAN_INTERFACE_RECEIVE: &str = "can0";
const MEASUREMENT_INTERVAL_SECONDS: u64 = 5;

fn main() -> Result<(), io::Error> {
    // Open CAN sockets for sending and receiving
    let socket_receive = CanSocket::open(CAN_INTERFACE_RECEIVE)?;

    // Initialize counters
    let mut message_count = 0;
    let mut start_time = Instant::now();

    // Read received messages on can interface
    loop {
        match socket_receive.read_frame() {
            Ok(_) => {
                message_count += 1;
                // Check if measurement interval has passed
                if start_time.elapsed() >= Duration::from_secs(MEASUREMENT_INTERVAL_SECONDS) {
                    let elapsed_time = start_time.elapsed().as_secs_f64();
                    let throughput = message_count as f64 / elapsed_time;

                    println!("Throughput: {:.2} messages/second\n\t\t{:.2}kbps", throughput,(throughput*8.0*8.0/1024.0));

                    // Reset counters
                    message_count = 0;
                    start_time = Instant::now();
                }
            }
            Err(err) => {
                eprintln!("Error reading CAN message: {:?}", err);
            }
        }
    }
}
