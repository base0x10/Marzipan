#include <iostream>
#include <assert.h>

#include "emulator.h"
#include "emulator-vars.h"

emulator::emulator(int offset, std::array<instruction, max_warrior_size> warrior1, std::array<instruction, max_warrior_size> warrior2) {
    
    //check that the start and end of each program in circular memory are sufficiantly far apart
    assert(offset >= min_separation && core_size - offset - (max_warrior_size*2) >= min_separation);

    core.fill(initial_instr);
    // copy the first warrior into the core at 0 offset
    std::copy(std::begin(warrior1), std::end(warrior1), std::begin(core));

    // copy over the second warrior at the specified offset
    std::copy(std::begin(warrior2), std::end(warrior2), std::begin(core) + offset + max_warrior_size);
}


int emulator::run() {

    int cur_task = 0;
    cur_task++;

    return num_threads;
}

void emulator::print(){

    for(auto i = 0u ; i < core.size() ; i++) {
        if (i%10 == 0) std::cout<<"\n";
        std::cout<<core[i].op<<" ";
    }
    std::cout<<std::endl;
}