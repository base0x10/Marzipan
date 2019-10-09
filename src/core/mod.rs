use crate::redcode;

mod micro_ops;
mod primatives;

struct Warrior {
    queue: Vec<usize>,
    pspace: [redcode::Instruction; redcode::PSPACESIZE],
}

pub struct Core {
    mem: [redcode::Instruction; redcode::CORESIZE],
    warriors: [Warrior; redcode::NUMWARRIORS],
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_core() {
        
    }
} 
