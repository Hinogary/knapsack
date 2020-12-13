use super::{sort_by_cost_weight_ratio, Item, Problem, Solution, SolverTrait};

use arrayvec::ArrayVec;
use itertools::izip;

#[derive(Debug, Clone)]
pub struct TabuSearchSolver {
    pub memory_size: usize,
    pub iterations: usize,
}

fn cost_weight(state: &[bool], items: &[Item]) -> (u32, u32) {
    state
        .iter()
        .zip(items.iter())
        .filter(|(&in_pack, _)| in_pack)
        .fold((0, 0), |(cost, weight), (_, item)| {
            (cost + item.cost, weight + item.weight)
        })
}

struct TabuMemory {
    tabu_raw: Vec<bool>,
    capacity: usize,
    size: usize,
    target: usize,
    problem_size: usize,
}


impl<'a> TabuMemory {
    fn new(problem_size: usize, memory_size: usize) -> TabuMemory {
        let tabu_raw = vec![false; problem_size * memory_size];
        TabuMemory {
            capacity: memory_size,
            tabu_raw,
            size: 0,
            problem_size,
            target: 0,
        }
    }

    fn blacklist<'b>(&self, state: &[bool], pass_memory: &'b mut [bool]) -> &'b [bool] {
        pass_memory
            .iter_mut()
            .for_each(|x| *x = false);
        self
            .tabu_raw
            .chunks(self.problem_size)
            .take(self.size)
            .fold(pass_memory, |blacklist, memory_state| {
                // finds at most 2 different bools beetwen state and items in memory
                // on 1 sets blacklist on 2 nothing
                match memory_state
                    .iter()
                    .zip(state.iter())
                    .enumerate()
                    .filter(|(_, (m, s))| m != s)
                    .collect::<ArrayVec<[_; 2]>>()
                    .as_slice()
                {
                    &[(i, _)] => blacklist[i] = true,
                    _ => (),
                };
                blacklist
            }) as &[bool]
    }

    fn insert(&mut self, state: &[bool]) {
        self.tabu_raw
            .chunks_mut(self.problem_size)
            .skip(self.target)
            .take(1)
            .for_each(|x| {
                x.iter_mut()
                    .zip(state.iter())
                    .for_each(|(mem, &state)| *mem = state)
            });
        self.size = (self.size + 1).min(self.capacity);
        self.target = (self.target + 1) % self.capacity;
    }
}

impl SolverTrait for TabuSearchSolver {
    fn construction(&self, problem: &Problem) -> Solution {

        // Maybe this mappings helps little? not sure
        let (items, mapping) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

        if items.len() == 0 {
            return Solution::empty(problem.id, problem.size);
        }

        let mut state = vec![true; items.len()];
        let mut best_solution = vec![false; items.len()];
        let mut best_cost = 0;

        let mut tabu = TabuMemory::new(items.len(), self.memory_size);

        let mut blacklist_for_less_allocations = vec![false; items.len()];

        for _ in 0..self.iterations {
            let (cost, weight) = cost_weight(&state, &items);

            let blacklist = tabu.blacklist(&state, &mut blacklist_for_less_allocations);
            //println!("{:?}", blacklist);
            // maximize function (- over_capacity, cost, _)
            let cost_fn = izip!((0..), state.iter(), items.iter(), blacklist.iter())
                .filter(|(_, _, _, &blacklist)| !blacklist)
                .map(|(i, &current_state, item, _)| {
                    let (new_weight, new_cost) = if current_state {
                        (weight - item.weight, cost - item.cost)
                    } else {
                        (weight + item.weight, cost + item.cost)
                    };
                    if new_weight > problem.max_weight {
                        (std::u32::MAX - new_weight + problem.max_weight, new_cost, i)
                    } else {
                        (std::u32::MAX, new_cost, i)
                    }
                })
                .max().unwrap_or_else(||{
                    // tries again without blacklist
                    izip!((0..), state.iter(), items.iter())
                    .map(|(i, &current_state, item)| {
                        let (new_weight, new_cost) = if current_state {
                            (weight - item.weight, cost - item.cost)
                        } else {
                            (weight + item.weight, cost + item.cost)
                        };
                        if new_weight > problem.max_weight {
                            (std::u32::MAX - new_weight + problem.max_weight, new_cost, i)
                        } else {
                            (std::u32::MAX, new_cost, i)
                        }
                    })
                    .max().unwrap()
                });
            tabu.insert(&state);
            let index_to_switch = cost_fn.2;
            state[index_to_switch] = !state[index_to_switch];

            if cost_fn.0 == std::u32::MAX && best_cost < cost_fn.1 {
                best_solution.iter_mut().zip(state.iter()).for_each(|(b, &s)| *b = s);
                best_cost = cost_fn.1;
            }
            //switch state
        }
        println!("{:?}", best_solution);
        if problem.max_weight < items.iter().zip(best_solution.iter()).map(|(item, &included)| if included {item.weight} else {0}).sum(){
            Solution::none(problem.id, problem.size)
        } else {
            Solution {
                id: problem.id,
                cost: best_cost,
                items: Some(best_solution.into_iter().enumerate().fold(
                    vec![false; problem.size],
                    |mut acc, (i, x)| {
                        if let Some(&mapping) = mapping.get(i) {
                            acc[mapping] = x;
                        }
                        acc
                    },
                )),
                size: problem.size,
            }
        }
    }
}
