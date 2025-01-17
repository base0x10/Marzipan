use num_traits::cast::{FromPrimitive, ToPrimitive};
use redcode;

/// Translates from a redcode instruction to the encoded bytecode representation
#[allow(
    clippy::unwrap_used,
    reason = "redcode guarantees that tests that all redcode types can be \
              converted to u8"
)]
pub fn encode(instr: redcode::Instruction) -> u32 {
    let op = instr.opcode.to_u8().unwrap();
    let modifier = instr.modifier.to_u8().unwrap();
    let a_mode = instr.a_addr_mode.to_u8().unwrap();
    let b_mode = instr.b_addr_mode.to_u8().unwrap();
    u32::from_be_bytes([op, modifier, a_mode, b_mode])
}

/// Translate from an encoded bytecode representation into a structured
/// instruction
///
/// Returns None if 'bytecode' was invalid
pub fn decode(bytecode: u32) -> Option<redcode::Instruction> {
    let parts = u32::to_be_bytes(bytecode);
    let op_part = parts[0];
    let modifier_part = parts[1];
    let a_mode_part = parts[2];
    let b_mode_part = parts[3];
    Some(redcode::Instruction {
        opcode: redcode::Opcode::from_u8(op_part)?,
        modifier: redcode::Modifier::from_u8(modifier_part)?,
        a_addr_mode: redcode::AddrMode::from_u8(a_mode_part)?,
        b_addr_mode: redcode::AddrMode::from_u8(b_mode_part)?,
    })
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod tests {
    // TODO(jespy) convert these to generic tests that can be used for any
    // bytecode implementations
    use std::collections::HashMap;

    use rand::Rng;
    use redcode::{test_utils::all_instructions, Instruction};

    use super::*;

    #[test]
    fn verify_roundtrip_conversion() {
        for instr in all_instructions() {
            let bytecode = encode(instr);
            let roundtrip_converted_instr = decode(bytecode);
            assert_eq!(
                instr,
                roundtrip_converted_instr.unwrap(),
                "An instruction should be unchanged after conversion into and \
                 then out of bytecode"
            );
        }
    }

    #[test]
    fn verify_unique_conversion_with_redcode_pairs() {
        // This test enumerates over 17 million unique pairs of instructions
        // At opt-level=1, it's instant (likely mostly compiled away)
        // but at opt-level=0 it takes a few seconds
        for (a_idx, a) in all_instructions().enumerate() {
            for (b_idx, b) in all_instructions().enumerate() {
                let a_bytecode = encode(a);
                let b_bytecode = encode(b);

                if a_idx != b_idx {
                    assert_ne!(
                        a, b,
                        "Sanity check: a and b have different indexes in \
                         all_instructions, and should not be equal"
                    );
                    assert_ne!(
                        a_bytecode, b_bytecode,
                        "Two distinct instructions shouldn't map to the same \
                         bytecode"
                    );
                } else {
                    assert_eq!(
                        a, b,
                        "Sanity check: a and b have the same index in \
                         all_instructions and should be equal"
                    );
                    assert_eq!(
                        a_bytecode, b_bytecode,
                        "Two equal instructions should always map to the same \
                         bytecode"
                    );
                }
            }
        }
    }

    #[test]
    fn bytecode_conversion_is_injection() {
        // Look for invalid injective mapping from a sample of bytecode space.
        // Only .002% of bytecode space is checked (SAMPLES/u32::MAX).
        // Most runs with a correct impl will never check a valid instruction.
        // But likely to catch an impl that return default values rather errors.
        const SAMPLES: u32 = 100_000;

        let mut bytecode_map: HashMap<Instruction, u32> = HashMap::new();
        let mut rng = rand::thread_rng();

        for _ in 0..SAMPLES {
            // rng is allowed to repeat a bytecode.
            // expected # of collisions is (N-1)^2/(u32::MAX*2)
            let bytecode: u32 = rng.gen();

            let conversion_result = decode(bytecode);
            match conversion_result {
                Some(instr) => {
                    // Check for different bytecode mapped to same instruction
                    // If bytecode are equal, it is an RNG collision
                    // If entry doesn't exist, this is the first bytecode we've
                    // found mapping to this instruction.
                    let existing_value = bytecode_map.insert(instr, bytecode);
                    assert_eq!(
                        bytecode,
                        existing_value.unwrap_or(bytecode),
                        "bytecode_to_redcode should be injective for valid
                        bytecode values.  Instead two different bytecode values
                        map to the same instruction."
                    );
                }
                // None is expected - most bytecode values are invalid
                None => {}
            }
        }
    }
}
