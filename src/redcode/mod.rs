pub const CORESIZE: usize = 8000;
pub const PSPACESIZE: usize = 80;
pub const NUMWARRIORS: usize = 2;
pub const WARRIORLEN: usize = 100;
pub const MAXNUMPROC: usize = 80_000;

mod redaddr;
pub use redaddr::RedAddr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    Dat,
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Jmp,
    Jmz,
    Jmn,
    Djn,
    Spl,
    Slt,
    Cmp,
    Seq,
    Sne,
    Nop,
    Ldp,
    Stp,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Modifier {
    A,
    B,
    AB,
    BA,
    F,
    X,
    I,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AddrMode {
    Immediate,
    Direct,
    IndirectB,
    PredecB,
    PostincB,
    IndirectA,
    PredecA,
    PostincA,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub a_addr_mode: AddrMode,
    pub b_addr_mode: AddrMode,
    pub a_value: RedAddr,
    pub b_value: RedAddr,
}

impl Default for Instruction {
    fn default() -> Instruction {
        Instruction {
            opcode: Opcode::Dat,
            modifier: Modifier::A,
            a_addr_mode: AddrMode::Immediate,
            b_addr_mode: AddrMode::Immediate,
            a_value: Default::default(),
            b_value: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_instr() {
        let def: Instruction = Default::default();
        let manual_def = Instruction {
            opcode: Opcode::Dat,
            modifier: Modifier::A,
            a_addr_mode: AddrMode::Immediate,
            b_addr_mode: AddrMode::Immediate,
            a_value: RedAddr::new(0),
            b_value: RedAddr::new(0),
        };
        assert_eq!(def, manual_def);
    }
}
