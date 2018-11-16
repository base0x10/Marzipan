#include <string>

// abstract class interface for all emulators
// provides abstraction api to inspect and set core values one at a time
// and to run the emulator for some number of steps (0 goes until end)

class Emulator {
    virtual int run(int steps) = 0;
    virtual std::string value_at(int addr) = 0;
    virtual void place(std::string instr, int addr) = 0;
};