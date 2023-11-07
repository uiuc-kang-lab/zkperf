import csv
import sys
import json

def get_kv(line):
  key = ' '.join(line.split("to ")[1:]).replace('\n', '')
  value = line.split("|")[-1].split()[0]
  if value.endswith('s'):
    value = value[:-1]
  value = float(value)
  return (key, value)

def get_csv_row(name, time, stack_len, max_depth):
  cur_depth = stack_len - 2
  return [""] * cur_depth + [name] + [time]

def get_breakdown(name, input_file, output_json, output_csv):
  with open(input_file, 'r') as in_file, open(output_json, 'w') as out_json, open(output_csv, 'w') as out_csv:
    csv_writer = csv.writer(out_csv, delimiter='\t')
    main_data = { "name": name }
    csv_writer.writerow([name])
    dict_stack = [main_data]
    time_stack = []
    prev_time = 0.
    prev_pipe_count = None
    max_depth = 2

    # Iterate through each line in the input file
    for line in in_file:
        # Check if the line contains "timing"
        if "timing" in line:
            pipe_count = line.count('|')

            # Find the index of the first occurrence of ']'
            index = line.index(']') + 1
            line_data = line[index:].strip()

            (name, time) = get_kv(line_data)
            if pipe_count == 0:
              dict_stack = [dict_stack[0]]
              current_dict = dict_stack[-1]
              current_dict[name] = {"time": time}
              dict_stack.append(current_dict[name])
              prev_time = time

              csv_row = get_csv_row(name, time, len(dict_stack), max_depth)
              csv_writer.writerow(csv_row)
            elif pipe_count > prev_pipe_count:
              current_dict = dict_stack[-1]
              current_dict[name + " breakdown"] = {}
              dict_stack.append(current_dict[name + " breakdown"])
              current_dict = dict_stack[-1]
              current_dict[name] = time

              time_stack.append(prev_time)
              top_time = time_stack[-1]
              csv_time = str(time) + " ({:.2f}%)".format(time/top_time * 100)
              csv_row = get_csv_row(name, csv_time, len(dict_stack), max_depth)
              csv_writer.writerow(csv_row)
            elif pipe_count == prev_pipe_count:
              current_dict = dict_stack[-1]
              current_dict[name] = time
              prev_time = time

              top_time = time_stack[-1]
              csv_time = str(time) + " ({:.2f}%)".format(time/top_time * 100)
              csv_row = get_csv_row(name, csv_time, len(dict_stack), max_depth)
              csv_writer.writerow(csv_row)
            elif pipe_count < prev_pipe_count:
              dict_stack.pop()
              current_dict = dict_stack[-1]
              current_dict[name] = time

              time_stack.pop()
              top_time = time_stack[-1]
              prev_time = top_time
              csv_time = str(time) + " ({:.2f}%)".format(time/top_time * 100)
              csv_row = get_csv_row(name, csv_time, len(dict_stack), max_depth)
              csv_writer.writerow(csv_row)

            prev_pipe_count = pipe_count

    json.dump(main_data, out_json, indent=4)

# Open the input and output files
# with open(input_file, 'r') as input_file, open(output_file, 'w', newline='') as output_file:
#     # Create a CSV writer
#     csv_writer = csv.writer(output_file)

#     # Iterate through each line in the input file
#     for line in input_file:
#         # Check if the line contains "timing"
#         if "timing" in line:
#             # Extract the part of the line after the ']' character
#             line = line[line.index(']')+1:]
#             # Replace all '|' characters with ','
#             line = line.replace('|', ',', 1)  # Replace only the first occurrence of '|'
#             line = line.replace('|', '')  # Remove remaining '|' characters
#             # Write the modified line to the CSV file
#             csv_writer.writerow([line.strip()])

def main():
  # Check if the correct number of arguments is provided
  if len(sys.argv) != 5:
    print("Usage: python breakdown.py <name> <input_file> <output_json> <output_csv>")
    sys.exit(1)

  name = sys.argv[1]
  input_file = sys.argv[2]
  output_json = sys.argv[3]
  output_csv = sys.argv[4]

  get_breakdown(name, input_file, output_json, output_csv)

if __name__ == '__main__':
  main()