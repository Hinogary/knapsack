use super::{
    ratio,
    utils::{
        best_valued_item_fit, calc_remaining_cost, calc_remaining_weight, max_cost_from_rem,
        sort_by_cost_weight_ratio,
    },
    Problem, Solution, SolverTrait,
};

#[derive(Debug, Clone)]
pub struct PruningSolver();

#[derive(Debug, Clone)]
struct ProblemWithRatios {
    p: Problem,
    ratios: Vec<ratio>,
    rem_weight: Vec<u32>,
    rem_cost: Vec<u32>,
    best_solution: Vec<bool>,
}

impl SolverTrait for PruningSolver {
    fn construction(&self, problem: &Problem) -> Solution {
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
                    || cost + problem.rem_cost[index] <= best_cost
                    // max_cost_from_rem is O(log n)
                    || best_cost >= cost + max_cost_from_rem(&problem.rem_cost[index..], &problem.rem_weight[index..], problem.p.max_weight - weight)
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

        let (items, ratios, mappings) =
            sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

        // items are already filtered by weight, so u32::MAX is fine
        let best_item = best_valued_item_fit(&items, u32::MAX);

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
}
