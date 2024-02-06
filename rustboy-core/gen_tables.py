import json
import requests

response = requests.get("https://gbdev.io/gb-opcodes/Opcodes.json")
data = response.json()
cnt = 0
unprefixed = data["unprefixed"]
prefixed = data["cbprefixed"]
print('''#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FlagEffect {
    Set,
    Unset,
    Conditional,
    NoEffect,
}
pub const T: FlagEffect = FlagEffect::Set;
pub const F: FlagEffect = FlagEffect::Unset;
pub const A: FlagEffect = FlagEffect::Conditional;
pub const B: FlagEffect = FlagEffect::NoEffect;
''')
print("pub const CLOCK:[u8; 256] = [")
for d in unprefixed:
    out ='{:2d}'.format(unprefixed[d]["cycles"][0]) 
    if len(unprefixed[d]["cycles"]) == 2:
        out ='{:2d}'.format(unprefixed[d]["cycles"][1]) 
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

print("pub const ALT_CLOCK:[u8; 256] = [")
for d in unprefixed:
    out ='{:2d}'.format(0) 
    if len(unprefixed[d]["cycles"]) == 2:
        out ='{:2d}'.format(unprefixed[d]["cycles"][0]) 
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

print("pub const CB_CLOCK:[u8; 256] = [")
for d in prefixed:
    out ='{:2d}'.format(prefixed[d]["cycles"][0]) 
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

def flag_table(name, id_char, is_prefixed):
    cnt = 0
    print("pub const " + name + ":[FlagEffect; 256] = [")
    if is_prefixed:
        table = prefixed
    else:
        table = unprefixed

    for d in table:
        out = ""
        curr = table[d]["flags"][id_char]
        if curr == "1":
            out = "T"
        elif curr == "0":
            out = "F"
        elif curr == id_char:
            out = "A"
        elif curr == "-":
            out = "B"
        else:
            out = "AHHHHH"
        print(out, end=", ")
        cnt += 1
        if cnt % 16 == 0:
            print()
    print("];")

flag_table("ZERO_FLAG", "Z", False)
flag_table("SUB_FLAG", "N", False)
flag_table("HALF_FLAG", "H", False)
flag_table("CARRY_FLAG", "C", False)
flag_table("CB_ZERO_FLAG", "Z", True)
flag_table("CB_SUB_FLAG", "N", True)
flag_table("CB_HALF_FLAG", "H", True)
flag_table("CB_CARRY_FLAG", "C", True)
