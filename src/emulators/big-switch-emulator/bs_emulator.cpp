#include "../../interfaces/bs_emulator.h"
#include "./bs_core.h"
#include "./redcode.h"
#include "../../interfaces/config.h"
#include <iostream>
#include <string>
#include <sstream>
#include <queue>

// every instruction must be in the canonical form "DAT.F #123, }345" without inferred modifiers or modes
struct Instruction str2instr(std::string instr) {
    struct Instruction instruction {DAT, F, IMMEDIATE, 0, IMMEDIATE, 0};

    int start = 0 , end = 0;
    
    
    while(instr[end] != '.') end++;
    std::string op = instr.substr(start, end - start);

    if (op.compare("DAT") == 0) instruction.op = DAT;
    else if (op.compare("MOV") == 0) instruction.op = MOV;
    else if (op.compare("ADD") == 0) instruction.op = ADD;
    else if (op.compare("SUB") == 0) instruction.op = SUB;
    else if (op.compare("MUL") == 0) instruction.op = MUL;
    else if (op.compare("DIV") == 0) instruction.op = DIV;
    else if (op.compare("MOD") == 0) instruction.op = MOD;
    else if (op.compare("JMP") == 0) instruction.op = JMP;
    else if (op.compare("JMZ") == 0) instruction.op = JMZ;
    else if (op.compare("JMN") == 0) instruction.op = JMN;
    else if (op.compare("DJN") == 0) instruction.op = DJN;
    else if (op.compare("SPL") == 0) instruction.op = SPL;
    else if (op.compare("SLT") == 0) instruction.op = SLT;
    else if (op.compare("CMP") == 0) instruction.op = CMP;
    else if (op.compare("SEQ") == 0) instruction.op = SEQ;
    else if (op.compare("SNE") == 0) instruction.op = SNE;
    else if (op.compare("NOP") == 0) instruction.op = NOP;
    else if (op.compare("LDP") == 0) instruction.op = LDP;
    else if (op.compare("STP") == 0) instruction.op = STP;
    else return instruction;

    // skip over '.'
    start = end + 1;

    if (instr.substr(start, 2).compare("AB") == 0) {
        start += 2;
        instruction.mod = AB;
    } else if (instr.substr(start, 2).compare("BA") == 0) {
        start += 2;
        instruction.mod = BA;
    } else if (instr.substr(start, 1).compare("A") == 0) {
        start += 1;
        instruction.mod = A;
    } else if (instr.substr(start, 1).compare("B") == 0) {
        start += 1;
        instruction.mod = B;
    } else if (instr.substr(start, 1).compare("F") == 0) {
        start += 1;
        instruction.mod = F;
    } else if (instr.substr(start, 1).compare("X") == 0) {
        start += 1;
        instruction.mod = X;
    } else if (instr.substr(start, 1).compare("I") == 0) {
        start += 1;
        instruction.mod = I;
    } else return instruction;

    // skip over space
    start += 1;

    if (instr[start] == '#') instruction.a_mode = IMMEDIATE;
    else if (instr[start] == '$') instruction.a_mode = DIRECT;
    else if (instr[start] == '*') instruction.a_mode = INDIRECT_A;
    else if (instr[start] == '@') instruction.a_mode = INDIRECT_B;
    else if (instr[start] == '{') instruction.a_mode = PREDEC_A;
    else if (instr[start] == '<') instruction.a_mode = PREDEC_B;
    else if (instr[start] == '}') instruction.a_mode = POSTINC_A;
    else if (instr[start] == '>') instruction.a_mode = POSTINC_B;
    else return instruction;
    start++;

    int val = std::stoi(instr.substr(start), nullptr);
    
    // sanitize address
    while (val < 0) val += core_size;
    val %= core_size;
    instruction.a_num = val;

    // skip over integer
    while (instr[start] >= '0' && instr[start] <= '9' ) start++;

    // skip over space and comma
    start += 2;

    if (instr[start] == '#') instruction.b_mode = IMMEDIATE;
    else if (instr[start] == '$') instruction.b_mode = DIRECT;
    else if (instr[start] == '*') instruction.b_mode = INDIRECT_A;
    else if (instr[start] == '@') instruction.b_mode = INDIRECT_B;
    else if (instr[start] == '{') instruction.b_mode = PREDEC_A;
    else if (instr[start] == '<') instruction.b_mode = PREDEC_B;
    else if (instr[start] == '}') instruction.b_mode = POSTINC_A;
    else if (instr[start] == '>') instruction.b_mode = POSTINC_B;
    else return instruction;
    start++;

