// Copyright 2018 Joseph Espy MIT LICENSE jespy@gwu.edu
#ifndef MARZIPAN_REDCODE_H_
#define MARZIPAN_REDCODE_H_

#include <stdint.h>

enum Opcode : char{
    DAT,
    MOV,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    JMP,
    JMZ,
    JMN,
    DJN,
    CMP,
    SLP,
    SPL
};

enum Modifier : char{
    A,
    B,
    AB,
    BA,
    F,
    X,
    I
};

enum Mode : char{
    IMMEDIATE,      // "#" prefix,
    DIRECT,         // "$" prefix,

    /* Description of indirect */

    INDIRECT_A,     // "*" prefix,
    INDIRECT_B,     // "@" prefix,

    /* Description of predecrement */
    PREDEC_A,       // "{" prefix,
    PREDEC_B,       // "<" prefix,

    /* Description of postincrement */
    POSTINC_A,      // ">" prefix
    POSTINC_B,      // "{" prefix
};


struct Instruction {
    Opcode op;
    Modifier mod;
    Mode a_mode;
    int16_t a_val;
    Mode b_mode;
    int16_t b_val;
};
#endif  // MARZIPAN_REDCODE_H_
