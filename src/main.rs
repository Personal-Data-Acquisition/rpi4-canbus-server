use std::str::from_utf8;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket};
use embedded_can::{Frame, Id, StandardId};
use nmea::{Nmea, SentenceType};
use half::f16;
use sqlx::SqlitePool;
use anyhow::{anyhow, Result};
use sqlx::sqlite::SqliteConnectOptions;

const SQLITE_DATABASE_PATH: &str = "sensor_data.db";

struct GpsParser {
    nmea: Nmea,
    buffer:Vec<u8>,
}

impl GpsParser {
    async fn new(pool: &SqlitePool) -> Result<Self> {
        // Create a GPS data table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS gps_data (
            fix_time TEXT,
            fix_date TEXT,
            fix_type TEXT,
            latitude REAL,
            longitude REAL,
            altitude REAL,
            speed_over_ground REAL,
            true_course REAL,
            num_of_fix_satellites INTEGER,
            hdop REAL,
            vdop REAL,
            pdop REAL,
            geoid_separation REAL,
            PRIMARY KEY (fix_time, fix_date)
        )",
        ).execute(pool).await?;

        Ok(Self {
            nmea: Nmea::default(),
            buffer: vec![],
        })
    }
    fn fix_type_to_string(fix_type: nmea::sentences::FixType) -> &'static str {
        match fix_type {
            nmea::sentences::FixType::Invalid => "Invalid",
            nmea::sentences::FixType::Gps => "Gps",
            nmea::sentences::FixType::DGps => "DGps",
            nmea::sentences::FixType::Pps => "Pps",
            nmea::sentences::FixType::Rtk => "Rtk",
            nmea::sentences::FixType::FloatRtk => "FloatRtk",
            nmea::sentences::FixType::Estimated => "Estimated",
            nmea::sentences::FixType::Manual => "Manual",
            nmea::sentences::FixType::Simulation => "Simulation",
        }
    }

    async fn parse(&mut self, frame_data:&[u8],pool: &SqlitePool)->Result<()> {
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
                    sqlx::query(
                        "INSERT INTO gps_data (fix_time, fix_date, fix_type, latitude, longitude, altitude, speed_over_ground, true_course, num_of_fix_satellites, hdop, vdop, pdop, geoid_separation) \
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                        .bind(self.nmea.fix_time.unwrap_or_default().to_string())
                        .bind(self.nmea.fix_date.unwrap_or_default().to_string())
                        .bind(Self::fix_type_to_string(self.nmea.fix_type.unwrap()))
                        .bind(self.nmea.latitude.unwrap_or_default())
                        .bind(self.nmea.longitude.unwrap_or_default())
                        .bind(self.nmea.altitude.unwrap_or_default())
                        .bind(self.nmea.speed_over_ground.unwrap_or_default())
                        .bind(self.nmea.true_course.unwrap_or_default())
                        .bind(self.nmea.num_of_fix_satellites.unwrap_or_default())
                        .bind(self.nmea.hdop.unwrap_or_default())
                        .bind(self.nmea.vdop.unwrap_or_default())
                        .bind(self.nmea.pdop.unwrap_or_default())
                        .bind(self.nmea.geoid_separation.unwrap_or_default())
                        .execute(pool).await?;


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
        Ok(())
    }
}

struct Mpu9250Parser {
    readings:[[f32; 3]; 3],
}
impl Mpu9250Parser {
    async fn new(pool: &SqlitePool) -> Result<Self> {
        // Create a GPS data table if it doesn't exist
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accelerometer_data (
            id INTEGER PRIMARY KEY,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            accelerometer_x REAL,
            accelerometer_y REAL,
            accelerometer_z REAL,
            magnetometer_x REAL,
            magnetometer_y REAL,
            magnetometer_z REAL,
            gyroscope_x REAL,
            gyroscope_y REAL,
            gyroscope_z REAL
        )",
        ).execute(pool).await?;

        Ok(Self {
            readings:[[0.0; 3]; 3],
        })
    }
    async fn parse(&mut self, frame_data:&[u8],pool: &SqlitePool)->Result<()> {
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

        // Populate readings array based on label
        let label_index = match label {
            ['a', 'c'] => 0,
            ['m', 'g'] => 1,
            ['g', 'y'] => 2,
            _ => return Err(anyhow!("Invalid label")),
        };
        self.readings[label_index] = data;

        //println!("{:?}: {:?}", label, data);

        if label_index==2 {
            sqlx::query(
                "INSERT INTO accelerometer_data (accelerometer_x, accelerometer_y, accelerometer_z, magnetometer_x, magnetometer_y, magnetometer_z, gyroscope_x, gyroscope_y, gyroscope_z)\
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(self.readings[0][0])
                .bind(self.readings[0][1])
                .bind(self.readings[0][2])
                .bind(self.readings[1][0])
                .bind(self.readings[1][1])
                .bind(self.readings[1][2])
                .bind(self.readings[2][0])
                .bind(self.readings[2][1])
                .bind(self.readings[2][2])
                .execute(pool).await?;
        }

        Ok(())
    }


}


const CAN_INTERFACE_0: &str = "can0";
//tokio for sql operations
#[tokio::main]
async fn main() -> Result<()> {

    // Open CAN sockets for sending and receiving
    let can_socket = CanSocket::open(CAN_INTERFACE_0)?;


    //load up database file
    let options = SqliteConnectOptions::new()
        .filename(SQLITE_DATABASE_PATH)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    let mut gps_parser= GpsParser::new(&pool).await?;
    let mut acc_parser= Mpu9250Parser::new(&pool).await?;

    // Read received messages on the other interface
    loop {
        match can_socket.read_frame() {
            Ok(frame) => {
                match frame.id() {
                    Id::Standard(s) if matches!(s.as_raw(), 0x50) =>{
                        gps_parser.parse(frame.data(),&pool).await?;
                    }
                    Id::Standard(s) if matches!(s.as_raw(), 0x60) =>{
                        acc_parser.parse(frame.data(),&pool).await?;
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