use std::collections::HashMap;

use genetic_algorithm::{
    allele::Allele,
    fitness::Fitness,
    genotype::{ListGenotype, RangeGenotype},
};
use itertools::Itertools;
use petgraph::{Direction, graph::NodeIndex};
use proptest::array::UniformArrayStrategy;

use crate::{
    gates::{Gate, NodeValue, Value},
    graph::PureCircuitGraph,
};

const MAX_DEGREE: usize = 2;
type Inner = (
    Gate,
    [Option<usize>; MAX_DEGREE],
    [Option<usize>; MAX_DEGREE],
);

#[derive(Debug, Clone)]
pub struct FitnessPureCircuit(Box<[Inner]>);

impl Allele for Value {}

impl Fitness for FitnessPureCircuit {
    type Genotype = ListGenotype<Value>;
    fn calculate_for_chromosome(
        &mut self,
        chromosome: &genetic_algorithm::fitness::prelude::FitnessChromosome<Self>,
        _genotype: &Self::Genotype,
    ) -> Option<genetic_algorithm::fitness::prelude::FitnessValue> {
        let mut errors = 0isize;
        let t = &chromosome.genes;
        for (g, ins, outs) in self.0.iter().copied() {
            let ins = ins
                .into_iter()
                .filter_map(|s| s.map(|ind| t[ind]))
                .collect_vec();
            let outs = outs
                .into_iter()
                .filter_map(|s| s.map(|ind| t[ind]))
                .collect_vec();
            match g.check(&ins, &outs) {
                Err(e) => {
                    dbg!(e);
                    return None;
                }
                Ok(b) => errors += !b as isize,
            }
        }

        Some(errors)
    }
}

impl<T, G> PureCircuitGraph<T, G> {
    fn to_chromosone(&self) -> Box<[Value]> {
        self.graph
            .node_weights()
            .filter_map(|n| match n.into_node() {
                NodeValue::ValueNode(value) => Some(value),
                NodeValue::GateNode { .. } => None,
            })
            .collect()
    }

    fn to_fitness_function(&self) -> Option<FitnessPureCircuit> {
        let map = self
            .graph
            .node_indices()
            .filter_map(|i| match self.graph[i].into_node() {
                NodeValue::ValueNode(_) => Some(i),
                NodeValue::GateNode { .. } => None,
            })
            .enumerate()
            .map(|(a, b)| (b, a))
            .collect::<HashMap<NodeIndex, usize>>();

        let mapper = self
            .graph
            .node_indices()
            .filter_map(|i| match self.graph[i].into_node() {
                NodeValue::ValueNode(_) => None,
                NodeValue::GateNode { gate, .. } => Some((i, gate)),
            })
            .map(|(nod_ind, gate)| {
                let mut ret_in = [None; MAX_DEGREE];
                let mut ret_out = [None; MAX_DEGREE];
                for (node, ind) in self
                    .graph
                    .neighbors_directed(nod_ind, Direction::Incoming)
                    .collect_vec()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    ret_in[node] = Some(ind)
                }

                for (node, ind) in self
                    .graph
                    .neighbors_directed(nod_ind, Direction::Outgoing)
                    .collect_vec()
                    .into_iter()
                    .rev()
                    .enumerate()
                {
                    ret_out[node] = Some(ind)
                }

                (gate, ret_in, ret_out)
            })
            .map::<Option<Inner>, _>(|(g, in_ind, out_ind)| {
                if g.arity()
                    == (
                        in_ind.iter().copied().filter(Option::is_some).count(),
                        out_ind.iter().copied().filter(Option::is_some).count(),
                    )
                {
                    let in_ind = in_ind
                        .iter()
                        .copied()
                        .map(|e| e.map(|e| *map.get(&e).unwrap()))
                        .collect_array::<MAX_DEGREE>()?;
                    let out_ind = out_ind
                        .iter()
                        .copied()
                        .map(|e| e.map(|e| *map.get(&e).unwrap()))
                        .collect_array::<MAX_DEGREE>()?;
                    Some((g, in_ind, out_ind))
                } else {
                    None
                }
            })
            .collect::<Option<Box<[Inner]>>>()?;
        Some(FitnessPureCircuit(mapper))
    }
}

#[cfg(test)]
mod test_evo {

    use super::*;
    use genetic_algorithm::strategy::evolve::prelude::*;
    use genetic_algorithm::strategy::hill_climb::prelude::*;

    use crate::{gates::NodeUnitialised, test_utils::enum_strategy};
    use itertools::Itertools;
    use proptest::prelude::{Strategy, *};

    // #[derive(Debug, Clone)]
    // struct PurifyFitness;

    mod conversion_tests {
        use super::*;

        fn generate_strategy() -> impl proptest::prelude::Strategy<Value = Vec<NodeUnitialised>> {
            proptest::collection::vec(
                prop_oneof![
                    enum_strategy::<Value>().prop_map(NodeUnitialised::from_value),
                    enum_strategy::<Gate>().prop_map(NodeUnitialised::from_gate)
                ],
                10..=250,
            )
        }

