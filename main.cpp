// Copyright 2018 Joseph Espy MIT LICENSE jespy@gwu.edu

#include <iostream>
#include <array>

#include "./emulator.h"
#include "./redcode.h"
#include "./warrior.h"

int main() {
    std::cout << "begin exeuction of marzipan" << std::endl;

    Warrior warrior1 = Warrior(0);
    Warrior warrior2 = Warrior(0);

    struct Instruction move {
        MOV,
        AB,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

    struct Instruction add {
        ADD,
        AB,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

    for (int i = 0 ; i < 10 ; i++) {
        warrior1.code[i] = add;
    }

    for (int i = 0 ; i < 10 ; i++) {
        warrior2.code[i] = move;
    }

    Emulator e;

    e.load_warriors(10, warrior1, warrior2);

    e.print();
}
