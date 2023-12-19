import numpy as np
import tensorflow as tf
import msgpack
import os
from io import BytesIO
import argparse

def load_tflite_model(model_path):
    # Load TFLite model
    interpreter = tf.lite.Interpreter(model_path=model_path, experimental_preserve_all_tensors=True)
    interpreter.allocate_tensors()
    # print(interpreter._get_ops_details())
    return interpreter

def load_input_data(npy_path):
    input_data = np.load(npy_path)
    input_data = np.reshape(input_data, (28, 28, 1))
    return input_data.astype(np.float32)

def run_inference(interpreter, input_data):
    # Set the input tensor values
    input_tensor_index = interpreter.get_input_details()[0]['index']
    interpreter.tensor(input_tensor_index)()[0] = input_data

    # Run inference
    interpreter.invoke()

    # Get the output tensor values
    # output_tensor_index = interpreter.get_output_details()[0]['index']
    # print(output_tensor_index)
    # output_data = interpreter.tensor(output_tensor_index)()[0]

    output_data = interpreter.get_tensor(38)
    print(output_data)
    return output_data

def save_output_to_msgpack(output_data, output_msgpack_path):
    # Save output data to msgpack file
    with open(output_msgpack_path, 'wb') as file:
        file.write(msgpack.packb(output_data.tolist()))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--model', type=str, required=True)
    parser.add_argument('--inputs', type=str, required=True)
    parser.add_argument('--outputs', type=str, required=True)
    args = parser.parse_args()

    model_path = args.model
    input_npy_dir = args.inputs
    output_msgpack_dir = args.outputs

    interpreter = load_tflite_model(model_path)
    files = [entry for entry in os.listdir(input_npy_dir) if os.path.isfile(os.path.join(input_npy_dir, entry))]

    for file in files:
        input_data = load_input_data(os.path.join(input_npy_dir, file))
        output_data = run_inference(interpreter, input_data)

        print("Inference Output:")
        print(output_data)

        output_file = file[:-len("npy")] + "msgpack"
        output_msgpack_path = os.path.join(output_msgpack_dir, output_file)
        save_output_to_msgpack(output_data, output_msgpack_path)
        print(f"Output saved to {output_msgpack_path}")

if __name__ == "__main__":
    main()