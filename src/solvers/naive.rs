use super::{Problem, Solution, SolverTrait};

#[derive(Debug, Clone)]
pub struct NaiveSolver();

// augumented problems used in recursion descent
#[derive(Debug, Clone)]
struct ProblemWithSol {
    p: Problem,
    best_solution: Vec<bool>,
}

impl SolverTrait for NaiveSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        fn rec_fn(
            problem: &mut ProblemWithSol,
            cost: u32,
            weight: u32,
            index: usize,
            best_cost: u32,
        ) -> u32 /* best_cost */ {
            if index < problem.p.size {
                let best_with_item = rec_fn(
                    problem,
                    cost + problem.p.items[index].cost,
                    weight + problem.p.items[index].weight,
                    index + 1,
                    best_cost,
                );
                let best_without_item = rec_fn(
                    problem,
                    cost,
                    weight,
                    index + 1,
                    best_cost.max(best_with_item),
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
            } else if weight <= problem.p.max_weight {
                best_cost.max(cost)
            } else {
                best_cost
            }
        }

        let mut aug_problem = ProblemWithSol {
            p: problem.clone(),
            best_solution: vec![false; problem.size],
        };
        let cost = rec_fn(&mut aug_problem, 0, 0, 0, 0);
        Solution {
            id: problem.id,
            size: problem.size,
            cost,
            items: Some(aug_problem.best_solution),
        }
    }
}
