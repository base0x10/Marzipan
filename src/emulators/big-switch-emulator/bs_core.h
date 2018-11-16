// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_BS_CORE_H_
#define MARZIPAN_BS_CORE_H_

/*
 * The Core class stores the entire state of the redcode virtual machine
 * different emulators can can work on copies of the same Core
 * but Core doesn't dictate policy for interacting with datastructures
 */

#include <queue>
#include <forward_list>

#include "./redcode.h"
#include "../../interfaces/config.h"

struct BS_Core {
    std::queue<int16_t> task_queue_w1;
    std::queue<int16_t> task_queue_w2;

    Instruction memory[core_size];

    std::forward_list<int> to_post_increment_a;
    std::forward_list<int> to_post_increment_b;


    // number of the steps that have been executed
    int counter;

    bool turn_w1;
};

#endif  // MARZIPAN_BS_CORE_H_
