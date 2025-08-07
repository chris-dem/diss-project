pub trait SolverTrait {
    type ParamSet;
    type Solution;

    fn find_solution(&self, param_set: Self::ParamSet) -> Option<Self::Solution>;
    fn find_solutions(&self, param_set: Self::ParamSet) -> impl Iterator<Item = Self::Solution>;
}
