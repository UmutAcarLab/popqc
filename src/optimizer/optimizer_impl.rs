use crate::oracles::oracle_utils::OracleEnum;
use crate::oracles::qiskit::qiskit_impl::Qiskit;
use crate::oracles::quartz::quartz_impl::Quartz;
use crate::oracles::roqc_oracle::Roqc;
use crate::oracles::tket::tket_impl::Tket;
use crate::oracles::voqc::Voqc;
use crate::utils::ftree::FenwickTree;
use circuit::config::{OracleName, SingleConfig};
use circuit::layer::Layout;
use circuit::{CircuitLayer, Gate};

use itertools::Itertools;
// use log::{debug, info};
use rayon::prelude::*;
pub struct SoamRunner {
    config: SingleConfig,
    omega: usize,
    pub oracle_runner: OracleEnum,
    ftree: FenwickTree,
    pub n_round: usize,
    pub time_oracle: f32,
    pub circ: CircuitLayer,
    pub layout: Layout,
    pub n_rounds: usize,
    pub n_seams_total: usize,
}

impl SoamRunner {
    pub fn new(config: SingleConfig, port: u16, circ: CircuitLayer, layout: Layout) -> Self {
        println!("First round: {:?}", config);
        let oracle_runner = match config.oracle_name {
            OracleName::Quartz(ref quartz_config) => {
                let oracle_runner = Quartz::new(quartz_config.clone(), port);
                OracleEnum::SyncQuartz(oracle_runner)
            }
            OracleName::Voqc(ref voqc_config) => {
                let oracle_runner = Voqc::new(voqc_config.clone());
                OracleEnum::Voqc(oracle_runner)
            }
            OracleName::Roqc(ref roqc_config) => {
                let oracle_runner = Roqc::new(roqc_config.clone());
                OracleEnum::Roqc(oracle_runner)
            }
            OracleName::Qiskit(ref qiskit_config) => {
                let oracle_runner = Qiskit::new(qiskit_config.clone());
                OracleEnum::Qiskit(oracle_runner)
            }
            OracleName::Tket(ref tket_config) => {
                let oracle_runner = Tket::new(tket_config.clone());
                OracleEnum::Tket(oracle_runner)
            }
        };

        SoamRunner {
            config: config.clone(),
            omega: config.omega,
            oracle_runner,
            ftree: FenwickTree::from_iter(vec![1; circ.len()]),
            n_round: 0,
            time_oracle: 0.0,
            circ: circ.clone(),
            layout,
            n_rounds: 0,
            n_seams_total: 0,
        }
    }
    fn id_of_non_empty_layer(&self, id: usize) -> usize {
        self.ftree.prefix_sum(id, 0)
    }

    fn reverse_id_of_non_empty_layer(&self, id: usize) -> usize {
        self.ftree.index_of(id)
    }
    fn find_seams(&self, seams: &Vec<usize>) -> (Vec<usize>, Vec<usize>) {
        // debug!("Finding seams: {:?}", seams);
        let len_seams = seams.len();
        let two_omega = self.omega * 2;
        let selected_seams_1: Vec<bool> = (0..len_seams)
            .into_par_iter()
            .map(|i| {
                if (i == 0
                    || (self.id_of_non_empty_layer(seams[i]) / two_omega
                        - self.id_of_non_empty_layer(seams[i - 1]) / two_omega
                        > 0))
                    && (self.id_of_non_empty_layer(seams[i]) / two_omega) % 2 == 0
                {
                    true
                } else {
                    false
                }
            })
            .collect();
        let n_selected_seams_1 = selected_seams_1.par_iter().filter(|x| **x).count();
        let selected_seams_2: Vec<bool> = (0..len_seams)
            .into_par_iter()
            .map(|i| {
                if (i == 0
                    || (self.id_of_non_empty_layer(seams[i]) / two_omega
                        - self.id_of_non_empty_layer(seams[i - 1]) / two_omega
                        > 0))
                    && (self.id_of_non_empty_layer(seams[i]) / two_omega) % 2 == 1
                {
                    true
                } else {
                    false
                }
            })
            .collect();
        let n_selected_seams_2 = selected_seams_2.par_iter().filter(|x| **x).count();
        if n_selected_seams_1 > n_selected_seams_2 {
            (
                seams
                    .par_iter()
                    .enumerate()
                    .filter(|(idx, _)| selected_seams_1[*idx])
                    .map(|(_, &x)| x)
                    .collect(),
                seams
                    .par_iter()
                    .enumerate()
                    .filter(|(idx, _)| !selected_seams_1[*idx])
                    .map(|(_, &x)| x)
                    .collect(),
            )
        } else {
            (
                seams
                    .par_iter()
                    .enumerate()
                    .filter(|(idx, _)| selected_seams_2[*idx])
                    .map(|(_, &x)| x)
                    .collect(),
                seams
                    .par_iter()
                    .enumerate()
                    .filter(|(idx, _)| !selected_seams_2[*idx])
                    .map(|(_, &x)| x)
                    .collect(),
            )
        }
    }

