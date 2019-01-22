#!/bin/bash

make clean 

# find reports all of the paths other than git and googletest
# the grep regex finds .h, .cpp and .c files
# and xargs puts it all into clang format, using google style guide
find . -type f -not -path "./googletest/*" -not -path "./.git/*" \
	| egrep '\.(h|c(pp)?)$' \
	|  xargs clang-format -style='google' -i

# Same process to run cpplint, over only cpp files
 find . -type f -not -path "./googletest/*" -not -path "./.git/*" \
	| egrep '\.(cpp|h|cc)$' \
	|  xargs python cpplint.py
