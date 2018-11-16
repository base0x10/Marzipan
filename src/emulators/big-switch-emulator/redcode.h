// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef REDCODE_H_
#define REDCODE_H_

#include <stdint.h>

// 5 bits, 19 vals
enum Opcode : char {
    DAT,    // terminate process
    MOV,    // move from A to B
    ADD,    // add A to B, store result in B
    SUB,    // subtract A from B, store result in B
    MUL,    // multiply A by B, store result in B
    DIV,    // divide B by A, store result in B if A <> 0, else terminate
    MOD,    // divide B by A, store remainder in B if A <> 0, else terminate
    JMP,    // transfer execution to A
    JMZ,    // transfer execution to A if B is zero
    JMN,    // transfer execution to A if B is non-zero
    DJN,    // decrement B, if B is non-zero, transfer execution to A
    SPL,    // split off process to A
    SLT,    // skip next instruction if A is less than B, b
    CMP,    // same as SEQ, so loader converts CMP to SEQ
    SEQ,    // (*) Skip next instruction if A is equal to B
    SNE,    // (*) Skip next instruction if A is not equal to B
    NOP,    // (*) No operation
    LDP,    // (+) Load P-space cell A into core address B, unimplemented
    STP     // (+) Store A-number into P-space cell B, unimplemented
};

enum Modifier : char {
    A,
    B,
    AB,
    BA,
    F,
    X,
    I
};

// 3 bits, 8 vals
enum Mode : char {
    IMMEDIATE,      // "#" prefix,
    DIRECT,         // "$" prefix,

    /* Description of indirect */

    INDIRECT_A,     // "*" prefix,
    INDIRECT_B,     // "@" prefix,

    /* Description of predecrement */
    PREDEC_A,       // "{" prefix,
    PREDEC_B,       // "<" prefix,

    /* Description of postincrement */
    POSTINC_A,      // "}" prefix
    POSTINC_B,      // ">" prefix
};

// 1064 values
struct Instruction {

    // enum for the 18 base ops
    Opcode op;

    // union of enums for exact modifier
    Modifier mod;

    // fields and their modes
    Mode a_mode;
    int16_t a_num;
    Mode b_mode;
    int16_t b_num;
};
#endif  // REDCODE_H_
