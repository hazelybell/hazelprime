import random

n_bits_per_line = 256
indent = 12

def make(bits):
    return random.randint(0, 2**bits+1)

def print_nicely(number):
    s = "{:X}".format(number);
    n_chars_per_line = n_bits_per_line//4
    first_len = (len(s)-1) % n_chars_per_line + 1
    first = s[0:first_len]
    padding = n_chars_per_line-len(first)
    ii = " " * padding + '"'
    l = [first]
    for i in range(first_len, len(s), n_chars_per_line):
        l.append(s[i:i+n_chars_per_line])
    sep = "\\\n" + (" " * (indent+1))
    s = sep.join(l)
    s = (" " * indent) + ii + s + '"'
    print(s)

a = make(1020);
b = make(1020);

print_nicely(a)
print_nicely(b)
print_nicely(a*b)
