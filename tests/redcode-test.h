// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef TESTS_REDCODE_TEST_H_
#define TESTS_REDCODE_TEST_H_

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

  int current;

  ReadableInstruction get_current() {
    ReadableInstruction instr = {};

    instr.op = static_cast<enum Opcode>(current % num_ops);
    current = current / num_ops;

    instr.mod = static_cast<enum Modifier>(current % num_mods);
    current = current / num_mods;

    instr.a_mode = static_cast<enum Mode>(current % num_modes);
    current = current / num_modes;

    instr.b_mode = static_cast<enum Mode>(current % num_modes);
    current = current / num_modes;

    return instr;
  }

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

TEST(Redcode, TestTranslations) {
  auto generator = InstructionGenerator();
  auto instructionSet = std::unordered_set<int>();

  while (generator.has_next()) {
    ReadableInstruction instr = generator.generate_next();

    int operation_int = static_cast<int>(instr2op(instr));

    EXPECT_TRUE(instructionSet.find(operation_int) != instructionSet.end());
    instructionSet.insert(operation_int);
  }
}

#endif  // TESTS_REDCODE_TEST_H_
