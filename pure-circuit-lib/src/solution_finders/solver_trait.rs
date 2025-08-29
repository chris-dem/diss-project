use anyhow::Result;
/// A trait for solvers that can find solutions given a set of parameters.
/// 
/// # Associated Types
/// * `ParamSet` - The type of parameters required by the solver
/// * `Solution` - The type of solution the solver produces
pub trait SolverTrait {
    /// The parameter set type required by this solver.
    type ParamSet;
    
    /// The solution type produced by this solver.
    type Solution;

    /// Finds a solution using the given parameters.
    /// 
    /// # Parameters
    /// * `param_set` - The parameters needed to solve the problem
    /// 
    /// # Returns
    /// Returns the solution if successful, or an error if solving fails.
    fn find_solution(&self, param_set: Self::ParamSet) -> Result<Self::Solution>;
}
