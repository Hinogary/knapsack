use itertools::Itertools;

use structopt::StructOpt;

use lazy_format::lazy_format;

use std::collections::HashMap;
use std::fs;

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

    let mut time = Duration::new(0, 0);

    let durations = input
        .0
        .iter()
        .map(|problem| {
            let start = Instant::now();
            (
                match problem.min_cost.is_none() || opts.force_construction {
                    true => solver.construction(&problem),
                    false => unimplemented!(),
                },
                start.elapsed(),
                problem,
            )
        })
        .map(|(solution, elapsed, problem)| {
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
                if solver.is_exact() {
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

                    if let FTPAS(_) = solver {
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

    println!("{} {}", max_time.as_secs_f64(), avg_time.as_secs_f64());
    Ok(())
}

#[derive(Display, Debug)]
#[display(fmt="{}", self.0)]
pub struct DisplayError(String);

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsack", author = "Martin Quarda <martin@quarda.cz>")]
pub struct Opts {
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
pub struct Item {
    weight: u32,
    cost: u32,
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
}
