use super::{greedy::GreedySolver, utils::best_valued_item_fit, Problem, Solution, SolverTrait};

#[derive(Debug, Clone)]
pub struct ReduxSolver();

impl SolverTrait for ReduxSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        let biggest_item_which_fit = best_valued_item_fit(&problem.items, problem.max_weight);
        let greedy = GreedySolver().construction(problem);
        if biggest_item_which_fit.0 > greedy.cost {
            Solution {
                id: problem.id,
                size: problem.size,
                cost: biggest_item_which_fit.0,
                items: Some(
                    (0..problem.size)
                        .map(|i| i == biggest_item_which_fit.1)
                        .collect(),
                ),
            }
        } else {
            greedy
        }
    }
}
