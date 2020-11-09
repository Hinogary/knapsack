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

use enum_dispatch::enum_dispatch;

pub use super::Opts;

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
}
pub use Solver::*;

#[enum_dispatch(Solver)]
pub trait SolverTrait {
    fn construction(&self, problem: &Problem) -> Solution;
}

impl Solver {
    pub fn is_exact(&self) -> bool {
        match self {
            Naive(_) | Pruning(_) | DynamicWeight(_) | DynamicCost(_) => true,
            Greedy(_) | Redux(_) | FTPAS(_) => false,
        }
    }

    pub fn from_opts(opts: &Opts) -> Solver {
        match (
            opts.naive,
            opts.pruning,
            opts.dynamic_weight,
            opts.dynamic_cost,
            opts.greedy,
            opts.redux,
            opts.ftpas.is_some(),
        ) {
            (true, false, false, false, false, false, false) => Naive(NaiveSolver()),
            (false, true, false, false, false, false, false) => Pruning(PruningSolver()),
            (false, false, true, false, false, false, false) => {
                DynamicWeight(DynamicWeightSolver())
            }
            (false, false, false, true, false, false, false) => DynamicCost(DynamicCostSolver()),
            (false, false, false, false, true, false, false) => Greedy(GreedySolver()),
            (false, false, false, false, false, true, false) => Redux(ReduxSolver()),
            (false, false, false, false, false, false, true) => FTPAS(FTPASSolver {
                gcd: opts.ftpas.unwrap(),
            }),
            (false, false, false, false, false, false, false) => panic!("Not any solver selected!"),
            _ => panic!("Too many solvers selected!"),
        }
    }
}
