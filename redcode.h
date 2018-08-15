#pragma once

enum opcode : char{
    DAT, // Is there anyway a modifier will have effect here?
    MOV, // modifiers?
    ADD, // can take AB or BA?
    SUB, // can take AB or BA?
    MUL, // can take AB or BA?
    DIV, // can take AB or BA?
    MOD, // can take AB or BA?
    JMP, // modifiers?
    JMZ, // A or B?
    JMN, // A or B?
    DJN, // DJN does what?
    CMP, // huh?
    SLP, // probably takes A or B
    SPL  // not sure
};

enum modifier : char{
    A,
    B,
    AB,
    BA,
    F,
    X,
    I
};

enum mode : char{
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


struct instruction {
    opcode op;
    modifier mod;
    mode a_mode;
    unsigned short a_val;
    mode b_mode;
    unsigned short b_val;
};
