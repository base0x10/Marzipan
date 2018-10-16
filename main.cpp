// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include <iostream>
#include <array>
#include <type_traits>

#include "optimized-emulator/emulator.h"
#include "optimized-emulator/redcode.h"
#include "./warrior.h"
#include "optimized-emulator/core.h"

int main() {
    std::cout << "begin exeuction of marzipan" << std::endl;

    Warrior warrior1 = Warrior(0);
    Warrior warrior2 = Warrior(0);

    struct Instruction move {
        MOV,
        I,
        DIRECT,
        0,
        DIRECT,
        1
    };

    /*struct Instruction add {
        ADD,
        AB,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };*/

    for (int i = 0 ; i < 10 ; i++) {
        warrior1.code[i] = move;
    }

    for (int i = 0 ; i < 10 ; i++) {
        warrior2.code[i] = move;
    }

    Emulator e {};

    e.load_warrior(warrior1, 0, 1);

    e.load_warrior(warrior2, 20, 2);

    e.print();

    int output = e.run(0);

    e.print();

    std::cout << output << std::endl;
}
