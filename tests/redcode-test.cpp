// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef TESTS_REDCODE_TEST_H_
#define TESTS_REDCODE_TEST_H_

#include <unordered_set>
#include <string>

#include "gtest/gtest.h"
// redcode is compiled as c
extern "C"
{
#include "../redcode.h"
}

// dumbed down input iterator to get each unique instruction exactly once
class InstructionGenerator
{
  const int num_ops = 19;
  const int num_mods = 7;
  const int num_modes = 8;
  const int num_instructions = num_ops * num_mods * num_modes * num_modes;

  ReadableInstruction get_current()
  {
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
  ReadableInstruction generate_next()
  {
    if (!this->has_next())
    {
      return ReadableInstruction();
    }
    current++;
    return this->get_current();
  }
};

TEST(Redcode, TestTranslations)
{
  auto generator = InstructionGenerator();
  auto operationSet = std::unordered_set<int>();
  auto strSet = std::unordered_set<std::string>();

  int num_iter = 0;

  while (generator.has_next())
  {
    ReadableInstruction rinstr = generator.generate_next();
    int operation_int = static_cast<int>(instr2op(rinstr));

    char char_str[25] = {};
    Instruction instr = {};
    instr.operation = operation_int;
    int len = instr2str(char_str, instr);
    len++;

    // each readable instruction should map to exactly one operation
    // so expect that if we search for it, we we will not find it
    EXPECT_TRUE(operationSet.find(operation_int) == operationSet.end())
        << "We expect that there is a bijection between readable instructions and operations" << std::endl
        << "The bitpacked instruction is " << operation_int << std::endl
        << "the redcode instruction is " << std::endl
        << "    op = " << rinstr.op << std::endl
        << "    mod = " << rinstr.mod << std::endl
        << "    a_mode = " << rinstr.a_mode << std::endl
        << "    b_mode = " << rinstr.b_mode << std::endl;
    operationSet.insert(operation_int);

    num_iter++;
  }

  EXPECT_EQ(num_iter, 19 * 7 * 8 * 8) << "The testing loop should have run exactly once for each instruction" << std::endl;
}

#endif // TESTS_REDCODE_TEST_H_
