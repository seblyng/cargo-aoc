import sys

with open(sys.argv[1]) as f:
    print(f.readline(), end="")
    print(f.readline(), end="")

