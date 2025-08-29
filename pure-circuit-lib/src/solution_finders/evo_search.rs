use std::marker::PhantomData;

use anyhow::{Result as ARes, anyhow};
use genetic_algorithm::{
    crossover,
    fitness::Fitness,
    genotype::{Genotype, ListGenotype},
    mutate, select,
    strategy::{
        evolve::prelude::*,
        hill_climb,
        prelude::{HillClimb, HillClimbVariant},
    },
};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    gates::Value,
    solution_finders::{base_finder::FitnessPureCircuit, solver_trait::SolverTrait},
};

impl Fitness for FitnessPureCircuit {
    type Genotype = ListGenotype<Value>;
    fn calculate_for_chromosome(
        &mut self,
        chromosome: &genetic_algorithm::fitness::prelude::FitnessChromosome<Self>,
        _genotype: &Self::Genotype,
    ) -> Option<genetic_algorithm::fitness::prelude::FitnessValue> {
        self.evaluate(&chromosome.genes).map(|x| x as isize)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SolverStruct<T> {
    phantom_data: PhantomData<T>,
}

#[derive(Debug)]
pub struct SolutionReturn {
    pub chromosone: Vec<Value>,
    pub errors: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvolutionaryAlgorithm<G: Fitness> {
    phantom_data: PhantomData<G>,
}
pub type SolverEvo = SolverStruct<EvolutionaryAlgorithm<FitnessPureCircuit>>;

#[derive(Debug, Clone, Copy, Default)]
pub struct HillClimbAlgorithm<G: Fitness> {
    phantom_data: PhantomData<G>,
}

pub type SolverHillClimb = SolverStruct<HillClimbAlgorithm<FitnessPureCircuit>>;

#[derive(Debug, Clone, Copy)]
pub struct NewType<T>(pub T);

impl PartialEq for NewType<HillClimbVariant> {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self.0, other.0),
            (
                HillClimbVariant::SteepestAscent,
                HillClimbVariant::SteepestAscent
            ) | (HillClimbVariant::Stochastic, HillClimbVariant::Stochastic)
        )
    }
}

impl Eq for NewType<HillClimbVariant> {}

impl PartialEq for NewType<SelectWrapper> {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (&self.0, &other.0),
            (SelectWrapper::Tournament(_), SelectWrapper::Tournament(_))
                | (SelectWrapper::Elite(_), SelectWrapper::Elite(_))
        )
    }
}

impl Eq for NewType<SelectWrapper> {}

impl PartialEq for NewType<CrossoverWrapper> {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (&self.0, &other.0),
            (CrossoverWrapper::Clone(_), CrossoverWrapper::Clone(_))
                | (
                    CrossoverWrapper::MultiGene(_),
                    CrossoverWrapper::MultiGene(_)
                )
                | (
                    CrossoverWrapper::MultiPoint(_),
                    CrossoverWrapper::MultiPoint(_)
                )
                | (
                    CrossoverWrapper::Rejuvenate(_),
                    CrossoverWrapper::Rejuvenate(_)
                )
                | (
                    CrossoverWrapper::SingleGene(_),
                    CrossoverWrapper::SingleGene(_)
                )
                | (
                    CrossoverWrapper::SinglePoint(_),
                    CrossoverWrapper::SinglePoint(_)
                )
                | (CrossoverWrapper::Uniform(_), CrossoverWrapper::Uniform(_))
        )
    }
}

impl Eq for NewType<CrossoverWrapper> {}

impl PartialEq for NewType<MutateWrapper> {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (&self.0, &other.0),
            (MutateWrapper::MultiGene(_), MutateWrapper::MultiGene(_))
                | (
                    MutateWrapper::MultiGeneDynamic(_),
                    MutateWrapper::MultiGeneDynamic(_)
                )
                | (
                    MutateWrapper::MultiGeneRange(_),
                    MutateWrapper::MultiGeneRange(_)
                )
                | (MutateWrapper::SingleGene(_), MutateWrapper::SingleGene(_))
                | (
                    MutateWrapper::SingleGeneDynamic(_),
                    MutateWrapper::SingleGeneDynamic(_)
                )
        )
    }
}

