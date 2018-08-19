// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_EMULATOR_H_
#define MARZIPAN_EMULATOR_H_

#include <array>

#include "./redcode.h"
#include "./emulator-vars.h"
#include "./warrior.h"
#include "./core.h"

class Emulator {
 public :
    void load_warrior(Warrior w, int offset, int war_num);
    int run(int steps);
    void print();
    void clear();
 private :
    Core core;
};

#endif  // MARZIPAN_EMULATOR_H_
