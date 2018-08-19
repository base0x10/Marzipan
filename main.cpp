// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include <iostream>
#include <array>
#include <type_traits>

#include "./emulator.h"
#include "./redcode.h"
#include "./warrior.h"
#include "./core.h"

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

    e.clear();
    
    e.load_warrior(warrior1, 0, 1);

    e.load_warrior(warrior2, 20, 2);

    e.print();
}
