/* Defines an abstraction over the link protocols that handles specifics related to the Cryomech API */

use anyhow::{Result, anyhow};
use smdp::{SmdpPacketV1, SmdpPacketV2};

const SMDP_OPCODE: u8 = 0x80;
pub(crate) enum RequestType {
    Read,
    /// Writes to dictionary values need data along with the
    /// dictionary hash/idx
    Write(u32),
}

/// Cryomech specific wrapper for SMDP packet format.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CPacketSmdp {
    addr: u8,
    data: Vec<u8>,
    srlno: Option<u8>,
}
impl CPacketSmdp {
    pub(crate) fn new(
        addr: u8,
        srlno: Option<u8>,
        req_type: RequestType,
        hashval: u16,
        array_idx: u8,
    ) -> Self {
        let (req_type_val, dict_write_data) = match req_type {
            RequestType::Read => (0x63u8, None),
            RequestType::Write(d) => (0x61, Some(d)),
        };

        let mut data = Vec::new();
        data.push(req_type_val);
        data.extend_from_slice(&hashval.to_be_bytes());
        data.push(array_idx);
        if let Some(dict_data) = dict_write_data {
            data.extend_from_slice(&dict_data.to_be_bytes());
        }
        Self { addr, data, srlno }
    }
    pub(crate) fn extract_data(&self) -> Result<u32> {
        // A well-formed response containing data should be 8-bytes
        if self.data.len() == 8 {
            self.data
                .get(4..)
                .and_then(|slice| slice.try_into().ok())
                .map(u32::from_be_bytes)
                .ok_or(anyhow!("Index into response data invalid."))
        } else {
            Err(anyhow!(
                "Response is malformed or is not a response packet."
            ))
        }
    }
    pub(crate) fn set_srlno(&mut self, srlno: u8) {
        self.srlno = Some(srlno)
    }
    pub(crate) fn get_srlno(&self) -> Option<u8> {
        self.srlno
    }
}
impl From<CPacketSmdp> for SmdpPacketV1 {
    fn from(cpkt: CPacketSmdp) -> Self {
        SmdpPacketV1::new(cpkt.addr, SMDP_OPCODE, cpkt.data)
    }
}
impl TryFrom<CPacketSmdp> for SmdpPacketV2 {
    type Error = anyhow::Error;

    fn try_from(cpkt: CPacketSmdp) -> Result<Self, Self::Error> {
        if let Some(srlno) = cpkt.srlno {
            Ok(SmdpPacketV2::new(cpkt.addr, SMDP_OPCODE, srlno, cpkt.data))
        } else {
            Err(anyhow!("Packet has no serial number."))
        }
    }
}
impl From<SmdpPacketV1> for CPacketSmdp {
    fn from(pkt: SmdpPacketV1) -> Self {
        Self {
            addr: pkt.addr(),
            data: Vec::from(pkt.data()),
            srlno: None,
        }
    }
}
impl From<SmdpPacketV2> for CPacketSmdp {
    fn from(pkt: SmdpPacketV2) -> Self {
        Self {
            addr: pkt.addr(),
            data: Vec::from(pkt.data()),
            srlno: Some(pkt.srlno()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cpkt_into_smdpv1() {
        let cpkt = CPacketSmdp {
            addr: 0x10,
            srlno: None,
            data: vec![1, 2, 3],
        };
        let smdpv1_pkt: SmdpPacketV1 = cpkt.clone().into();
        assert_eq!(smdpv1_pkt.data(), cpkt.data);
        assert_eq!(smdpv1_pkt.addr(), cpkt.addr);
        assert_eq!(smdpv1_pkt.cmd_rsp(), SMDP_OPCODE);
    }
    #[test]
    fn test_cpkt_into_smdpv2_ok() {
        let cpkt = CPacketSmdp {
            addr: 0x10,
            srlno: Some(0x17),
            data: vec![1, 2, 3],
        };
        let smdpv2_pkt: SmdpPacketV2 = cpkt.clone().try_into().unwrap();
        assert_eq!(smdpv2_pkt.data(), cpkt.data);
        assert_eq!(smdpv2_pkt.addr(), cpkt.addr);
        assert_eq!(smdpv2_pkt.cmd_rsp(), SMDP_OPCODE);
        assert_eq!(smdpv2_pkt.srlno(), cpkt.srlno.unwrap());
    }
    #[test]
    fn test_cpkt_into_smdpv2_err() {
        let cpkt = CPacketSmdp {
            addr: 0x10,
            srlno: None,
            data: vec![1, 2, 3],
        };
        let result: Result<SmdpPacketV2, _> = cpkt.try_into();
        assert!(result.is_err());
    }
    #[test]
    fn test_smdpv1_into_cpkt() {
        let addr = 0x20;
        let data = vec![4, 5, 6];
        let smdpv1_pkt = SmdpPacketV1::new(addr, SMDP_OPCODE, data.clone());
        let cpkt: CPacketSmdp = smdpv1_pkt.into();
        assert_eq!(cpkt.addr, addr);
        assert_eq!(cpkt.data, data);
        assert_eq!(cpkt.srlno, None);
    }

    #[test]
    fn test_smdpv2_into_cpkt() {
        let addr = 0x30;
        let srlno = 0x42;
        let data = vec![7, 8, 9];
        let smdpv2_pkt = SmdpPacketV2::new(addr, SMDP_OPCODE, srlno, data.clone());
        let cpkt: CPacketSmdp = smdpv2_pkt.into();
        assert_eq!(cpkt.addr, addr);
        assert_eq!(cpkt.data, data);
        assert_eq!(cpkt.srlno, Some(srlno));
    }
}
