// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_EMULATOR_VARS_H_
#define MARZIPAN_EMULATOR_VARS_H_

#include <string>

#define DEBUG_SPEC

#ifdef KOTH

constexpr int core_size             = 8000;
constexpr int cycles_before_tie     = 80000;
constexpr int max_warrior_size      = 100;
constexpr int max_num_tasks         = 8000;
constexpr int min_separation        = 100;

// use min_sep as separation rather than random
constexpr bool const_separation     = false;

constexpr std::string initial_instr("DAT.F #0, #0");

#endif

#ifdef ICWS86

constexpr int core_size             = 8192;
constexpr int cycles_before_tie     = 100000;
constexpr int max_warrior_size      = 300;
constexpr int max_num_tasks         = 64;
constexpr int min_separation        = 300;

// use min_sep as separation rather than random
constexpr bool const_separation     = false;

constexpr std::string initial_instr("DAT.F #0, #0");

#endif

// simpilier spec for debugging emulator behavior
#ifdef DEBUG_SPEC

constexpr int core_size             = 40;
constexpr int cycles_before_tie     = 1000;
constexpr int max_warrior_size      = 10;
constexpr int max_num_tasks         = 32;
constexpr int min_separation        = 10;

// use min_sep as separation rather than random
constexpr bool const_separation     = true;

#endif

/* The following are not configurable but conform to both KOTH and ICWS86:
    read_distance                   = core_size;
    write_distance                  = core_size;
    warriors                        = 2;
*/

#endif  // MARZIPAN_EMULATOR_VARS_H_
