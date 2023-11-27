import argparse
import json
import math
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("num_constraint")

if __name__ == "__main__":

    args = parser.parse_args()
    num_constraint = args.num_constraint
    k = int(math.ceil(math.log2(int(num_constraint))))
    actual_constraint = 2**(k-1)

    with open("dummy_main.circom", "w") as f:
        f.writelines([
            "pragma circom 2.0.0;\n",
            'include "dummy.circom";\n',
            "component main = Dummy({});\n".format(str(actual_constraint))
        ])

    with open("input.json", "w") as f:
        json.dump({
            "a": [1] * actual_constraint,
            "out": 1
        }, f)
    
    if k <= 8:
        url = "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_08.ptau"
    elif k == 9:
        url = "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_09.ptau"
    else:
        url = "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_{}.ptau".format(k)
    

    subprocess.run(["wget", "-O", "powers.ptau", url])
    