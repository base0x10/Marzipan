#pragma once

#include <array>

#include "redcode.h"
#include "emulator-vars.h"

class emulator {
    public :
        emulator(int offset, std::array<instruction, max_warrior_size> warrior1, std::array<instruction, max_warrior_size> warrior2);
        int run();
        int run_until(int steps);
        void print();
    private :
        std::array<instruction, core_size> core;
        std::array<instruction, max_num_tasks> task_queue;
};