cppsrc = cli.cpp
csrc = redcode.c
obj = $(csrc:.c=.o) $(cppsrc:.cpp=.o)

COPTS = -Wall -Wextra -Werror -pedantic-errors -O3

CXXFLAGS = -std=c++11  $(COPTS)
CFLAGS = -std=c99 $(COPTS) -Wmissing-prototypes -Wstrict-prototypes -Wold-style-definition
LDFLAGS = -lstdc++

marzipan: $(obj)
	$(CC) -o $@ $^ $(LDFLAGS)

.PHONY: clean
clean:
	rm -f $(obj) marzipan