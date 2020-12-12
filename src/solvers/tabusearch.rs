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
        let blacklist = self
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
                    &[] => unreachable!(),
                    &[(i, _)] => blacklist[i] = true,
                    _ => (),
                };
                blacklist
            });
        blacklist
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
        let mut state = vec![true; problem.size];

        // Maybe this mappings helps little? not sure
        let (items, mapping) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
        let items = items.into_iter().rev().collect::<Vec<_>>();
        let mappings = mapping
            .into_iter()
            .rev()
            .map(|i| items.len() - 1 - i)
            .collect::<Vec<_>>();

        let mut tabu = TabuMemory::new(problem.size, self.memory_size);

        let mut blacklist_for_less_allocations = vec![false; problem.size];

        for _ in 0..self.iterations {
            let (cost, weight) = cost_weight(&state, &problem.items);
            blacklist_for_less_allocations
                .iter_mut()
                .for_each(|x| *x = false);
            let blacklist = tabu.blacklist(&state, &mut blacklist_for_less_allocations);
            // maximize function (- over_capacity, cost, _)
            let cost_fn = izip!((0..), state.iter(), problem.items.iter(), blacklist.iter())
                .filter(|(_, _, _, &blacklist)| !blacklist)
                .map(|(i, &current_state, item, _)| {
                    let (new_weight, new_cost) = if current_state {
                        (weight - item.weight, cost - item.cost)
                    } else {
                        (weight + item.weight, cost + item.cost)
                    };
                    if new_weight > problem.max_weight {
                        (std::u32::MAX - new_weight, new_cost, i)
                    } else {
                        (std::u32::MAX, new_cost, i)
                    }
                })
                .max();

            let index_to_switch = cost_fn.map(|(_, _, i)| i);

            tabu.insert(&state);
            //switch state
            if let Some(i) = index_to_switch {
                state[i] = !state[i];
            } else {
                panic!("Tabu stuck in local minimum (all near states are in tabu).");
            }
        }
        let (cost, weight) = cost_weight(&state, &problem.items);
        if weight > problem.max_weight {
            Solution::empty(problem.id, problem.size)
        } else {
            Solution {
                id: problem.id,
                cost,
                items: Some(state.into_iter().enumerate().fold(
                    vec![false; problem.size],
                    |mut acc, (i, x)| {
                        if let Some(&mapping) = mappings.get(i) {
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
