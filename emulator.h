// Copyright 2018 Joseph Espy MIT LICENSE jespy@gwu.edu
#ifndef MARZIPAN_EMULATOR_H_
#define MARZIPAN_EMULATOR_H_

#include <array>

#include "./redcode.h"
#include "./emulator-vars.h"
#include "./warrior.h"

class Emulator {
 public :
    void load_warriors(int offset, Warrior w1, Warrior w2);
    int run();
    int run_until(int steps);
    void print();
    void clear();
 private :
    std::array<Instruction, core_size> core;
    std::array<int, max_num_tasks> task_queue;
};

#endif  // MARZIPAN_EMULATOR_H_
