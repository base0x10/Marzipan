# Marzipan

Marzipan is a collection of tools and and emulators for CoreWar and the Redcode
assembly language written in Rust.

## What is CoreWar?

CoreWar is a game played by humans who write programs in the Redcode assembly
language.  These programs fight to the death in a single virtual computer core.

### Links

 * [Koth.org](https://koth.org) hosts continuous King Of The Hill tournaments
   that have been running for decades, and links to various guides and
   community resources.
 * [The Beginner's Guide To Corewar](https://vyznev.net/corewar/guide.html) by
   Ilmari Karonen is the definitive best place to start to learn the Redcode
   assembly language.  

## Current Status

Everything is WIP as I consolidate various libraries and programs.  I'm working
towards a "0.1.0" release with a minimal but complete and well tested feature
set.  

### Alpha Release Goals

| Component          | Feature                                         | Status                                    |
|--------------------|-------------------------------------------------|-------------------------------------------|
| redcode            | ICWS 88 and 94 representation and serialization | Feature complete, nearly complete testing |
| redcode-parser     | ICWS 88 and 94 loadfile parsing                 | Feature complete, very incomplete testing |
| marzipan-cli       | pMARS style emulator binary                     | Planned                                   |
| marzipan-core      | Low level generic emulator interface            | Complete                                  |
| marzipan-core      | Reference emulator implementation               | Feature complete, very incomplete testing |
| marzipan-core      | Optimized nursery emulator implementation       | Planned                                   |
| marzipan-core      | Optimized bytecode interpreter implementation   | Yet unpublished POC =)                    |
| verification-tests | Cycle accurate pMARS differential testing       | Yet unpublished POC =)                    |
| verification-tests | Permuted 2-warrior battle pMARS diff. testing   | Yet unpublished POC =)                    |

## Components

 * [marzipan-cli](./marzipan-cli/) is (*will be*) a pMARS style stand alone
   emulator binary
 * [marzaipan-core](./marzipan-core/) contains low level emulator
   implementations.
 * [redcode](./redcode/) is a small, reuseable library for Redcode
   representation types in rust, with Serde serialization.
 * [redcode-parser](./redcode-parser/) contains libraries for parsing warriors
   written in Redcode assembly.  Currently only the loadfile _assembled_ subset
   of the language is supported.  pMARS or a similar tool should be used to
   assemble warriors with macros or omitted modifiers to the loadfile format.
 * [verification-tests](./verification-tests/) (*will*) run differential testing
   between pMARS and Marzipan emulator implementations.


## Copyright, Contribution, and Licensing

Copyright (C) 2023 Joseph Espy

Marzipan is dual-licensed under (your choice) of MIT and Apache-2.0.

Contributions are welcome!
