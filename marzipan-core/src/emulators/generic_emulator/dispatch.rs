use itertools::Itertools;
use redcode;

use super::{
    bytecode,
    emulation_operations::{
        arithmetic_op, cmp_op, dat_op, djn_op, jmn_op, jmp_op, jmz_op, ldp_op,
        mov_op, nop_op, slt_op, sne_op, spl_op, stp_op, OpInputs,
    },
    operands,
    processes::ProcessQueueSet,
    pspace,
};
use crate::{
    emulator_core::{
        CoreSettings, EmulatorCore, EmulatorError, EmulatorResult,
    },
    BytecodeInstructionIdentifier, CoreAddr,
};

/// Contains the state for the generic redcode emulator
pub struct Emulator {
    /// Core memory, process queues, and pspsace
    state: EmulatorState,
    /// Active settings applied to this emulator
    config: CoreSettings,
}

/// Mutable state of the current emulator and core memory
struct EmulatorState {
    /// Per-warrior FIFO queue of instruction addresses, indexed by warrior IDs
    /// from 0 to [`CoreSettings`]'s `warriors - 1`
    pq: ProcessQueueSet,
    /// Instruction and field values currently stored in the core
    core: Vec<redcode::CompleteInstruction>,
    /// Pspace state for warriors that have been assigned PINs
    pspace: pspace::PSpace,
}

