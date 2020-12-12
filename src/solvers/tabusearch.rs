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

impl SolverTrait for TabuSearchSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        let mut state = vec![true; problem.size];

        let mut memory_raw = vec![false; problem.size * self.memory_size];
        let mut full_memory = memory_raw.chunks_mut(problem.size).collect::<Vec<_>>();

        let (items, mapping) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
        let items = items.into_iter().rev().collect::<Vec<_>>();
        let mappings = mapping
            .into_iter()
            .rev()
            .map(|i| items.len() - 1 - i)
            .collect::<Vec<_>>();

        let mut index_to_rewrite = 0;
        let mut tabu_size = 0;

        for _ in 0..self.iterations {
            let (cost, weight) = cost_weight(&state, &problem.items);
            let tabu = full_memory[..tabu_size].iter().fold(
                vec![false; problem.size],
                |mut blacklist, memory_state| {
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
                },
            );
            // maximize function (- over_capacity, cost, _)
            let cost_fn = izip!((0..), state.iter(), problem.items.iter(), tabu.into_iter())
                .filter(|(_, _, _, tabu)| !tabu)
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
            //println!("{:?}", cost_fn.map(|(x, y, z)|(std::u32::MAX - x, y, z)).unwrap());
            let index_to_switch = cost_fn.map(|(_, _, i)| i);

            //insert to tabu
            full_memory[index_to_rewrite]
                .iter_mut()
                .zip(state.iter())
                .for_each(|(mem, &state)| *mem = state);
            tabu_size = (tabu_size + 1).min(self.memory_size);
            index_to_rewrite = (index_to_rewrite + 1) % self.memory_size;
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
