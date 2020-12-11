use super::{
    greedy::construction_greedy_inner, utils::sort_by_cost_weight_ratio, Item, Problem, Solution,
    SolverTrait,
};

#[derive(Debug, Clone)]
pub struct TabuSearchSolver {
    memory_size: usize,
    iterations: usize,
}

impl SolverTrait for TabuSearchSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        let (items, mappings) = sort_by_cost_weight_ratio(&problem.items, problem.max_weight);
        let (items, cost) =
            construction_greedy_inner(&items, &mappings, problem.size, problem.max_weight);
        let raw_memory = vec![false; self.memory_size * problem.size];
        let memory = raw_memory.chunks(problem.size).collect::<Vec<_>>();
        Solution {
            id: problem.id,
            size: problem.size,
            items: Some(items),
            cost,
        }
    }
}
