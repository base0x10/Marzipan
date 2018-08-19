// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include "./warrior.h"
#include "./redcode.h"
#include "./emulator-vars.h"

Warrior::Warrior(int start) {
    code.fill(initial_instr);
    start_pos = start;
}

void Warrior::clear() {
    code.fill(initial_instr);
}
