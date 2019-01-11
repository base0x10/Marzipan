// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include "./redcode.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/*
 * TODO: what does this file do
 */

#define OP_MASK (0x1F)
#define MOD_MASK (0x7 << 5)
#define MODE_A_MASK (0x7 << 8)
#define MODE_B_MASK (0x7 << 11)

// converts from the simplier ReadableInstruction RedCode struct format to the
// internal numerical representation of an operation
packed_operation_t instr2op(struct ReadableInstruction instr) {
  // rightmost 5 bits are the opcode
  packed_operation_t operation = instr.op & OP_MASK;

  // next rightmost 3 bits are the modifier
  operation |= (instr.mod << 5 & MOD_MASK);

  // next rightmost three bits are the first mode
  operation |= (instr.a_mode << 8 & MODE_A_MASK);

  // next three rightmost bits are the second mode
  operation |= (instr.b_mode << 11 & MODE_B_MASK);

  return operation;
}

// converts back from a compact representation to a readableInstruction struct
// representation
struct ReadableInstruction op2instr(packed_operation_t operation) {
  struct ReadableInstruction instr = (struct ReadableInstruction){
      (enum Opcode)(operation & OP_MASK),
      (enum Modifier)((operation & MOD_MASK) >> 5),
      (enum Mode)((operation & MODE_A_MASK) >> 8),
      (enum Mode)((operation & MODE_B_MASK) >> 11)};
  return instr;
}

/*
 * lookup tables for translating between ReadableInstruction and strings
 */

const char *op2str[19] = {"DAT", "MOV", "ADD", "SUB", "MUL", "DIV", "MOD",
                          "JMP", "JMZ", "JMN", "DJN", "SPL", "SLT", "CMP",
                          "SEQ", "SNE", "NOP", "LDP", "STP"};

const char *mode2str[8] = {"#", "$", "*", "@", "{", "<", "}", ">"};

const char *mod2str[7] = {"A", "B", "AB", "BA", "F", "X", "I"};

// writes the instruction specified into the string and returns
// the number of characters written.  With valid inputs, writes
// between 13 and 20 bytes (assuming that fields are 1-4 digits base 10)
int rinstr2str(char *dest, struct ReadableInstruction instruction,
               core_offset_t a_field, core_offset_t b_field) {
  if (dest == NULL) return 0;

  // format along the lines of "MOD.AB #8000, >7999\n"

  sprintf(dest, "%s.%s %s%d, %s%d\n", op2str[instruction.op],
          mod2str[instruction.mod], mode2str[instruction.a_mode], a_field,
          mode2str[instruction.b_mode], b_field);

  return strlen(dest);
}

// convert from internal bitpacked instruction format to string
int instr2str(char *dest, struct Instruction instr) {
  struct ReadableInstruction decoded = op2instr(instr.operation);
  return rinstr2str(dest, decoded, instr.a_field, instr.b_field);
}

// decode a string into struct ReadableInstruction
// returns number of bytes read from string for one instruction
int str2rinstr(char *str, struct ReadableInstruction *to_fill, int *a_field,
               int *b_field) {
  char *initial_str = str;

  // used when parsing integers.
  char *endptr = NULL;

  int set_flag = 0;
  // search for matching operation
  for (int i = 0; i < 19; i++) {
    if (strncmp(str, op2str[i], 3) == 0) {
      to_fill->op = (enum Opcode)i;
      set_flag = 1;
      break;
    }
  }
  if (set_flag == 0) goto BADPARSE;

  // move cursor 3 over for op and 1 for "."
  str += 4;
  if (str[-1] != '.') goto BADPARSE;

  // the modifier is variable length and BA needs to be matched before B etc
  if (strncmp(str, "AB", 2) == 0) {
    to_fill->mod = AB;
    str += 1;
  } else if (strncmp(str, "BA", 2) == 0) {
    to_fill->mod = BA;
    str += 1;
  } else if (strncmp(str, "A", 1) == 0)
    to_fill->mod = A;
  else if (strncmp(str, "B", 1) == 0)
    to_fill->mod = B;
  else if (strncmp(str, "F", 1) == 0)
    to_fill->mod = F;
  else if (strncmp(str, "X", 1) == 0)
    to_fill->mod = X;
  else if (strncmp(str, "I", 1) == 0)
    to_fill->mod = I;
  else
    goto BADPARSE;

  // skip over modifier and a space
  if (str[1] != ' ') goto BADPARSE;
  str += 2;

  // search for a_mode
  set_flag = 0;
  for (int i = 0; i < 8; i++) {
    if (mode2str[i][0] == str[0]) {
      to_fill->a_mode = (enum Mode)i;
      set_flag = 1;
      break;
    }
  }
  if (set_flag == 0) goto BADPARSE;
  str += 1;

  // convert a_field from a string to integer type
  endptr = NULL;
  *a_field = strtol(str, &endptr, 10);
  if (str == endptr) goto BADPARSE;
  str = endptr;

  // skip over ", "
  if (strncmp(str, ", ", 2) != 0) goto BADPARSE;
  str += 2;

  // search for b_mode
  set_flag = 0;
  for (int i = 0; i < 8; i++) {
    if (mode2str[i][0] == str[0]) {
      to_fill->b_mode = (enum Mode)i;
      set_flag = 1;
      break;
    }
  }
  if (set_flag == 0) goto BADPARSE;
  str += 1;

  // convert a_field from a string to integer type
  endptr = NULL;
  *b_field = strtol(str, &endptr, 10);
  if (str == endptr) goto BADPARSE;
  str = endptr;

  // skip over "\n"
  if (str[0] != '\n') goto BADPARSE;
  str += 1;

  return str - initial_str;

BADPARSE:
  // encountered a bad string, so zero everything and return 0
  *to_fill = (struct ReadableInstruction){(enum Opcode)0, (enum Modifier)0,
                                          (enum Mode)0, (enum Mode)0};
  *a_field = 0;
  *b_field = 0;
  return 0;
}

// convert from a string to the bitpacked internal instruction format
// returns number of bytes read on success and 0 on failure
int str2instr(char *str, struct Instruction *to_fill) {
  struct ReadableInstruction rinstr;
  int a_field, b_field;

  int bytes_read = str2rinstr(str, &rinstr, &a_field, &b_field);

  if (bytes_read == 0) return 0;

  to_fill->a_field = a_field;
  to_fill->b_field = b_field;
  to_fill->operation = instr2op(rinstr);
  return bytes_read;
}
