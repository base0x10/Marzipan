#!/bin/bash
make clean && find . -type f | egrep '\.(h|c(pp)?)$' |  xargs clang-format -style='google' -i
find . -type f | egrep '\.(h|cpp)$' |  xargs python cpplint.py


