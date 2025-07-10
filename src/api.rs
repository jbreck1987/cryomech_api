use std::io::{Read, Write};

/* The user facing API for communication with Cryomech compressors */
use anyhow::Result;
use serialport::SerialPort;
use smdp::SmdpPacketHandler;

pub enum SmdpVersion {
    // Version 1 has no SRLNO field
    V1,
    // Versions 2 and above have SRLNO field
    V2Plus,
}

/// SMDP API to Cryomech devices. Assumes point-to-point communication, not multi-drop.
pub struct CryomechApiSmdp<T: Read + Write> {
    smdp_handler: SmdpPacketHandler<T>,
    read_timeout: usize,
    com_port: String,
    dev_addr: u8,
    version: SmdpVersion,
    srlno: usize,
}
impl CryomechApiSmdp<Box<dyn SerialPort>> {
    pub fn new(
        com_port: &str,
        baud: u32,
        read_timeout_ms: usize,
        dev_addr: u8,
        max_framesize: usize,
        version: SmdpVersion,
    ) -> Result<Self> {
        // Build serialport instance then self
        let io = serialport::new(com_port, baud).open()?;
        Ok(Self {
            smdp_handler: SmdpPacketHandler::new(io, read_timeout_ms, max_framesize),
            read_timeout: read_timeout_ms,
            com_port: com_port.into(),
            dev_addr,
            version,
            srlno: 0,
        })
    }
}
