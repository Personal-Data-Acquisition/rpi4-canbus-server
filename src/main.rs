use std::io;
use std::str::from_utf8;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket};
use embedded_can::{Frame, Id, StandardId};
use nmea::{Error, Nmea, SentenceType};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use half::f16;

const GPS_CSV_FILE_PATH: &str = "gps_data.csv";

struct GpsParser {
    nmea: Nmea,
    buffer:Vec<u8>,
    gps_csv_file:File,
}

impl GpsParser {
    fn new() -> io::Result<GpsParser> {
        // Check if the file already exists
        let file_exists = Path::new(GPS_CSV_FILE_PATH).exists();

        // Open or create a file for writing
        let mut gps_csv_file = File::create(GPS_CSV_FILE_PATH)?;

        // Write the CSV header only if the file is newly created
        if !file_exists {
            writeln!(
                gps_csv_file,
                "Fix Time,Fix Date,Fix Type,Latitude,Longitude,Altitude,Speed over Ground,True Course,Number of Fix Satellites,HDOP,VDOP,PDOP,Geoid Separation"
            )?;
        }

        Ok(GpsParser {
            nmea: Nmea::default(),
            buffer: vec![],
            gps_csv_file,
        })
    }
    fn parse(&mut self, frame_data:&[u8]) {
        self.buffer.extend_from_slice(frame_data);
        //nmea lines start with '$', we need to remove anything before that
        //sometimes there are leading \0 we need to skip
        //Find the position of the first '$' character in the buffer
        if let Some(dollar_pos) = self.buffer.iter().position(|&b| b == b'$') {
            //Discard everything before the first '$' character
            self.buffer.drain(..dollar_pos);
        }
        //check if there is a complete message, if there are none, leave the buffer alone
        while let Some(newline_pos) = self.buffer.iter().position(|&b| b == b'\n') {
            //get the next message
            let message = self.buffer.drain(..=newline_pos).collect::<Vec<_>>();
            // Convert to UTF-8 string
            let message_str = from_utf8(&message).unwrap();
            //println!("recieved:{}\n",message_str);
            // Process the NMEA message
            match self.nmea.parse(message_str) {
                Err(err) => {
                    // Handle the error
                    println!("Error parsing NMEA message: {}", err);
                }
                //each message blocked capped off with GLL message
                Ok(SentenceType::GLL)=>{
                    writeln!(self.gps_csv_file, "{},{},{:?},{:.4},{:.4},{},{},{},{},{},{},{},{}",
                             self.nmea.fix_time.unwrap_or_default(),
                             self.nmea.fix_date.unwrap_or_default(),
                             self.nmea.fix_type,
                             self.nmea.latitude.unwrap_or_default(),
                             self.nmea.longitude.unwrap_or_default(),
                             self.nmea.altitude.unwrap_or_default(),
                             self.nmea.speed_over_ground.unwrap_or_default(),
                             self.nmea.true_course.unwrap_or_default(),
                             self.nmea.num_of_fix_satellites.unwrap_or_default(),
                             self.nmea.hdop.unwrap_or_default(),
                             self.nmea.vdop.unwrap_or_default(),
                             self.nmea.pdop.unwrap_or_default(),
                             self.nmea.geoid_separation.unwrap_or_default()).expect("Parse error");

                    println!("recieved:{}",message_str);
                    println!("Fix Time: {}", self.nmea.fix_time.unwrap_or_default());
                    println!("Fix Date: {}", self.nmea.fix_date.unwrap_or_default());
                    println!("Fix Type: {:?}", self.nmea.fix_type);
                    println!("Latitude: {:.4}", self.nmea.latitude.unwrap_or_default());
                    println!("Longitude: {:.4}", self.nmea.longitude.unwrap_or_default());
                    println!("Altitude: {} meters", self.nmea.altitude.unwrap_or_default());
                    println!("Speed over Ground: {} m/s", self.nmea.speed_over_ground.unwrap_or_default());
                    println!("True Course: {} degrees", self.nmea.true_course.unwrap_or_default());
                    println!("Number of Fix Satellites: {}", self.nmea.num_of_fix_satellites.unwrap_or_default());
                    println!("HDOP: {}", self.nmea.hdop.unwrap_or_default());
                    println!("VDOP: {}", self.nmea.vdop.unwrap_or_default());
                    println!("PDOP: {}", self.nmea.pdop.unwrap_or_default());
                    println!("Geoid Separation: {} meters", self.nmea.geoid_separation.unwrap_or_default());
                    println!("Fix Satellites PRNs: {:?}\n\n\n", self.nmea.fix_satellites_prns);
                }
                _ => {}
            }
        }
    }
}


const CAN_INTERFACE_0: &str = "can0";

fn main() -> Result<(), io::Error> {

    // Open CAN sockets for sending and receiving
    let can_socket = CanSocket::open(CAN_INTERFACE_0)?;

    let mut gps_parser= GpsParser::new()?;
    // Read received messages on the other interface
    loop {
        match can_socket.read_frame() {
            Ok(frame) => {
                match frame.id() {
                    Id::Standard(s) if matches!(s.as_raw(), 0x50) =>{
                        gps_parser.parse(frame.data());
                    }
                    Id::Standard(s) if matches!(s.as_raw(), 0x60) =>{
                        println!("message from 0x60");
                        let frame_data=frame.data();
                        let label = [frame_data[0] as char, frame_data[1] as char];

                        // Extract data
                        let mut data = [0.0; 3];
                        let mut index = 2;
                        for i in 0..3 {
                            let f16_bytes: [u8; 2] = [frame_data[index], frame_data[index + 1]];
                            let f16_value = f16::from_le_bytes(f16_bytes);
                            data[i] = f16_value.to_f32();
                            index += 2;
                        }
                        println!("{label:?}: {data:?}")

                        //Some((label[0], label[1], data))
                    }
                    Id::Standard(_) => {}
                    Id::Extended(_) => {}
                }
            }
            Err(err) => {
                eprintln!("Error reading CAN message: {:?}", err);
            }
        }
    }
    Ok(())
}