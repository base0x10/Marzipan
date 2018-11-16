#include <iostream>
#include <fstream>
#include "../interfaces/bs_emulator.h"

using namespace std;

int main(int argc, char *argv[]) {

    if (argc != 6) {
        cout << "please specify two warrior files, their initial offsets, and a number of rounds to run" << endl;
        cout << "e.g. ./marzipan 10 warrior1.red 0 warrior2.red 100" << endl;

        return 0;
    }

    BS_Emulator emu(stoi(argv[1]), stoi(argv[3]));

    int i = stoi(argv[1]);
    ifstream warrior1(argv[2]);
    string instruction;
    while (getline(warrior1, instruction)) {
        emu.place(instruction, i++);
    }

    i = stoi(argv[3]);
    ifstream warrior2(argv[4]);
    while (getline(warrior2, instruction)) {
        emu.place(instruction, i++);
    }


    for (int rounds = 1 ; rounds<stoi(argv[5]) ; rounds++) {
        for (int i = 0 ; i < core_size ; i++) {
            cout << emu.value_at(i) << endl;
        }
        cout << "----------------------------------------------" << endl;

        int res = emu.run(1);
    
        if (res ==  PAUSED_EXECUTION) continue;

        if (res == WON_BY_W1) {
            cout << "Warrior 1 won at round " << rounds << endl;
        }

        if (res == WON_BY_W2) {
            cout << "Warrior 2 won at round " << rounds << endl;
        }

        if (res == TIE) {
            cout << "A tie was reached, no warrior was killed by cycles_before_tie" <<endl;
        }

        if (res == NO_EXECUTION) {
            cout << "Both warriors died before either warrior successfully executed an instruction" <<endl;
        }

        return 0;
    }

    cout << "A tie was reached, no warrior was killed by "<< argv[5] << " cycles " << endl;
}
