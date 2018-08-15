#include "redcode.h"
#pragma once

/* ~~~ THINGS YOU CAN CONFIGURE ~~~ */

// change to ICWS86 to compile with ICWS86 compatability
#define DEBUG_SPEC

// change the size of the threading pool
constexpr int num_threads = 1;

// optimize for low-fitness warriors TODO: write documentation on this
constexpr bool predictive_execution = true;

/* ~~~ THINGS YOU SHOULD NOT CHANGE ~~~ */

#ifdef KOTH

constexpr int core_size             = 8000;
constexpr int cycles_before_tie     = 80000;
constexpr int max_warrior_size      = 100;
constexpr int max_num_tasks         = 8000;
constexpr int min_separation        = 100;
constexpr bool const_separation     = false;    // use min_sep as separation rather than random

constexpr struct instruction initial_instr {
        DAT,
        F,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

#endif

#ifdef ICWS86

constexpr int core_size             = 8192;
constexpr int cycles_before_tie     = 100000;
constexpr int max_warrior_size      = 300;
constexpr int max_num_tasks         = 64; 
constexpr int min_separation        = 300;
constexpr bool const_separation     = false;    // use min_sep as separation rather than random

constexpr struct instruction initial_instr {
        DAT,
        F,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };
#endif

// simpilier spec for debugging emulator behavior
#ifdef DEBUG_SPEC

constexpr int core_size             = 40;
constexpr int cycles_before_tie     = 1000;
constexpr int max_warrior_size      = 10;
constexpr int max_num_tasks         = 32;
constexpr int min_separation        = 10;
constexpr bool const_separation     = true;    // use min_sep as separation rather than random

constexpr struct instruction initial_instr {
        DAT,
        F,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

#endif

/* The following cannot be modified but conform to both KOTH and ICWS86:
    read_distance                   = core_size;
    write_distance                  = core_size;
    warriors                        = 2;
*/
