// Redcode instructions require a bit of complexity to interpret
// Implementations usually fall into two camps:
//  - Big Switch: 
//      Using either templates or macros or script-generated code to 
//      write the logic for each Opcode X Modifier X Mode X Mode
//      A single large switch statement which dispatches to the correct behavior
//      Marzipan was origionally a c++ big-switch emulator
//  - Interpreter
//      Break up the code for each behavior, and seperate reusable logic
//      
//  Big Switch _should_ be faster, as the compilier has lots of information
//  and there is a minimum of one unpredictable branch per instruction, used
//  to index into the list of instructions.  Of course the compilier can (read will)
//  insert more unpredictable jumps, be we assume the compilier is smart enough
//  to do this well.  
//
//  The interpreter method is much much easiser to write and debug.  And an ideal
//  compilier would be able to generate the same machine code for two programs
//  with the same visable behavior.  
//
//  All of this is to say that marzipan does not take either path.  The emulator
//  should be debugable AND fast.
//
//  Marzipan translates each instruction into a series of micro-ops, and then executes
//  the micro-ops.  