        proptest! {

            #[test]
            fn to_chromosone_alignment(s in generate_strategy()) {
                let mut pc = PureCircuitGraph::<(), ()>::new();
                for el in s.iter().copied() {
                    pc.add_node(el, ());
                }
                let el1 = pc.to_chromosone();
                let el2 = s.into_iter().filter_map(|e| if let NodeValue::ValueNode(v) = e {
                    Some(v)
                } else { None }).collect::<Box<[_]>>();
                prop_assert_eq!(el1, el2);

            }
        }

        #[test]
        fn check_mapping_1() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(1), Some(0)], [Some(2), None])])
            );
        }

        #[test]
        fn check_mapping_2() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(0), Some(1)], [Some(2), None])])
            );
        }

        #[test]
        fn check_mapping_3() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_3, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_2, ()).unwrap();
            let fitness = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fitness.0,
                *Box::new([(Gate::And, [Some(0), Some(2)], [Some(1), None])])
            );
        }

        #[test]
        fn check_mapping_4() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_2, ()).unwrap();
            assert!(matches!(pc.to_fitness_function(), None));
        }

        #[test]
        fn check_mapping_big_1() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let val_4 = pc.add_node(NodeUnitialised::from_value(Value::One), ());
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            pc.add_edge(val_3, gate_2, ()).unwrap();
            pc.add_edge(gate_2, val_4, ()).unwrap();
            let fit = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fit.0,
                *Box::new([
                    (Gate::And, [Some(0), Some(1)], [Some(2), None]),
                    (Gate::Copy, [Some(2), None], [Some(3), None]),
                ])
            );
        }

        #[test]
        fn check_mapping_big_2() {
            let mut pc = PureCircuitGraph::<(), ()>::new();
            let val_1 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 0
            let val_3 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 1
            let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
            let val_4 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); // 2
            let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Copy), ());
            let val_2 = pc.add_node(NodeUnitialised::from_value(Value::One), ()); //3
            pc.add_edge(gate_2, val_4, ()).unwrap();
            pc.add_edge(val_1, gate_1, ()).unwrap();
            pc.add_edge(gate_1, val_3, ()).unwrap();
            pc.add_edge(val_2, gate_1, ()).unwrap();
            pc.add_edge(val_3, gate_2, ()).unwrap();
            let fit = pc.to_fitness_function().unwrap();
            assert_eq!(
                *fit.0,
                *Box::new([
                    (Gate::And, [Some(0), Some(3)], [Some(1), None]),
                    (Gate::Copy, [Some(1), None], [Some(2), None]),
                ])
            );
        }
    }

    mod fitness_tests {
        use genetic_algorithm::chromosome::GenesOwner;
        use strum::IntoEnumIterator;

        use super::*;

        proptest! {

                #[test]
                fn check_simple_case(x in enum_strategy::<Value>(), y in enum_strategy::<Value>(), z in enum_strategy::<Value>()) {
                    let mut pc = PureCircuitGraph::<(), ()>::new();
                    let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::And), ());
                    let val_1 = pc.add_node(NodeUnitialised::from_value(x), ());
                    let val_2 = pc.add_node(NodeUnitialised::from_value(y), ());
                    let val_3 = pc.add_node(NodeUnitialised::from_value(z), ());
                    pc.add_edge(val_2, gate_1, ()).unwrap();
                    pc.add_edge(val_1, gate_1, ()).unwrap();
                    pc.add_edge(gate_1, val_3, ()).unwrap();
                    let mut fitness = pc.to_fitness_function().unwrap();
                    let genotype = ListGenotype::builder()
                        .with_allele_list(Value::iter().collect_vec())
                        .with_genes_size(3)
                        .build()
                        .unwrap();
                    let chrom = ListChromosome::new(vec![x,y,z]);
                    let s = fitness.calculate_for_chromosome(&chrom, &genotype);
                    assert_eq!(s, Some(
                        (Gate::And.check(&[x,y], &[z]).unwrap() == false) as isize
                    ), "We are getting {}", Gate::And.check(&[x,y], &[z]).unwrap());
                }

                #[test]
                fn check_complex_case(s in prop::array::uniform5(enum_strategy::<Value>())) {
                    let mut pc = PureCircuitGraph::<(), ()>::new();
                    let gate_1 = pc.add_node(NodeUnitialised::from_gate(Gate::Purify), ());
                    let val_1 = pc.add_node(NodeUnitialised::from_value(s[0]), ());
                    let val_2 = pc.add_node(NodeUnitialised::from_value(s[1]), ());
                    let gate_2 = pc.add_node(NodeUnitialised::from_gate(Gate::Or), ());
                    let val_3 = pc.add_node(NodeUnitialised::from_value(s[2]), ());
                    let val_4 = pc.add_node(NodeUnitialised::from_value(s[3]), ());
                    let val_5 = pc.add_node(NodeUnitialised::from_value(s[4]), ());
                    pc.add_edge(val_2, gate_1, ()).unwrap();
                    pc.add_edge(gate_1, val_1, ()).unwrap();
                    pc.add_edge(gate_1, val_3, ()).unwrap();
                    pc.add_edge(val_3, gate_2, ()).unwrap();
                    pc.add_edge(val_4, gate_2, ()).unwrap();
                    pc.add_edge(gate_2, val_5, ()).unwrap();
                    let mut fitness = pc.to_fitness_function().unwrap();
                    let genotype = ListGenotype::builder()
                        .with_allele_list(Value::iter().collect_vec())
                        .with_genes_size(3)
                        .build()
                        .unwrap();
                    let chrom = ListChromosome::new(s.to_vec());
                    let res = fitness.calculate_for_chromosome(&chrom, &genotype);
                    assert_eq!(res, Some(
                        (Gate::Purify.check(&[s[1]], &[s[0], s[2]]).unwrap() == false) as isize
                        + (Gate::Or.check(&[s[2], s[3]], &[s[4]]).unwrap() == false) as isize
                    ));
            }
        }
    }

    // impl Fitness for PurifyFitness {
    //     type Genotype = RangeGenotype<u8>;

    //     fn calculate_for_chromosome(
    //         &mut self,
    //         chromosome: &FitnessChromosome<Self>,
    //         _genotype: &Self::Genotype,
    //     ) -> Option<FitnessValue> {
    //         let mut score = (chromosome.genes[0] != 1) as isize;
    //         for ((ind, s1), s2) in chromosome
    //             .genes
    //             .iter()
    //             .copied()
    //             .zip(chromosome.genes.iter().copied().skip(1))
    //             .zip(chromosome.genes.iter().copied().skip(2))
    //             .step_by(2)
    //         {
    //             score += 1 - match ind {
    //                 s @ (0 | 2) => (s1, s2) == (s, s),
    //                 1 => [(0, 1), (1, 2), (0, 2)].contains(&(s1, s2)),
    //                 d => {
    //                     panic!("Something went wrong {d}");
    //                 }
    //             } as isize;
    //         }
    //         Some(score)
    //     }
    // }

    // #[test]
    // fn test_lib() {
    //     env_logger::init();
    //     let genotype = RangeGenotype::builder()
    //         .with_genes_size(3 + 10 * 2)
    //         .with_allele_range(0u8..=2u8)
    //         .with_genes_hashing(true)
    //         .build()
    //         .unwrap();

    //     println!("{genotype}");

    //     let mut evolve = Evolve::builder()
    //         .with_genotype(genotype.clone())
    //         .with_target_population_size(100)
    //         .with_select(SelectElite::new(0.5, 0.02))
    //         .with_crossover(CrossoverUniform::new(0.7, 0.8))
    //         .with_mutate(MutateSingleGene::new(0.2))
    //         .with_fitness(PurifyFitness)
    //         .with_fitness_ordering(FitnessOrdering::Minimize)
    //         .with_fitness_cache(10_000)
    //         .with_target_fitness_score(0)
    //         // .with_reporter(EvolveReporterSimple::new(100))
    //         .with_max_stale_generations(10_000)
    //         .build()
    //         .unwrap();

    //     let mut hill_climb = HillClimb::builder()
    //         .with_genotype(genotype.clone())
    //         .with_variant(HillClimbVariant::Stochastic)
    //         .with_max_stale_generations(10000)
    //         // .with_variant(HillClimbVariant::SteepestAscent)
    //         // .with_max_stale_generations(5) // needs to a little bit above 1
    //         .with_fitness(PurifyFitness)
    //         .with_fitness_ordering(FitnessOrdering::Minimize)
    //         .with_target_fitness_score(0)
    //         .build()
    //         .unwrap();

    //     if let Some((best_genes, fitness_score)) = evolve.best_genes_and_fitness_score() {
    //         let p = best_genes
    //             .iter()
    //             .rev()
    //             .enumerate()
    //             .filter_map(|(ind, el)| {
    //                 if ind < 2 || ind % 2 == 1 {
    //                     Some(*el)
    //                 } else {
    //                     None
    //                 }
    //             })
    //             .rev()
    //             .collect_vec();
    //         dbg!(p);
    //         if fitness_score == 0 {
    //             println!("Valid solution with fitness score: {}", fitness_score);
    //         } else {
    //             println!("Wrong solution with fitness score: {}", fitness_score);
    //         }
    //     } else {
    //         println!("Invalid solution with fitness score: None");
    //     }

    //     println!("HILL CLIMB");
    //     if let Some((best_genes, fitness_score)) = hill_climb.best_genes_and_fitness_score() {
    //         let p = best_genes
    //             .iter()
    //             .rev()
    //             .enumerate()
    //             .filter_map(|(ind, el)| {
    //                 if ind < 2 || ind % 2 == 1 {
    //                     Some(*el)
    //                 } else {
    //                     None
    //                 }
    //             })
    //             .rev()
    //             .collect_vec();
    //         dbg!(p);
    //         if fitness_score == 0 {
    //             println!("Valid solution with fitness score: {}", fitness_score);
    //         } else {
    //             println!("Wrong solution with fitness score: {}", fitness_score);
    //         }
    //     } else {
    //         println!("Invalid solution with fitness score: None");
    //     }
    // }
}
