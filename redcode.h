// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef REDCODE_H_
#define REDCODE_H_

#include "./types.h"

/*
 * This file defines a redcode instruction as it is represented in the simulated
 * core memory as well as a constructor for the bitpacked operation field in
 * terms of a ReadableInstruction struct
 */

struct Instruction {
  packed_operation_t operation;
  core_offset_t a_field;
  core_offset_t b_field;
};

/*
 *   These enums and struct ReadableInstruction are used to construct struct
 * Instruction which is used everywhere, but is optimized to make emulation fast
 */

// 5 importent bits, 19 vals
enum Opcode {
  DAT = 0x0,   // terminate process
  MOV = 0x1,   // move from A to B
  ADD = 0x2,   // add A to B, store result in B
  SUB = 0x3,   // subtract A from B, store result in B
  MUL = 0x4,   // multiply A by B, store result in B
  DIV = 0x5,   // divide B by A, store result in B if A <> 0, else terminate
  MOD = 0x6,   // divide B by A, store remainder in B if A <> 0, else
               // terminate
  JMP = 0x7,   // transfer execution to A
  JMZ = 0x8,   // transfer execution to A if B is zero
  JMN = 0x9,   // transfer execution to A if B is non-zero
  DJN = 0xa,   // decrement B, if B is non-zero, transfer execution to A
  SPL = 0xb,   // split off process to A
  SLT = 0xc,   // skip next instruction if A is less than B, b
  CMP = 0xd,   // same as SEQ, so loader converts CMP to SEQ
  SEQ = 0xe,   // Sip next instruction if A is equal to B
  SNE = 0xf,   // Skip next instruction if A is not equal to B
  NOP = 0x10,  // No operation
  LDP = 0x11,  // Load P-space cell A into core address B
  STP = 0x12   // Store A-number into P-space cell B
};

// 3 importent bits, 7 values
enum Modifier {
  A = 0x0,
  B = 0x1,
  AB = 0x2,
  BA = 0x3,
  F = 0x4,
  X = 0x5,
  I = 0x6
};

// 3 importent bits, 8 vals
enum Mode {
  IMMEDIATE = 0x0,   // "#" prefix,
  DIRECT = 0x1,      // "$" prefix,
  INDIRECT_A = 0x2,  // "*" prefix,
  INDIRECT_B = 0x3,  // "@" prefix,
  PREDEC_A = 0x4,    // "{" prefix,
  PREDEC_B = 0x5,    // "<" prefix,
  POSTINC_A = 0x6,   // "}" prefix
  POSTINC_B = 0x7    // ">" prefix
};

struct ReadableInstruction {
  enum Opcode op;
  enum Modifier mod;
  enum Mode a_mode;
  enum Mode b_mode;
};

// converts from the simplier ReadableInstruction RedCode struct format to the
// internal numerical representation of an operation
packed_operation_t instr2op(struct ReadableInstruction instr);

// converts back from a compact representation to a readableInstruction struct
// representation
struct ReadableInstruction op2instr(packed_operation_t operation);

// writes the instruction specified into the string and returns
// the number of characters written.  With valid inputs, writes
// between 13 and 20 bytes (assuming that fields are 1-4 digits base 10)
int rinstr2str(char *dest, struct ReadableInstruction instruction,
               core_offset_t a_field, core_offset_t b_field);

// convert from internal bitpacked instruction format to string
int instr2str(char *dest, struct Instruction instr);

// decode a string into struct ReadableInstruction
// returns number of bytes read from string for one instruction
int str2rinstr(char *str, struct ReadableInstruction *to_fill, int *a_field,
               int *b_field);

// convert from a string to the bitpacked internal instruction format
// returns number of bytes read on success and 0 on failure
int str2instr(char *str, struct Instruction *to_fill);

#endif  // REDCODE_H_
