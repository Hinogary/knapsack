use itertools::Itertools;

use structopt::StructOpt;

use lazy_format::lazy_format;

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::mem::size_of;
use std::str::FromStr;
use std::time::{Duration, Instant};

use gcd::Gcd;

fn main() {
    let opts = Opts::from_args();

    let solver = Solver::from_opts(&opts);

    let input =
        fs::read_to_string(opts.input_task.clone()).expect("Failed to read input_task file!");

    let ref_solutions = if let Some(ref sol) = opts.solution {
        Some(
            fs::read_to_string(sol)
                .expect("Failed to read reference soulions file!")
                .lines()
                .map(parse_solution_line)
                .fold(HashMap::new(), |mut map, value| {
                    map.insert(value.id, value);
                    map
                }),
        )
    } else {
        None
    };

    let mut time = Duration::new(0, 0);

    let durations = input
        .lines()
        .map(parse_problem_line)
        .map(|problem| {
            let start = Instant::now();
            (
                match (
                    solver,
                    problem.min_cost.is_none() || opts.force_construction,
                ) {
                    (Naive, true) => construction_naive(&problem),
                    (Pruning, true) => construction_pruning(&problem),
                    (DynamicWeight, true) => construction_dynamic_weight(&problem),
                    (DynamicCost, true) => construction_dynamic_cost(&problem),
                    (Redux, true) => construction_redux(&problem),
                    (Greedy, true) => construction_greedy(&problem),
                    #[allow(unreachable_patterns)]
                    _ => unimplemented!(),
                },
                start.elapsed(),
                problem,
                solver.is_exact(),
            )
        })
        .map(|(solution, elapsed, problem, is_exact)| {
            let mut output = String::new();
            time += elapsed;
            output += format!("{} {} {}", solution.id, solution.size, solution.cost).as_str();
            if let Some(items) = &solution.items {
                output += items
                    .iter()
                    .map(|&i| if i { " 1" } else { " 0" })
                    .join("")
                    .as_str();
            }
            let mut additional_info = "".to_string();
            if ref_solutions.is_some() && (problem.min_cost.is_none() || opts.force_construction) {
                let reference = ref_solutions.as_ref().unwrap().get(&solution.id).unwrap();
                if is_exact {
                    if *reference != solution
                        && reference.cost == solution.cost
                        && reference.size == solution.size
                    {
                        println!("Same cost, but different solution!");
                    } else {
                        assert_eq!(*reference, solution);
                    }
                } else {
                    additional_info = format!(
                        " ratio of ref solution: {}",
                        solution.cost as f32 / reference.cost as f32
                    );
                }
            }
            println!("time: {:?} {}\n{}", elapsed, additional_info, output);

            (solution.id, elapsed)
        })
        .collect::<Vec<_>>();

    // output file, which is easier to parse for nice graphs
    if let Some(path) = opts.save_durations {
        fs::write(
            path,
            durations
                .iter()
                .map(|(id, elapsed)| lazy_format!("{} {}", id, elapsed.as_secs_f64()))
                .join("\n"),
        )
        .expect("Failed to save durations!");
    }

    println!(
        "Maximum time: {:?} Average time: {:?}",
        durations
            .iter()
            .map(|(_, elapsed)| *elapsed)
            .max()
            .unwrap_or(time),
        durations
            .iter()
            .map(|(_, elapsed)| *elapsed)
            .fold(Duration::new(0, 0), |acc, x| acc + x)
            / (durations.len() as u32)
    );

    println!("Total time: {:?}", time);
}

#[derive(Debug, Copy, Clone)]
enum Solver {
    Naive,
    Pruning,
    DynamicWeight,
    DynamicCost,
    Greedy,
    Redux,
    FTPAS,
}
use Solver::*;

impl Solver {
    fn is_exact(self) -> bool {
        match self {
            Naive | Pruning | DynamicWeight | DynamicCost => true,
            Greedy | Redux | FTPAS => false,
        }
    }

