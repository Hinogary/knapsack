use super::{utils::sort_by_cost_weight_ratio, Item, Problem, Solution, SolverTrait};

#[derive(Debug, Clone)]
pub struct GreedySolver();

impl SolverTrait for GreedySolver {
    fn construction(&self, problem: &Problem) -> Solution {
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
}

pub fn construction_greedy_inner(
    items: &[Item],
    mappings: &[usize],
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
