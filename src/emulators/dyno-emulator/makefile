OBJ_DIR = ../../../obj
OBJS = emulator.cpp

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++

$(OBJ_DIR)/dyno-emulator.o :
	$(CC) -c $(CPPFLAGS) $(CXXFLAGS) -o $@ $(OBJS)

clean:
	rm *.o || :