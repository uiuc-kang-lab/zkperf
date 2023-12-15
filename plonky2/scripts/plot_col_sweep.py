import json
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import numpy as np
import os

def main():
  directory_path = "logs/column_sweep/"

  cols = set()
  for filename in os.listdir(directory_path):
    if filename.endswith(".json"):
        col = int(filename.split('_')[0])
        cols.add(col)

  cols = sorted(list(cols))
  times = {"dlrm": [], "mnist": [], "ecdsa": [], "merkle": [], "mnist_no_lookup": []}

  fig=plt.figure(figsize=(4 * 3+1, 2.2 * 2 + .5))
#   fig, axes = plt.subplots(
#         figsize=(4 * 3, 3.3 * 2), ncols=3, nrows=2,
#         sharex=False
#   )
  fig.tight_layout()

  for task in times:
    min_time = 100000
    min_col = 0
    for c in cols:
        with open(os.path.join(directory_path, str(c)+"_"+task+".json"), 'r') as file:
            data = json.load(file)
            prover_time = data["ProverTime"]
            times[task].append((c, prover_time))
            if min_time > prover_time:
                min_time = prover_time
                min_col = c
    print(task, cols, times[task])
    default_time = next(tup for tup in times[task] if tup[0] == 80)[1]
    speedup = default_time / min_time
    print("{} min time: {}, col: {}. speedup from default: {}".format(task, min_time, min_col, speedup))

  ax1 = fig.add_subplot(2, 3, 1)
  df = pd.DataFrame(data=times['mnist'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=ax1)
  ax1.plot(df['Columns'], df['Proving time (s)'], marker='o')
  ax1.set_ylim(0)
  ax1.set_title('a) plonky2 MNIST')

  ax2 = fig.add_subplot(2, 3, 2)
  df = pd.DataFrame(data=times['ecdsa'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=ax2)
  ax2.plot(df['Columns'], df['Proving time (s)'], marker='o')
  ax2.set_ylim(0)
  ax2.set_title('b) plonky2 ECDSA')
  ax2.set_ylabel('')

  ax3 = fig.add_subplot(2, 3, 3)
  df = pd.DataFrame(data=times['merkle'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=ax3)
  ax3.plot(df['Columns'], df['Proving time (s)'], marker='o')
  ax3.set_ylim(0)
  ax3.set_title('c) plonky2 Merkle Tree')
  ax3.set_ylabel('')

  ax4 = fig.add_subplot(2, 3, 4)
  df = pd.DataFrame(data=times['dlrm'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=ax4, markers=True)
  ax4.plot(df['Columns'], df['Proving time (s)'], marker='o')
  ax4.set_ylim(0)
  ax4.set_title('d) plonky2 DLRM')

  bench = []
  halo_path = "../halo2/zkml/experiments"
  for file in os.listdir(halo_path):
    if file.endswith(".txt"):
      name = file.replace(".txt", "")
      cols = int(name.split("_")[-2])
      with open(os.path.join(halo_path, file)) as f:
        lines = f.readlines()
        for line in lines:
          line = line.strip()
          elems = line.split(": ")
          if elems[0] == "Proving time":
            bench.append((cols, float(elems[1][:-1])))

  ax5 = fig.add_subplot(2, 3, 5)
  df = pd.DataFrame(data=bench, columns=['Columns', 'Proving time (s)'])
  sns.scatterplot(data=df, x='Columns', y='Proving time (s)', marker="o", ax=ax5)
  ax5.set_xlabel("Columns")
  ax5.set_title("e) halo2 zkml DLRM")
  ax5.set_ylabel('')
  plt.subplots_adjust(hspace=0.5)

  plt.savefig('col_sweep.pdf', bbox_inches='tight', pad_inches=0)

  fig, axes = plt.subplots(
        figsize=(1 * 4, 2.2), ncols=1, nrows=1,
        sharex=False
  )
  fig.tight_layout()

  df_lookup = pd.DataFrame(data=times['mnist'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df_lookup)
  axes.plot(df_lookup['Columns'], df_lookup['Proving time (s)'], marker='o', label="with lookups")
  df = pd.DataFrame(data=times['mnist_no_lookup'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df)
  axes.plot(df['Columns'], df['Proving time (s)'], marker='o', label="without lookups")
  axes.set_ylim(0)
  axes.set_title('plonky2 MNIST with/without lookups')
  plt.legend()
  axes.set_ylabel('')

  plt.savefig('mnist_lookups.pdf', bbox_inches='tight', pad_inches=0)

if __name__ == '__main__':
  main()