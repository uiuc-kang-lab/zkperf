import argparse
from datetime import datetime
import json

parser = argparse.ArgumentParser()
parser.add_argument("name")
parser.add_argument("input")
parser.add_argument("output")

if __name__ == "__main__":

    args = parser.parse_args()
    phase_time = {}
    record_phase = ""
    record_time = None
    beginning = None
    with open(args.input, "r") as f:
        raw = f.readlines()
        for line in raw:
            line = line.strip()
            elems = line.split(" ")
            
            time = datetime.fromisoformat(elems[0])
            if elems[-1] == "Start":
                beginning = time
            elif elems[-1] == "Complete":
                phase_time[phase] = (time - record_time).total_seconds()*1000
                phase_time["Other"] = (time-beginning).total_seconds()*1000 - \
                    sum([val for _, val in phase_time.items()])
            else:
                phase_idx = elems.index("Phase")
                phase = " ".join(elems[phase_idx:])
                if record_time is None:
                    record_time = time
                    record_phase = phase
                else:
                    phase_time[record_phase] = (time - record_time).total_seconds()*1000
                    record_time = time
                    record_phase = phase
    with open(args.output, "w") as f:
        json.dump({
            "name": args.name,
            "phase_time": phase_time
        }, f)