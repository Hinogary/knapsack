use super::{ratio, Item, Problem};
use itertools::Itertools;
use std::cmp::Reverse;

pub fn calculate_practical_ftpas_error(problem: &Problem, gcd: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};
    let mut items = problem.items.clone();
    items.sort_by_key(|x| x.weight);
    #[allow(deprecated)]
    let m = items
        .iter()
        .fold_while((0, 0), |(weight, i), item| {
            if weight + item.weight <= problem.max_weight {
                Continue((weight + item.weight, i + 1))
            } else {
                Done((0, i))
            }
        })
        .into_inner()
        .1;
    let mut gcds = items
        .into_iter()
        .map(|item| item.cost % gcd)
        .collect::<Vec<_>>();
    gcds.sort_unstable_by_key(|&x| Reverse(x));
    gcds[0..m].iter().sum()
}

// returns (new items, cost/weight ratios descending, mapping [new array] -> [original array])
pub fn sort_by_cost_weight_ratio(
    items: &[Item],
    max_weight: u32,
) -> (Vec<Item>, Vec<ratio>, Vec<usize>) {
    let len = items.len();
    let mut vec = items
        .iter()
        .enumerate()
        .map(|(index, item)| (*item, ratio::new(item.cost, item.weight), index))
        .filter(|(item, _, _)| item.weight <= max_weight)
        .collect::<Vec<_>>();
    vec.sort_unstable_by_key(|x| Reverse((x.1, x.0.weight)));
    vec.into_iter().fold(
        (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        ),
        |(mut items, mut ratios, mut mappings), (item, ratio, mapping)| {
            items.push(item);
            ratios.push(ratio);
            mappings.push(mapping);
            (items, ratios, mappings)
        },
    )
}

// O(ln n)
pub fn max_cost_from_rem(rem_costs: &[u32], rem_weights: &[u32], max_weight: u32) -> u32 {
    let mut l = 0;
    let mut r = rem_costs.len() - 2;
    if rem_weights[0] <= max_weight {
        return rem_costs[0];
    }
    // binary search
    while l != r {
        let next = (l + r) / 2;
        let weight = rem_weights[0] - rem_weights[next + 1];
        if weight <= max_weight {
            l = next + 1;
        } else {
            r = next;
        }
    }
    let rem_weight = (max_weight + rem_weights[l] - rem_weights[0]).min(rem_weights[0]);
    let last_weight = rem_weights[l] - rem_weights[l + 1];
    let last_cost = rem_costs[l] - rem_costs[l + 1];
    rem_costs[0] - rem_costs[l]
        + if rem_weight == 0 {
            0
        } else {
            last_cost * last_weight / rem_weight
        }
}

// Calculates maximum possible cost ... takes sorted items by cost/weight ratios
// O(n)
pub fn max_cost(items: &[Item], max_weight: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};
    #[allow(deprecated)] // fold_while no longer deprecated in master
    items
        .iter()
        .fold_while((0, 0), |(weight, cost), x| {
            if weight + x.weight <= max_weight {
                Continue((weight + x.weight, cost + x.cost))
            } else if weight == max_weight {
                Done((0, cost))
            } else {
                Done((0, cost + x.cost * x.weight / (max_weight - weight)))
            }
        })
        .into_inner()
        .1
}

pub fn best_valued_item_fit(items: &[Item], max_weight: u32) -> (u32, usize) {
    items
        .iter()
        .enumerate()
        .fold((0, 0), |(cost, index), (i, item)| {
            if item.cost > cost && item.weight <= max_weight {
                (item.cost, i)
            } else {
                (cost, index)
            }
        })
}

pub fn calc_remaining_weight(items: &[Item]) -> Vec<u32> {
    //reverse items[..].cost, then accumule them into vector, where x[i] = x[i-1] + items[i].cost (x[-1] = 0), then again reverse
    items
        .iter()
        .rev()
        .map(|item| item.weight)
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain([0u32].iter().cloned())
        .collect()
}

pub fn calc_remaining_cost(items: &[Item]) -> Vec<u32> {
    //reverse items[..].cost, then accumule them into vector, where x[i] = x[i-1] + items[i].cost (x[-1] = 0), then again reverse
    items
        .iter()
        .rev()
        .map(|item| item.cost)
        .scan(0, |state, x| {
            *state += x;
            Some(*state)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain([0u32].iter().cloned())
        .collect()
}
