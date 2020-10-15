from matplotlib import pyplot
data_txt = open('durations_ZR27_pruning.txt').read()
data = list(map(lambda x: list(map(float, x.split(' '))), data_txt.split('\n')))
x = list(map(lambda x: x[3], data))
pyplot.hist(x, bins=30)
pyplot.xlabel('p_visits')
pyplot.ylabel('počet')
pyplot.title(r'Histogram sady ZR27 algoritmem prořezávání')
pyplot.show()
