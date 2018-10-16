# Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

SRC := main.cpp visualizer.cpp warrior.cpp

OBJDIR = obj
OBJS = $(SRC:.cpp=.o) 
OBJ_PATHS = $(addprefix $(OBJDIR)/, $(OBJS)) optimized-emulator/emu.o

OUTPUT_OPTION = -o $(OBJDIR)/$@

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++


marzipan : $(SRC:.cpp=.o) 
	$(CC) $(CPPFLAGS) -o marzipan $(OBJ_PATHS)

$(OBJS) : $(SRC) optimized-emulator/emu.o

optimized-emulator/emu.o:
	$(MAKE) -C optimized-emulator

clean :
	$(MAKE) -C optimized-emulator clean 
	rm $(OBJ_PATHS) marzipanthe 