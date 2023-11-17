import argparse
import json
import math
import matplotlib.pyplot as plt
import numpy as np
import os


def plot_phases_log(data, name):
    tasks = [val["name"] for val in data]
    phases = list(data[0]["phase_time"].keys())
    times = {}
    for phase in phases:
        times[phase] = np.array([
            val["phase_time"][phase] for val in data
        ])

    plt.rcParams["figure.figsize"] = (15,4)
    fig, ax = plt.subplots()
    
    bottom = np.zeros(len(tasks))
    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time
    pos = ax.get_position()
    ax.set_yscale("log")
    ax.set_position([pos.x0, pos.y0, pos.width * 0.5, pos.height])
    ax.legend(loc='center right', bbox_to_anchor=(1.65, 0.5))
    
    ax.set_xlabel('Benchmark Task')
    ax.set_ylabel('Time in log scale(ms)')
    ax.set_title('Circom {} Phases Breakdown'.format(name.capitalize()))
    plt.savefig('circom_{}_phase_breakdown_log.png'.format(name))

def plot_phases(data, name):
    tasks = [val["name"] for val in data]
    phases = list(data[0]["phase_time"].keys())
    times = {}
    for phase in phases:
        times[phase] = np.array([
            val["phase_time"][phase] for val in data
        ])

    plt.rcParams["figure.figsize"] = (15,4)
    fig, ax = plt.subplots()
    
    bottom = np.zeros(len(tasks))
    bar_width = 0.35
    for category, time in times.items():
        p = ax.bar(tasks, time, bar_width, label=category, bottom=bottom)
        bottom += time
    pos = ax.get_position()
    ax.set_position([pos.x0, pos.y0, pos.width * 0.5, pos.height])
    ax.legend(loc='center right', bbox_to_anchor=(1.65, 0.5))
    
    ax.set_xlabel('Benchmark Task')
    ax.set_ylabel('Time(ms)')
    ax.set_title('Circom {} Phases Breakdown'.format(name.capitalize()))
    plt.savefig('circom_{}_phase_breakdown.png'.format(name))


def plot_ops_pie(data, name):
    
    labels = ["FFT/IFFT", "MSM", "OTHER"]
    percentage = [
        sum([val["phase_time"]["Phase 2: Transform A, B, C"]/sum([ti for _, ti in val["phase_time"].items()]) for val in data])/len(data),
        sum([val["phase_time"]["Phase 4: Compute A, B, C, H Commitment"]/sum([ti for _, ti in val["phase_time"].items()]) for val in data])/len(data),
    ]
    percentage.append(1.0 - sum(percentage))
    plt.rcParams["figure.figsize"] = plt.rcParamsDefault["figure.figsize"]
    fig, ax = plt.subplots()
    ax.pie(percentage, labels=labels, autopct='%1.1f%%')
    ax.set_title('Average Circom {} Breakdown over Tasks'.format(name.capitalize()))
    plt.savefig('circom_{}_breakdown.png'.format(name))

parser = argparse.ArgumentParser()
parser.add_argument("dir")

if __name__ == "__main__":
    args = parser.parse_args()
    path = args.dir
    prover_breakdown_files = []
    for file in os.listdir(path):
        if file.endswith(".json") and "prove" in file:
            prover_breakdown_files.append(file)
    
    prover_breakdowns = []
    for file in prover_breakdown_files:
        with open(os.path.join(path, file), "r") as f:
            prover_breakdowns.append(json.load(f))
    
    plot_phases_log(prover_breakdowns, "prove")
    plot_phases(prover_breakdowns, "prove")
    plot_ops_pie(prover_breakdowns, "prove")
    