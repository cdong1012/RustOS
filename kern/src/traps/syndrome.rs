use aarch64::ESR_EL1;
use crate::console::{kprintln};
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        let hsvc_imm = ESR_EL1::get_value(val as u64, ESR_EL1::ISS_HSVC_IMM);
        let ifsc = hsvc_imm & 0b111111; // bit 0 to 5
        let fault_type = ifsc >> 2; // upper 4 bits of ifsc
        let level = (ifsc & 0b11) as u8; // lower 2 bits of ifsc
        let fault = match fault_type {
            0b0000 => Fault::AddressSize,
            0b0001 => Fault::Translation,
            0b0010 => Fault::AccessFlag,
            0b0011 => Fault::Permission,
            0b1100 => Fault::TlbConflict,
            0b1000 => Fault::Alignment,
            _      => Fault::Other(ifsc as u8),
        };
        fault
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    SimdFp,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;
        let ec = ESR_EL1::get_value(esr as u64, ESR_EL1::EC);
        let iss = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS);
        let hsvc_imm = ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_HSVC_IMM);
        let syndrome : Syndrome = match ec {
            0b000000 => Syndrome::Unknown,
            0b000001 => Syndrome::WfiWfe,
            0b000111 => Syndrome::SimdFp,
            0b001110 => Syndrome::IllegalExecutionState,
            0b010101 => Syndrome::Svc(hsvc_imm as u16), // get the last 16 bits
            0b010110 => Syndrome::Hvc(hsvc_imm as u16),
            0b010111 => Syndrome::Smc(hsvc_imm as u16),
            0b011000 => Syndrome::MsrMrsSystem,
            0b100000 | 0b100001 => { // Instruction abort
                Syndrome::InstructionAbort {
                    kind : Fault::from(esr),
                    level : (hsvc_imm & 0b11) as u8,
                }
            },
            0b100010 => Syndrome::PCAlignmentFault,
            0b100100 | 0b100101 => { // data abort
                let level = (hsvc_imm & 0b11) as u8; // lower 2 bits of ifsc
                Syndrome::DataAbort {
                    kind : Fault::from(esr),
                    level : level,
                }
            },
            0b100110 => Syndrome::SpAlignmentFault,
            0b101100 => Syndrome::TrappedFpu,
            0b101111 => Syndrome::SError,
            0b110000 | 0b110001 => Syndrome::Breakpoint,
            0b110010 | 0b110011 => Syndrome::Step,
            0b110100 | 0b110101 => Syndrome::Watchpoint,
            0b111100 => Syndrome::Brk(ESR_EL1::get_value(esr as u64, ESR_EL1::ISS_BRK_CMMT) as u16),
            _ => Syndrome::Other(esr),
        };
        syndrome
    }
}
