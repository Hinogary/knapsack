from subprocess import Popen, PIPE
import os
from concurrent.futures import ThreadPoolExecutor

from time import time

def time_it(task, alg):
    args = ["./target/release/knapsack", "../{}_inst.dat".format(task), "--" + alg]
    process = Popen(args, stdout=PIPE, stderr=PIPE)
    (output, err) = process.communicate()
    exit_code = process.wait()
    output = output.decode("utf8")
    if err:
        print(" ".join(args))
        print(err.decode("utf8"))
        raise Exception()
    return list(map(float, output.split('\n')[-2].split(' ')))

def main(cases=['NK','ZKC','ZKW'], enable_naive=lambda x: x <= 25, max_workers=4):
    start = time()
    data = {}
    dir = os.listdir('..')
    executor = ThreadPoolExecutor(max_workers=max_workers)
    futures = []
    for t in cases:
        instance = data.get(t, {})
        data[t] = instance
        for file in dir:
            if file[:len(t)] == t and file[-9:] == "_inst.dat":
                l = file[len(t):-9]

                instance[l] = ach = instance.get(l,{})
                for alg in ["pruning", "dynamic-cost", "dynamic-weight"]:
                    if not enable_naive(l) and alg=="naive":
                        continue
                    f = executor.submit(fn, t, l, alg, ach)
                    futures.append(f)
    for f in futures:
        f.result()

    end = time()
    return data, end-start

def fn(t, l, alg, ach):
    tests = 10 if alg != "naive" else 1
    values = []
    for i in range(tests):
        if sum(map(lambda x: x[1], values)) > 0.1:
            tests = i
            break
        values.append(time_it( t + str(l), alg ))
    avg = sum(map(lambda x: x[1], values)) / tests
    max = sum(map(lambda x: x[0], values)) / tests
    print("{} {} {} {} {}".format(t, l, alg, max, avg))
    ach[alg] =  {"avg": avg, "max": max}