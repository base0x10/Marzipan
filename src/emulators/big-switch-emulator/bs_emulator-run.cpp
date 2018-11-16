// Copyright 2018 Joseph Espy MIT LICENSE jespy@JosephEspy.com

/*
 *  Here be dragons - This is where we emulate the redcode virtual machine
 *
 *  There are a number of fundimental tradeoffs to be considered for the 
 *      design of an emulator
 * 
 *      - Minimized branching factor vs minimized redundent computation:
 *          for instance, some instructions do not need to evaluate the a/b_mode
 *          But it may be faster to calculate it than increase the branching factor
 * 
 *      - Minimized code size vs data size:
 *          We want to keep the entire instruction pipeline in cache, as well as the entire
 *          core array.  We can minify the size of the array by packing modifier and modes 
 *          into fewer bits, but this requries a more cycles and complexity/code size
 *          to decode.  
 * 
 *  This implementation trades data size to optimize code size/complexity
 *  Takes a middle of the road approach to redundent computation vs branching factor
 *
 *  The following tradeoffs are made:
 *      - A redcode instruction is 10 bytes (on my x86-64 gcc)
 *          it could fit into 6 byts (or fewer with ints smaller than 16 bit)
 *
 *      - To fit it into even smaller instruction, would need to mask/shift
 *          out relevent bits to extract modifiers - extra few cycles per loop
 *
 *      - Basic switch at the instruciton level, could switch at the
 *          instruction/modifier pair level
 *              - this might be compilied away, don't want to premature optimize
 *
 *      - We setup operands outside of the opcode switch statement
 *          this make the code smaller and more IC cache friendly
 *          but sometimes this work is not needed
 */

/*
 * DONE:
 *  - DAT
 *  - NOP
 *  - SPL
 *  - JMZ
 *  - DJN
 *  - JMN
 *  - LDP
 *  - STP
 *  - ADD
 *  - SUB
 *  - MUL
 *  - DIV
 *  - MOD
 *  - MOV
 *  - SLT
 *  - CMP
 *  - SEQ
 *  - JMP
 *
 * NOT DONE:
 *  - SNE
 */

#include <iostream>
#include <cassert>
#include <cstring>

#include "../../interfaces/bs_emulator.h"
#include "./bs_core.h"
#include "./redcode.h"
#include "../../interfaces/config.h"

#define likely(x) __builtin_expect(!!(x), 1)
#define unlikely(x) __builtin_expect(!!(x), 0)

// adds two positive numbers and returns the value modulo core_size
inline int16_t add(int16_t a, int16_t b) {
    return (a+b) % core_size;
}

/* returns the positive number less than core_size
 * which is congruent to a-b in the field specified by modulo core_size
 * assuming that a and b are less than core_size and positive
 */
inline int16_t sub(int16_t a, int16_t b) {
    return (a - b + core_size) % core_size;
}

/* The post-increemnt modifiers requre an address to be incremented after
 * the logic of an operation has concluded.  
 * This function is called during the cleanup prior to switching turns
 */
inline void BS_Emulator::post_increment() {

    while (!core.to_post_increment_a.empty()) {
        auto addr = core.to_post_increment_a.front();
        core.to_post_increment_a.pop_front();

        core.memory[addr].a_num = add(core.memory[addr].a_num, 1);
    }

    while (!core.to_post_increment_b.empty()) {
        int addr = core.to_post_increment_b.front();
        core.to_post_increment_b.pop_front();

        core.memory[addr].b_num = add(core.memory[addr].b_num, 1);
    }
}

inline void BS_Emulator::queue_task(int addr) {
    std::queue<int16_t> & queue =
        (core.turn_w1 ? core.task_queue_w1 : core.task_queue_w2);
    if(likely(queue.size() < max_num_tasks)) {
        queue.push(addr);
    }
}

/*
 * @param steps: number of steps to run before returning or zero for no limit
 * @return WON_BY_W1, WON_BY_W2, TIE, or PAUSED_EXECUTION
 *
 * precondition: Core is loaded with valid instructions (including empty)
 * postcondition: Will return correct winner or tie signal or
 *   if tie or winner has not been reached by steps, PAUSED_EXECUTION
 *   if the game ends before any instructions are executed, NO_EXECUTION
 */

