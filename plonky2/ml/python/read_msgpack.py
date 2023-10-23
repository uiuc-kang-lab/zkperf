import argparse
import msgpack
import json

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input',  type=str, required=True)
    parser.add_argument('--output', type=str, required=True)
    args = parser.parse_args()

    with open(args.input, "rb") as data_file:
        byte_data = data_file.read()

    data_loaded = msgpack.unpackb(byte_data)
    s = json.dumps(data_loaded)
    with open(args.output, "w") as f:
	    print(s, file=f)



if __name__ == '__main__':
  main()
