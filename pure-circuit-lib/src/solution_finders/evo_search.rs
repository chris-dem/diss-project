use std::marker::PhantomData;

use anyhow::{Result as ARes, anyhow};
use genetic_algorithm::{
    crossover,
    fitness::{self, Fitness},
    genotype::{Genotype, ListGenotype},
    mutate, select,
    strategy::{
        evolve::prelude::*,
        prelude::{HillClimb, HillClimbVariant},
    },
};
use itertools::Itertools;
use petgraph::Direction::Incoming;
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

#[derive(Debug, Clone, Copy)]
pub struct EvolutionaryAlgorithm;

#[derive(Debug, Clone, Copy, Default)]
pub struct HillClimbAlgorithm<G: Fitness> {
    phantom_data: PhantomData<G>,
}

pub type SolverHillClimb = SolverStruct<HillClimbAlgorithm<FitnessPureCircuit>>;

// pub struct EvoParamSet {
//     pub fitness_func: FitnessPureCircuit,
//     pub gene_size: usize,
//     pub gene_hashing: bool,
//     pub selection: select::SelectWrapper,
//     pub crossover: crossover::CrossoverWrapper,
//     pub mutate: mutate::MutateWrapper,
//     pub fitness_cache: usize,
//     pub stale_generations: usize,
//     pub population_size: usize,
// }

// impl SolverTrait for SolverStruct<EvolutionaryAlgorithm> {
//     type ParamSet = EvoParamSet;
//     type Solution = SolutionReturn;
//     fn find_solution(&self, param_set: Self::ParamSet) -> Option<Self::Solution> {
//         let genotype = ListGenotype::builder()
//             .with_allele_list(Value::iter().collect_vec())
//             .with_genes_size(param_set.gene_size)
//             .with_genes_hashing(param_set.gene_hashing)
//             .build()
//             .ok()?;

//         let mut evolve = Evolve::builder()
//             .with_genotype(genotype.clone())
//             .with_target_population_size(param_set.population_size)
//             // Experiment
//             // .with_select(SelectElite::new(0.5, 0.02))
//             .with_select(param_set.selection)
//             // .with_crossover(CrossoverUniform::new(0.7, 0.8))
//             .with_crossover(param_set.crossover)
//             // .with_mutate(MutateSingleGene::new(0.2))
//             .with_mutate(param_set.mutate)
//             .with_fitness(param_set.fitness_func)
//             .with_fitness_ordering(FitnessOrdering::Minimize)
//             .with_fitness_cache(param_set.fitness_cache)
//             .with_target_fitness_score(0)
//             // .with_reporter(EvolveReporterSimple::new(100))
//             .with_max_stale_generations(param_set.stale_generations)
//             .build()
//             .ok()?;
//         evolve.call();
//         let (a, b) = evolve.best_genes_and_fitness_score()?;

//         Some(SolutionReturn {
//             chromosone: a,
//             errors: b as usize,
//         })
//     }
// }

pub struct Instance<T: Fitness<Genotype = ListGenotype<Value>>> {
    func: T,
    size: usize,
}

impl<T: Fitness<Genotype = ListGenotype<Value>>> Instance<T> {
    pub fn new(func: T, size: usize) -> Self {
        Self { func, size }
    }
}

pub struct Build;

pub struct HillParamSet<T> {
    pub param_type: T,
    pub gene_hashing: bool,
    pub hill_variant: HillClimbVariant,
    pub fitness_cache: usize,
    pub stale_generations: usize,
    pub population_size: usize,
}

impl Default for HillParamSet<Build> {
    fn default() -> Self {
        Self {
            param_type: Build,
            gene_hashing: true,
            hill_variant: HillClimbVariant::Stochastic,
            fitness_cache: 250,
            stale_generations: 10_000,
            population_size: 250,
        }
    }
}

impl<G: Fitness<Genotype = ListGenotype<Value>>> HillParamSet<Instance<G>> {
    pub fn build(instance: Instance<G>, builder: Option<HillParamSet<Build>>) -> Self {
        let t = builder.unwrap_or_default();
        Self {
            param_type: instance,
            gene_hashing: t.gene_hashing,
            hill_variant: t.hill_variant,
            fitness_cache: t.fitness_cache,
            stale_generations: t.stale_generations,
            population_size: t.population_size,
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

        let mut hill_climb = HillClimb::builder()
            .with_genotype(genotype.clone())
            .with_variant(param_set.hill_variant)
            .with_max_stale_generations(param_set.stale_generations)
            .with_fitness(param_set.param_type.func)
            .with_fitness_cache(param_set.fitness_cache)
            .with_fitness_ordering(FitnessOrdering::Minimize)
            .with_target_fitness_score(0)
            .build()
            .map_err(|e| anyhow!(e.0))?;

        hill_climb.call();
        let (a, b) = hill_climb
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
    use mockall::*;

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

    // fn setup_bad_graph() -> PureCircuitGraph {
    //     let mut pc_resource = PureCircuitGraph::new();
    //     let val_1 = pc_resource.add_node(NodeUnitialised::from_value(Value::Bot), ());
    //     let val_2 = pc_resource.add_node(NodeUnitialised::from_value(Value::Bot), ());
    //     let gate_1 = pc_resource.add_node(NodeUnitialised::from_gate(Gate::And), ());
    //     pc_resource.add_edge(val_1, gate_1, ()).unwrap();
    //     pc_resource.add_edge(val_2, gate_1, ()).unwrap();
    //     pc_resource
    // }

    #[test]
    fn should_run_for_correct() {
        let pc = setup_good_graph();
        let fit = pc.to_fitness_function().unwrap();
        let ind = pc.graph.node_count();
        let s = SolverStruct::<HillClimbAlgorithm<FitnessPureCircuit>>::default();
        let params = HillParamSet::build(
            Instance {
                func: fit,
                size: ind,
            },
            None,
        );

        assert!(s.find_solution(params).is_ok());
    }

    mock! {
        MyStruct {}     // Name of the mock struct, less the "Mock" prefix
        impl Clone for MyStruct {
            fn clone(&self) -> Self;
        }

        impl Debug for MyStruct {
            fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result;
        }

        impl Fitness for MyStruct {
            type Genotype = ListGenotype<Value>;
            fn calculate_for_chromosome(
                &mut self,
                _chromosome: &FitnessChromosome<Self>,
                _genotype: &<MockMyStruct as Fitness>::Genotype,
            ) -> Option<FitnessValue>;
        }
    }

    #[test]
    fn should_run_for_correct_mock() {
        let pc = setup_good_graph();
        let mut fit = MockMyStruct::new();
        fit.expect_calculate_for_chromosome().return_const(None);
        let ind = pc.graph.node_count();
        let s = SolverStruct::<HillClimbAlgorithm<_>>::default();
        let params = HillParamSet::build(
            Instance {
                func: fit,
                size: ind,
            },
            None,
        );

        assert!(s.find_solution(params).is_err());
    }
}
