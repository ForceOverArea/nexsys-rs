import nexsys

# get code from demo.nxs file
with open("./demo.nxs","r") as f:    
    nexsys_code = f.read()

soln, _ = nexsys.solve(nexsys_code)

print(soln)