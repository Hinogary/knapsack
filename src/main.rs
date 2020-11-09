use itertools::Itertools;

use structopt::StructOpt;

use lazy_format::lazy_format;

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::str::FromStr;
use std::time::{Duration, Instant};

use derive_more::Display;
use structopt::clap::{Error};

use gcd::Gcd;

use num_rational::Ratio;

type ratio = Ratio<u32>;

fn main() -> Result<(), Error>  {
    let opts = Opts::from_args();

    let solver = Solver::from_opts(&opts);

    let input = &opts.input_task;

    let ref_solutions = if let Some(ref sol) = opts.solution {
        Some(sol.0.iter().fold(HashMap::new(), |mut map, value| {
            map.insert(value.id, value);
            map
        }))
    } else {
        None
    };

    let mut time = Duration::new(0, 0);

    let durations = input
        .0
        .iter()
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
                    (FTPAS, true) => construction_ftpas(&problem, opts.ftpas.unwrap()),
                    #[allow(unreachable_patterns)]
                    _ => unimplemented!(),
                },
                start.elapsed(),
                problem,
                solver.is_exact(),
                solver,
            )
        })
        .map(|(solution, elapsed, problem, is_exact, solver)| {
            let mut output = String::new();
            //println!("{:?}", problem);
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
                    if **reference != solution
                        && reference.cost == solution.cost
                        && reference.size == solution.size
                    {
                        println!("Same cost, but different solution!");
                    } else {
                        assert_eq!(**reference, solution);
                    }
                } else {
                    let absolute_error = reference.cost - solution.cost;
                    let ref_cost = reference.cost as f32;
                    let cost = solution.cost as f32;
                    let relative_error = (ref_cost - cost) / ref_cost;

                    if solver == FTPAS {
                        let gcd = opts.ftpas.unwrap();
                        let practical_error = calculate_practical_ftpas_error(&problem, gcd);

                        additional_info = format!(
                            " errors: ratio: {} absolute: {} max possible: {} ratio: {}",
                            relative_error,
                            absolute_error,
                            practical_error,
                            absolute_error as f32 / practical_error as f32
                        );
                    } else {
                        additional_info = format!(
                            "errors: ratio: {} absolute: {}",
                            relative_error, absolute_error
                        );
                    }
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

    let max_time = durations
        .iter()
        .map(|(_, elapsed)| *elapsed)
        .max()
        .unwrap_or(time);

    let avg_time = durations
        .iter()
        .map(|(_, elapsed)| *elapsed)
        .fold(Duration::new(0, 0), |acc, x| acc + x)
        / (durations.len() as u32);

    println!("Maximum time: {:?} Average time: {:?}", max_time, avg_time);

    println!("Total time: {:?}", time);

    println!("{} {}", max_time.as_secs_f64(), avg_time.as_secs_f64())
    Ok(())
}

fn calc_remaining_weight(items: &[Item]) -> Vec<u32> {
    //reverse items[..].cost, then accumule them into vector, where x[i] = x[i-1] + items[i].cost (x[-1] = 0), then again reverse
    items
        .iter()
        .rev()
        .map(|item| item.weight)
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain([0u32].iter().cloned())
        .collect()
}

fn calc_remaining_cost(items: &[Item]) -> Vec<u32> {
    //reverse items[..].cost, then accumule them into vector, where x[i] = x[i-1] + items[i].cost (x[-1] = 0), then again reverse
    items
        .iter()
        .rev()
        .map(|item| item.cost)
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain([0u32].iter().cloned())
        .collect()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

#[derive(Display, Debug)]
#[display(fmt="{}", self.0)]
struct DisplayError(String);

#[derive(Debug)]
struct SolutionsFromFile(Vec<Solution>);

impl FromStr for SolutionsFromFile {
    type Err = DisplayError;
    fn from_str(file_name: &str) -> Result<SolutionsFromFile, DisplayError> {
        Ok(SolutionsFromFile(
            fs::read_to_string(file_name)
                .map_err(|e| {
                    DisplayError(format!(
                        "Could not solution load file: {}, because: {}",
                        file_name, e
                    ))
                })?
                .lines()
                .map(|line| {
                    parse_solution_line(line).map_err(|e| format!("{}\nSolution Line: {}", e, line))
                })
                .collect::<Result<_, _>>()
                .map_err(|s| DisplayError(s))?,
        ))
    }
}

#[derive(Debug)]
struct ProblemFromfile(Vec<Problem>);

impl FromStr for ProblemFromfile {
    type Err = DisplayError;
    fn from_str(file_name: &str) -> Result<ProblemFromfile, DisplayError> {
        Ok(ProblemFromfile(
            fs::read_to_string(file_name)
                .map_err(|e| {
                    DisplayError(format!(
                        "Could not problem load file: {}, because: {}",
                        file_name, e
                    ))
                })?
                .lines()
                .map(|line| {
                    parse_problem_line(line).map_err(|e| format!("{}\nProblem Line: {}", e, line))
                })
                .collect::<Result<_, _>>()
                .map_err(|s| DisplayError(s))?,
        ))
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsack", author = "Martin Quarda <martin@quarda.cz>")]
struct Opts {
    input_task: ProblemFromfile,
    solution: Option<SolutionsFromFile>,
    #[structopt(long)]
    naive: bool,
    #[structopt(long)]
    pruning: bool,
    #[structopt(long)]
    dynamic_weight: bool,
    #[structopt(long)]
    dynamic_cost: bool,
    #[structopt(long)]
    greedy: bool,
    #[structopt(long)]
    redux: bool,
    #[structopt(long)]
    ftpas: Option<u32>,
    #[structopt(long)]
    save_durations: Option<String>,
    #[structopt(long)]
    force_construction: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    ratios: Vec<ratio>,
    rem_weight: Vec<u32>,
    rem_cost: Vec<u32>,
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
}

fn calculate_practical_ftpas_error(problem: &Problem, gcd: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};
    let mut items = problem.items.clone();
    items.sort_by(|a, b| a.weight.cmp(&b.weight));
    #[allow(deprecated)]
    let m = items
        .iter()
        .fold_while((0, 0), |(weight, i), item| {
            if weight + item.weight <= problem.max_weight {
                Continue((weight + item.weight, i + 1))
            } else {
                Done((0, i))
            }
        })
        .into_inner()
        .1;
    let mut gcds = items
        .into_iter()
        .map(|item| item.cost % gcd)
        .collect::<Vec<_>>();
    gcds.sort_unstable_by(|a, b| b.cmp(a));
    gcds[0..m].iter().sum()
}

fn calculate_practical_ftpas_error(problem: &Problem, gcd: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};
    let mut items = problem.items.clone();
    items.sort_by(|a, b| a.weight.cmp(&b.weight));
    #[allow(deprecated)]
    let m = items
        .iter()
        .fold_while((0, 0), |(weight, i), item| {
            if weight + item.weight <= problem.max_weight {
                Continue((weight + item.weight, i + 1))
            } else {
                Done((0, i))
            }
        })
        .into_inner()
        .1;
    let mut gcds = items
        .into_iter()
        .map(|item| item.cost % gcd)
        .collect::<Vec<_>>();
    gcds.sort_unstable_by(|a, b| b.cmp(a));
    gcds[0..m].iter().sum()
}

// returns (new items, cost/weight ratios descending, mapping [new array] -> [original array])
fn sort_by_cost_weight_ratio(items: &[Item], max_weight: u32) -> (Vec<Item>, Vec<ratio>, Vec<usize>) {
    use std::cmp::Ordering;
    let len = items.len();
    let mut vec = items
        .iter()
        .enumerate()
        .map(|(index, item)| (*item, ratio::new(item.cost, item.weight), index))
        .filter(|(item, _, _)| item.weight <= max_weight)
        .collect::<Vec<_>>();
    vec.sort_unstable_by(|a, b| match b.1.cmp(&a.1) {
        Ordering::Equal => b.0.weight.cmp(&a.0.weight),
        other => other,
    });
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

// O(ln n)
fn max_cost_from_rem(rem_costs: &[u32], rem_weights: &[u32], max_weight: u32) -> u32 {
    let mut l = 0;
    let mut r = rem_costs.len() - 2;
    if rem_weights[0] <= max_weight {
        return rem_costs[0];
    }
    // binary search
    while l != r {
        let next = (l + r) / 2;
        let weight = rem_weights[0] - rem_weights[next + 1];
        if weight <= max_weight {
            l = next + 1;
        } else {
            r = next;
        }
    }
    let rem_weight = (max_weight + rem_weights[l] - rem_weights[0]).min(rem_weights[0]);
    let last_weight = rem_weights[l] - rem_weights[l + 1];
    let last_cost = rem_costs[l] - rem_costs[l + 1];
    let cost = rem_costs[0] - rem_costs[l]
        + if rem_weight == 0 {
            0
        } else {
            last_cost * last_weight / rem_weight
        };
    cost
}

// Calculates maximum possible cost ... takes sorted items by cost/weight ratios
// O(n)
fn max_cost(items: &[Item], max_weight: u32) -> u32 {
    let mut i = 0;
    use itertools::FoldWhile::{Continue, Done};
    #[allow(deprecated)] // fold_while no longer deprecated in master
    items
        .iter()
        .fold_while((0, 0), |(weight, cost), x| {
            if weight + x.weight <= max_weight {
                i += 1;
                Continue((weight + x.weight, cost + x.cost))
            } else {
                if weight == max_weight {
                    Done((0, cost))
                } else {
                    Done((0, cost + x.cost * x.weight / (max_weight - weight)))
                }
            }
        })
        .into_inner()
        .1
}

fn next_parse_with_err<'a, T, K>(iter: &mut T) -> Result<K, String>
where
    T: Iterator<Item = &'a str>,
    K: FromStr,
    <K as std::str::FromStr>::Err: std::fmt::Debug,
{
    Ok(iter
        .next()
        .ok_or_else(|| format!("Line exhasted, but next item was expecting"))?
        .parse()
        .map_err(|e| format!("Could not parse number {:?}", e))?)
}

fn parse_problem_line(line: &str) -> Result<Problem, String> {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id: i32 = next_parse_with_err(&mut iter)?;
    let size = next_parse_with_err(&mut iter)?;
    let max_weight = next_parse_with_err(&mut iter)?;
    let min_cost = match () {
        () if id < 0 => Ok(Some(next_parse_with_err(&mut iter)?)),
        () if id > 0 => Ok(None),
        _ => Err(format!("zero id not permitted")),
    }?;
    let items = (0..size)
        .map(|_|{
            let weight =  next_parse_with_err(&mut iter)?;
            let cost =  next_parse_with_err(&mut iter)?;
            Ok(Item {
                weight,
                cost
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    assert_eq!(
        iter.next(),
        None,
        "Line was not exhausted, wrong problem line!"
    );
    Ok(Problem {
        id: id.abs() as u32,
        max_weight,
        size,
        min_cost,
        items,
    })
}

fn parse_solution_line(line: &str) -> Result<Solution, String> {
    let mut iter = line.split(' ').filter(|x| !x.is_empty());
    let id = next_parse_with_err(&mut iter)?;
    let size = next_parse_with_err(&mut iter)?;
    let cost = next_parse_with_err(&mut iter)?;
    let items = Some(
        (0..size)
            .map(|_| {
                match iter
                    .next()
                    .ok_or_else(|| format!("Not enough bits in line!"))?
                {
                    "1" => Ok(true),
                    "0" => Ok(false),
                    _ => Err(format!("Reference solution is not in (0, 1)!")),
                }
            })
            .collect::<Result<Vec<_>, String>>()?,
    );
    if iter.next() != None {
        return Err(format!("Line was not exhausted, wrong solution line!"));
    }
    Ok(Solution {
        id,
        size,
        cost,
        items,
    })
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
        last_selected: bool,
    ) -> u32 {
        if index < problem.p.items.len() {
            let ratio = problem.ratios[index];
            if (problem.p.max_weight - weight).min(problem.rem_weight[index])
                * ratio.numer() / ratio.denom()
                + cost
                < best_cost
                || cost + problem.rem_cost[index] < best_cost
                // max_cost_from_rem is O(log n)
                || best_cost > cost + max_cost_from_rem(&problem.rem_cost[index..], &problem.rem_weight[index..], problem.p.max_weight - weight)
            {
                return best_cost;
            }
            let cur_item = problem.p.items[index];
            let new_weight = weight + cur_item.weight;
            let best_with_item = if new_weight <= problem.p.max_weight
            //     v this condition prohibits from trying all permutations of same item
                && (last_selected || problem.p.items[index - 1] != cur_item)
            {
                rec_fn(
                    problem,
                    cost + cur_item.cost,
                    weight + cur_item.weight,
                    index + 1,
                    best_cost,
                    true,
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
                false,
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
        rem_cost: calc_remaining_cost(&items),
        rem_weight: calc_remaining_weight(&items),
        best_solution: (0..items.len()).map(|i| i == best_item.1).collect(),
        p: Problem { items, ..*problem },
        ratios,
    };

    let cost = rec_fn(&mut aug_problem, 0, 0, 0, best_item.0, true);

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
    // backtracking only

    let (mut items, _ratios, mut mapping) =
        sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

    if items.len() == 0 {
        items.push(Item { weight: 1, cost: 0 });
        mapping.push(0);
    }

    let gcd = items
        .iter()
        .fold(items[0].weight, |acc, x| acc.gcd(x.weight));

    if gcd > 1 {
        for item in &mut items {
            item.weight /= gcd;
        }
    }

    let max_weight = problem.max_weight / gcd;

    let size = (problem.max_weight / gcd) as usize + 1;
    let ilen = items.len();

    let mut table_raw: Vec<Option<(u32, bool)>> = vec![None; size * (ilen + 1)];
    let mut table_base = table_raw
        .as_mut_slice()
        .chunks_mut(size)
        .collect::<Vec<_>>();

    //tabulka je převrácená oproti přednáškám .. řešení je v [0][0]
    //první index určuje přidané předměty .. např i => jsou přidané věci i..ilen
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
        if with_item.1 <= max_weight {
            if let Some(cell) = table[with_item.0][with_item.1 as usize] {
                me_cell = Some((cell.0 + items[*item].cost, true));
            } else {
                stack.push(with_item);
                continue;
            }
        }
        if let Some(cell) = table[without_item.0][without_item.1 as usize] {
            let cost_with_item = me_cell.map(|x| x.0).unwrap_or(0);
            if cell.0 >= cost_with_item {
                me_cell = Some((cell.0, false));
            }
        } else {
            stack.push(without_item);
            continue;
        }
        table[*item][*weight as usize] = me_cell;
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
                        let added = x[w as usize].unwrap().1;
                        vec[mapping[i]] = added;
                        (i + 1, if added { w + items[i].weight } else { w }, vec)
                    },
                )
                .2,
        ),
    }
}

fn construction_dynamic_cost(problem: &Problem) -> Solution {
    // mainly foward tracking but backtracing solution
    let (mut items, mut ratios, mut mappings) =
        sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
    ratios.push(ratio::new(0,1));

    if items.len() == 0 {
        items.push(Item {
            weight: std::u32::MAX,
            cost: 0,
        });
        mappings.push(0);
    }

    let cost_gcd = items.iter().fold(items[0].cost, |acc, x| acc.gcd(x.cost));
    let weight_gcd = items
        .iter()
        .fold(items[0].weight, |acc, x| acc.gcd(x.weight));
    let max_weight = (problem.max_weight - problem.max_weight % weight_gcd).max(1);
    let ilen = items.len();

    if cost_gcd > 1 {
        for item in &mut items {
            item.cost /= cost_gcd;
        }

        for ratio in &mut ratios {
            *ratio /= cost_gcd;
        }
    }

    // if we take first k items and part of the first item, which do not fit, we get maximal possible cost
    let max_cost = max_cost(&items, max_weight);

    let size = max_cost as usize + 1;

    if max_cost == 0 {
        return Solution::empty(problem.id, problem.size);
    }

    let rem_cost = calc_remaining_cost(&items);
    let rem_weight = calc_remaining_weight(&items);

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
    let redux_solution = {
        let (solution, cost) =
            construction_greedy_inner(&items, &mappings, problem.size, problem.max_weight);
        let (item_cost, index) = best_valued_item_fit(&problem.items, problem.max_weight);
        if cost > item_cost {
            (solution, cost)
        } else {
            ((0..problem.size).map(|i| i == index).collect(), item_cost)
        }
    };
    let mut best_cost = redux_solution.1 / cost_gcd;

    while !queue.is_empty() {
        let (item, cost) = queue.pop_front().unwrap();
        if item >= ilen {
            continue;
        }
        let with_item = (item + 1, cost + items[item].cost);
        let without_item = (item + 1, cost);
        let weight = table[item][cost as usize].unwrap();
        let new_weight = weight + items[item].weight;
        if new_weight <= max_weight {
            best_cost = best_cost.max(with_item.1);
            if let Some(value) = table[with_item.0][with_item.1 as usize] {
                if value > new_weight {
                    table[with_item.0][with_item.1 as usize] = Some(new_weight);
                    // it is already in queue, no need to push again
                }
            } else {
                table[with_item.0][with_item.1 as usize] = Some(new_weight);
                queue.push_back(with_item);
            }
        }

        // pruning same as in recursive pruning
        let ratio = ratios[without_item.0];
        if (max_weight - weight).min(rem_weight[without_item.0]) * ratio.numer() / ratio.denom()
            + cost
            < best_cost
            || cost + rem_cost[without_item.0] < best_cost
        {
            continue;
        }

        if let Some(value) = table[without_item.0][without_item.1 as usize] {
            if value > weight {
                table[without_item.0][without_item.1 as usize] = Some(weight);
                // it is already in queue, no need to push again
            }
        } else {
            table[without_item.0][without_item.1 as usize] = Some(weight);
            queue.push_back(without_item);
        }
    }

    let best_solution = if best_cost * cost_gcd == redux_solution.1 {
        redux_solution.0
    } else {
        table
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
                    let (new_cost, rem_weight) = if let Some(w) = x[rem_cost as usize] {
                        if w <= rem_weight {
                            (rem_cost, w)
                        } else {
                            vec[mappings[i]] = true;
                            let rem_cost = rem_cost - items[i].cost;
                            (rem_cost, x[rem_cost as usize].unwrap())
                        }
                    } else {
                        vec[mappings[i]] = true;
                        let rem_cost = rem_cost - items[i].cost;
                        (rem_cost, x[rem_cost as usize].unwrap())
                    };
                    (i.max(1) - 1, new_cost, rem_weight, vec)
                },
            )
            .3
    };

    Solution {
        id: problem.id,
        size: problem.size,
        cost: best_cost * cost_gcd,
        items: Some(best_solution),
    }
}

fn construction_greedy_inner(
    items: &Vec<Item>,
    mappings: &Vec<usize>,
    size: usize,
    max_weight: u32,
) -> (Vec<bool>, u32) {
    let (items, _, cost) = items.iter().enumerate().fold(
        (vec![false; size], max_weight, 0),
        |(mut items, rem_weight, cost), (i, item)| {
            if rem_weight >= item.weight {
                items[mappings[i]] = true;
                (items, rem_weight - item.weight, cost + item.cost)
            } else {
                (items, rem_weight, cost)
            }
        },
    );
    (items, cost)
}

fn construction_greedy(problem: &Problem) -> Solution {
    let (items, _, mappings) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
    let (items, cost) =
        construction_greedy_inner(&items, &mappings, problem.size, problem.max_weight);
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

fn construction_ftpas(problem: &Problem, forced_gcd: u32) -> Solution {
    let transformed_items = problem
        .items
        .iter()
        .map(|&item| Item {
            cost: item.cost / forced_gcd,
            ..item
        })
        .collect();

    let solution = construction_dynamic_cost(&Problem {
        items: transformed_items,
        ..*problem
    });

    Solution {
        cost: if let Some(ref items) = solution.items {
            items.iter().enumerate().fold(0, |acc, (i, &used)| {
                if used {
                    acc + problem.items[i].cost
                } else {
                    acc
                }
            })
        } else {
            0
        },
        ..solution
    }
}
