// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include <algorithm>
#include <iostream>
#include <cassert>

#include "./emulator.h"
#include "../emulator-vars.h"
#include "./core.h"
#include "../warrior.h"

/*
 * The base emulator class includes most of the fast paths in the system
 * especially load_warrior, clear, and run(0)
 *
 * base emulator does little error checking or input validation.
 * It does not insure that it has been setup properly before it begins
 *
 * its derived classes may behave differently and check inputs
 */


// returns the distance that must be traversed through circular memory
// to walk forward from addr_1, addr_2, eg. dist(0, core_size) == 0
int dist(int addr_1, int addr_2) {
    // due to modulo arithmatic, this works for all values of addr_{1,2}
    // this would be simplier if % did modulo rather than remainder
    addr_1 = addr_1 % core_size;
    return (addr_2 - addr_1 + core_size) % core_size;
}

void Emulator::load_warrior(Warrior w, int offset, int war_num) {
    // make offset valid address, and blow up on negative offset/start_pos
    assert(offset >= 0 && w.start_pos >= 0);
    offset = (offset + w.start_pos) % core_size;

    if (war_num == 1) {
        // input validation
        assert(offset_w1 == NOT_LOADED);

        if (offset_w2 != NOT_LOADED) {
            assert(
                dist(offset, offset_w2) >= max_warrior_size + min_separation);
            assert(
                dist(offset_w2, offset) >= max_warrior_size + min_separation);
        }

        offset_w1 = offset;

        // you can only set your initial address within your own memory
        core.task_queue_w1.push(offset);
        std::cout<<"Just pushed to queue 1 "<< core.task_queue_w1.front();

    } else if (war_num == 2) {
        // input validation
        assert(offset_w2 == NOT_LOADED);

        // check that min_separation is observed
        if (offset_w1 != NOT_LOADED) {
            assert(
                dist(offset, offset_w1) >= max_warrior_size + min_separation);
            assert(
                dist(offset_w1, offset) >= max_warrior_size + min_separation);
        }

        offset_w2 = offset;

        // you can only set your initial address within your own memory
        core.task_queue_w2.push(offset);
        std::cout<<"Just pushed to queue 2 "<< core.task_queue_w2.front();

    } else {
        assert(0 && "tried to load a warrior with number other than 1 or 2");
    }

    std::copy(std::begin(w.code),
        std::end(w.code),
        std::begin(core.memory) + offset);
}

// Print should only ever be used for debugging.
// to query the internal state of the emulator, use a derived interactive class
void Emulator::print() {
    for (auto i = 0u ; i < core_size ; i++) {
        if (i%16 == 0) std::cout << "\n";
        std::cout << core.memory[i].op << " ";
    }
    std::cout << std::endl;
}

void Emulator::clear() {
    // reset task queues, swap might be marginally faster, but this clearer
    core.task_queue_w1 = std::queue<int16_t>();
    core.task_queue_w2 = std::queue<int16_t>();

    // this should be empty
    core.to_post_increment_a.clear();
    core.to_post_increment_b.clear();


    // reset memory array - this seems to be the most idiomatic way to do it
    std::fill(std::begin(core.memory), std::end(core.memory), initial_instr);

    offset_w1 = NOT_LOADED;
    offset_w2 = NOT_LOADED;

    core.counter = 0;
    core.turn_w1 = true;
}
