/* The user facing API for communication with Cryomech compressors */

use crate::packet::{CPacketSmdp, RequestType};
use anyhow::{Result, anyhow};
use serialport::SerialPort;
use smdp::{PacketFormat, SmdpPacketHandler, SmdpPacketV1, SmdpPacketV2, format::ResponseCode};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmdpVersion {
    // Version 1 has no SRLNO field
    V1,
    // Versions 2 and above have SRLNO field
    V2Plus,
}

/// SMDP API to Cryomech devices. Assumes point-to-point communication, not multi-drop.
#[derive(Debug)]
pub struct CryomechApiSmdp<T: Read + Write> {
    smdp_handler: SmdpPacketHandler<T>,
    read_timeout: usize,
    com_port: String,
    dev_addr: u8,
    version: SmdpVersion,
    srlno: u8,
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
            srlno: 0x17,
        })
    }
    /// Increments SRLNO using appropriate logic (valid SRLNO: [16 - 255]). Returns the current
    /// value of the srlno for use.
    fn increment_srlno(&mut self) -> u8 {
        let ret = self.srlno;
        if self.srlno == u8::MAX {
            self.srlno = 0x11
        } else {
            self.srlno += 1;
        }
        ret
    }
    /// Helper function that writes/reads to/from the wire and handles
    /// SMDP protocol error checking
    fn comm_handler(
        &mut self,
        req_type: RequestType,
        hashval: u16,
        array_idx: u8,
    ) -> Result<Option<u32>> {
        let is_read = matches!(req_type, RequestType::Read);
        let mut cpkt = CPacketSmdp::new(self.dev_addr, None, req_type, hashval, array_idx);

        // Write and read to/from wire and convert back into CPacketSmdp
        let resp_cpkt: CPacketSmdp = match self.version {
            SmdpVersion::V1 => {
                let req_smdp: SmdpPacketV1 = cpkt.into();
                self.smdp_handler.write_once(&req_smdp)?;
                let resp_smdp: SmdpPacketV1 = self.smdp_handler.poll_once()?;
                match resp_smdp.rsp()? {
                    ResponseCode::Ok => resp_smdp.into(),
                    other => return Err(anyhow!("RSP not OK: {:?}", other)),
                }
            }
            SmdpVersion::V2Plus => {
                cpkt.set_srlno(self.increment_srlno());
                let req_smdp: SmdpPacketV2 = cpkt.try_into().expect("Just set srlno");
                self.smdp_handler.write_once(&req_smdp)?;
                let resp_smdp: SmdpPacketV2 = self.smdp_handler.poll_once()?;
                if resp_smdp.srlno() != req_smdp.srlno() {
                    return Err(anyhow!("SRLNO mismatch"));
                }
                match resp_smdp.rsp()? {
                    ResponseCode::Ok => resp_smdp.into(),
                    other => return Err(anyhow!("RSP not OK: {:?}", other)),
                }
            }
        };
        // Extract data and return (if read-only), otherwise return None.
        if is_read {
            resp_cpkt.extract_data().map(Some)
        } else {
            Ok(None)
        }
    }
}

/* READ-ONLY METHODS */
impl CryomechApiSmdp<Box<dyn SerialPort>> {
    /// Firmware checksum
    pub fn fw_checksum(&mut self) -> Result<String> {
        todo!()
    }
    /// True if nonvolatile memory was lost
    pub fn mem_loss(&mut self) -> Result<bool> {
        todo!()
    }
    /// CPU temperature (°C)
    pub fn cpu_temp(&mut self) -> Result<f32> {
        todo!()
    }
    /// True if clock battery OK
    pub fn clock_batt_ok(&mut self) -> Result<bool> {
        todo!()
    }
    /// True if clock battery low
    pub fn clock_batt_low(&mut self) -> Result<bool> {
        todo!()
    }
    /// Elapsed compressor minutes
    pub fn comp_minutes(&mut self) -> Result<u32> {
        todo!()
    }
    /// Compressor motor current draw, in Amps
    pub fn motor_current_amps(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn input_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn output_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn helium_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn oil_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn min_input_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn min_output_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn min_helium_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn min_oil_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn max_input_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn max_output_water_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn max_helium_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In °C
    pub fn max_oil_temp(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn high_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn low_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn max_high_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn max_low_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn min_high_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn min_low_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn avg_high_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn avg_low_side_pressure(&mut self) -> Result<u32> {
        todo!()
    }
    /// In PSI Absolute
    pub fn high_side_pressure_deriv(&mut self) -> Result<u32> {
        todo!()
    }
    /// True if the compressor is actively running
    pub fn comp_on(&mut self) -> Result<bool> {
        todo!()
    }
    /// True indicates one or more active errors or warnings.
    pub fn err_code_status(&mut self) -> Result<bool> {
        todo!()
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

#[cfg(test)]
mod test {
    use super::*;
}