impl Emulator {
    /// Construct and initialize the emulator
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] for an incompatible set of
    /// parameters.
    pub fn new(
        core_size: u64,
        pspace_size: u64,
        warriors: u64,
        processes: u64,
    ) -> EmulatorResult<Self> {
        if core_size > u64::from(CoreAddr::MAX) {
            Err(EmulatorError::InvalidParam("core_size is too large"))
        } else if pspace_size > core_size {
            Err(EmulatorError::InvalidParam("pspace_size is too large"))
        } else {
            Ok(())
        }?;
        let config = CoreSettings {
            core_size,
            pspace_size,
            warriors,
            processes,
            bytecode_format: None,
        };
        let state = EmulatorState {
            pq: ProcessQueueSet::new(&config),
            core: vec![
                redcode::CompleteInstruction::default();
                usize::try_from(core_size).map_err(|_err| {
                    EmulatorError::InternalError("impossibly large core_size")
                })?
            ],
            pspace: pspace::PSpace::new(
                pspace_size.try_into().unwrap_or_default(),
            ),
        };
        Ok(Self { state, config })
    }

    /// Removes any existing pspace state, and writes a configuration based on
    /// the provided pspace map from pin to warrior id
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if any of the warriors in the
    /// pspace map is not valid
    pub fn initialize_pspace(
        &mut self,
        pspace_map: &[(u64, u64)],
    ) -> EmulatorResult<()> {
        let (pins, warriors): (Vec<u64>, Vec<u64>) =
            pspace_map.iter().copied().unzip();
        for w in warriors {
            self.validate_warrior_param(w, "invalid warrior ID in pspace map")?;
        }
        // Write an empty pspace config
        self.state.pspace = pspace::PSpace::new(
            self.config.pspace_size.try_into().unwrap_or_default(),
        );
        // Create a space for each pin
        for pin in pins.iter().unique() {
            self.state.pspace.add_pspace(*pin)?;
        }
        // Perform warrior to pin assignments
        for &(pin, warrior_id) in pspace_map.iter() {
            self.state.pspace.assign_pspace(warrior_id, pin)?;
        }
        Ok(())
    }

    /// executes a single instruction at pc as this warrior
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if `pc` or `warrior_id` are
    /// invalid
    fn step_emulator(
        &mut self,
        pc: redcode::FieldValue,
        warrior_id: u64,
    ) -> EmulatorResult<()> {
        // Evaluate A and B operands
        // Cache the indexes and values at PC, a_target, b_target
        let regs = operands::evaluate(pc, &mut self.state.core)?;
        let inputs = OpInputs {
            warrior_id,
            regs: &regs,
            core_size: self.config.core_size.try_into().map_or(
                Err(EmulatorError::InternalError("impossibly large core_size")),
                Ok,
            )?,
            pq: &mut self.state.pq,
            core: &mut self.state.core,
            pspace: &mut self.state.pspace,
        };

        // Execute the instruction at PC
        match regs.current.instr.opcode {
            redcode::Opcode::Dat => dat_op(inputs),
            redcode::Opcode::Mov => mov_op(inputs),
            redcode::Opcode::Add
            | redcode::Opcode::Sub
            | redcode::Opcode::Mul
            | redcode::Opcode::Div
            | redcode::Opcode::Mod => arithmetic_op(inputs),
            redcode::Opcode::Jmp => jmp_op(inputs),
            redcode::Opcode::Jmz => jmz_op(inputs),
            redcode::Opcode::Jmn => jmn_op(inputs),
            redcode::Opcode::Djn => djn_op(inputs),
            redcode::Opcode::Spl => spl_op(inputs),
            redcode::Opcode::Slt => slt_op(inputs),
            redcode::Opcode::Cmp | redcode::Opcode::Seq => cmp_op(inputs),
            redcode::Opcode::Sne => sne_op(inputs),
            redcode::Opcode::Nop => nop_op(inputs),
            redcode::Opcode::Ldp => ldp_op(inputs),
            redcode::Opcode::Stp => stp_op(inputs),
        }
    }

    /// Checks that a core address or value parameter is valid for this core
    /// size
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if `val` is greater than the
    /// core size minus 1
    fn validate_addr_param(
        &self,
        val: CoreAddr,
        msg: &'static str,
    ) -> EmulatorResult<CoreAddr> {
        if u64::from(val) < self.config.core_size {
            Ok(val)
        } else {
            Err(EmulatorError::InvalidParam(msg))
        }
    }

    /// Checks that a warrior id parameter is valid for the number of warriors
    /// that this core is currently configured for.
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InvalidParam`] if `val` is greater than
    /// `warriors - 1`
    const fn validate_warrior_param(
        &self,
        warrior_id: u64,
        msg: &'static str,
    ) -> EmulatorResult<u64> {
        if warrior_id < self.config.warriors {
            Ok(warrior_id)
        } else {
            Err(EmulatorError::InvalidParam(msg))
        }
    }
}

impl EmulatorCore for Emulator {
    fn step(&mut self, warrior_id: u64) -> EmulatorResult<Option<CoreAddr>> {
        self.validate_warrior_param(warrior_id, "invalid warrior id")?;
        Ok(match self.state.pq.pop(warrior_id)? {
            // This warrior has an active process
            Some(pc) => {
                self.step_emulator(pc, warrior_id)?;
                Some(pc)
            }
            // This warrior has no active processes
            None => None,
        })
    }

    fn run(
        &mut self,
        cycles: u64,
        warriors_remaining: u64,
    ) -> EmulatorResult<u64> {
        let mut cycles_executed = 0;
        // This has the effect that two warriors may kill each other during one
        // cycle resulting in a tie for both
        // TODO(jespy) compare this behavior w/ pmars
        while cycles_executed < cycles
            && self.active_warrior_set().len() as u64 > warriors_remaining
        {
            for w in self.active_warrior_set() {
                self.step(w)?;
            }
            cycles_executed = cycles_executed.saturating_add_signed(1);
        }
        Ok(cycles_executed)
    }

    /// Query per-core settings such as `bytecode_format` and `core_size`.
    fn core_settings(&self) -> CoreSettings {
        self.config.clone()
    }

    /// Query the value stored at an address in the core.
    fn read_core(
        &self,
        addr: CoreAddr,
    ) -> EmulatorResult<(BytecodeInstructionIdentifier, CoreAddr, CoreAddr)>
    {
        self.validate_addr_param(addr, "invalid address to read from core")?;
        let redcode_res = self.state.core.get(addr as usize).ok_or(
            EmulatorError::InternalError(
                "core isn't large enough to read from a valid address",
            ),
        )?;
        let bytecode = self.rc_to_bytecode(redcode_res.instr);
        Ok((bytecode, redcode_res.a_field, redcode_res.b_field))
    }

    /// Modify the value stored at an address in the core.
    fn write_core(
        &mut self,
        addr: CoreAddr,
        insn: BytecodeInstructionIdentifier,
        a_field: CoreAddr,
        b_field: CoreAddr,
    ) -> EmulatorResult<()> {
        self.validate_addr_param(addr, "invalid address to write to core")?;
        self.validate_addr_param(addr, "invalid a_field to write to core")?;
        self.validate_addr_param(addr, "invalid b_field to write to core")?;
        let instr = self.bytecode_to_rc(insn)?;
        let complete_value = redcode::CompleteInstruction {
            instr,
            a_field,
            b_field,
        };
        self.state.core.get_mut(addr as usize).map_or(
            Err(EmulatorError::InternalError(
                "generic_emulator core isn't large enough to write to a valid \
                 address",
            )),
            |location| {
                *location = complete_value;
                Ok(())
            },
        )
    }

    /// Read a value from the pspace owned by some warrior.
    fn read_pspace(
        &self,
        warrior_id: u64,
        addr: CoreAddr,
    ) -> EmulatorResult<CoreAddr> {
        self.validate_warrior_param(
            warrior_id,
            "invalid warrior to read pspace",
        )?;
        if u64::from(addr) >= self.config.pspace_size {
            return Err(EmulatorError::InvalidParam(
                "pspace address larger than configured pspace size",
            ));
        }
        self.state.pspace.read(addr, warrior_id).map_or(
            Err(EmulatorError::InvalidParam(
                "pspace not configured for this warrior",
            )),
            Ok,
        )
    }

    /// Write a value to the pspace owned by some warrior.
    ///
    /// Aside from the special address of zero, warriors that share a pin
    /// read from and write to the same pspace.
    fn write_pspace(
        &mut self,
        warrior_id: u64,
        addr: CoreAddr,
        value: CoreAddr,
    ) -> EmulatorResult<()> {
        self.validate_addr_param(
            value,
            "tried to write invalid value to pspace",
        )?;
        self.validate_warrior_param(
            warrior_id,
            "invalid warrior to read pspace",
        )?;
        if u64::from(addr) >= self.config.pspace_size {
            return Err(EmulatorError::InvalidParam(
                "pspace address larger than configured pspace size",
            ));
        }
        match self.state.pspace.write(addr, value, warrior_id) {
            Ok(()) => Ok(()),
            // TODO(jespy) implement a way to check if a warrior should be
            // configured with a pspace rather than assume that
            // internal error is only caused by requesting for warrior without
            // pspace
            Err(_) => Err(EmulatorError::InvalidParam(
                "pspace not configured for this warrior",
            )),
        }
    }

    /// Removes any state associated with the core.  Writes the new values to
    /// the entire core.  All observable state is removed including process
    /// queues, partial-cycle state, and all pspace mapping and values.  
    fn reset_core(
        &mut self,
        initial_instr: BytecodeInstructionIdentifier,
        initial_a: CoreAddr,
        initial_b: CoreAddr,
    ) -> EmulatorResult<()> {
        let initial_complete_instr = redcode::CompleteInstruction {
            instr: self.bytecode_to_rc(initial_instr)?,
            a_field: initial_a,
            b_field: initial_b,
        };
        self.state.pq.reset_queues();
        self.state.core =
            vec![
                initial_complete_instr;
                usize::try_from(self.config.core_size).map_err(|_err| {
                    EmulatorError::InternalError("impossibly large core_size")
                })?
            ];
        self.state.pspace = pspace::PSpace::new(
            CoreAddr::try_from(self.config.pspace_size).unwrap_or_default(),
        );
        Ok(())
    }

    /// Returns the set of warriors with non-empty process queues.
    fn active_warrior_set(&self) -> Vec<u64> {
        self.state.pq.active_warriors()
    }

    /// Returns a copy of the process queue for a warrior.  This will be empty
    /// for inactive warriors.  Otherwise the next process to execute is first.
    fn read_process_queue(
        &self,
        warrior_id: u64,
    ) -> EmulatorResult<Vec<CoreAddr>> {
        self.state.pq.read_queue(warrior_id)
    }

    /// Replaces the warriors current processes with the values in the input.
    fn replace_process_queue(
        &mut self,
        warrior_id: u64,
        process_queue: &[CoreAddr],
    ) -> EmulatorResult<()> {
        for e in process_queue {
            self.validate_addr_param(
                *e,
                "attempted to add an invalid core address to a process queue",
            )?;
        }
        self.state.pq.replace_queue(warrior_id, process_queue)
    }

    /// Convert this emulator's bytecode representation to a redcode Instruction
    ///
    /// # Errors
    ///
    /// Returns an error if the bytecode instruction cannot be decoded
    fn bytecode_to_rc(
        &self,
        bytecode_instr: BytecodeInstructionIdentifier,
    ) -> EmulatorResult<redcode::Instruction> {
        bytecode::decode(bytecode_instr)
            .ok_or(EmulatorError::InvalidParam("invalid bytecode instruction"))
    }

    /// Convert a redcode instruction to this emulator's bytecode representation
    fn rc_to_bytecode(
        &self,
        redcode_instr: redcode::Instruction,
    ) -> BytecodeInstructionIdentifier {
        bytecode::encode(redcode_instr)
    }
}
