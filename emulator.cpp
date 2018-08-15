#include "iostream"

#include "emulator.h"
#include "emulator-vars.h"

emulator::emulator(int offset, std::array<instruction, max_warrior_size> warrior1, std::array<instruction, max_warrior_size> warrior2) {
    
    // TODO sanity checks for these inputs

    core.fill(initial_instr);
    // copy the first warrior into the core at 0 offset
    std::copy(std::begin(warrior1), std::end(warrior1), std::begin(core));

    std::copy(std::begin(warrior2), std::end(warrior2), std::begin(core) + offset);
}


int emulator::run() {

    int cur_task = 0;
    cur_task++;

    return num_threads;
}

void emulator::print(){

    for(auto i = 0u ; i < core.size() ; i++) {
        if (i%64 == 0) std::cout<<"\n";
        std::cout<<core[i].op<<" ";
    }
    std::cout<<std::endl;
}