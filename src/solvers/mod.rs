pub mod utils;
pub use utils::*;

mod naive;
use naive::NaiveSolver;

mod pruning;
use pruning::PruningSolver;

mod dynamic_weight;
use dynamic_weight::DynamicWeightSolver;

mod dynamic_cost;
use dynamic_cost::DynamicCostSolver;

mod ftpas;
use ftpas::FTPASSolver;

mod greedy;
use greedy::GreedySolver;

mod redux;
use redux::ReduxSolver;

mod tabusearch;
use tabusearch::TabuSearchSolver;

mod approx_pruning;
use approx_pruning::ApproxPruningSolver;

use enum_dispatch::enum_dispatch;

pub use super::Opts;

use super::DisplayError;
pub use super::{Item, Problem, Solution};

use num_rational::Ratio;

#[allow(non_camel_case_types)]
pub type ratio = Ratio<u32>;

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum Solver {
    Naive(NaiveSolver),
    Pruning(PruningSolver),
    DynamicWeight(DynamicWeightSolver),
    DynamicCost(DynamicCostSolver),
    Greedy(GreedySolver),
    Redux(ReduxSolver),
    FTPAS(FTPASSolver),
    TabuSearch(TabuSearchSolver),
    ApproxPruning(ApproxPruningSolver),
}
pub use Solver::*;

#[derive(Debug, Clone, Copy)]
pub enum Methods {
    Naive,
    Pruning,
    DynamicWeight,
    DynamicCost,
    Greedy,
    Redux,
    FTPAS,
    TabuSearch,
    ApproxPruning,
}

use itertools::Itertools;
use std::str::FromStr;

impl FromStr for Methods {
    type Err = DisplayError;
    fn from_str(name: &str) -> Result<Methods, DisplayError> {
        let methods = [
            ("naive", Self::Naive),
            ("pruning", Self::Pruning),
            ("dynamic-weight", Self::DynamicWeight),
            ("dynamic-cost", Self::DynamicCost),
            ("greedy", Self::Greedy),
            ("redux", Self::Redux),
            ("ftpas", Self::FTPAS),
            ("tabu-search", Self::TabuSearch),
            ("approx-pruning", Self::ApproxPruning),
        ];
        methods
            .iter()
            .map(|(method_name, method)| {
                if *method_name == name {
                    Some(Ok(*method))
                } else {
                    None
                }
            })
            .find(|x| x.is_some())
            .unwrap_or(None)
            .unwrap_or_else(|| {
                Err(format!(
                    "Method {:?} not found, following are valid: {}.",
                    name,
                    methods.iter().map(|x| x.0).join(", ")
                )
                .into())
            })
    }
}

#[enum_dispatch(Solver)]
pub trait SolverTrait {
    fn construction(&self, problem: &Problem) -> Solution;
    // method can specialize better decision
    fn decision(&self, problem: &Problem) -> Solution {
        let constr_sol = self.construction(problem);
        if constr_sol.cost >= problem.min_cost.unwrap() {
            constr_sol
        } else {
            Solution::none(problem.id, problem.size)
        }
    }
}

impl Solver {
    pub fn is_exact(&self) -> bool {
        match self {
            Naive(_) | Pruning(_) | DynamicWeight(_) | DynamicCost(_) => true,
            Greedy(_) | Redux(_) | FTPAS(_) | ApproxPruning(_) | TabuSearch(_) => false,
        }
    }

    pub fn from_opts(opts: &Opts) -> Result<Solver, DisplayError> {
        Ok(match opts.method {
            Methods::Naive => Naive(NaiveSolver()),
            Methods::Pruning => Pruning(PruningSolver()),
            Methods::DynamicWeight => DynamicWeight(DynamicWeightSolver()),
            Methods::DynamicCost => DynamicCost(DynamicCostSolver()),
            Methods::Greedy => Greedy(GreedySolver()),
            Methods::Redux => Redux(ReduxSolver()),
            Methods::FTPAS => FTPAS(FTPASSolver {
                gcd: if let Some(p) = opts.precision {
                    p
                } else {
                    return Err("Missing precision option.".into());
                },
            }),
            Methods::ApproxPruning => ApproxPruning(ApproxPruningSolver {
                precision: if let Some(p) = opts.precision {
                    p
                } else {
                    return Err("Missing precision option.".into());
                },
            }),
            Methods::TabuSearch => TabuSearch(TabuSearchSolver {
                memory_size: if let Some(m) = opts.memory_size {
                    m
                } else {
                    return Err("Missing memory option.".into());
                },
                iterations: if let Some(i) = opts.iterations {
                    i
                } else {
                    return Err("Missing iterations option.".into());
                },
            }),
        })
    }
}