    pub fn pair_and_optimize(&mut self, seams: Vec<usize>) -> Vec<usize> {
        // info!("new cycle");
        let (selected_seams, remaining_seams) = self.find_seams(&seams);
        // debug!("selected_seams: {:?}", selected_seams);
        // debug!("remaining_seams: {:?}", remaining_seams);
        let tasks: Vec<(usize, usize)> = selected_seams
            .par_iter()
            .map(|&seam| {
                (
                    self.reverse_id_of_non_empty_layer(
                        self.id_of_non_empty_layer(seam).saturating_sub(self.omega),
                    ),
                    self.reverse_id_of_non_empty_layer(
                        (self.id_of_non_empty_layer(seam) + self.omega).min(self.circ.len()),
                    ),
                )
            })
            .collect();
        let now = std::time::Instant::now();
        let ((new_seams, tree_updates), circ_updates): (
            (Vec<Vec<usize>>, Vec<Vec<(usize, isize)>>),
            Vec<Vec<(usize, Vec<Gate>)>>,
        ) = tasks
            .par_iter()
            .enumerate()
            .map(|(task_id, task)| {
                let (left, right) = task;
                let segment_before_optimize = self.circ.get(*left, *right);
                // println!(
                //     "segment_before_optimize: {:?},left: {:?},right: {:?}",
                //     segment_before_optimize.cost(&self.config.cost),
                //     *left,
                //     *right
                // );
                let segment_after_optimize = CircuitLayer::from_seq(
                    self.oracle_runner
                        .run_single(segment_before_optimize.to_seq(), task_id),
                    self.layout.clone(),
                );
                if segment_after_optimize.cost(&self.config.cost)
                    < segment_before_optimize.cost(&self.config.cost)
                    && segment_after_optimize.len() <= segment_before_optimize.len()
                {
                    let mut tree_updates: Vec<(usize, isize)> = vec![];
                    let mut circ_updates: Vec<(usize, Vec<Gate>)> = vec![];
                    for i in 0..*right - *left {
                        if i < segment_after_optimize.len() {
                            circ_updates
                                .push((i + *left, segment_after_optimize.get_one(i).clone()));
                            if self.circ.is_empty(i + *left) {
                                tree_updates.push((i + *left, 1));
                            }
                        } else {
                            circ_updates.push((i + *left, vec![]));
                            if !self.circ.is_empty(i + *left) {
                                tree_updates.push((i + *left, -1));
                            }
                        }
                    }

                    ((vec![*left, *right - 1], tree_updates), circ_updates)
                } else {
                    ((vec![], vec![]), vec![])
                }
            })
            .unzip();
        let time_oracle = now.elapsed().as_secs_f32();
        self.time_oracle += time_oracle;
        self.n_round += 1;
        let circ_updates: Vec<_> = circ_updates.into_par_iter().flatten().collect();
        self.circ.par_set(circ_updates);
        let tree_updates: Vec<_> = tree_updates.into_par_iter().flatten().collect();
        self.ftree.add_at_batch(tree_updates.clone());
        let new_seams: Vec<usize> = new_seams.into_par_iter().flatten().collect();
        // println!("new_seams: {:?}", new_seams);
        self.n_seams_total += new_seams.len();
        let new_seams: Vec<usize> = new_seams
            .iter()
            .merge(remaining_seams.iter())
            .map(|&x| x)
            .dedup()
            .collect();
        new_seams
    }
    pub fn soam(&mut self) {
        if self.config.use_soam {
            let mut seams: Vec<usize> = (0..1 + (self.circ.len() / self.config.omega))
                .map(|i| i * self.config.omega)
                .collect();
            self.n_seams_total = seams.len();
            while !seams.is_empty() {
                self.n_rounds += 1;
                seams = self.pair_and_optimize(seams);
            }
            println!("Finished!");
            println!("Number of rounds: {:?}", self.n_round);
            println!("Oracle running time: {:?}", self.time_oracle);
        } else {
            self.circ = CircuitLayer::from_seq(
                self.oracle_runner.run_single(self.circ.to_seq(), 0),
                self.layout.clone(),
            );
        }
    }
    pub fn correctness_check(&self, circ: &CircuitLayer) {
        let len = circ.layers.len();
        let correctness: Vec<bool> = (0..len.saturating_sub(self.omega))
            .into_par_iter()
            .map(|i| {
                let slice = circ.get(i, i + self.omega);
                let optimized_result = CircuitLayer::from_seq(
                    self.oracle_runner.run_single(slice.to_seq(), i),
                    self.layout.clone(),
                );
                optimized_result.cost(&self.config.cost) >= slice.cost(&self.config.cost)
            })
            .collect();
        // println!("correctness: {:?}", correctness);
        let n_violations = correctness.iter().filter(|x| !**x).count();
        if n_violations > 0 {
            println!(
                "Number of violations: {:?}",
                correctness.iter().filter(|x| !**x).count()
            );
        }
    }
}
