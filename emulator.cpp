// Copyright 2018 Joseph Espy MIT LICENSE jespy@gwu.edu

#include "./emulator.h"
#include "./emulator-vars.h"

#include <iostream>
#include <cassert>
#include <algorithm>

void Emulator::load_warriors(int offset, Warrior w1, Warrior w2) {
    this->clear();

    // check for an offset that keeps min_separation
    assert(offset >= min_separation
        && core_size - offset - (max_warrior_size*2) >= min_separation);


    // copy the first warrior into the core at 0 offset
    std::copy(std::begin(w1.code), std::end(w1.code), std::begin(core));

    // copy over the second warrior at the specified offset
    std::copy(std::begin(w2.code),
        std::end(w2.code),
        std::begin(core) + offset + max_warrior_size);
}


int Emulator::run() {
    int cur_task = 0;
    cur_task++;

    return num_threads;
}

void Emulator::print() {
    for (auto i = 0u ; i < core.size() ; i++) {
        if (i%10 == 0) std::cout << "\n";
        std::cout << core[i].op << " ";
    }
    std::cout << std::endl;
}

void Emulator::clear() {
    core.fill(initial_instr);
    task_queue.fill(0);
}
