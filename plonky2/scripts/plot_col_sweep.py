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
  times = {"dlrm": [], "mnist": [], "ecdsa": [], "merkle": []}

  fig, axes = plt.subplots(
        figsize=(4 * 4, 3.3), ncols=4, nrows=1,
        sharex=False
  )
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
    speedup = (default_time - min_time) / default_time
    print("{} min time: {}, col: {}. speedup from default: {}".format(task, min_time, min_col, speedup))

  df = pd.DataFrame(data=times['dlrm'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=axes[0], markers=True)
  axes[0].plot(df['Columns'], df['Proving time (s)'], marker='o')
  axes[0].set_ylim(0)
  axes[0].set_title('plonky2 DLRM')

  df = pd.DataFrame(data=times['mnist'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=axes[1])
  axes[1].plot(df['Columns'], df['Proving time (s)'], marker='o')
  axes[1].set_ylim(0)
  axes[1].set_title('plonky2 MNIST')
  axes[1].set_ylabel('')

  df = pd.DataFrame(data=times['ecdsa'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=axes[2])
  axes[2].plot(df['Columns'], df['Proving time (s)'], marker='o')
  axes[2].set_ylim(0)
  axes[2].set_title('plonky2 ECDSA')
  axes[2].set_ylabel('')

  df = pd.DataFrame(data=times['merkle'], columns=['Columns', 'Proving time (s)'])
  sns.lineplot(x='Columns', y='Proving time (s)', data=df, ax=axes[3])
  axes[3].plot(df['Columns'], df['Proving time (s)'], marker='o')
  axes[3].set_ylim(0)
  axes[3].set_title('plonky2 Merkle Tree')
  axes[3].set_ylabel('')

  plt.savefig('col_sweep.pdf', bbox_inches='tight', pad_inches=0)

if __name__ == '__main__':
  main()