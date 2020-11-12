use super::{utils::sort_by_cost_weight_ratio, Item, Problem, Solution, SolverTrait};
use gcd::Gcd;

#[derive(Debug, Clone)]
pub struct DynamicWeightSolver();

impl SolverTrait for DynamicWeightSolver {
    fn construction(&self, problem: &Problem) -> Solution {
        // backtracking only
        let (mut items, _ratios, mut mapping) =
            sort_by_cost_weight_ratio(&problem.items, problem.max_weight);

        if items.is_empty() {
            items.push(Item { weight: 1, cost: 0 });
            mapping.push(0);
        }

        let gcd = items
            .iter()
            .fold(items[0].weight, |acc, x| acc.gcd(x.weight));

        if gcd > 1 {
            for item in &mut items {
                item.weight /= gcd;
            }
        }

        let max_weight = problem.max_weight / gcd;

        let size = (problem.max_weight / gcd) as usize + 1;
        let ilen = items.len();

        let mut table_raw: Vec<Option<(u32, bool)>> = vec![None; size * (ilen + 1)];
        let mut table_base = table_raw
            .as_mut_slice()
            .chunks_mut(size)
            .collect::<Vec<_>>();

        //tabulka je převrácená oproti přednáškám .. řešení je v [0][0]
        //první index určuje přidané předměty .. např i => jsou přidané věci i..ilen
        //druhý index určuje zbývající kapacitu a nevyužitá kapacita je "nahoře"

        let table = table_base.as_mut_slice();

        // last row is having zero cost
        for x in table.last_mut().unwrap().iter_mut() {
            *x = Some((0, false));
        }

        // DFS průchod

        let mut stack = Vec::with_capacity(ilen);

        stack.push((0usize, 0u32));
        while !stack.is_empty() {
            let (item, weight) = stack.last().unwrap();
            let with_item = (item + 1, weight + items[*item].weight);
            let without_item = (item + 1, *weight);
            let mut me_cell = None;
            if with_item.1 <= max_weight {
                if let Some(cell) = table[with_item.0][with_item.1 as usize] {
                    me_cell = Some((cell.0 + items[*item].cost, true));
                } else {
                    stack.push(with_item);
                    continue;
                }
            }
            if let Some(cell) = table[without_item.0][without_item.1 as usize] {
                let cost_with_item = me_cell.map(|x| x.0).unwrap_or(0);
                if cell.0 >= cost_with_item {
                    me_cell = Some((cell.0, false));
                }
            } else {
                stack.push(without_item);
                continue;
            }
            table[*item][*weight as usize] = me_cell;
            stack.pop();
        }
        Solution {
            id: problem.id,
            size: problem.size,
            cost: table[0][0].unwrap().0,
            items: Some(
                table
                    .iter()
                    .take(ilen) //skip last
                    .fold(
                        (0, 0u32, vec![false; problem.items.len()]),
                        |(i, w, mut vec), x| {
                            let added = x[w as usize].unwrap().1;
                            vec[mapping[i]] = added;
                            (i + 1, if added { w + items[i].weight } else { w }, vec)
                        },
                    )
                    .2,
            ),
        }
    }
}
