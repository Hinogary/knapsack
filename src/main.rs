use itertools::Itertools;

use structopt::StructOpt;

use std::collections::HashMap;

use std::time::{Duration, Instant};

use derive_more::Display;
use structopt::clap::{Error, ErrorKind};

mod ioutils;
mod solvers;
use ioutils::*;
use solvers::{utils::calculate_practical_ftpas_error, *};

fn main() -> Result<(), Error> {
    let opts = Opts::from_args();

    let solver = Solver::from_opts(&opts)
        .map_err(|e| Error::with_description(&e.0, ErrorKind::ArgumentConflict))?;

    let input = &opts.input_task;

    let ref_solutions = if let Some(ref sol) = opts.solution {
        Some(sol.0.iter().fold(HashMap::new(), |mut map, value| {
            map.insert(value.id, value);
            map
        }))
    } else {
        None
    };

    let mut stats = Stats::default();

    let durations = input
        .0
        .iter()
        .map(|problem| {
            let start = Instant::now();
            (
                match problem.min_cost.is_none() || opts.force_construction {
                    true => solver.construction(&problem),
                    false => solver.decision(&problem),
                },
                start.elapsed(),
                problem,
            )
        })
        .map(|(solution, elapsed, problem)| {
            let mut output = String::new();
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
                additional_info +=
                    &check_solution(reference, &solution, problem, &solver, &opts, &mut stats);
            }
            println!("time: {:?} {}\n{}", elapsed, additional_info, output);

            elapsed
        })
        .collect::<Vec<_>>();

    let max_time = durations.iter().max().cloned().unwrap();

    let avg_time = durations
        .iter()
        .fold(Duration::new(0, 0), |acc, x| acc + *x)
        / (durations.len() as u32);

    let total_time: Duration = durations.iter().sum();

    println!("Maximum time: {:?} Average time: {:?}", max_time, avg_time);

    println!("Total time: {:?}", total_time);

    if !solver.is_exact() && ref_solutions.is_some() {
        println!(
            "Maximum error: {} Average error: {} No solution: {}",
            stats.relative_error_max,
            stats.relative_error_total / stats.instances as f64,
            stats.no_solution,
        );
    }

    println!("{} {}", max_time.as_secs_f64(), avg_time.as_secs_f64());
    Ok(())
}

#[derive(Default)]
struct Stats {
    instances: usize,
    relative_error_total: f64,
    relative_error_max: f64,
    no_solution: usize,
}

fn check_solution(
    reference: &Solution,
    solution: &Solution,
    problem: &Problem,
    solver: &Solver,
    opts: &Opts,
    stats: &mut Stats,
) -> String {
    stats.instances += 1;
    if !solution.items.is_some(){
        stats.no_solution += 1;
        " No solution found".to_string()
    } else if solver.is_exact() {
        if *reference != *solution
            && reference.cost == solution.cost
            && reference.size == solution.size
        {
            println!("Same cost, but different solution!");
        } else {
            assert_eq!(reference, solution);
        }
        "".to_string()
    } else {
        let absolute_error = reference.cost - solution.cost;
        let ref_cost = reference.cost as f64;
        let cost = solution.cost as f64;
        let relative_error = (ref_cost - cost) / ref_cost.max(1.0);

        stats.relative_error_max = stats.relative_error_max.max(relative_error);
        stats.relative_error_total += relative_error;

        if let FTPAS(_) = solver {
            let gcd = opts.precision.unwrap();
            let practical_error = calculate_practical_ftpas_error(&problem, gcd);

            format!(
                " errors: ratio: {} absolute: {} max possible: {} ratio: {}",
                relative_error,
                absolute_error,
                practical_error,
                absolute_error as f32 / practical_error as f32
            )
        } else {
            format!(
                "errors: ratio: {} absolute: {}",
                relative_error, absolute_error
            )
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt="{}", self.0)]
pub struct DisplayError(String);

impl std::convert::From<&str> for DisplayError {
    fn from(err: &str) -> DisplayError {
        DisplayError(err.to_string())
    }
}

impl std::convert::From<String> for DisplayError {
    fn from(err: String) -> DisplayError {
        DisplayError(err)
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsack", author = "Martin Quarda <martin@quarda.cz>")]
pub struct Opts {
    method: Methods,
    input_task: ProblemFromfile,
    solution: Option<SolutionsFromFile>,
    #[structopt(long)]
    precision: Option<u32>,
    #[structopt(long)]
    force_construction: bool,
    #[structopt(long)]
    memory_size: Option<usize>,
    #[structopt(long)]
    iterations: Option<usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Item {
    cost: u32,
    weight: u32,
}

use solvers::ratio;

impl Item {
    fn cost_weight_ratio(&self) -> ratio {
        return ratio::new_raw(self.cost, self.weight);
    }
}

#[derive(Debug, Clone)]
pub struct Problem {
    id: u32,
    max_weight: u32,
    size: usize,
    // switch between decision and construction problem
    min_cost: Option<u32>,
    items: Vec<Item>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Solution {
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
