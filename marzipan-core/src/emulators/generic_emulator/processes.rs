use alloc::collections::vec_deque::VecDeque;

use itertools::Itertools;

use crate::{
    emulator_core::{EmulatorError, EmulatorResult},
    CoreAddr, CoreSettings,
};

/// Stores program counters for each warrior, up to a capacity defined by
/// `CoreSettings`.
pub struct ProcessQueueSet {
    /// Individual process queues numbered by `warrior_id`s
    queues: Vec<VecDeque<CoreAddr>>,
    /// Number of processes beyond which additional calls to
    /// [`ProcessQueueSet::push_back`] will have no effect.
    max_processes: usize,
}

impl ProcessQueueSet {
    /// If one exists, the next program counter for a warrior
    ///
    /// Returns an [`InternalError`] if `warrior_id` is invalid
    pub fn pop(&mut self, warrior_id: u64) -> EmulatorResult<Option<CoreAddr>> {
        Ok(self
            .queues
            .get_mut(convert_warrior_id(warrior_id)?)
            .ok_or(EmulatorError::InternalError(
                "tried to pop from the process queue for a warrior that \
                 doesn't exist",
            ))?
            .pop_front())
    }

    /// Adds a program counter for a warrior if that warrior is not already at
    /// capacity
    pub fn push_back(
        &mut self,
        value: CoreAddr,
        warrior_id: u64,
    ) -> EmulatorResult<()> {
        let pq = self.queues.get_mut(convert_warrior_id(warrior_id)?).ok_or(
            EmulatorError::InternalError(
                "a process queue doesn't exist for this warrior",
            ),
        )?;
        if pq.len() < self.max_processes {
            pq.push_back(value);
        }
        Ok(())
    }

    /// Empties the process queues for all warriors
    pub fn reset_queues(&mut self) {
        self.queues = vec![VecDeque::new(); self.queues.len()];
    }

    /// Returns the set of `warrior_ids` with non-empty process queues
    pub fn active_warriors(&self) -> Vec<u64> {
        return self
            .queues
            .iter()
            .zip(0..)
            .filter(|&(pq, _)| !pq.is_empty())
            .map(|(_, idx)| idx)
            .collect();
    }

    /// Replace the process queue for a warrior with the input queue, in order
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InternalError`] if any value in the input is
    /// not within the appropriate range for this core size, or if `warrior_id`
    /// is invalid.
    pub fn replace_queue(
        &mut self,
        warrior_id: u64,
        process_queue: &[CoreAddr],
    ) -> EmulatorResult<()> {
        if convert_warrior_id(warrior_id)? >= self.queues.len() {
            Err(EmulatorError::InternalError(
                "attempting to replace the process queue for a warrior id not \
                 currently associated with a process queue",
            ))
        } else if process_queue.len() > self.max_processes {
            Err(EmulatorError::InternalError(
                "Unable to replace a process queue where the length of the
                 new queue is greater the allowed number of processes",
            ))
        } else {
            Ok(())
        }?;
        let queue = self
            .queues
            .get_mut(convert_warrior_id(warrior_id)?)
            .ok_or(EmulatorError::InternalError("Invalid Warrior id"))?;
        *queue = VecDeque::from(process_queue.iter().copied().collect_vec());
        Ok(())
    }

    /// Constructs empty process queues for each warrior from 0 to `warriors-1`
    /// from [`CoreSettings`]
    pub fn new(settings: &CoreSettings) -> Self {
        // If you request more processes or warriors that usize::MAX, you are in
        // for a bad time
        Self {
            queues: vec![
                VecDeque::new();
                usize::try_from(settings.warriors).unwrap_or_default()
            ],
            max_processes: usize::try_from(settings.processes)
                .unwrap_or_default(),
        }
    }

    /// Extract the entire content of a warrior's process queue in order
    ///
    /// # Errors
    ///
    /// Returns an [`EmulatorError::InternalError`] if `warrior_id` is invalid
    pub fn read_queue(&self, warrior_id: u64) -> EmulatorResult<Vec<CoreAddr>> {
        let q = self.queues.get(convert_warrior_id(warrior_id)?).ok_or(
            EmulatorError::InternalError(
                "attempting to read the process queue of a warrior with no \
                 process queue",
            ),
        )?;
        let v: Vec<u32> = q.iter().copied().collect_vec();
        Ok(v)
    }
}

/// Convert u64 warrior to a usize as an index into the queue of process queues
///
/// # Errors
///
/// Returns an error if a `warrior_id` is greater than `usize::MAX`
fn convert_warrior_id(warrior_id: u64) -> EmulatorResult<usize> {
    usize::try_from(warrior_id).map_or(
        Err(EmulatorError::InternalError(
            "unable to convert warrior id into usize",
        )),
        Ok,
    )
}
