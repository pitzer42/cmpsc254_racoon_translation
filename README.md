# cmpsc254_racoon_translation

## firmware folder 
The firmware folder was copied from here: https://github.com/ucsbieee/mapache64/tree/main/firmware#creating-a-game 

This gives us the framework to write the fibonacci code in the template folder which we use to benchmark py65 and our rust implementation.

The folder benchmark1/ holds the entire template code for benchmark1, which is a straight copy of the firmware/template/ folder because we only did one benchmark. It also holds the output of our trials in python_benchmark1.txt and rust_benchmark1.txt. There is also a description of the commands we used for our benchmarks.

## main.rs

Our code is very similar to the structure of the py65 emulator found here: https://github.com/ucsbieee/py65/tree/main/py65/devices. 

The main function should load a mapache64.bin (located in firmware/template/dump/) file created by the make commands in the firmware/template/ folder. There is a copy of this bin file in benchmark1/template/dump/mapache64.bin.

This bin file is the entire contents of the memory before running the code.

The main function then steps through the memory starting at the Mpu6502.start_pc address. It will stop and dump out a binary file of the memory when it runs into the 0xdb opcode which is the opcode of the STP assembly instruction.

There are many helper functions defined within the Mpu6502 struct. These are all meant to be used by the opcodes themselves when doing their actual operations. The register contents are stored in these variables: pc (program counter), acc (accumulator), p (status register), sp (stack pointer), x, y (both used for addressing). See: https://en.wikibooks.org/wiki/6502_Assembly

Finally, the initializeInstructions() function creates a hashmap which maps the instruction opcodes to their individual operations. This allows the step function to quickly get the operation associated with an opcode. We use opcodes from both the 6502 device and the 65C02 device: https://github.com/ucsbieee/py65/tree/main/py65/devices. Not all of them are implemented, but enough to run our fibonacci benchmark.

Ignore the excycles and processorCycles members. These are not accurately kept up to date in our program.

## needed_instructions.py

This is a utility script for converting the python decorators found in these files: https://github.com/ucsbieee/py65/tree/main/py65/devices to rust HashMap.insert statements. Not a perfect conversion, some manual editing still needs to be done. It will read through the assembly instructions to find which ones need to be implemented. However, lots of opcodes that were not in the assembly were still in the machine code, so maybe better to just convert all the instructions.

## monitor.py

This file is included for the purpose of showing what changes we made to the original monitor.py from  for the purpose of benchmarking. Ctrl+F the file for "rs65" and you will see our changes. The change to the constructor arguments is for the purpose of using a basic array for memory access instead of checking for observers on each access. This is the original: https://github.com/ucsbieee/py65/blob/main/py65/monitor.py

