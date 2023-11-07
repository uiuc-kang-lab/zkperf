import json
import matplotlib.pyplot as plt
import numpy as np

# Load data from the JSON file
def plot_overall(data):
    tasks = []
    wire_commits = []
    arg_polys = []
    quotients = []
    openings = []
    for dict in data:
        tasks.append(dict["name"])
        d = dict["prove"]["prove breakdown"]
        generator_key = [key for key in d if "generators" in key][0]
        wire_commit_total = d[generator_key] + d["compute full witness"] + d["compute wire polynomials"] + d["compute wires commitment"]
        arg_polys_total = d["compute partial products"] + d["commit  partial products, Z's and, if any, lookup polynomials"]
        quotient_total = d["compute quotient polys"] + d["split up quotient polys"] + d["commit  quotient polys"]
        opening_total = d["construct the opening set, including lookups"] + d["split up quotient polys"] + d["commit  quotient polys"]
        wire_commits.append(wire_commit_total)
        arg_polys.append(arg_polys_total)
        quotients.append(quotient_total)
        openings.append(opening_total)

    times = {
        "wire commitments": np.array(wire_commits),
        "argument polynomials": np.array(arg_polys),
        "quotient polynomials": np.array(quotients),
        "opening set + proof": np.array(openings)
    }
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))

    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time

    plt.xlabel('task')
    plt.ylabel('time')
    plt.title('plonky2 overall breakdown')
    plt.legend()

    # Save the plot to an image file
    plt.savefig('overall_breakdown.png')

def plot_wire_commit(data):
    tasks = []
    generators = []
    witnesses = []
    wire_polys = []
    iffts = []
    ffts = []
    ldes = []
    merkles = []
    for dict in data:
        tasks.append(dict["name"])
        d = dict["prove"]["prove breakdown"]
        bd_dict = d["compute wires commitment breakdown"]
        generator_key = [key for key in d if "generators" in key][0]
        generator = d[generator_key]
        witness = d["compute full witness"]
        poly = d["compute wire polynomials"]
        ifft = bd_dict["IFFT"]
        fft = bd_dict["FFT + blinding"]
        lde = bd_dict["transpose LDEs"]
        merkle = bd_dict["build Merkle tree"]
        generators.append(generator)
        witnesses.append(witness)
        wire_polys.append(poly)
        iffts.append(ifft)
        ffts.append(fft)
        ldes.append(lde)
        merkles.append(merkle)

    times = {
        "run generators": np.array(generators),
        "compute full witness": np.array(witnesses),
        "compute wire polynomials": np.array(wire_polys),
        "wire commitment IFFT": np.array(iffts),
        "wire commitment FFT + blinding": np.array(ffts),
        "wire commitment transpose LDEs": np.array(ldes),
        "wire commitment Merkle tree": np.array(merkle),
    }
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))

    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time

    plt.xlabel('task')
    plt.ylabel('time')
    plt.title('plonky2 wire commitment breakdown')
    plt.legend()

    # Save the plot to an image file
    plt.savefig('wire_breakdown.png')

def plot_arg_commit(data):
    tasks = []
    prods = []
    iffts = []
    ffts = []
    ldes = []
    merkles = []
    for dict in data:
        tasks.append(dict["name"])
        d = dict["prove"]["prove breakdown"]
        bd_dict = d["commit  partial products, Z's and, if any, lookup polynomials breakdown"]
        prod = d["compute partial products"]
        ifft = bd_dict["IFFT"]
        fft = bd_dict["FFT + blinding"]
        lde = bd_dict["transpose LDEs"]
        merkle = bd_dict["build Merkle tree"]
        prods.append(prod)
        iffts.append(ifft)
        ffts.append(fft)
        ldes.append(lde)
        merkles.append(merkle)

    times = {
        "compute partial products": np.array(prods),
        "argument commitment IFFT": np.array(iffts),
        "argument commitment FFT + blinding": np.array(ffts),
        "argument commitment transpose LDEs": np.array(ldes),
        "argument commitment Merkle tree": np.array(merkle),
    }
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))

    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time

    plt.xlabel('task')
    plt.ylabel('time')
    plt.title("plonky2 partial products, Z's, lookup polynomials breakdown")
    plt.legend()

    # Save the plot to an image file
    plt.savefig('arg_breakdown.png')

def plot_quotient_commit(data):
    tasks = []
    quots = []
    splits = []
    ffts = []
    ldes = []
    merkles = []
    for dict in data:
        tasks.append(dict["name"])
        d = dict["prove"]["prove breakdown"]
        bd_dict = d["commit  quotient polys breakdown"]
        quot = d["compute quotient polys"]
        split = d["split up quotient polys"]
        fft = bd_dict["FFT + blinding"]
        lde = bd_dict["transpose LDEs"]
        merkle = bd_dict["build Merkle tree"]
        quots.append(quot)
        splits.append(split)
        ffts.append(fft)
        ldes.append(lde)
        merkles.append(merkle)

    times = {
        "compute quotient polys": np.array(quots),
        "split up quotient polys": np.array(splits),
        "quotient commitment FFT + blinding": np.array(ffts),
        "quotient commitment transpose LDEs": np.array(ldes),
        "quotient commitment Merkle tree": np.array(merkle),
    }
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))

    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time

    plt.xlabel('task')
    plt.ylabel('time')
    plt.title("plonky2 quotient polynomials breakdown")
    plt.legend()

    # Save the plot to an image file
    plt.savefig('quotient_breakdown.png')

def plot_openings(data):
    tasks = []
    op_sets = []
    reds = []
    ffts = []
    folds = []
    witnesses = []
    for dict in data:
        tasks.append(dict["name"])
        d = dict["prove"]["prove breakdown"]
        bd_dict = d["compute opening proofs breakdown"]
        op_set = d["construct the opening set, including lookups"]
        reduce_keys = [key for key in bd_dict if "reduce" in key]
        red = bd_dict[reduce_keys[0]] + bd_dict[reduce_keys[1]]
        fft_key = [key for key in bd_dict if "final FFT" in key][0]
        fft = bd_dict[fft_key]
        fold = bd_dict["fold codewords in the commitment phase"]
        witness = bd_dict["find proof-of-work witness"]
        op_sets.append(op_set)
        reds.append(red)
        ffts.append(fft)
        folds.append(fold)
        witnesses.append(witness)

    times = {
        "construct the opening set, including lookups": np.array(op_sets),
        "reduce polynomial batch twice": np.array(reds),
        "perform final FFT": np.array(ffts),
        "fold codewords in the commitment phase": np.array(fold),
        "find proof-of-work witness": np.array(witness),
    }
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))

    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time

    plt.xlabel('task')
    plt.ylabel('time')
    plt.title("plonky2 opening set + proof breakdown")
    plt.legend()

    # Save the plot to an image file
    plt.savefig('opening_breakdown.png')

def main():
  # Check if the correct number of arguments is provided
  with open('breakdowns.json', 'r') as file:
    data = json.load(file)

  plot_overall(data)
  plot_wire_commit(data)
  plot_arg_commit(data)
  plot_quotient_commit(data)
  plot_openings(data)

if __name__ == '__main__':
  main()