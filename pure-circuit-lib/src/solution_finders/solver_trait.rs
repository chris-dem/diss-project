use anyhow::Result;
pub trait SolverTrait {
    type ParamSet;
    type Solution;

    fn find_solution(&self, param_set: Self::ParamSet) -> Result<Self::Solution>;
}

pub trait TotalSolverTrait: SolverTrait {
    type ManySolution;
    fn find_solutions(&self, param_set: Self::ParamSet)
    -> impl Iterator<Item = Self::ManySolution>;
}
