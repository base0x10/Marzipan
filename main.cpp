#include <iostream>
#include <array>

#include "emulator.h"
#include "redcode.h"

int main () {

    std::cout<<"begin exeuction of marzipan"<<std::endl;

    std::array<instruction, max_warrior_size> warrior1;
    warrior1.fill(initial_instr);
    std::array<instruction, max_warrior_size> warrior2;
    warrior2.fill(initial_instr);

    struct instruction move {
        MOV,
        AB,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

    struct instruction add {
        ADD,
        AB,
        IMMEDIATE,
        0,
        IMMEDIATE,
        0
    };

    for(int i = 0 ; i < 10 ; i++) {
        warrior1[i] = add;
    }

    for(int i = 0 ; i < 10 ; i++) {
        warrior2[i] = move;
    }

    emulator e = emulator(64, warrior1, warrior2);

    e.print();
}