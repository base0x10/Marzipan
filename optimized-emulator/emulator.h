// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

#ifndef MARZIPAN_EMULATOR_H_
#define MARZIPAN_EMULATOR_H_

#include <array>

#include "./redcode.h"
#include "../emulator-vars.h"
#include "../warrior.h"
#include "./core.h"

// initial value of offset_w1 and offset_w2
#define NOT_LOADED (-1)

// return signals of Core::run(int steps)
#define WON_BY_W1 (1)
#define WON_BY_W2 (2)
#define TIE (0)
#define PAUSED_EXECUTION (-1)
#define NO_EXECUTION (-2)

#define likely(x) __builtin_expect(!!(x), 1)
#define unlikely(x) __builtin_expect(!!(x), 0)

class Emulator {
 public :
    void load_warrior(Warrior w, int offset, int war_num);
    int run(int steps);
    void print();
    void clear();
    Emulator() {
        clear();
    }
 private :
    Core core;
    inline void post_increment();
    inline void queue_task(int addr);

    int offset_w1 = NOT_LOADED;
    int offset_w2 = NOT_LOADED;
};

#endif  // MARZIPAN_EMULATOR_H_
