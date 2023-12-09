"""Partially converts the python decorators
   For instructions into rust code. 
   The conversion is not perfect however and
   requires some manual changes."""
instructions = set()

with open('./firmware/template/build/main.c.s') as assembly:
    for line in assembly:
        line = line.strip()

        if (len(line) == 0 or ';' == line[0] or '.' == line[0]):
            continue
        instruction = ""

        if (':' in line):
            line = line.split(':')
            instruction = line[1].strip().split(' ')[0]
        else:
            instruction = line.split(' ')[0]
        instructions.add(instruction)
    print(instructions)

with open("converted_instructions.rs", 'w') as converted_instructions_file:

    with open("./65c02_instrs.py") as py_instrs_file:
        line = next(py_instrs_file)
        while (True):
            go_next = True

            if ('@instruction' in line):
                is_needed = False
                curr_instr = ""
                for instr in instructions:
                    instr: str
                    if (f'name=\"{instr.upper()}\"' in line):
                        is_needed = True
                        curr_instr = instr
                        break

                if not is_needed:
                    line = next(py_instrs_file)
                    continue
                
                decorator = line
                line = next(py_instrs_file)
                idx = line.find("_") + 1
                opcode = line[idx:line.find('(')]
                converted_instructions_file.write(f'// {decorator}')
                converted_instructions_file.write(f'instructions.insert({opcode}, |self2| '+"{\n")

                while (True):
                    line = next(py_instrs_file)
                    line = line.replace("self", "self2")
                    if ("@instruction" in line):
                        converted_instructions_file.write("});\n")
                        go_next = False
                        break
                    if (line.strip() == ""):
                        converted_instructions_file.write(line)
                        
                    else:
                        converted_instructions_file.write(line.strip('\n')+";\n")
            if (go_next):
                line = next(py_instrs_file)
                    


