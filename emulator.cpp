// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include <algorithm>
#include <iostream>
#include <cassert>

#include "./emulator.h"
#include "./emulator-vars.h"
#include "./core.h"
#include "./warrior.h"

/* 
 * The base emulator class includes most of the fast paths in the system
 * especially load_warrior, clear, and run(0)
 * 
 * base emulator does little error checking or input validation.  
 * It does not insure that it has been setup properly before it begins
 * 
 * its derived classes may behave differently and check inputs
 */

int Emulator::run(int steps) {
    assert(steps == 0 && "Base emulator does not support partial runs");

    return 1;
}

void Emulator::load_warrior(Warrior w, int offset, int war_num) {
    if (war_num == 1) {
        core.task_queue_w1.emplace_front(w.start_pos);
    } else if (war_num == 2) {
        core.task_queue_w2.emplace_front(w.start_pos);
    } else {
        assert(0 && "tried to load a warrior with number other than 1 or 2");
    }

    std::copy(std::begin(w.code),
        std::end(w.code),
        std::begin(core.memory) + offset);
}

// Print should only ever be used for debugging.
// to query the internal state of the emulator, use a derived class
void Emulator::print() {
    for (auto i = 0u ; i < core_size ; i++) {
        if (i%16 == 0) std::cout << "\n";
        std::cout << core.memory[i].op << " ";
    }
    std::cout << std::endl;
}

void Emulator::clear() {
    // reset task queues
    core.task_queue_w1.clear();
    core.task_queue_w2.clear();
    // reset memory array - this seems to be the most idiomatic way to do it
    std::fill(std::begin(core.memory), std::end(core.memory), initial_instr);
}
