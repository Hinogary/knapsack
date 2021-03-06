use super::{Item, Problem};
use itertools::Itertools;
use std::cmp::Reverse;

pub fn calculate_practical_ftpas_error(problem: &Problem, gcd: u32) -> u32 {
    use itertools::FoldWhile::{Continue, Done};
    #[allow(deprecated)] // fold_while no longer deprecated in master
    let m = problem
        .items
        .iter()
        .sorted_by_key(|x| x.weight)
        .fold_while((0, 0), |(weight, i), item| {
            if weight + item.weight <= problem.max_weight {
                Continue((weight + item.weight, i + 1))
            } else {
                Done((0, i))
            }
        })
        .into_inner()
        .1;
    let gcds = problem
        .items
        .iter()
        .filter(|item| item.weight <= problem.max_weight)
        .map(|item| item.cost % gcd)
        .sorted_by_key(|&x| Reverse(x))
        .collect::<Vec<_>>();
    gcds[0..m].iter().sum()
}

// returns (new items, cost/weight ratios descending, mapping [new array] -> [original array])
pub fn sort_by_cost_weight_ratio(items: &[Item], max_weight: u32) -> (Vec<Item>, Vec<usize>) {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| (*item, index))
        .filter(|(item, _)| item.weight <= max_weight)
        .sorted_by_key(|x| Reverse((x.0.cost_weight_ratio(), x.0.weight)))
        .unzip()
}

// O(ln n)
pub fn max_cost_from_rem(rem_costs: &[u32], rem_weights: &[u32], max_weight: u32) -> u32 {
    // skips first weights, that are less than max_weight, speed ups easy cases, slower hard cases
    let skip = {
        rem_weights
            .iter()
            .zip(rem_weights.iter().skip(1))
            .map(|(l, r)| l - r)
            .enumerate()
            .find(|(_, w)| *w <= max_weight)
            .map(|x| x.0)
            .unwrap_or(rem_costs.len() - 2)
    } as usize;
    let rem_costs = &rem_costs[skip..];
    let rem_weights = &rem_weights[skip..];
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

/// final result is k[i] = f(ts)[i..x.len()].sum() in O(n) time with 2 allocations
fn desc_sum_vec_with_fn<T, K, F>(ts: &[T], f: F) -> Vec<K>
where
    F: Fn(&T) -> K,
    K: std::ops::AddAssign<K>,
    K: Clone,
    K: Default,
{
    // reverse ts, then accumule them into vector, where x[i] = x[i-1] + f(t), (x[-1] = 0), then again reverse
    ts.iter()
        .rev()
        .map(f)
        .scan(K::default(), |state, x| {
            *state += x;
            Some(state.clone())
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .chain([K::default()].iter().cloned())
        .collect()
}

pub fn calc_remaining_weight(items: &[Item]) -> Vec<u32> {
    desc_sum_vec_with_fn(items, |item| item.weight)
}

pub fn calc_remaining_cost(items: &[Item]) -> Vec<u32> {
    desc_sum_vec_with_fn(items, |item| item.cost)
}
