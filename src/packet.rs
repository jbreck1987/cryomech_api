/* Defines an abstraction over the link protocols that handles specifics related to the Cryomech API */

use anyhow::anyhow;
use smdp::{SmdpPacketV1, SmdpPacketV2};

const SMDP_OPCODE: u8 = 0x80;

pub enum SmdpVersion {
    // Version 1 has no SRLNO field
    V1,
    // Versions 2 and above have SRLNO field
    V2Plus,
}
/// Cryomech specific wrapper for SMDP packet format.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CPacketSmdp {
    addr: u8,
    data: Vec<u8>,
    srlno: Option<u8>,
}
impl CPacketSmdp {
    pub fn new(addr: u8, srlno: Option<u8>, data: Vec<u8>) -> Self {
        Self { addr, data, srlno }
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
        let cpkt = CPacketSmdp::new(0x10, None, vec![1, 2, 3]);
        let smdpv1_pkt: SmdpPacketV1 = cpkt.clone().into();
        assert_eq!(smdpv1_pkt.data(), cpkt.data);
        assert_eq!(smdpv1_pkt.addr(), cpkt.addr);
        assert_eq!(smdpv1_pkt.cmd_rsp(), SMDP_OPCODE);
    }
    #[test]
    fn test_cpkt_into_smdpv2_ok() {
        let cpkt = CPacketSmdp::new(0x10, Some(0x10), vec![1, 2, 3]);
        let smdpv2_pkt: SmdpPacketV2 = cpkt.clone().try_into().unwrap();
        assert_eq!(smdpv2_pkt.data(), cpkt.data);
        assert_eq!(smdpv2_pkt.addr(), cpkt.addr);
        assert_eq!(smdpv2_pkt.cmd_rsp(), SMDP_OPCODE);
        assert_eq!(smdpv2_pkt.srlno(), cpkt.srlno.unwrap());
    }
    #[test]
    fn test_cpkt_into_smdpv2_err() {
        let cpkt = CPacketSmdp::new(0x10, None, vec![1, 2, 3]);
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
