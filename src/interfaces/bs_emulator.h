#ifndef MARZIPAN_BS_EMULATOR_H_
#define MARZIPAN_BS_EMULATOR_H_

#include "./emulator.h"
#include "./config.h"
#include "../emulators/big-switch-emulator/bs_core.h"

#include <string>

// return signals of run(int steps)
#define WON_BY_W1 (1)
#define WON_BY_W2 (2)
#define TIE (0)
#define PAUSED_EXECUTION (-1)
#define NO_EXECUTION (-2)


class BS_Emulator: Emulator {
 public:
    int run(int steps);
    std::string value_at(int addr);
    void place(std::string instr, int addr);
    void reset(int w1_start, int w2_start);
    BS_Emulator(int start_w1, int start_w2) {
        reset(start_w1, start_w2);
    }
 private:
    BS_Core core;
    inline void queue_task(int addr);
    inline void post_increment();

};

#endif  // MARZIPAN_BS_EMULATOR_H_
 