CPPSRC =
CSRC = redcode.c
OBJ = $(CSRC:.c=.o) $(CPPSRC:.cpp=.o)
TESTS = $(wildcard tests/*.cpp)

COPTS = -Wall -Wextra -Werror -pedantic-errors -O3

CXXFLAGS = -std=c++11  $(COPTS)
CFLAGS = -std=c99 $(COPTS) -Wmissing-prototypes -Wstrict-prototypes -Wold-style-definition
LDFLAGS = -lstdc++

GTEST_DIR = googletest/googletest
TESTFLAGS = $(CPPFLAGS) $(CXXFLAGS) -isystem $(GTEST_DIR)/include -pthread

marzipan: $(OBJ) cli.cpp
	$(CC) -o $@ $^ $(LDFLAGS)

libgtest.a:
	g++ -std=c++11 -isystem $(GTEST_DIR)/include -I$(GTEST_DIR) -pthread -c $(GTEST_DIR)/src/gtest-all.cc
	ar -rv libgtest.a gtest-all.o

unit-test: tests/unit-test.cpp $(OBJ) libgtest.a $(TESTS)
	$(CXX) $(TESTFLAGS) -o $@ $^

.PHONY: clean
clean:
	rm -f $(OBJ) marzipan unit-test gtest-all.o libgtest.a unit-test.o