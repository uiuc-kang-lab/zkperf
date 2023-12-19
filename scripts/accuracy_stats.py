import msgpack
import numpy as np
import os
import sys

def load_msgpack_file(file_path):
    with open(file_path, 'rb') as file:
        data = msgpack.unpack(file)
    return data

def accuracy(data, tag):
    correct = 0.
    for k in data:
        output = data[k][tag]
        label = data[k]["label"]
        pred = np.argmax(output)
        if label == pred:
            correct += 1
    return correct / len(data)

def error(data, tag):
    correct = 0.
    err = np.zeros(10)
    max_err = np.zeros(10)
    percent_err = np.zeros(10)
    for k in data:
        circuit_output = data[k][tag]
        model_output = data[k]["model"]
        err += circuit_output - model_output
        abs_err = np.abs(circuit_output - model_output)
        max_err = np.maximum(max_err, abs_err)
        percent_err += (circuit_output - model_output) / model_output
    return (err / len(data), max_err, percent_err / len(data) * 100)


def collect_data(zkml_dir, model_dir):
    files = [entry for entry in os.listdir(model_dir) if os.path.isfile(os.path.join(model_dir, entry))]

    data = {}
    for file in files:
        zkml_output = load_msgpack_file(os.path.join(zkml_dir, file))
        zkml_output = np.array(zkml_output) / 512
        model_output = load_msgpack_file(os.path.join(model_dir, file))
        model_output = np.array(model_output).flatten()

        label, _ = os.path.splitext(file)
        item = label.split('_')[0]
        label = int(label.split('_')[1])
        outputs = {"zkml": zkml_output, "model": model_output, "label": label}
        data[item] = outputs
    return data

def main():
    zkml_dir = sys.argv[1]
    model_dir = sys.argv[2]
    # Specify paths to the msgpack files

    data = collect_data(zkml_dir, model_dir)

    zkml_err, zkml_max_err, zkml_percent_err = error(data, "zkml")
    print("zkml avg error: ", np.mean(zkml_err))
    print("zkml avg percent error: {}%".format(np.mean(zkml_percent_err)))
    print("zkml max error: ", np.max(zkml_max_err))
    print("model accuracy: ", accuracy(data, "model"))
    print("zkml accuracy: ", accuracy(data, "zkml"))

    # % accuracy
    # error in values
    # % error in values a - e / e

if __name__ == "__main__":
    main()