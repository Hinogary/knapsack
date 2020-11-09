use super::{dynamic_cost::DynamicCostSolver, Item, Problem, Solution, SolverTrait};

#[derive(Debug, Clone)]
pub struct FTPASSolver {
    pub gcd: u32,
}

impl SolverTrait for FTPASSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        let transformed_items = problem
            .items
            .iter()
            .map(|&item| Item {
                cost: item.cost / self.gcd,
                ..item
            })
            .collect();

        let solution = DynamicCostSolver().construction(&Problem {
            items: transformed_items,
            ..*problem
        });

        Solution {
            cost: if let Some(ref items) = solution.items {
                items.iter().enumerate().fold(0, |acc, (i, &used)| {
                    if used {
                        acc + problem.items[i].cost
                    } else {
                        acc
                    }
                })
            } else {
                0
            },
            ..solution
        }
    }
}
