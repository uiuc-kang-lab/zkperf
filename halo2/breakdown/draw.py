import argparse
import json
import math
import matplotlib.pyplot as plt
import numpy as np
import os


def plot_prover_phases(data):
    tasks = [val["name"] for val in data]
    phases = list(data[0]["phase_ops"].keys())
    times = {}
    for phase in phases:
        if phase == "Other":
            times[phase] = np.array([
                val["phase_ops"][phase] for val in data
            ])
        else:
            times[phase] = np.array([
                val["phase_ops"][phase]["total"] for val in data
            ])
    plt.rcParams["figure.figsize"] = (15,4)
    fig, ax = plt.subplots()
    bottom = np.zeros(len(tasks))
    bar_width = 0.35
    for category, time in times.items():
        time = [math.log(ti) for ti in time]
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time
    pos = ax.get_position()
    ax.set_position([pos.x0, pos.y0, pos.width * 0.5, pos.height])
    ax.legend(loc='center right', bbox_to_anchor=(1.9, 0.5))
    
    # plt.xlabel('Benchmark Task')
    # plt.ylabel('Time in log scale(ms)')
    # plt.title('Halo2 Phases Breakdown')
    ax.set_xlabel('Benchmark Task')
    ax.set_ylabel('Time in log scale(ms)')
    ax.set_title('Halo2 Prover Phases Breakdown')
    plt.savefig('halo2_prover_phase_breakdown.png')

def plot_ops_pie(data):
    
    labels = ["FFT", "IFFT", "MSM", "OTHER"]
    percentage = [
        sum([val["entire_ops"]["prove"]["fft"]/val["entire_ops"]["prove"]["total"] for val in data])/len(data),
        sum([val["entire_ops"]["prove"]["ifft"]/val["entire_ops"]["prove"]["total"] for val in data])/len(data),
        sum([val["entire_ops"]["prove"]["msm"]/val["entire_ops"]["prove"]["total"] for val in data])/len(data),
        sum([val["entire_ops"]["prove"]["other"]/val["entire_ops"]["prove"]["total"] for val in data])/len(data),
    ]
    plt.rcParams["figure.figsize"] = plt.rcParamsDefault["figure.figsize"]
    fig, ax = plt.subplots()
    ax.pie(percentage, labels=labels, autopct='%1.1f%%')
    ax.set_title('Average Halo2 Prover Breakdown over Tasks')
    plt.savefig('halo2_prover_breakdown.png')

parser = argparse.ArgumentParser()
parser.add_argument("dir")

if __name__ == "__main__":
    args = parser.parse_args()
    path = args.dir
    breakdown_files = []
    for file in os.listdir(path):
        if file.endswith(".json"):
            breakdown_files.append(file)
    
    breakdowns = []
    for file in breakdown_files:
        with open(os.path.join(path, file), "r") as f:
            breakdowns.append(json.load(f))
    
    plot_prover_phases(breakdowns)
    plot_ops_pie(breakdowns)
    