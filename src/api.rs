/* The user facing API for communication with Cryomech compressors */

use crate::{
    CResult, Error,
    packet::{CPacketSmdp, RequestType},
};
use serialport::SerialPort;
use smdp::{SmdpPacketHandler, SmdpPacketV2, SmdpPacketV3, format::ResponseCode};
use std::{
    io::{Read, Write},
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Flags the SMDP frame format to be used.
pub enum SmdpVersion {
    // Version 2 has no SRLNO field
    V2,
    // Versions 3 and above have SRLNO field
    V3Plus,
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
    ) -> CResult<Self> {
        // Build serialport instance then self
        let io = serialport::new(com_port, baud)
            .open()
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(Self {
            smdp_handler: SmdpPacketHandler::new(io, read_timeout_ms, max_framesize),
            read_timeout: read_timeout_ms,
            com_port: com_port.into(),
            dev_addr,
            version,
            srlno: 0x17,
        })
    }
    /// In ms
    pub fn read_timeout(&self) -> usize {
        self.read_timeout
    }
    pub fn com_port(&self) -> &str {
        &self.com_port
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
    ) -> CResult<Option<u32>> {
        let is_read = matches!(req_type, RequestType::Read);
        let mut cpkt = CPacketSmdp::new(self.dev_addr, None, req_type, hashval, array_idx);

        // Write and read to/from wire and convert back into CPacketSmdp
        let resp_cpkt: CPacketSmdp = match self.version {
            SmdpVersion::V2 => {
                let req_smdp: SmdpPacketV2 = cpkt.into();
                self.smdp_handler
                    .write_once(&req_smdp)
                    .map_err(Error::propagate_smdp_io)?;
                let resp_smdp: SmdpPacketV2 = self
                    .smdp_handler
                    .poll_once()
                    .map_err(Error::propagate_smdp_io)?;
                match resp_smdp.rsp().map_err(|e| Error::Smdp(e.to_string()))? {
                    ResponseCode::Ok => resp_smdp.into(),
                    other => return Err(Error::InvalidFormat(format!("RSP not OK: {:?}", other))),
                }
            }
            SmdpVersion::V3Plus => {
                cpkt.set_srlno(self.increment_srlno());
                let req_smdp: SmdpPacketV3 = cpkt.try_into().expect("Just set srlno");
                self.smdp_handler
                    .write_once(&req_smdp)
                    .map_err(Error::propagate_smdp_io)?;
                let resp_smdp: SmdpPacketV3 = self
                    .smdp_handler
                    .poll_once()
                    .map_err(Error::propagate_smdp_io)?;
                if resp_smdp.srlno() != req_smdp.srlno() {
                    return Err(Error::InvalidFormat("SRLNO mismatch".to_string()));
                }
                match resp_smdp.rsp().map_err(|e| Error::Smdp(e.to_string()))? {
                    ResponseCode::Ok => resp_smdp.into(),
                    other => return Err(Error::InvalidFormat(format!("RSP not OK: {:?}", other))),
                }
            }
        };
        // Extract data and return (if read-only).
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
    pub fn fw_checksum(&mut self) -> CResult<u32> {
        let data =
            self.comm_handler(RequestType::Read, 0x2B0D, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data)
    }
    /// True if nonvolatile memory was lost
    pub fn mem_loss(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0x801A, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// CPU temperature (°C)
    pub fn cpu_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x3574, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// True if clock battery OK
    pub fn clock_batt_ok(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0xA37A, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// True if clock battery low
    pub fn clock_batt_low(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0x0B8B, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// Elapsed compressor minutes
    pub fn comp_minutes(&mut self) -> CResult<u32> {
        let data =
            self.comm_handler(RequestType::Read, 0x454C, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data)
    }
    /// Compressor motor current draw, in Amps
    pub fn motor_current_amps(&mut self) -> CResult<u32> {
        let data =
            self.comm_handler(RequestType::Read, 0x638B, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data)
    }
    /// In °C
    pub fn input_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x0D8F, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn output_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x0D8F, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn helium_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x0D8F, 0x02)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn oil_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x0D8F, 0x03)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn min_input_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x6E58, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn min_output_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x6E58, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn min_helium_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x6E58, 0x02)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn min_oil_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x6E58, 0x03)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn max_input_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x8A1C, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn max_output_water_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x8A1C, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn max_helium_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x8A1C, 0x02)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In °C
    pub fn max_oil_temp(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x8A1C, 0x03)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// True if a temperature sensor has failed
    pub fn temp_sensor_fail(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0x6E2D, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// True if a pressure sensor has failed
    pub fn pressure_sensor_fail(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0xF82B, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// In PSI Absolute
    pub fn high_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0xAA50, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn low_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0xAA50, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn max_high_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x7A62, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn max_low_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x7A62, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn min_high_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x5E0B, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn min_low_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x5E0B, 0x01)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn avg_high_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x7E90, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// In PSI Absolute
    pub fn avg_low_side_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0xBB94, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// Also known as "bounce". In PSI Absolute
    pub fn high_side_pressure_deriv(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x66FA, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// Average difference in High/Low side pressures in PSI Absolute.
    pub fn avg_delta_pressure(&mut self) -> CResult<f32> {
        let data =
            self.comm_handler(RequestType::Read, 0x319C, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data as f32 * 0.1)
    }
    /// True if the compressor is actively running
    pub fn comp_on(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0x5F95, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
    /// True indicates one or more active errors or warnings.
    pub fn err_code_status(&mut self) -> CResult<bool> {
        let data =
            self.comm_handler(RequestType::Read, 0x65A4, 0x00)?
                .ok_or(Error::InvalidFormat(
                    "Expected data in response, got none.".to_string(),
                ))?;
        Ok(data == 1)
    }
}

/* WRITE METHODS */
impl CryomechApiSmdp<Box<dyn SerialPort>> {
    /// Clears the min/max values for both pressure and temp
    pub fn clear_press_temp_min_max(&mut self) -> CResult<()> {
        let _ = self.comm_handler(RequestType::Write(0x0001), 0xD3DB, 0x00)?;
        Ok(())
    }
    /// Activates the compressor. Returns true if verification successful.
    pub fn start_compressor(&mut self) -> CResult<bool> {
        let _ = self.comm_handler(RequestType::Write(0x0001), 0xD501, 0x00)?;
        std::thread::sleep(Duration::from_secs(1));
        self.comp_on()
    }
    /// Deactivates the compressor. Returns true if verification successful.
    pub fn stop_compressor(&mut self) -> CResult<bool> {
        let _ = self.comm_handler(RequestType::Write(0x0000), 0xC598, 0x00)?;
        std::thread::sleep(Duration::from_secs(1));
        self.comp_on().map(|b| !b)
    }
}

/// Builder for the SMDP API type
pub struct CryomechApiSmdpBuilder {
    read_timeout: usize,
    baud: u32,
    com_port: String,
    dev_addr: u8,
    max_framesize: usize,
    version: SmdpVersion,
}
impl CryomechApiSmdpBuilder {
    pub fn new(com_port: &str) -> Self {
        Self {
            read_timeout: 80,
            baud: 115200,
            com_port: com_port.into(),
            dev_addr: 0x10,
            max_framesize: 64,
            version: SmdpVersion::V2,
        }
    }
    pub fn read_timeout_ms(mut self, timeout: usize) -> Self {
        self.read_timeout = timeout;
        self
    }
    pub fn device_addr(mut self, addr: u8) -> Self {
        self.dev_addr = addr;
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
    pub fn build(self) -> CResult<CryomechApiSmdp<Box<dyn SerialPort>>> {
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
