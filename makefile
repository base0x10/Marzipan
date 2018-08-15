SRC := emulator.cpp main.cpp visualizer.cpp

OBJDIR = obj
OBJS = $(SRC:.cpp=.o)
OBJ_PATHS = $(addprefix $(OBJDIR)/, $(OBJS))

OUTPUT_OPTION = -o $(OBJDIR)/$@

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++

marzipan : $(SRC:.cpp=.o)
	$(CC) $(CPPFLAGS) -o marzipan $(OBJ_PATHS)

$(OBJS) : $(SRC)

clean :
	rm $(OBJ_PATHS) marzipan