def gen_file():
    with open("rot_break.txt", 'w') as f:
        for i in range(2,101):
            f.write(f"cx q[0], q[{i}]")
            f.write("rz(1.5) q[0]")
