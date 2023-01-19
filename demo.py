import nexsys

soln, log = nexsys.py_solve("""
keep x on [-10, 0]
guess -2.5 for x

x^2 = 9
""",
1E-10, 300, False)