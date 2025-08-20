import json


with open("systolic.json") as f:
    netlist = json.load(f)
    print("Successfully loaded JSON netlist from systolic.json")