OBJ_DIR = ../../../obj
SRC = bs_emulator.cpp bs_emulator-run.cpp
OBJS = $(SRC:.cpp=.o) 

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++

CC = g++

$(OBJ_DIR)/bs_emulator.o : $(OBJS)
	ld -r $(OBJS) -o $@ 

$(OBJS) : $(SRC)

clean:
	rm *.o || :