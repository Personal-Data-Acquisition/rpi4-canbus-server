use std::io;
use std::str::from_utf8;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket};
use embedded_can::{Frame, StandardId};
use nmea::Nmea;



const CAN_INTERFACE_0: &str = "can0";

fn main() -> Result<(), io::Error> {
    // Open CAN sockets for sending and receiving
    //let socket_send = CanSocket::open(CAN_INTERFACE_SEND)?;
    let can_socket = CanSocket::open(CAN_INTERFACE_0)?;

    let mut nmea = Nmea::default();
    let mut buffer = Vec::new();
    // Read received messages on the other interface
    loop {
        match can_socket.read_frame() {
            Ok(frame) => {
                buffer.extend_from_slice(frame.data());
                //nmea lines start with '$', we need to remove anything before that
                //sometimes there are leading \0 we need to skip
                //Find the position of the first '$' character in the buffer
                if let Some(dollar_pos) = buffer.iter().position(|&b| b == b'$') {
                    //Discard everything before the first '$' character
                    buffer.drain(..dollar_pos);
                }
                //check if there is a complete message, if there are none, leave the buffer alone
                while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                    //get the next message
                    let message = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                    // Convert to UTF-8 string
                    let message_str = from_utf8(&message).unwrap();
                    //println!("recieved:{}\n",message_str);
                    // Process the NMEA message
                    nmea.parse(message_str).unwrap();
                    println!("Fix Time: {}", nmea.fix_time.unwrap_or_default());
                    println!("Fix Date: {}", nmea.fix_date.unwrap_or_default());
                    println!("Fix Type: {:?}", nmea.fix_type);
                    println!("Latitude: {:.4}", nmea.latitude.unwrap_or_default());
                    println!("Longitude: {:.4}", nmea.longitude.unwrap_or_default());
                    println!("Altitude: {} meters", nmea.altitude.unwrap_or_default());
                    println!("Speed over Ground: {} m/s", nmea.speed_over_ground.unwrap_or_default());
                    println!("True Course: {} degrees", nmea.true_course.unwrap_or_default());
                    println!("Number of Fix Satellites: {}", nmea.num_of_fix_satellites.unwrap_or_default());
                    println!("HDOP: {}", nmea.hdop.unwrap_or_default());
                    println!("VDOP: {}", nmea.vdop.unwrap_or_default());
                    println!("PDOP: {}", nmea.pdop.unwrap_or_default());
                    println!("Geoid Separation: {} meters", nmea.geoid_separation.unwrap_or_default());
                    println!("Fix Satellites PRNs: {:?}", nmea.fix_satellites_prns);
                }

            }
            Err(err) => {
                eprintln!("Error reading CAN message: {:?}", err);
            }
        }
    }
    Ok(())
}