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

pub struct CryomechApiSmdpBuilder {
    read_timeout: usize,
    baud: u32,
    com_port: String,
    dev_addr: u8,
    max_framesize: usize,
    version: SmdpVersion,
}
impl CryomechApiSmdpBuilder {
    pub fn new(com_port: &str, dev_addr: u8) -> Self {
        Self {
            read_timeout: 80,
            baud: 115200,
            com_port: com_port.into(),
            dev_addr,
            max_framesize: 64,
            version: SmdpVersion::V1,
        }
    }
    pub fn read_timeout_ms(mut self, timeout: usize) -> Self {
        self.read_timeout = timeout;
        self
    }
    pub fn version(mut self, version: SmdpVersion) -> Self {
        self.version = version;
        self
    }
    pub fn baud(mut self, baud: u32) -> Self {
        self.baud = baud;
        self
    }
    pub fn max_framesize(mut self, size: usize) -> Self {
        self.max_framesize = size;
        self
    }
    pub fn build(self) -> Result<CryomechApiSmdp<Box<dyn SerialPort>>> {
        CryomechApiSmdp::new(
            &self.com_port,
            self.baud,
            self.read_timeout,
            self.dev_addr,
            self.max_framesize,
            self.version,
        )
    }
}
