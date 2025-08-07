use crate::solution_finders::solver_trait::SolverTrait;

#[cfg(test)]
mod test_evo {

    use std::ops::Range;

    use super::*;
    use genetic_algorithm::strategy::evolve::prelude::*;
    use genetic_algorithm::strategy::hill_climb::prelude::*;
    use itertools::Itertools;

    #[derive(Debug, Clone)]
    struct PurifyFitness;

    impl Fitness for PurifyFitness {
        type Genotype = RangeGenotype<u8>;

        fn calculate_for_chromosome(
            &mut self,
            chromosome: &FitnessChromosome<Self>,
            _genotype: &Self::Genotype,
        ) -> Option<FitnessValue> {
            let mut score = (chromosome.genes[0] != 1) as isize;
            for ((ind, s1), s2) in chromosome
                .genes
                .iter()
                .copied()
                .zip(chromosome.genes.iter().copied().skip(1))
                .zip(chromosome.genes.iter().copied().skip(2))
                .step_by(2)
            {
                score += 1 - match ind {
                    s @ (0 | 2) => (s1, s2) == (s, s),
                    1 => [(0, 1), (1, 2), (0, 2)].contains(&(s1, s2)),
                    d => {
                        panic!("Something went wrong {d}");
                    }
                } as isize;
            }
            Some(score)
        }
    }

    #[test]

    fn test_lib() {
        env_logger::init();
        let genotype = RangeGenotype::builder()
            .with_genes_size(3 + 10 * 2)
            .with_allele_range(0u8..=2u8)
            .with_genes_hashing(true)
            .build()
            .unwrap();

        println!("{genotype}");

        let mut evolve = Evolve::builder()
            .with_genotype(genotype.clone())
            .with_target_population_size(100)
            .with_select(SelectElite::new(0.5, 0.02))
            .with_crossover(CrossoverUniform::new(0.7, 0.8))
            .with_mutate(MutateSingleGene::new(0.2))
            .with_fitness(PurifyFitness)
            .with_fitness_ordering(FitnessOrdering::Minimize)
            .with_fitness_cache(10_000)
            .with_target_fitness_score(0)
            // .with_reporter(EvolveReporterSimple::new(100))
            .with_max_stale_generations(10_000)
            .build()
            .unwrap();

        let mut hill_climb = HillClimb::builder()
            .with_genotype(genotype.clone())
            .with_variant(HillClimbVariant::Stochastic)
            .with_max_stale_generations(10000)
            // .with_variant(HillClimbVariant::SteepestAscent)
            // .with_max_stale_generations(5) // needs to a little bit above 1
            .with_fitness(PurifyFitness)
            .with_fitness_ordering(FitnessOrdering::Minimize)
            .with_target_fitness_score(0)
            .build()
            .unwrap();

        if let Some((best_genes, fitness_score)) = evolve.best_genes_and_fitness_score() {
            let p = best_genes
                .iter()
                .rev()
                .enumerate()
                .filter_map(|(ind, el)| {
                    if ind < 2 || ind % 2 == 1 {
                        Some(*el)
                    } else {
                        None
                    }
                })
                .rev()
                .collect_vec();
            dbg!(p);
            if fitness_score == 0 {
                println!("Valid solution with fitness score: {}", fitness_score);
            } else {
                println!("Wrong solution with fitness score: {}", fitness_score);
            }
        } else {
            println!("Invalid solution with fitness score: None");
        }

        println!("HILL CLIMB");
        if let Some((best_genes, fitness_score)) = hill_climb.best_genes_and_fitness_score() {
            let p = best_genes
                .iter()
                .rev()
                .enumerate()
                .filter_map(|(ind, el)| {
                    if ind < 2 || ind % 2 == 1 {
                        Some(*el)
                    } else {
                        None
                    }
                })
                .rev()
                .collect_vec();
            dbg!(p);
            if fitness_score == 0 {
                println!("Valid solution with fitness score: {}", fitness_score);
            } else {
                println!("Wrong solution with fitness score: {}", fitness_score);
            }
        } else {
            println!("Invalid solution with fitness score: None");
        }
    }
}