    val = std::stoi(instr.substr(start), nullptr);
    
    // sanitize address
    while (val < 0) val += core_size;
    val %= core_size;
    instruction.b_num = val;
    return instruction;
}

// returns a string of the instruction in canonical form
std::string instr2str(struct Instruction instr) {
    std::string str{};

    switch(instr.op)
    {
        case DAT:
            str+="DAT";
            break;
        case MOV:
            str+="MOV";
            break;
        case ADD:
            str+="ADD";
            break;
        case SUB:
            str+="SUB";
            break;
        case MUL:
            str+="MUL";
            break;
        case DIV:
            str+="DIV";
            break;
        case MOD:
            str+="MOD";
            break;
        case JMP:
            str+="JMP";
            break;
        case JMZ:
            str+="JMZ";
            break;
        case JMN:
            str+="JMN";
            break;
        case DJN:
            str+="DJN";
            break;
        case SPL:
            str+="SPL";
            break;
        case SLT:
            str+="SLT";
            break;
        case CMP:
            str+="CMP";
            break;
        case SEQ:
            str+="SEQ";
            break;
        case SNE:
            str+="SNE";
            break;
        case NOP:
            str+="NOP";
            break;
        case LDP:
            str+="LDP";
            break;
        case STP:
            str+="STP";
            break;
        default:
            break;
    }

    switch(instr.mod)
    {
        case A:
            str+=".A ";
            break;
        case B:
            str+=".B ";
            break;
        case AB:
            str+=".AB ";
            break;
        case BA:
            str+=".BA ";
            break;
        case F:
            str+=".F ";
            break;
        case X:
            str+=".X ";
            break;
        case I:
            str+=".I ";
        default:
            break;
    }

    switch(instr.a_mode)
    {
        case IMMEDIATE:
            str+="#";
            break;
        case DIRECT:
            str+="$";
            break;
        case INDIRECT_A:
            str+="*";
            break;
        case INDIRECT_B:
            str+="@";
            break;
        case PREDEC_A:
            str+="{";
            break;
        case PREDEC_B:
            str+="<";
            break;
        case POSTINC_A:
            str+="}";
            break;
        case POSTINC_B:
            str+=">";
            break;
        default:
            break;
    }

    str.append(std::to_string(instr.a_num)+", ");
    
    switch(instr.b_mode)
    {
        case IMMEDIATE:
            str+="#";
            break;
        case DIRECT:
            str+="$";
            break;
        case INDIRECT_A:
            str+="*";
            break;
        case INDIRECT_B:
            str+="@";
            break;
        case PREDEC_A:
            str+="{";
            break;
        case PREDEC_B:
            str+="<";
            break;
        case POSTINC_A:
            str+="}";
            break;
        case POSTINC_B:
            str+=">";
            break;
        default:
            break;
    }

    str.append(std::to_string(instr.b_num));

    return str;
}

std::string BS_Emulator::value_at(int addr) {
    while (addr < 0) addr += core_size;
    addr %= core_size;
    return instr2str(core.memory[addr]);
}

void BS_Emulator::place(std::string instr, int addr){
    while (addr < 0) addr += core_size;
    addr %= core_size;

    core.memory[addr] = str2instr(instr);

    return;
}

void BS_Emulator::reset(int w1_start, int w2_start) {

    while (w1_start < 0) w1_start += core_size;
    w1_start %= core_size;
    
    while (w2_start < 0) w2_start += core_size;
    w2_start %= core_size;

    core.task_queue_w1 = {};
    core.task_queue_w2 = {};

    core.task_queue_w1.push(w1_start);
    core.task_queue_w2.push(w2_start);

    // this should be empty
    core.to_post_increment_a.clear();
    core.to_post_increment_b.clear();


    // reset memory array
    std::fill(std::begin(core.memory), std::end(core.memory), str2instr(std::string("DAT.F #0, #0")));

    core.counter = 0;
    core.turn_w1 = false;
}