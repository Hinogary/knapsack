from subprocess import Popen, PIPE

from time import time

import re

def time_it(task, alg, gcd = None):
    args = ["./target/release/knapsack", "../{}_inst.dat".format(task), "../{}_sol.dat".format(task), "--" + alg, str(gcd)]
    if alg != "ftpas":
        args = args[:-1]
    process = Popen(args, stdout=PIPE, stderr=PIPE)
    (output, err) = process.communicate()
    exit_code = process.wait()
    output = output.decode("utf8")
    if err:
        print(" ".join(args))
        print(err.decode("utf8"))
        raise Exception()
    values = list(map(int, re.findall("absolute: ([0-9]+)", output)))
    err_ratios = list(map(float, re.findall("practical ratio: ([0-9.]+)", output)))
    errors = sum(map(lambda x: 1 if x > 0 else 0, values))
    avg_error = sum(values) / max(errors, 1)
    max_error = max(values)
    avg_relative_error = sum(err_ratios) / max(errors, 1)
    max_time, avg_time = list(map(float, output.split('\n')[-2].split(' ')))
    return errors, avg_error, max_error, max_time, avg_time, avg_relative_error, max(err_ratios or [0.0])

def main():
    pass