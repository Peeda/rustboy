import json
import requests

response = requests.get("https://gbdev.io/gb-opcodes/Opcodes.json")
data = response.json()
cnt = 0
unprefixed = data["unprefixed"]
print('''
#[derive(Debug, PartialEq, Eq)]
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

print("pub const ZERO_FLAG:[FlagEffect; 256] = [")
for d in unprefixed:
    out = ""
    match unprefixed[d]["flags"]["Z"]:
        case "1":
            out = "T"
        case "0":
            out = "F"
        case "Z":
            out = "A"
        case "-":
            out = "B"
        case other:
            out = "AHHHH"
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

print("pub const SUB_FLAG:[FlagEffect; 256] = [")
for d in unprefixed:
    out = ""
    match unprefixed[d]["flags"]["N"]:
        case "1":
            out = "T"
        case "0":
            out = "F"
        case "N":
            out = "A"
        case "-":
            out = "B"
        case other:
            out = "AHHHH"
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

print("pub const HALF_FLAG:[FlagEffect; 256] = [")
for d in unprefixed:
    out = ""
    match unprefixed[d]["flags"]["H"]:
        case "1":
            out = "T"
        case "0":
            out = "F"
        case "H":
            out = "A"
        case "-":
            out = "B"
        case other:
            out = "AHHHH"
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")

print("pub const CARRY_FLAG:[FlagEffect; 256] = [")
for d in unprefixed:
    out = ""
    match unprefixed[d]["flags"]["C"]:
        case "1":
            out = "T"
        case "0":
            out = "F"
        case "C":
            out = "A"
        case "-":
            out = "B"
        case other:
            out = "AHHHH"
    print(out, end=", ")
    cnt += 1
    if cnt % 16 == 0:
        print()
print("];")
