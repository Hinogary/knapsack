from matplotlib import pyplot as plt
import json


def make_graph(data, xlabel="velikost"):

    colors = {
        "naive": "red",
        "pruning": "purple",
        "dynamic-cost": "green",
        "dynamic-weight": "blue",
    }

    line_styles = {
        "max": "dotted",
        "avg": "solid"
    }

    xd = [(x, y) for x in ["avg", "max"] for y in ["pruning", "naive", "dynamic-cost", "dynamic-weight"]]

    for typ in data:
        print(data)
        ldata = data[typ]
        ddata = [ [(ldata[y][x[1]][x[0]], float(y)) for y in ldata if x[1] in ldata[y]] for x in xd]
        for d in ddata:
            d.sort(key=lambda x: x[1])
        xdata = [[x[0] for x in d] for d in ddata]
        ydata = [[x[1] for x in d] for d in ddata]
        fig = plt.figure(figsize=(12,8))
        ax = fig.add_subplot(1, 1, 1)
        #plt.margins(0,0)
        plt.subplots_adjust(top = 0.95, bottom = 0.05, right = 0.99, left = 0.07,
                hspace = 0, wspace = 0)
        plt.yscale("log")
        plt.ylabel("čas [s]")
        plt.xlabel(xlabel)
        ax.grid(True, linestyle='-.')
        plt.title("Sada "+typ)
        for x, y, t in zip(xdata, ydata, xd):
            ax.plot(y, x, linestyle=line_styles[t[0]], color=colors[t[1]], marker=".")
        plt.savefig(typ+".png")

if __name__=='__main__':
    data = json.loads(open("data.json").read())
    make_graph(data)