impl Eq for NewType<MutateWrapper> {}

#[derive(Debug, Clone)]
pub struct EvoParamSet<T> {
    pub param_type: T,
    pub gene_hashing: bool,
    pub selection: NewType<select::SelectWrapper>,
    pub crossover: NewType<crossover::CrossoverWrapper>,
    pub mutate: NewType<mutate::MutateWrapper>,
    pub fitness_cache: usize,
    pub stale_generations: usize,
    pub population_size: usize,
    pub num_of_species: usize,
    pub with_parallel: bool,
}

impl Default for EvoParamSet<Build> {
    fn default() -> Self {
        Self {
            param_type: Build,
            gene_hashing: true,
            fitness_cache: 250,
            stale_generations: 10_000,
            population_size: 250,
            num_of_species: 15,
            selection: NewType(SelectWrapper::Elite(SelectElite::new(0.05, 0.02))),
            crossover: NewType(CrossoverWrapper::Uniform(CrossoverUniform::new(0.5, 0.1))),
            mutate: NewType(MutateWrapper::MultiGene(MutateMultiGene::new(1, 0.2))),
            with_parallel: false,
        }
    }
}

impl EvoParamSet<Build> {
    pub fn build<G: Fitness<Genotype = ListGenotype<Value>>>(
        &self,
        instance: Instance<G>,
    ) -> EvoParamSet<Instance<G>> {
        EvoParamSet {
            param_type: instance,
            gene_hashing: self.gene_hashing,
            fitness_cache: self.fitness_cache,
            stale_generations: self.stale_generations,
            population_size: self.population_size,
            selection: self.selection.clone(),
            crossover: self.crossover.clone(),
            mutate: self.mutate.clone(),
            num_of_species: self.num_of_species,
            with_parallel: self.with_parallel,
        }
    }
}

impl<G: Fitness<Genotype = ListGenotype<Value>>> SolverTrait
    for SolverStruct<EvolutionaryAlgorithm<G>>
{
    type ParamSet = EvoParamSet<Instance<G>>;

    type Solution = SolutionReturn;

    fn find_solution(&self, param_set: Self::ParamSet) -> ARes<Self::Solution> {
        let genotype = ListGenotype::builder()
            .with_allele_list(Value::iter().collect_vec())
            .with_genes_size(param_set.param_type.size)
            .with_genes_hashing(param_set.gene_hashing)
            .build()
            .map_err(|e| anyhow!(e.0))?;

        let evolve = Evolve::builder()
            .with_genotype(genotype.clone())
            .with_target_population_size(param_set.population_size)
            // Experiment
            .with_select(param_set.selection.0)
            .with_crossover(param_set.crossover.0)
            .with_mutate(param_set.mutate.0)
            .with_fitness(param_set.param_type.func)
            .with_fitness_ordering(FitnessOrdering::Minimize)
            .with_fitness_cache(param_set.fitness_cache)
            .with_target_fitness_score(0)
            .with_max_stale_generations(param_set.stale_generations);

        let evolve = if param_set.with_parallel {
            evolve
                .with_par_fitness(true)
                .call_par_speciated(param_set.num_of_species)
                .map_err(|e| anyhow!(e.0))?
        } else {
            evolve
                .with_par_fitness(false)
                .call_speciated(param_set.num_of_species)
                .map_err(|e| anyhow!(e.0))?
        };
        let (a, b) = evolve
            .0
            .best_genes_and_fitness_score()
            .ok_or(anyhow!("Error when computing score"))?;

        Ok(SolutionReturn {
            chromosone: a,
            errors: b as usize,
        })
    }
}

pub struct Instance<T: Fitness<Genotype = ListGenotype<Value>>> {
    func: T,
    size: usize,
}