int BS_Emulator::run(int steps) {

    // run until i == cycles_before_tie or steps this run == steps
    int remaining_steps = cycles_before_tie - core.counter;
    // The value to return should we finish without a winner
    int ret_val;

    // set steps to the min(steps, remaining_steps)
    if (steps == 0) {
        ret_val = TIE;
        steps = remaining_steps;
    } else if (steps < remaining_steps) {
        ret_val = PAUSED_EXECUTION;
    } else {
        steps = remaining_steps;
    }

    // The core is not in a runnable state.
    // Did you clear it and then load two warriors
    if (steps < 1 ||
            core.task_queue_w1.empty() ||
            core.task_queue_w2.empty()) {
        return NO_EXECUTION;
    }

    /*
     * overview of what happens in one cycle:
     *
     * - query the forward_list task queues for each warrior
     * - load the copy of instruction at the addresses returned by the queue
     *
     * - Four step evaluation, each with a switch statement
     *      1) evaluate a-operand
     *              - compute a-pointer
     *              - copy cached a-instruction
     *      2) evaluate b-operand
     *              - compute b_pointer
     *              - copy cached b-instruction
     *      3) evaluate modifier/op pair in three nested switches
     *              a) evaluate modifier-type
     *                      - determine if comparisons happen between values,
     *                          pairs, or entire instructions
     *              b) evaluate operand
     *                      - with modifier type and op, the instruction can be
     *                          executed, we just don't know where the result goes
     *                      - this is also where we add to instruction queue
     *                      - dat, div, mod check if queue becomes empty
     *              c) evaluate modifier
     *                      - modifier determines where result is written
     *      4) post-increment values that are stored for postincrement
     */

    /* Preconditions at the top of the loop:
     * - both task queues are non-empty
     * - all a-numbers and b-numbers in core are positive and < core_size
     * - all opcode.modifier pairs are valid (see ICWS.94-5.1)
     */

    for (int i = 0 ; i < steps ; i++) {
        core.turn_w1 = !core.turn_w1;
        // get the queue of the current warrior
        std::queue<int16_t>& cur_queue 
            { (core.turn_w1 ? core.task_queue_w1 : core.task_queue_w2) };
        /* ~~ These are "Cached Values" ~~
         * They are computed or read once per cycle, and subsequent
         *   modification to the core them will not update them
         *
         * program_counter, current_instr (including all fields),
         * a_ptr, a_instr, b_ptr, b_isntr
         */

        // We check for empty queues after a DAT/DIV/MOD instructions,
        // the only instructions that do not append to the queue
        auto program_counter = cur_queue.front();
        cur_queue.pop();

        // addresses are %core_size before being appended to task queue
        auto current_instr = core.memory[program_counter];

        // note: this is absolute while ICWS defines it as relative
        int a_ptr {};
        Instruction a_instr {};

        // TODO(base0x10): write unit tests about pre-decerement behavior

        // Based on a_mode and a_num, set values for a_ptr and a_instr
        // note that predec modifies values that maybe read late in cycle
        switch (current_instr.a_mode) {
            // sets the ptr to PC and a_instr to core[PC]
            case IMMEDIATE :
            {
                a_ptr = program_counter;
                a_instr = current_instr;
                break;
            }

            // a_ptr is a_num + pc
            case DIRECT :
            {
                a_ptr = add(program_counter, current_instr.a_num);
                a_instr = core.memory[a_ptr];
                break;
            }

            // a_ptr points to the value pointed to by a_num of a_num + PC
            case INDIRECT_A :
            {
                // the value of the a_num pointed to by current a_num
                int tmp_a_ptr = add(program_counter, current_instr.a_num);
                int tmp_a = core.memory[tmp_a_ptr].a_num;
                a_ptr = add(tmp_a_ptr, tmp_a);
                a_instr = core.memory[a_ptr];
                break;
            }

            // a_ptr points to the value pointed to by b_num of a_num + PC
            case INDIRECT_B :
            {
                int tmp_a_ptr = add(program_counter, current_instr.a_num);
                int tmp_b = core.memory[tmp_a_ptr].b_num;
                a_ptr = add(tmp_a_ptr, tmp_b);
                a_instr = core.memory[a_ptr];
                break;
            }

            // TODO(base0x10): can you predecrement with 0 value
            // eg. decrement yourself
            // and what is the correct behavior
            // a_ptr points to the value pointed to by --a_num of a_num + PC
            case PREDEC_A :
            {
                // the value of the a_num pointed to by current a_num
                int tmp_a_ptr = add(program_counter, current_instr.a_num);

                core.memory[tmp_a_ptr].a_num =
                    sub(core.memory[tmp_a_ptr].a_num, 1);

                int tmp_a = core.memory[tmp_a_ptr].a_num;

                a_ptr = add(tmp_a_ptr, tmp_a);
                a_instr = core.memory[a_ptr];
                break;
            }

            // TODO(base0x10): can you predecrement with 0 value
            // eg. decrement yourself
            // and what is the correct behavior
            // a_ptr points to the value pointed to by --b_num of a_num + PC
            case PREDEC_B :
            {
                // the value of the a_num pointed to by current a_num
                int tmp_a_ptr = add(program_counter, current_instr.a_num);

                core.memory[tmp_a_ptr].b_num =
                    sub(core.memory[tmp_a_ptr].b_num, 1);

                int tmp_b = core.memory[tmp_a_ptr].a_num;

                a_ptr = add(tmp_a_ptr, tmp_b);
                a_instr = core.memory[a_ptr];
                break;
            }

            // similar to INDIRECT_A but postincrement tmp_a at end of cycle
            case POSTINC_A :
            {
                // the value of the a_num pointed to by current a_num
                int tmp_a_ptr = add(program_counter, current_instr.a_num);

                // add address to later increment a field
                core.to_post_increment_a.push_front(tmp_a_ptr);

                int tmp_a = core.memory[tmp_a_ptr].a_num;
                a_ptr = add(tmp_a_ptr, tmp_a);
                a_instr = core.memory[a_ptr];
                break;
            }

            // similar to INDIRECT_B but postincrement tmp_b at end of cycle
            case POSTINC_B :
            {
                int tmp_a_ptr = add(program_counter, current_instr.a_num);

                // after execution, increment the b field of this addr
                core.to_post_increment_b.push_front(tmp_a_ptr);

                int tmp_b = core.memory[tmp_a_ptr].b_num;
                a_ptr = add(tmp_a_ptr, tmp_b);
                a_instr = core.memory[a_ptr];
                break;
            }

            default :
                assert(0 && "Invalid modifier in core");
        }

        int b_ptr {};
        Instruction b_instr {};

        // Based on a_mode and a_num, set values for a_ptr and a_instr
        // note that predec modifies values that maybe read late in cycle
        switch (current_instr.b_mode) {
            // sets the b_ptr to PC and b_instr to core[PC]
            case IMMEDIATE :
            {
                b_ptr = program_counter;
                b_instr = core.memory[program_counter];
                break;
            }

            // b_ptr is b_num + pc
            case DIRECT :
            {
                b_ptr = add(program_counter, current_instr.b_num);
                b_instr = core.memory[b_ptr];
                break;
            }

            // b_ptr points to the value pointed to by a_num of b_num + PC
            case INDIRECT_A :
            {
                // the value of the a_num pointed to by current b_num
                int tmp_b_ptr = add(program_counter, current_instr.b_num);
                int tmp_a = core.memory[tmp_b_ptr].a_num;
                b_ptr = add(tmp_b_ptr, tmp_a);
                b_instr = core.memory[b_ptr];
                break;
            }

            // b_ptr points to the value pointed to by b_num of b_num + PC
            case INDIRECT_B :
            {
                int tmp_b_ptr = add(program_counter, current_instr.b_num);
                int tmp_b = core.memory[tmp_b_ptr].b_num;
                b_ptr = add(tmp_b_ptr, tmp_b);
                b_instr = core.memory[b_ptr];
                break;
            }

            // TODO(base0x10): write better comments, indirection is tough
            case PREDEC_A :
            {
                int tmp_b_ptr = add(program_counter, current_instr.b_num);

                core.memory[tmp_b_ptr].a_num =
                    sub(core.memory[tmp_b_ptr].a_num, 1);

                int tmp_a = core.memory[tmp_b_ptr].a_num;

                b_ptr = add(tmp_b_ptr, tmp_a);
                b_instr = core.memory[b_ptr];
                break;
            }

            // TODO(base0x10): documentation is hard, man
            case PREDEC_B :
            {
                int tmp_b_ptr = add(program_counter, current_instr.b_num);

                core.memory[tmp_b_ptr].b_num =
                    sub(core.memory[tmp_b_ptr].b_num, 1);

                int tmp_b = core.memory[tmp_b_ptr].b_num;

                b_ptr = add(tmp_b_ptr, tmp_b);
                b_instr = core.memory[b_ptr];
                break;
            }

            // TODO(base0x10): writing documentation for untested code is amoral
            case POSTINC_A :
            {
                /*
                 * go to the location specified by b_num
                 * specify that the a field of this should be incremented later
                 * use the location specified by this instructions a field
                 */
                int tmp_b_ptr = add(program_counter, current_instr.b_num);

                // add to list that will have b field incremented later
                core.to_post_increment_a.push_front(tmp_b_ptr);

                int tmp_a = core.memory[tmp_b_ptr].a_num;
                b_ptr = add(tmp_b_ptr, tmp_a);
                b_instr = core.memory[b_ptr];
                break;
            }

            // TODO(base0x10): documentation slows compilation, this is unacceptable
            case POSTINC_B :
            {
                /*
                 * go to the location specified by b_num
                 * specify that its b field should be incremented later
                 * use the location specified by this instruction's b field
                 */
                int tmp_b_ptr = add(program_counter, current_instr.b_num);

                // add to list that will have b field incremented later
                core.to_post_increment_b.push_front(tmp_b_ptr);

                int tmp_b = core.memory[tmp_b_ptr].b_num;
                b_ptr = add(tmp_b_ptr, tmp_b);
                b_instr = core.memory[b_ptr];
                break;
            }

            default :
                assert(0 && "Invalid modifier in core");
        }

        // The following instructions either do not use their modifier
        // or use them differently than most instructions.
        switch(current_instr.op) {
            case DAT:
            {
                if (cur_queue.empty()) {
                    return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                }
                post_increment();
                continue;
            }
            case NOP:
            {
                queue_task(add(program_counter, 1));
                post_increment();
                continue;
            }
            case JMP:
            {
                queue_task(a_ptr);
                post_increment();
                continue;
            }
            case SPL:
            {
                queue_task(add(program_counter, 1));
                queue_task(a_ptr);
                post_increment();
                continue;
            }
            case JMZ:
            {
                int b_val;
                switch (current_instr.mod) {
                    case A:  // FALLTHROUGH, same behavior as BA modifier
                    case BA:
                    {
                        b_val = b_instr.a_num;
                        break;
                    }
                    case B:  // FALLTHROUGH, same behavior as AB modifier
                    case AB:
                    {
                        b_val = b_instr.b_num;
                        break;
                    }
                    case F:  // FALLTHROUGH, same as I
                    case X:  // FALLTHROUGH, same as I
                    case I:
                    {
                        // we only care if both are 0 or not, so bitwise &
                        b_val = b_instr.a_num & b_instr.b_num;
                        break;
                    }
                }
                if (b_val == 0) {
                    queue_task(a_ptr);
                } else {
                    queue_task(add(program_counter, 1));
                }
                post_increment();
                continue;

            }
            case DJN:
            {
                int b_val;
                switch (current_instr.mod) {
                    case A:  // FALLTHROUGH, same behavior as BA modifier
                    case BA:
                    {
                        // b_instruction is always cache of core.memory[b_ptr]
                        core.memory[b_ptr].a_num
                            = sub(core.memory[b_ptr].a_num, 1);

                        b_instr.a_num = sub(b_instr.a_num, 1);
                        b_val = b_instr.a_num;
                        break;
                    }
                    case B:  // FALLTHROUGH, same behavior as AB modifier
                    case AB:
                    {
                        // b_instruction is always cache of core.memory[b_ptr]
                        core.memory[b_ptr].b_num
                            = sub(core.memory[b_ptr].b_num, 1);

                        b_instr.b_num = sub(b_instr.b_num, 1);
                        b_val = b_instr.b_num;
                        break;
                    }
                    case F:  // FALLTHROUGH, same as I
                    case X:  // FALLTHROUGH, same as I
                    case I:
                    {

                        core.memory[b_ptr].b_num 
                            = sub(core.memory[b_ptr].b_num, 1);

                        core.memory[b_ptr].a_num 
                            = sub(core.memory[b_ptr].a_num, 1);

                        // decrement both values in b_instr
                        b_instr.b_num = sub(b_instr.b_num, 1);
                        b_instr.a_num = sub(b_instr.a_num, 1);

                        // we only care if both are 0 or not, so bitwise &
                        b_val = b_instr.a_num & b_instr.b_num;
                        break;
                    }
                }
                if (b_val != 0) {
                    queue_task(a_ptr);
                } else {
                    queue_task(add(program_counter, 1));
                }
                post_increment();
                continue;

            }
            case JMN:
            {
                int b_val;
                switch (current_instr.mod) {
                    case A:  // FALLTHROUGH, same behavior as BA modifier
                    case BA:
                    {
                        b_val = b_instr.a_num;
                        break;
                    }
                    case B:  // FALLTHROUGH, same behavior as AB modifier
                    case AB:
                    {
                        b_val = b_instr.b_num;
                        break;
                    }
                    case F:  // FALLTHROUGH, same as I
                    case X:  // FALLTHROUGH, same as I
                    case I:
                    {
                        // we only care if both are 0 or not, so bitwise &
                        b_val = b_instr.a_num & b_instr.b_num;
                        break;
                    }
                }
                if (b_val != 0) {
                    queue_task(a_ptr);
                } else {
                    queue_task(add(program_counter, 1));
                }
                post_increment();
                continue;
            }
            case LDP:
            {
                assert(0 && "Have not implemented p-space yet");
            }
            case STP:
            {
                assert(0 && "Have not implemented p-space yet");
            }
            default:
            {
                break;
            }
        }

        // From below this point, all instructions make use of their modifiers
        // so we unconditionally compute the single value modifiers

        // outputs of single value modifiers A, B, AB, BA
        int a_val, b_val;
        int16_t *b_target; 

        // This switch computes single value modifiers
        switch (current_instr.mod) {
            case A:
            {
                a_val = a_instr.a_num;
                b_val = b_instr.a_num;
                b_target = &core.memory[b_ptr].a_num;
                break;
            }
            case B:
            {
                a_val = a_instr.b_num;
                b_val = b_instr.b_num;
                b_target = &core.memory[b_ptr].b_num;
                break;
            }
            case AB:
            {
                a_val = a_instr.a_num;
                b_val = b_instr.b_num;
                b_target = &core.memory[b_ptr].b_num;
                break;
            }
            case BA:
            {
                a_val = a_instr.b_num;
                b_val = b_instr.a_num;
                b_target = &core.memory[b_ptr].a_num;
                break;
            }
            default: 
            {
                break;
            }
        }

        // This switch catches ops that use mod and some that alter control flow
        switch (current_instr.op) {
            /*
             * Behavior of div:
             * If you divide by zero then your process is removed from queue
             * if you are dividing pairs of values, if either divide by zero,
             * the process is removed from queue, but the other division
             * still happens
             */
            case DIV:
            {
                switch (current_instr.mod) {
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        // If you try to divide by zero, remove from queue
                        if(b_val == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        }
                        post_increment();
                        queue_task(add(program_counter, 1));
                        *b_target = a_val / b_val;
                        break;
                    }
                    case I:  // FALLTHROUGH
                    case F:
                    {
                        if (a_instr.a_num) core.memory[b_ptr].a_num =
                            (b_instr.a_num / a_instr.a_num) % core_size;
                        if (a_instr.b_num) core.memory[b_ptr].b_num = 
                            (b_instr.b_num / a_instr.b_num) % core_size;

                        if(a_instr.a_num == 0 || a_instr.b_num == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        }
                        break;
                    }
                    case X:
                    {
                        if (a_instr.a_num) core.memory[b_ptr].a_num =
                            (b_instr.b_num / a_instr.a_num) % core_size;
                        if (a_instr.b_num) core.memory[b_ptr].b_num = 
                            (b_instr.a_num / a_instr.b_num) % core_size;

                        if(a_instr.a_num == 0 || a_instr.b_num == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                }
                queue_task(add(program_counter, 1));
                post_increment();
                continue;
            }
            case MOD:
            {
                switch (current_instr.mod) {
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        // If you try to divide by zero, remove from queue
                        if(b_val == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        }
                        *b_target = a_val / b_val;
                        break;
                    }
                    case I:  // FALLTHROUGH
                    case F:
                    {
                        if (a_instr.a_num) core.memory[b_ptr].a_num =
                            (b_instr.a_num % a_instr.a_num) % core_size;
                        if (a_instr.b_num) core.memory[b_ptr].b_num = 
                            (b_instr.b_num % a_instr.b_num) % core_size;

                        if(a_instr.a_num == 0 || a_instr.b_num == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        }
                        break;
                    }
                    case X:
                    {
                        if (a_instr.a_num) core.memory[b_ptr].a_num =
                            (b_instr.b_num % a_instr.a_num) % core_size;
                        if (a_instr.b_num) core.memory[b_ptr].b_num = 
                            (b_instr.a_num % a_instr.b_num) % core_size;

                        if(a_instr.a_num == 0 || a_instr.b_num == 0) {
                            if (cur_queue.empty()) {
                                return core.turn_w1 ? WON_BY_W2 : WON_BY_W1;
                            }
                            post_increment();
                            continue;
                        }
                        break;
                    }
                }
                queue_task(add(program_counter, 1));
                post_increment();
                continue;
            }
            case SLT:
            {
                // if the a_val is less than the b_val, queue two after pc, else queue normally
                switch (current_instr.mod) {
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        if (a_val < b_val) {
                            queue_task(add(program_counter, 2));
                        } else  {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case X:
                    {
                        if (a_instr.a_num < b_instr.b_num && a_instr.b_num < b_instr.a_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case F:
                    case I:
                    {
                        if (a_instr.a_num < b_instr.a_num && a_instr.b_num < b_instr.b_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                }
                post_increment();
                continue;
            }
            case MOV:
            {
                switch (current_instr.mod) {

                    // b_target and a_val are already computed, so we can fallthrough
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        *b_target = a_val;
                        break;
                    }

                    // copies both fields of a_instr into b_ptr but swapping them
                    case X:
                    {
                        core.memory[b_ptr].a_num = a_instr.b_num;
                        core.memory[b_ptr].b_num = a_instr.a_num;
                        break;
                    }
                    // copies both fields if a_instr into b_ptr
                    case F:
                    {
                        core.memory[b_ptr].a_num = a_instr.a_num;
                        core.memory[b_ptr].b_num = a_instr.b_num;
                        break;
                    }
                    // copies entire instruction from a_instr to b_ptr
                    case I:
                    {
                        core.memory[b_ptr] = a_instr;
                        break;
                    }
                }
                queue_task(add(program_counter, 1));
                post_increment();
                continue;
            }
            case CMP:
            case SEQ:
            {
                // if the a_val is equal to the b_val, queue two after pc, else queue normally
                switch (current_instr.mod) {
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        if (a_val == b_val) {
                            queue_task(add(program_counter, 2));
                        } else  {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case X:
                    {
                        if (a_instr.a_num == b_instr.b_num && a_instr.b_num == b_instr.a_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case F:
                    {
                        if (a_instr.a_num == b_instr.a_num && a_instr.b_num == b_instr.b_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;                    
                    }
                    case I:
                    {
                        // compare entire instructions for equality
                        if (memcmp(&a_instr, &b_instr, sizeof(Instruction)) == 0) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;                    
                    }
                }
                post_increment();
                continue;
            }
            case SNE:
            {
                // if the a_val is equal to the b_val, queue two after pc, else queue normally
                switch (current_instr.mod) {
                    case A:
                    case B:
                    case AB:
                    case BA:
                    {
                        if (a_val != b_val) {
                            queue_task(add(program_counter, 2));
                        } else  {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case X:
                    {
                        if (a_instr.a_num != b_instr.b_num && a_instr.b_num != b_instr.a_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;
                    }
                    case F:
                    {
                        if (a_instr.a_num != b_instr.a_num && a_instr.b_num != b_instr.b_num) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;                    
                    }
                    case I:
                    {
                        // compare entire instructions for equality
                        if (memcmp(&a_instr, &b_instr, sizeof(Instruction)) != 0) {
                            queue_task(add(program_counter, 2));
                        } else {
                            queue_task(add(program_counter, 1));
                        }
                        break;                    
                    }
                }
                post_increment();
                continue;
            }
            default: 
            {
                break;
            }
        }

        // ADD, SUB, MUL write their outputs in the same way,
        // and cannot alter control flow
        switch (current_instr.mod) {
            case A:
            case B:
            case AB:
            case BA:
            {
                switch (current_instr.op){
                    case ADD: 
                    {
                        *b_target = add(a_val, b_val);
                        break;
                    }
                    case SUB:
                    {
                        *b_target = sub(b_val, a_val);
                        break;
                    }
                    case MUL:
                    {
                        *b_target = (a_val * b_val)%core_size;
                        break;
                    }
                    default: assert(0 && 
                        "unreachable, other ops have been dealt with");
                }
                break;

            }
            case I:  // FALLTHROUGH to .F
            case F:
            {
                Instruction *b_target_instr = &core.memory[b_ptr];
                switch (current_instr.op) {
                    case ADD: 
                    {
                        b_target_instr->a_num = add(b_instr.a_num, a_instr.a_num);
                        b_target_instr->b_num = add(b_instr.b_num, a_instr.b_num);

                        break;
                    }
                    case SUB:
                    {
                        b_target_instr->a_num = sub(b_instr.a_num, a_instr.a_num);
                        b_target_instr->b_num = sub(b_instr.b_num, a_instr.b_num);
                        break;
                    }
                    case MUL:
                    {
                        b_target_instr->a_num =
                            (b_instr.a_num * a_instr.a_num) % core_size;
                        b_target_instr->b_num = 
                            (b_instr.b_num * a_instr.b_num) % core_size;
                        break;
                    }
                    default: assert(0 && 
                        "unreachable, other ops have been dealt with");
                }
                break;
            }
            case X:
            {
                Instruction *b_target_instr = &core.memory[b_ptr];
                switch (current_instr.op) {
                    case ADD: 
                    {
                        b_target_instr->a_num = add(b_instr.b_num, a_instr.a_num);
                        b_target_instr->b_num = add(b_instr.a_num, a_instr.b_num);

                        break;
                    }
                    case SUB:
                    {
                        b_target_instr->a_num = sub(b_instr.b_num, a_instr.a_num);
                        b_target_instr->b_num = sub(b_instr.a_num, a_instr.b_num);

                        break;
                    }
                    case MUL:
                    {
                        b_target_instr->a_num =
                            (b_instr.b_num * a_instr.a_num) % core_size;
                        b_target_instr->b_num =
                            (b_instr.a_num * a_instr.b_num) % core_size;

                        break;
                    }
                    default: assert(0 && 
                        "unreachable, other ops have been dealt with");
                }
                break;
            }

        }


        post_increment();
        queue_task(add(program_counter, 1));

    }
    return ret_val;
}
