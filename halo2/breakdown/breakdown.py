import argparse
import json

parser = argparse.ArgumentParser()
parser.add_argument("name")
parser.add_argument("input")
parser.add_argument("output")


def get_time(line):
    elems = line.strip().split(".")
    if "ms" in elems[-1]:
        ti = float(elems[-2]+"."+elems[-1][:-2])
    elif "µs" in elems[-1]:
        ti = float(elems[-2]+"."+elems[-1][:-2])/1000
    elif "ns" in elems[-1]:
        ti = float(elems[-2]+"."+elems[-1][:-2])/1000000
    else:
        ti = float(elems[-2]+"."+elems[-1][:-2]) * 1000 

    return ti

if __name__ == "__main__":

    args = parser.parse_args()

    entire_ops = {
        "prove": {
            "fft": 0,
            "ifft": 0,
            "msm": 0,
            "other": 0,
            "total": 0,
        },
        "verify": {
            "fft": 0,
            "ifft": 0,
            "msm": 0,
            "pairing": 0,
            "other": 0,
            "total": 0,
        },
    }
    phase_ops = {
        "prove": {

        },
        "verify": {

        }
    }

    process = ""
    phase = ""
    parse_flag = False

    with open(args.input, "r") as f:
        raw = f.readlines()
        for line in raw:
            if line.strip() == "Start:   Prover":
                parse_flag = True
                process = "prove"
            elif line.strip() == "Start:   Verifier":
                parse_flag = True
                process = "verify"
            elif parse_flag:
                line = line.strip()
                if line.startswith("··Start:   "):
                    temp = line.replace("··Start:   ", "")
                    if temp.startswith("Phase"):
                        phase = temp

                elif line.startswith("End:     "):
                    temp = line.replace("End:     ", "")
                    ti = get_time(temp)
                    entire_ops[process]["total"] = ti
                    entire_ops[process]["other"] = \
                        ti - entire_ops[process].get("fft") - \
                        entire_ops[process].get("ifft") - \
                        entire_ops[process].get("msm") - \
                        entire_ops[process].get("pair", 0)
                    phase_ops[process]["Other"] = ti - \
                        sum([val["total"] for _, val in phase_ops[process].items()])

                    parse_flag = False 
                    process = ""
                elif line.startswith("··End:     "):
                    temp = line.replace("··End:     ", "")
                    if temp.startswith("Phase"):
                        ti = get_time(temp)
                        phase_breakdown = phase_ops[process].get(phase, {
                            "fft": 0,
                            "ifft": 0,
                            "msm": 0,
                            "pairing": 0,
                            "other": 0,
                            "total": 0,
                        })
                        phase_breakdown["total"] = ti
                        phase_breakdown["other"] = \
                            ti - phase_breakdown.get("fft") - \
                            phase_breakdown.get("ifft") - \
                            phase_breakdown.get("msm") - \
                            phase_breakdown.get("pair", 0)
                        phase_ops[process][phase] = phase_breakdown

                    else:
                        ti = get_time(temp)
                        entire_ops[process]["ifft"] += ti
                    
                    phase = ""
                elif line.startswith("····End:     "):
                    temp = line.replace("····End:     ", "")
                    ti = get_time(temp)
                    elems = temp.split(".")
                    op = elems[0].strip().split("-")[0]
                    entire_ops[process][op] += ti 
                    phase_breakdown = phase_ops[process].get(phase, {
                        "fft": 0,
                        "ifft": 0,
                        "msm": 0,
                        "pairing": 0,
                        "other": 0,
                        "total": 0,
                    })
                    phase_breakdown[op] += ti
                    phase_ops[process][phase] = phase_breakdown
    with open(args.output, "w") as f:
        json.dump({
            "name": args.name,
            "entire_ops": entire_ops,
            "phase_ops": phase_ops
        }, f)
