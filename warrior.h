// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_WARRIOR_H_
#define MARZIPAN_WARRIOR_H_

#include <array>

#include "./redcode.h"
#include "./emulator-vars.h"

class Warrior {
 public :
    std::array<Instruction, max_warrior_size> code;
    int start_pos;
    explicit Warrior(int start);
    void clear();
};

#endif  // MARZIPAN_WARRIOR_H_
