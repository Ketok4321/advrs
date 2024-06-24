def char(c, nl=False):
    print(f"""class '{c}' extends Character:
    method equals(c):
        return c is '{c}'
    end{"""

    method isNewline():
        return True
    end""" if nl else ""}
end""")

for i in range(32, 126+1):
    c = chr(i)
    if c == "\'": c = "\\'"
    if c == "\\": c = "\\\\"
    char(c)
    print()
char("\\n", nl=True)