impl<T: Fitness<Genotype = ListGenotype<Value>>> Instance<T> {
    pub fn new(func: T, size: usize) -> Self {
        Self { func, size }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Build;

#[derive(Clone, Debug, Copy)]
pub struct HillParamSet<T> {
    pub param_type: T,
    pub gene_hashing: bool,
    pub hill_variant: NewType<HillClimbVariant>,
    pub fitness_cache: usize,
    pub stale_generations: usize,
    pub population_size: usize,
    pub num_of_runs: usize,
    pub with_parallel: bool,
}

impl Default for HillParamSet<Build> {
    fn default() -> Self {
        Self {
            param_type: Build,
            gene_hashing: true,
            hill_variant: NewType(HillClimbVariant::Stochastic),
            fitness_cache: 250,
            stale_generations: 10_000,
            population_size: 250,
            num_of_runs: 15,
            with_parallel: false,
        }
    }
}

impl HillParamSet<Build> {
    pub fn build<G: Fitness<Genotype = ListGenotype<Value>>>(
        &self,
        instance: Instance<G>,
    ) -> HillParamSet<Instance<G>> {
        HillParamSet {
            param_type: instance,
            gene_hashing: self.gene_hashing,
            hill_variant: self.hill_variant,
            fitness_cache: self.fitness_cache,
            stale_generations: self.stale_generations,
            population_size: self.population_size,
            num_of_runs: self.num_of_runs,
            with_parallel: self.with_parallel,
        }
    }
}

impl<G: Fitness<Genotype = ListGenotype<Value>>> SolverTrait
    for SolverStruct<HillClimbAlgorithm<G>>
{
    type ParamSet = HillParamSet<Instance<G>>;

    type Solution = SolutionReturn;

    fn find_solution(&self, param_set: Self::ParamSet) -> ARes<Self::Solution> {
        let genotype = ListGenotype::builder()
            .with_allele_list(Value::iter().collect_vec())
            .with_genes_size(param_set.param_type.size)
            .with_genes_hashing(param_set.gene_hashing)
            .build()
            .map_err(|e| anyhow!(e.0))?;

        let hill_climb = HillClimb::builder()
            .with_genotype(genotype.clone())
            .with_variant(param_set.hill_variant.0)
            .with_max_stale_generations(param_set.stale_generations)
            .with_fitness(param_set.param_type.func)
            .with_fitness_cache(param_set.fitness_cache)
            .with_fitness_ordering(FitnessOrdering::Minimize)
            .with_target_fitness_score(0);

        let hill_climb = if param_set.with_parallel {
            hill_climb
                .with_par_fitness(true)
                .call_par_repeatedly(50)
                .map_err(|e| anyhow!(e.0))?
        } else {
            hill_climb
                .with_par_fitness(false)
                .call_repeatedly(50)
                .map_err(|e| anyhow!(e.0))?
        };

        let (a, b) = hill_climb
            .0
            .best_genes_and_fitness_score()
            .ok_or(anyhow!("Error when computing score"))?;

        Ok(SolutionReturn {
            chromosone: a,
            errors: b as usize,
        })
    }
}

#[cfg(test)]
mod evo_testers {
    use std::fmt::Debug;

    use crate::{
        gates::{Gate, NodeUnitialised},
        graph::PureCircuitGraph,
    };
    use itertools::all;

    use super::*;
    fn setup_good_graph() -> PureCircuitGraph {
        let mut pc_resource = PureCircuitGraph::new();
        let val_1 = pc_resource.add_node(NodeUnitialised::from_value(Value::Bot), ());
        let val_2 = pc_resource.add_node(NodeUnitialised::from_value(Value::Bot), ());
        let val_3 = pc_resource.add_node(NodeUnitialised::from_value(Value::Bot), ());
        let gate_1 = pc_resource.add_node(NodeUnitialised::from_gate(Gate::And), ());
        pc_resource.add_edge(val_1, gate_1, ()).unwrap();
        pc_resource.add_edge(val_2, gate_1, ()).unwrap();
        pc_resource.add_edge(gate_1, val_3, ()).unwrap();
        pc_resource
    }
}
