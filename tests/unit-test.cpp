// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#include <iostream>
#include "gtest/gtest.h"

int main(int argc, char **argv) {
  // allow googletest to parse cli flags and setup tests
  ::testing::InitGoogleTest(&argc, argv);

  return RUN_ALL_TESTS();
}
