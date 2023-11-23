use std::io;
use std::time::Duration;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket};
use embedded_can::{Frame, StandardId};

const CAN_INTERFACE_SEND: &str = "can0";
const CAN_INTERFACE_RECEIVE: &str = "can1";

fn main() -> Result<(), io::Error> {
    // Open CAN sockets for sending and receiving
    let socket_send = CanSocket::open(CAN_INTERFACE_SEND)?;
    let socket_receive = CanSocket::open(CAN_INTERFACE_RECEIVE)?;

    // Send a CAN message 
    let id = StandardId::new(0x100).unwrap();
    let message = CanFrame::new(id, &[1, 2, 3, 4]).unwrap(); 
    socket_send.write_frame(&message)?;
   
    std::thread::sleep(Duration::from_secs(1));
     // Read received messages on the other interface     
     loop {         
         match socket_receive.read_frame() {
            Ok(frame) => {                 
                println!("Received CAN message on {}: {:?}", CAN_INTERFACE_RECEIVE, frame);
                break; 
            }             
            Err(err) => {       
                    eprintln!("Error reading CAN message: {:?}", err); 
                    break;                
                          
           }        
        }    
    }
    Ok(())
}
