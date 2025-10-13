import koicore_py

parser = koicore_py.PyParser("#hello world\nThis is text.", command_threshold=1)
for cmd in parser:
    print(f"Command: {cmd.name}")
    for param in cmd.params:
        if param.is_basic():
            value = param.as_value()
            print(f"  - {value}")
        elif param.is_composite():
            name, value = param.as_composite()
            print(f"  - {name}({value})")

parser2 = koicore_py.PyParser("#draw Line 2")
cmd = parser2.next_command()
if cmd:
    print(cmd.name)