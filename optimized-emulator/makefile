# Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

SRC := emulator.cpp emulator-run.cpp
HDR := emulator.h core.h redcode.h

OBJDIR = obj
OBJS = $(SRC:.cpp=.o)
OBJ_PATHS = $(addprefix $(OBJDIR)/, $(OBJS))

OUTPUT_OPTION = -o $(OBJDIR)/$@

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++

emu.o : $(OBJS)
	ld -r -o emu.o $(OBJ_PATHS)

$(OBJS) : $(SRC) $(HDR)

clean:
	rm *.o