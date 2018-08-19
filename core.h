// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_CORE_H_
#define MARZIPAN_CORE_H_

/*
 * The Core class stores the entire state of the redcode virtual machine
 * different emulators can can work on copies of the same Core
 * but Core doesn't dictate policy for interacting with datastructures
 */

#include "./redcode.h"
#include "./emulator-vars.h"

#include <forward_list>

#define likely(x)      __builtin_expect(!!(x), 1)
#define unlikely(x)    __builtin_expect(!!(x), 0)
#define EMPTY_TASK_QUEUE -1

struct Core {
 public :
    std::forward_list<int> task_queue_w1;
    std::forward_list<int> task_queue_w2;

    Instruction memory[core_size];

};

#endif  // MARZIPAN_CORE_H_
