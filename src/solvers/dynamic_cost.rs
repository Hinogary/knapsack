use super::{
    greedy::construction_greedy_inner,
    ratio,
    utils::{
        best_valued_item_fit, calc_remaining_cost, calc_remaining_weight, max_cost,
        sort_by_cost_weight_ratio,
    },
    Item, Problem, Solution, SolverTrait,
};
use gcd::Gcd;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DynamicCostSolver();

impl SolverTrait for DynamicCostSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        // mainly foward tracking but backtracing solution
        let (mut items, mut mappings) =
            sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

        if items.is_empty() {
            items.push(Item {
                weight: std::u32::MAX,
                cost: 0,
            });
            mappings.push(0);
        }

        let cost_gcd = items.iter().fold(items[0].cost, |acc, x| acc.gcd(x.cost));
        let weight_gcd = items
            .iter()
            .fold(items[0].weight, |acc, x| acc.gcd(x.weight));
        let max_weight = (problem.max_weight - problem.max_weight % weight_gcd).max(1);
        let ilen = items.len();

        if cost_gcd > 1 {
            for item in &mut items {
                item.cost /= cost_gcd;
            }
        }

        // if we take first k items and part of the first item, which do not fit, we get maximal possible cost
        let max_cost = max_cost(&items, max_weight);

        let size = max_cost as usize + 1;

        if max_cost == 0 {
            return Solution::empty(problem.id, problem.size);
        }

        let rem_cost = calc_remaining_cost(&items);
        let rem_weight = calc_remaining_weight(&items);

        let mut table_raw: Vec<Option<u32>> = vec![None; size * (ilen + 1)];
        let mut table_base = table_raw
            .as_mut_slice()
            .chunks_mut(size)
            .collect::<Vec<_>>();

        // první index určuje přidané věci ... jsou přidané věci 0..index
        // druhý index určuje dosáhnutou hodnotu
        // hodnota určuje využitou váhu, tu se snažím minimalizovat

        let table = table_base.as_mut_slice();

        table[0][0] = Some(0);

        // BFS průchod, VecDeque je ring buffer
        // indexy v queue jsou seřazené podle indexů itemů, ale ne nutně v pořadí podle ceny

        let mut queue = VecDeque::with_capacity(size);

        queue.push_back((0, 0));

        // best_cost je nejlepší cena dosáhnuta za 0..(index aktuálního předmětu)
        let redux_solution = {
            let (solution, cost) =
                construction_greedy_inner(&items, &mappings, problem.size, problem.max_weight);
            let (item_cost, index) = best_valued_item_fit(&problem.items, problem.max_weight);
            if cost > item_cost {
                (solution, cost)
            } else {
                ((0..problem.size).map(|i| i == index).collect(), item_cost)
            }
        };
        let mut best_cost = redux_solution.1 / cost_gcd;

        while !queue.is_empty() {
            let (item, cost) = queue.pop_front().unwrap();
            if item >= ilen {
                continue;
            }
            let with_item = (item + 1, cost + items[item].cost);
            let without_item = (item + 1, cost);
            let weight = table[item][cost as usize].unwrap();
            let new_weight = weight + items[item].weight;
            if new_weight <= max_weight {
                best_cost = best_cost.max(with_item.1);
                if let Some(value) = table[with_item.0][with_item.1 as usize] {
                    if value > new_weight {
                        table[with_item.0][with_item.1 as usize] = Some(new_weight);
                        // it is already in queue, no need to push again
                    }
                } else {
                    table[with_item.0][with_item.1 as usize] = Some(new_weight);
                    queue.push_back(with_item);
                }
            }

            // pruning same as in recursive pruning
            let ratio = items
                .get(without_item.0)
                .map(|x| x.cost_weight_ratio())
                .unwrap_or(ratio::new(0, 1)); //ratios[without_item.0];
            if (max_weight - weight).min(rem_weight[without_item.0]) * ratio.numer() / ratio.denom()
                + cost
                < best_cost
                || cost + rem_cost[without_item.0] < best_cost
            {
                continue;
            }

            if let Some(value) = table[without_item.0][without_item.1 as usize] {
                if value > weight {
                    table[without_item.0][without_item.1 as usize] = Some(weight);
                    // it is already in queue, no need to push again
                }
            } else {
                table[without_item.0][without_item.1 as usize] = Some(weight);
                queue.push_back(without_item);
            }
        }

        let best_solution = if best_cost * cost_gcd == redux_solution.1 {
            redux_solution.0
        } else {
            table
                .iter()
                .rev()
                .skip(1)
                .fold(
                    (
                        ilen - 1,
                        best_cost,
                        problem.max_weight,
                        vec![false; problem.items.len()],
                    ),
                    |(i, rem_cost, rem_weight, mut vec), x| {
                        let (new_cost, rem_weight) = if let Some(w) = x[rem_cost as usize] {
                            if w <= rem_weight {
                                (rem_cost, w)
                            } else {
                                vec[mappings[i]] = true;
                                let rem_cost = rem_cost - items[i].cost;
                                (rem_cost, x[rem_cost as usize].unwrap())
                            }
                        } else {
                            vec[mappings[i]] = true;
                            let rem_cost = rem_cost - items[i].cost;
                            (rem_cost, x[rem_cost as usize].unwrap())
                        };
                        (i.max(1) - 1, new_cost, rem_weight, vec)
                    },
                )
                .3
        };

        Solution {
            id: problem.id,
            size: problem.size,
            cost: best_cost * cost_gcd,
            items: Some(best_solution),
        }
    }
}
