use std::collections::HashMap;

use crate::{
    emulator_core::{EmulatorError, EmulatorResult},
    CoreAddr,
};

/// Contains all pspace mappings and values for all warriors
#[derive(Default)]
pub struct PSpace {
    /// number of elements in the pspace of each warrior
    pspace_size: u32,
    /// mapping from a warrior to the pin identifying the pspace which is uses
    warrior_to_pin: HashMap<u64, u64>,
    /// Special `pspace[0]` values which are not shared between warriors
    /// sharing a pin
    zero_index_values: HashMap<u64, CoreAddr>,
    /// pspace buffers indexed by the pins from `warrior_to_pin`
    ///
    /// The 0 index in each pspace buffer is unused.  
    pin_to_pspace: HashMap<u64, Vec<CoreAddr>>,
}

impl PSpace {
    /// construct an empty pspace with no existing warrior mappings
    pub fn new(pspace_size: u32) -> Self {
        Self {
            pspace_size,
            ..Default::default()
        }
    }

    /// read a value from `location` in the pspace owned by `warrior_id`
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InternalError`] if this warrior has no
    /// pspace, or if `location` is not a valid pspace address
    pub fn read(
        &self,
        location: CoreAddr,
        warrior_id: u64,
    ) -> EmulatorResult<CoreAddr> {
        match location {
            0 => self.zero_index_values.get(&warrior_id),
            _ => self
                .warrior_to_pin
                .get(&warrior_id)
                .and_then(|pin| self.pin_to_pspace.get(pin))
                .and_then(|pspace| pspace.get(location as usize)),
        }
        .ok_or(EmulatorError::InternalError("invalid pspace reference"))
        .copied()
    }

    /// Write 'value' to 'location' in the pspsace owned by `warrior_id`
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InternalError`] if this warrior has no
    /// pspace, or if `location` is not a valid pspace address
    pub fn write(
        &mut self,
        location: CoreAddr,
        value: CoreAddr,
        warrior_id: u64,
    ) -> EmulatorResult<()> {
        let location = match location {
            0 => self.zero_index_values.get_mut(&warrior_id),
            _ => self
                .warrior_to_pin
                .get(&warrior_id)
                .and_then(|pin| self.pin_to_pspace.get_mut(pin))
                .and_then(|pspace| pspace.get_mut(location as usize)),
        }
        .ok_or(EmulatorError::InternalError("invalid pspace reference"))?;
        *location = value;
        Ok(())
    }

    /// Allocates a pspace identified by this pin
    ///
    /// # Errors
    ///
    /// If a pspace already exists with this pin, returns an
    /// [`EmulatorError::InternalError`]
    pub fn add_pspace(&mut self, pin: u64) -> EmulatorResult<()> {
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.pin_to_pspace.entry(pin)
        {
            e.insert(vec![0; self.pspace_size as usize]);
            Ok(())
        } else {
            Err(EmulatorError::InternalError(
                "a pspace already exists with this PIN",
            ))
        }
    }

    /// Give a warrior access to an existing pspace with this pin
    ///
    /// # Errors
    ///
    /// If no pspace is associated with this pin, returns an
    /// [`EmulatorError::InternalError`]
    pub fn assign_pspace(
        &mut self,
        warrior_id: u64,
        pin: u64,
    ) -> EmulatorResult<()> {
        if !self.pin_to_pspace.contains_key(&pin) {
            return Err(EmulatorError::InternalError(
                "Pspace with pin {pin} doesn't exist",
            ));
        }
        self.warrior_to_pin.insert(warrior_id, pin);
        self.zero_index_values.insert(warrior_id, 0);
        Ok(())
    }
}
