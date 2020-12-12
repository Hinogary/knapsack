use super::{ Problem, Solution,
    SolverTrait,
    Item,
};

use arrayvec::ArrayVec;
use itertools::izip;

#[derive(Debug, Clone)]
pub struct TabuSearchSolver {
    memory_size: usize,
    iterations: usize,
}

fn cost_weight(state: &[bool], items: &[Item]) -> (u32, u32) {
    state
    .iter()
    .zip(items.iter())
    .filter(|(&in_pack, _)| in_pack)
    .fold((0, 0), |(cost, weight), (_, item)| (cost + item.cost, weight + item.weight))
}

impl SolverTrait for TabuSearchSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        let mut state = vec![true; problem.size];

        // cost_fn = (- over_capacity, cost)
        let mut memory_raw = vec![false; problem.size * self.memory_size];
        let mut full_memory = memory_raw.chunks_mut(problem.size).collect::<Vec<_>>();

        let mut index_to_rewrite = 0;

        for _ in 0..self.iterations {
            let (cost, weight) = cost_weight(&state, &problem.items);
            let tabu = full_memory
                .iter()
                .fold(vec![false; problem.size], |mut blacklist, memory_state| {
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
            let index_to_switch = izip!((0..), state.iter(), problem.items.iter(), tabu.into_iter())
                .filter(|(_, _, _, tabu)| !tabu)
                .map(
                    |(i, &current_state, item, _)|
                    {
                        let (new_weight, new_cost) = if current_state {
                            (weight - item.weight, cost - item.cost)
                        } else {
                            (weight + item.weight, cost + item.cost)
                        };
                        if new_weight > item.weight {
                            (std::u32::MAX - new_weight, new_cost, i)
                        } else {
                            (std::u32::MAX, new_cost, i)
                        }
                    }
                ).max().map(|(_, _, i)| i);
            if let Some(i) = index_to_switch {
                state[i] = !state[i];
            } else {
                panic!("Tabu stuck in local minimum (all near states are in tabu).");
            }
            full_memory[index_to_rewrite].iter_mut().zip(state.iter()).for_each(|(mem, &state)| *mem = state);
            index_to_rewrite = (index_to_rewrite + 1) % self.memory_size;
        }
        let (cost, weight) = cost_weight(&state, &problem.items);
        Solution{
            id: problem.id,
            cost,
            items: Some(state),
            size: problem.size,
        }
    }
}