    fn from_opts(opts: &Opts) -> Solver {
        match (
            opts.naive,
            opts.pruning,
            opts.dynamic_weight,
            opts.dynamic_cost,
            opts.greedy,
            opts.redux,
            opts.ftpas.is_some(),
        ) {
            (true, false, false, false, false, false, false) => Naive,
            (false, true, false, false, false, false, false) => Pruning,
            (false, false, true, false, false, false, false) => DynamicWeight,
            (false, false, false, true, false, false, false) => DynamicCost,
            (false, false, false, false, true, false, false) => Greedy,
            (false, false, false, false, false, true, false) => Redux,
            (false, false, false, false, false, false, true) => FTPAS,
            (false, false, false, false, false, false, false) => panic!("Not any solver selected!"),
            _ => panic!("Too many solvers selected!"),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsack", author = "Martin Quarda <martin@quarda.cz>")]
struct Opts {
    input_task: String,
    solution: Option<String>,
    #[structopt(short, long)]
    naive: bool,
    #[structopt(short, long)]
    pruning: bool,
    #[structopt(long)]
    dynamic_weight: bool,
    #[structopt(long)]
    dynamic_cost: bool,
    #[structopt(short, long)]
    greedy: bool,
    #[structopt(short, long)]
    redux: bool,
    #[structopt(short, long)]
    ftpas: Option<f32>,
    #[structopt(long)]
    save_durations: Option<String>,
    #[structopt(long)]
    force_construction: bool,
}

#[derive(Debug, Clone, Copy)]
struct Item {
    weight: u32,
    cost: u32,
}

#[derive(Debug, Clone)]
struct Problem {
    id: u32,
    max_weight: u32,
    size: usize,
    // switch between decision and construction problem
    min_cost: Option<u32>,
    items: Vec<Item>,
}

// augumented problems used in recursion descent
#[derive(Debug, Clone)]
struct ProblemWithSol {
    p: Problem,
    best_solution: Vec<bool>,
}

#[derive(Debug, Clone)]
struct ProblemWithRatios {
    p: Problem,
    ratios: Vec<f32>,
    best_solution: Vec<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Solution {
    id: u32,
    size: usize,
    cost: u32,
    items: Option<Vec<bool>>,
}

impl Solution {
    fn empty(id: u32, size: usize) -> Solution {
        Solution {
            id,
            size,
            cost: 0,
            items: Some(vec![false; size]),
        }
    }

    fn none(id: u32, size: usize) -> Solution {
        Solution {
            id,
            size,
            cost: 0,
            items: None,
        }
    }
}

// returns (new items, cost/weight ratios descending, mapping [new array] -> [original array])
fn sort_by_cost_weight_ratio(
    items: &Vec<Item>,
    max_weight: u32,
) -> (Vec<Item>, Vec<f32>, Vec<usize>) {
    let len = items.len();
    let mut vec = items
        .iter()
        .enumerate()
        .map(|(index, item)| (*item, item.cost as f32 / item.weight as f32, index))
        .filter(|(item, _, _)| item.weight <= max_weight)
        .collect::<Vec<_>>();
    vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    vec.into_iter().fold(
        (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        ),
        |(mut items, mut ratios, mut mappings), (item, ratio, mapping)| {
            items.push(item);
            ratios.push(ratio);
            mappings.push(mapping);
            (items, ratios, mappings)
        },
    )
}

// Calculates maximum possible cost ... takes sorted items by cost/weight ratios
fn max_cost(items: &Vec<Item>, max_weight: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};

    #[allow(deprecated)] // fold_while no longer deprecated in master
    items
        .iter()
        .fold_while((0, 0), |(weight, cost), x| {
            if weight + x.weight < max_weight {
                Continue((weight + x.weight, cost + x.cost))
            } else {
                Done((0, cost + x.cost * x.weight / (max_weight - weight)))
            }
        })
        .into_inner()
        .1
}

fn next_parse<'a, T, K>(iter: &mut T) -> K
where
    T: Iterator<Item = &'a str>,
    K: FromStr,
    <K as std::str::FromStr>::Err: std::fmt::Debug,
{
    iter.next().unwrap().parse().unwrap()
}

fn parse_problem_line(line: &str) -> Problem {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id: i32 = next_parse(&mut iter);
    let size = next_parse(&mut iter);
    let max_weight = next_parse(&mut iter);
    let min_cost = match () {
        () if id < 0 => Some(next_parse(&mut iter)),
        () if id > 0 => None,
        _ => panic!("zero id not permitted"),
    };
    let items = (0..size)
        .map(|_| Item {
            weight: next_parse(&mut iter),
            cost: next_parse(&mut iter),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        iter.next(),
        None,
        "Line was not exhausted, wrong problem line!"
    );
    Problem {
        id: id.abs() as u32,
        max_weight,
        size,
        min_cost,
        items,
    }
}

fn parse_solution_line(line: &str) -> Solution {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id = next_parse(&mut iter);
    let size = next_parse(&mut iter);
    let cost = next_parse(&mut iter);
    let items = Some(
        (0..size)
            .map(|_| match iter.next().unwrap() {
                "1" => true,
                "0" => false,
                _ => panic!("Reference solution is not in (0, 1)!"),
            })
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        iter.next(),
        None,
        "Line was not exhausted, wrong solution line!"
    );
    Solution {
        id,
        size,
        cost,
        items,
    }
}

fn construction_naive(problem: &Problem) -> Solution {
    fn rec_fn(
        problem: &mut ProblemWithSol,
        cost: u32,
        weight: u32,
        index: usize,
        best_cost: u32,
    ) -> u32 /* best_cost */ {
        if index < problem.p.size {
            let best_with_item = rec_fn(
                problem,
                cost + problem.p.items[index].cost,
                weight + problem.p.items[index].weight,
                index + 1,
                best_cost,
            );
            let best_without_item = rec_fn(
                problem,
                cost,
                weight,
                index + 1,
                best_cost.max(best_with_item),
            );
            match best_cost.max(best_with_item).max(best_without_item) {
                x if best_cost == x => best_cost, //best_cost didnt change, so it was not in this recursion (at best was same, which we ignore)
                x if best_with_item == x => {
                    problem.best_solution[index] = true;
                    best_with_item
                }
                x if best_without_item == x => {
                    problem.best_solution[index] = false;
                    best_without_item
                }
                _ => unreachable!(),
            }
        } else {
            if weight <= problem.p.max_weight {
                best_cost.max(cost)
            } else {
                best_cost
            }
        }
    }

    let mut aug_problem = ProblemWithSol {
        p: problem.clone(),
        best_solution: vec![false; problem.size],
    };
    let cost = rec_fn(&mut aug_problem, 0, 0, 0, 0);
    Solution {
        id: problem.id,
        size: problem.size,
        cost,
        items: Some(aug_problem.best_solution),
    }
}

fn construction_pruning(problem: &Problem) -> Solution {
    fn rec_fn(
        problem: &mut ProblemWithRatios,
        cost: u32,
        weight: u32,
        index: usize,
        best_cost: u32,
    ) -> u32 {
        if index < problem.p.items.len() {
            if (problem.p.max_weight - weight) as f32 * problem.ratios[index] + (cost as f32)
                    < best_cost as f32
            {
                return best_cost;
            }
            let new_weight = weight + problem.p.items[index].weight;
            let best_with_item = if new_weight <= problem.p.max_weight {
                rec_fn(
                    problem,
                    cost + problem.p.items[index].cost,
                    weight + problem.p.items[index].weight,
                    index + 1,
                    best_cost,
                )
            } else {
                best_cost
            };
            let best_without_item = rec_fn(
                problem,
                cost,
                weight,
                index + 1,
                best_cost.max(best_with_item),
            );
            match best_cost.max(best_with_item).max(best_without_item) {
                x if best_cost == x => best_cost, //best_cost didnt change, so it was not in this recursion (at best was same, which we ignore)
                x if best_with_item == x => {
                    problem.best_solution[index] = true;
                    best_with_item
                }
                x if best_without_item == x => {
                    problem.best_solution[index] = false;
                    best_without_item
                }
                _ => unreachable!(),
            }
        } else {
            // I check weight before even going deeper, so it's not important here
            best_cost.max(cost)
        }
    }

    let (items, ratios, mappings) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

    // items are already filtered by weight, so u32::MAX is fine
    let best_item = best_valued_item_fit(&items, std::u32::MAX);

    let mut aug_problem = ProblemWithRatios {
        best_solution: (0..items.len()).map(|i| i==best_item.1).collect(),
        p: Problem { items, ..*problem },
        ratios,
    };

    let cost = rec_fn(&mut aug_problem, 0, 0, 0, best_item.0);

    Solution {
        id: problem.id,
        size: problem.size,
        cost,
        items: Some(aug_problem.best_solution.into_iter().enumerate().fold(
            vec![false; problem.size],
            |mut acc, (i, x)| {
                if let Some(&mapping) = mappings.get(i) {
                    acc[mapping] = x;
                }
                acc
            },
        )),
    }
}

fn construction_dynamic_weight(problem: &Problem) -> Solution {
    let (mut items, _ratios, mut mapping) =
        sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

    if items.len() == 0 {
        items.push(Item { weight: 1, cost: 0 });
        mapping.push(0);
    }

    let gcd = items
        .iter()
        .fold(items[0].weight, |acc, x| acc.gcd(x.weight)) as usize;
    let size = problem.max_weight as usize / gcd + 1;
    let ilen = items.len();

    let mut table_raw: Vec<Option<(u32, bool)>> = vec![None; size * (ilen + 1)];
    let mut table_base = table_raw
        .as_mut_slice()
        .chunks_mut(size)
        .collect::<Vec<_>>();

    //tabulka je převrácená oproti přednáškám .. řešení je v [0][0]
    //první index určuje přidané předměty .. např 0 => jsou přidané věci 0..ilen
    //druhý index určuje zbývající kapacitu a nevyužitá kapacita je "nahoře"

    let table = table_base.as_mut_slice();

    // last row is having zero cost
    for x in table.last_mut().unwrap().iter_mut() {
        *x = Some((0, false));
    }

    // DFS průchod

    let mut stack = Vec::with_capacity(ilen);

    stack.push((0usize, 0u32));
    while !stack.is_empty() {
        let (item, weight) = stack.last().unwrap();
        let with_item = (item + 1, weight + items[*item].weight);
        let without_item = (item + 1, *weight);
        let mut me_cell = None;
        if with_item.1 <= problem.max_weight {
            if let Some(cell) = table[with_item.0][with_item.1 as usize / gcd] {
                me_cell = Some((cell.0 + items[*item].cost, true));
            } else {
                stack.push(with_item);
                continue;
            }
        }
        if let Some(cell) = table[without_item.0][without_item.1 as usize / gcd] {
            let cost_with_item = me_cell.map(|x| x.0).unwrap_or(0);
            if cell.0 >= cost_with_item {
                me_cell = Some((cell.0, false));
            }
        } else {
            stack.push(without_item);
            continue;
        }
        table[*item][*weight as usize / gcd] = me_cell;
        stack.pop();
    }
    Solution {
        id: problem.id,
        size: problem.size,
        cost: table[0][0].unwrap().0,
        items: Some(
            table
                .iter()
                .take(ilen) //skip last
                .fold(
                    (0, 0u32, vec![false; problem.items.len()]),
                    |(i, w, mut vec), x| {
                        let added = x[w as usize / gcd].unwrap().1;
                        vec[mapping[i]] = added;
                        (i + 1, if added { w + items[i].weight } else { w }, vec)
                    },
                )
                .2,
        ),
    }
}

fn construction_dynamic_cost(problem: &Problem) -> Solution {
    let (mut items, mut ratios, mut mapping) =
        sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
    ratios.push(0.0);

    if items.len() == 0 {
        items.push(Item {
            weight: std::u32::MAX,
            cost: 0,
        });
        mapping.push(0);
    }

    let gcd = items.iter().fold(items[0].cost, |acc, x| acc.gcd(x.cost)) as usize;
    let ilen = items.len();

    // if we take first k items and part of the first item, which do not fit, we get maximal possible cost
    #[allow(deprecated)] // fold_while is not anymore deprecated in master
    let max_cost = max_cost(&items, problem.max_weight);

    let size = max_cost as usize / gcd.max(1) + 1;

    if max_cost == 0 {
        return Solution::empty(problem.id, problem.size);
    }

    if size * (ilen + 1) * size_of::<Option<u32>>() > 8_000_000_000 {
        println!("Needed allocations are over 8 GB!");
        return Solution::none(problem.id, problem.size);
    }

    let mut table_raw: Vec<Option<u32>> = vec![None; size * (ilen + 1)];
    let mut table_base = table_raw
        .as_mut_slice()
        .chunks_mut(size)
        .collect::<Vec<_>>();

    // první index určuje přidané věci ... jsou přidané věci 0..index
    // druhý index určuje dosáhnutou hodnotu
    // hodnota určuje využitou váhu, tu se snažím minimalizovat

    let table = table_base.as_mut_slice();

    table[0][0] = Some(0);

    // BFS průchod, VecDeque je ring buffer
    // indexy v queue jsou seřazené podle indexů itemů, ale ne nutně v pořadí podle ceny

    let mut queue = VecDeque::with_capacity(size);

    queue.push_back((0, 0));

    // best_cost je nejlepší cena dosáhnuta za 0..(index aktuálního předmětu)
    let mut best_cost = construction_redux(problem).cost;

    while !queue.is_empty() {
        let (item, cost) = queue.pop_front().unwrap();
        if item >= ilen {
            continue;
        }
        let with_item = (item + 1, cost + items[item].cost);
        let without_item = (item + 1, cost);
        let weight = table[item][cost as usize / gcd].unwrap();
        let new_weight = weight + items[item].weight;
        if new_weight <= problem.max_weight {
            best_cost = best_cost.max(with_item.1);
            if let Some(value) = table[with_item.0][with_item.1 as usize / gcd] {
                if value > new_weight {
                    table[with_item.0][with_item.1 as usize / gcd] = Some(new_weight);
                    // it is already in queue, no need to push again
                }
            } else {
                table[with_item.0][with_item.1 as usize / gcd] = Some(new_weight);
                queue.push_back(with_item);
            }
        }

        // pruning same as in recursive pruning
        if (problem.max_weight - weight) as f32 * ratios[without_item.0] + (cost as f32)
                < best_cost as f32
        {
            continue;
        }

        if let Some(value) = table[without_item.0][without_item.1 as usize / gcd] {
            if value > weight {
                table[without_item.0][without_item.1 as usize / gcd] = Some(weight);
                // it is already in queue, no need to push again
            }
        } else {
            table[without_item.0][without_item.1 as usize / gcd] = Some(weight);
            queue.push_back(without_item);
        }
    }

    let best_cost = (table
        .last()
        .unwrap()
        .iter()
        .enumerate()
        .rev()
        .find(|(_, x)| x.is_some())
        .unwrap()
        .0
        * gcd) as u32;

    let best_solution = table
        .iter()
        .rev()
        .skip(1)
        .fold(
            (
                ilen - 1,
                best_cost,
                problem.max_weight,
                vec![false; problem.items.len()],
            ),
            |(i, rem_cost, rem_weight, mut vec), x| {
                let (new_cost, rem_weight) = if let Some(w) = x[rem_cost as usize / gcd] {
                    if w <= rem_weight {
                        (rem_cost, w)
                    } else {
                        vec[mapping[i]] = true;
                        let rem_cost = rem_cost - items[i].cost;
                        (rem_cost, x[rem_cost as usize / gcd].unwrap())
                    }
                } else {
                    vec[mapping[i]] = true;
                    let rem_cost = rem_cost - items[i].cost;
                    (rem_cost, x[rem_cost as usize / gcd].unwrap())
                };
                (i - 1, new_cost, rem_weight, vec)
            },
        )
        .3;

    Solution {
        id: problem.id,
        size: problem.size,
        cost: best_cost,
        items: Some(best_solution),
    }
}

fn construction_greedy(problem: &Problem) -> Solution {
    let (items, _, mappings) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
    let (items, _, cost) = items.iter().enumerate().fold(
        (vec![false; problem.items.len()], problem.max_weight, 0),
        |(mut items, rem_weight, cost), (i, item)| {
            if rem_weight >= item.weight {
                items[mappings[i]] = true;
                (items, rem_weight - item.weight, cost + item.cost)
            } else {
                (items, rem_weight, cost)
            }
        },
    );
    Solution {
        id: problem.id,
        size: problem.size,
        items: Some(items),
        cost,
    }
}

fn best_valued_item_fit(items: &Vec<Item>, max_weight: u32) -> (u32, usize) {
    items
    .iter()
    .enumerate()
    .fold((0, 0), |(cost, index), (i, item)| {
        if item.cost > cost && item.weight <= max_weight {
            (item.cost, i)
        } else {
            (cost, index)
        }
    })
}

fn construction_redux(problem: &Problem) -> Solution {
    let biggest_item_which_fit = best_valued_item_fit(&problem.items, problem.max_weight);
    let greedy = construction_greedy(problem);
    if biggest_item_which_fit.0 > greedy.cost {
        Solution {
            id: problem.id,
            size: problem.size,
            cost: biggest_item_which_fit.0,
            items: Some(
                (0..problem.size)
                    .map(|i| i == biggest_item_which_fit.1)
                    .collect(),
            ),
        }
    } else {
        greedy
    }
}
