use itertools::Itertools;

use structopt::StructOpt;

use lazy_format::lazy_format;

use std::collections::HashMap;
use std::fs;
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
                    (Naive, false) => decision_naive(&problem),
                    (Pruning, true) => construction_pruning(&problem),
                    (Pruning, false) => decision_pruning(&problem),
                    (DynamicWeight, true) => construction_dynamic_weight(&problem),
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
            if ref_solutions.is_some() && problem.min_cost.is_none() {
                let reference = ref_solutions.as_ref().unwrap().get(&solution.id).unwrap();
                if is_exact {
                    if reference != &solution && reference.cost == solution.cost && reference.size == solution.size {
                        println!("Same cost, but different solution!");
                    } else {
                        assert_eq!(*reference, solution);
                    }
                } else {
                    additional_info = format!(" ratio of ref solution: {}", solution.cost as f32 / reference.cost as f32);
                }
            }
            println!(
                "time: {:?} {}\n{}",
                elapsed,
                additional_info,
                output
            );

            (
                solution.id,
                elapsed,
            )
        })
        .collect::<Vec<_>>();

    // output file, which is easier to parse for nice graphs
    if let Some(path) = opts.save_durations {
        fs::write(
            path,
            durations
                .iter()
                .map(|(id, elapsed)| {
                    lazy_format!("{} {}", id, elapsed.as_secs_f64())
                })
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
    #[structopt(short, long)]
    dynamic_weight: bool,
    #[structopt(short, long)]
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
struct ProblemWithCostRem {
    p: Problem,
    costs_rem: Vec<u32>,
    best_solution: Vec<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Solution {
    id: u32,
    size: usize,
    cost: u32,
    items: Option<Vec<bool>>,
}

fn calc_remaining_cost(problem: &Problem) -> Vec<u32> {
    //reverse items[..].cost, then accumule them into vector, where x[i] = x[i-1] + items[i].cost (x[-1] = 0), then again reverse
    problem
        .items
        .iter()
        .rev()
        .map(|item| item.cost)
        .fold(Vec::with_capacity(problem.size), |mut acc, x| {
            acc.push(x + acc.last().unwrap_or(&0));
            acc
        })
        .into_iter()
        .rev()
        .collect()
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

fn decision_naive(problem: &Problem) -> Solution {
    fn rec_fn(
        problem: &mut ProblemWithSol,
        cost: u32,
        weight: u32,
        index: usize,
        min_cost: u32,
    ) -> u32 /* best_cost */ {
        if index < problem.p.size {
            let best_with_item = rec_fn(
                problem,
                cost + problem.p.items[index].cost,
                weight + problem.p.items[index].weight,
                index + 1,
                min_cost,
            );
            let best_without_item = if best_with_item == 0 {
                rec_fn(problem, cost, weight, index + 1, min_cost)
            } else {
                0
            };
            match best_with_item.max(best_without_item) {
                0 => 0,
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
            if weight <= problem.p.max_weight && cost >= min_cost {
                cost
            } else {
                0
            }
        }
    }

    let mut aug_problem = ProblemWithSol {
        p: problem.clone(),
        best_solution: vec![false; problem.size],
    };
    let cost = rec_fn(&mut aug_problem, 0, 0, 0, problem.min_cost.unwrap());
     Solution {
            id: problem.id,
            size: problem.size,
            cost,
            items: if cost > problem.min_cost.unwrap() {
                Some(aug_problem.best_solution)
            } else {
                None
            },
        }
}

fn construction_pruning(problem: &Problem) -> Solution {
    fn rec_fn(
        problem: &mut ProblemWithCostRem,
        cost: u32,
        weight: u32,
        index: usize,
        best_cost: u32,
    ) -> u32 {
        if index < problem.p.size {
            if cost + problem.costs_rem[index] < best_cost {
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

    let mut aug_problem = ProblemWithCostRem {
        p: (*problem).clone(),
        costs_rem: calc_remaining_cost(problem),
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

fn decision_pruning(problem: &Problem) -> Solution {
    fn rec_fn(
        problem: &mut ProblemWithCostRem,
        cost: u32,
        weight: u32,
        index: usize,
        min_cost: u32,
    ) -> u32 {
        if index < problem.p.size {
            if cost + problem.costs_rem[index] < min_cost {
                return 0;
            }
            let new_weight = weight + problem.p.items[index].weight;
            let best_with_item = if new_weight <= problem.p.max_weight {
                rec_fn(
                    problem,
                    cost + problem.p.items[index].cost,
                    new_weight,
                    index + 1,
                    min_cost,
                )
            } else {
                0
            };
            let best_without_item = if best_with_item == 0 {
                rec_fn(problem, cost, weight, index + 1, min_cost)
            } else {
                0
            };
            match best_with_item.max(best_without_item) {
                0 => 0, //best_cost didnt change, so it was not in this recursion (at best was same, which we ignore)
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
            if cost >= min_cost {
                cost
            } else {
                0
            }
        }
    }

    let mut aug_problem = ProblemWithCostRem {
        p: (*problem).clone(),
        costs_rem: calc_remaining_cost(problem),
        best_solution: vec![false; problem.size],
    };
    let cost = rec_fn(&mut aug_problem, 0, 0, 0, problem.min_cost.unwrap());
        Solution {
            id: problem.id,
            size: problem.size,
            cost,
            items: if cost > problem.min_cost.unwrap() {
                Some(aug_problem.best_solution)
            } else {
                None
            }
        }
}

fn construction_dynamic_weight(problem: &Problem) -> Solution {
    unimplemented!()
}
