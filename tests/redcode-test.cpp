// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef TESTS_REDCODE_TEST_H_
#define TESTS_REDCODE_TEST_H_

#include <algorithm>
#include <string>
#include <unordered_set>

#include "gtest/gtest.h"
// redcode is compiled as c
extern "C" {
#include "../redcode.h"
}

// dumbed down input iterator to get each unique instruction exactly once
class InstructionGenerator {
  const int num_ops = 19;
  const int num_mods = 7;
  const int num_modes = 8;
  const int num_instructions = num_ops * num_mods * num_modes * num_modes;

  ReadableInstruction get_current() {
    auto value = current;
    ReadableInstruction instr = {};

    instr.op = static_cast<enum Opcode>(value % num_ops);
    value = value / num_ops;

    instr.mod = static_cast<enum Modifier>(value % num_mods);
    value = value / num_mods;

    instr.a_mode = static_cast<enum Mode>(value % num_modes);
    value = value / num_modes;

    instr.b_mode = static_cast<enum Mode>(value % num_modes);
    value = value / num_modes;

    return instr;
  }
  int current;

 public:
  // default constructor
  InstructionGenerator() { current = -1; }
  // check if user can ask for a new instruction
  bool has_next() { return current < num_instructions - 1; }
  ReadableInstruction generate_next() {
    if (!this->has_next()) {
      return ReadableInstruction();
    }
    current++;
    return this->get_current();
  }
};

// This test is a bit spaghetti because it tests low level c api for redcode
TEST(Redcode, TestAllTranslations) {
  /*
   * This test generates every unique redcode instruction
   * for each instruction, there are three representations
   * we test that every translation between representation is correct
   * and that no representation occurs twice
   */

  auto generator = InstructionGenerator();
  auto operationSet = std::unordered_set<int>();
  auto strSet = std::unordered_set<std::string>();

  int num_iter = 0;

  while (generator.has_next()) {
    /*
     * The following triangle is the equivalence relation that we are testing
     * against
     *
     *    readable instruction
     *         ^       ^
     *        /         \
     *       /           \
     *      v             v
     *    str <-----> instruction
     *
     * For each side, there are two edges that reach it, clockwise and
     * counterclockwise So this test verifies that the clockwise and
     * counterclockwise agree. For our starting point on the triangle (readable
     * instruction), we also test against our initial value
     */

    ReadableInstruction rinstr_orig = generator.generate_next();

    Instruction clockwiseInstruction = {};
    clockwiseInstruction.operation = static_cast<int>(instr2op(rinstr_orig));

    char char_str[25] = {};
    int len = instr2str(char_str, clockwiseInstruction);
    ASSERT_TRUE(len > 0)
        << "instr2str failed with valid instruction, instr is: "
        << clockwiseInstruction.operation << std::endl;

    std::string clockwiseString{char_str};
    ReadableInstruction clockwiseReadableInstruction = {};
    int a_field_clockwise, b_field_clockwise;
    str2rinstr(char_str, &clockwiseReadableInstruction, &a_field_clockwise,
               &b_field_clockwise);

    // reset buffer to 0
    std::fill(char_str, char_str + 15, 0);

    len = rinstr2str(char_str, rinstr_orig, 0, 0);
    ASSERT_TRUE(len > 0)
        << "rinstr2str failed with valid instruction, instr is: "
        << clockwiseInstruction.operation << std::endl;

    std::string counterClockwiseString{char_str};

    Instruction counterClockwiseInstruction = {};
    str2instr(char_str, &counterClockwiseInstruction);

    ReadableInstruction counterClockwiseReadableInstruction =
        op2instr(counterClockwiseInstruction.operation);

    std::string error_description{
        "Instructions, ReadableInstructions and char * representations should "
        "all represent the same data and translations should not alter data"};

    EXPECT_EQ(clockwiseString, counterClockwiseString)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * != "
           "ReadableInstruction -> char * for the readable instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.op,
              counterClockwiseReadableInstruction.op)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction -> char * -> "
           "Instruction -> ReadableInstruction for the readable instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.op, rinstr_orig.op)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction for the readable "
           "instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.mod,
              counterClockwiseReadableInstruction.mod)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction -> char * -> "
           "Instruction -> ReadableInstruction for the readable instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.mod, rinstr_orig.mod)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction for the readable "
           "instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.a_mode,
              counterClockwiseReadableInstruction.a_mode)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction -> char * -> "
           "Instruction -> ReadableInstruction for the readable instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.a_mode, rinstr_orig.a_mode)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction for the readable "
           "instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.b_mode,
              counterClockwiseReadableInstruction.b_mode)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction -> char * -> "
           "Instruction -> ReadableInstruction for the readable instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseReadableInstruction.b_mode, rinstr_orig.b_mode)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction -> char * -> "
           "ReadableInstruction != ReadableInstruction for the readable "
           "instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    EXPECT_EQ(clockwiseInstruction.operation,
              counterClockwiseInstruction.operation)
        << error_description << std::endl
        << "Specifically, ReadableInstruction -> Instruction != "
           "ReadableInstruction -> char * -> Instruction for the readable "
           "instruction: "
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;

    // each readable instruction should map to exactly one operation
    // so expect that if we search for it, we we will not find it
    EXPECT_TRUE(operationSet.find(clockwiseInstruction.operation) ==
                operationSet.end())
        << "We expect that there is a bijection between readable instructions "
           "and operations"
        << std::endl
        << "The bitpacked instruction is " << clockwiseInstruction.operation
        << std::endl
        << "the redcode instruction is " << std::endl
        << "    op = " << rinstr_orig.op << std::endl
        << "    mod = " << rinstr_orig.mod << std::endl
        << "    a_mode = " << rinstr_orig.a_mode << std::endl
        << "    b_mode = " << rinstr_orig.b_mode << std::endl;
    operationSet.insert(clockwiseInstruction.operation);

    num_iter++;
  }

  EXPECT_EQ(num_iter, 19 * 7 * 8 * 8)
      << "The testing loop should have run exactly once for each instruction"
      << std::endl;
}

#endif  // TESTS_REDCODE_TEST_H_
