OBJ_DIR = ../../obj
OBJS = $(SRC:.cpp=.o) 
OBJ_PATHS = $(addprefix $(OBJ_DIR)/, $(OBJS))

CXXFLAGS = -Wall -Wextra -Werror -std=c++11 -pedantic-errors
CPPFLAGS = -lstdc++

$(OBJ_DIR)/cli-server.o : cli-server.cpp
	$(CC) -c $(CPPFLAGS) $(CXXFLAGS) -o $(OBJ_DIR)/cli-server.o cli-server.cpp

clean:
	rm *.o || :