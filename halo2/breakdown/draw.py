import argparse
import json
import matplotlib.pyplot as plt
import numpy as np
import os


def plot_phases_log(data, name):
    tasks = [val["name"] for val in data]
    phases = list(data[0]["phase_ops"][name].keys())
    times = {}
    for phase in phases:
        if phase == "Other":
            times[phase] = np.array([
                val["phase_ops"][name][phase] for val in data
            ])
        else:
            times[phase] = np.array([
                val["phase_ops"][name][phase]["total"] for val in data
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
    ax.legend(loc='center right', bbox_to_anchor=(1.9, 0.5))
    
    ax.set_xlabel('Benchmark Task')
    ax.set_ylabel('Time in log scale(ms)')
    ax.set_title('Halo2 {} Phases Breakdown'.format(name.capitalize()))
    plt.savefig('halo2_{}_phase_breakdown_log.png'.format(name))

def plot_phases(data, name):
    tasks = [val["name"] for val in data]
    phases = list(data[0]["phase_ops"][name].keys())
    times = {}
    for phase in phases:
        if phase == "Other":
            times[phase] = np.array([
                val["phase_ops"][phase][name] for val in data
            ])
        else:
            times[phase] = np.array([
                val["phase_ops"][phase][name]["total"] for val in data
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
    ax.legend(loc='center right', bbox_to_anchor=(1.9, 0.5))

    ax.set_xlabel('Benchmark Task')
    ax.set_ylabel('Time(ms)')
    ax.set_title('Halo2 {} Phases Breakdown'.format(name.capitalize()))
    plt.savefig('halo2_{}_phase_breakdown.png'.format(name))



def plot_ops_pie(data, name):
    
    labels = ["FFT", "IFFT", "MSM", "OTHER"]
    percentage = [
        sum([val["entire_ops"][name]["fft"]/val["entire_ops"][name]["total"] for val in data])/len(data),
        sum([val["entire_ops"][name]["ifft"]/val["entire_ops"][name]["total"] for val in data])/len(data),
        sum([val["entire_ops"][name]["msm"]/val["entire_ops"][name]["total"] for val in data])/len(data),
        sum([val["entire_ops"][name]["other"]/val["entire_ops"][name]["total"] for val in data])/len(data),
    ]
    plt.rcParams["figure.figsize"] = plt.rcParamsDefault["figure.figsize"]
    fig, ax = plt.subplots()
    ax.pie(percentage, labels=labels, autopct='%1.1f%%')
    ax.set_title('Average Halo2 {} Breakdown over Tasks'.format(name.capitalize()))
    plt.savefig('halo2_{}_breakdown.png'.format(name))

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
    
    plot_phases_log(breakdowns, "prove")
    plot_phases(breakdowns, "prove")
    plot_ops_pie(breakdowns, "prove")
    