use std::fs;
use std::process::Command;

use plotly::{Plot as PlotlyPlot, Scatter};
use rand::distr::{weighted::WeightedIndex, Bernoulli, Distribution, Uniform};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Exp, Normal, Poisson};

pub(super) fn random_arrow_sequence(rng: &mut impl Rng) -> String {
    let mut arrows = "➡⬈⬆⬉⬅⬋⬇⬊".chars().collect::<Vec<_>>();
    arrows.shuffle(rng);
    arrows.into_iter().collect()
}

use super::App;

impl App {
    pub(super) fn save_preset(&mut self) {
        match serde_json::to_string_pretty(&self.state)
            .map_err(|error| error.to_string())
            .and_then(|json| fs::write("rand_explorer_preset.json", json).map_err(|error| error.to_string()))
        {
            Ok(()) => self.status_message = "Preset saved to rand_explorer_preset.json".to_owned(),
            Err(error) => self.status_message = format!("Save failed: {error}"),
        }
    }

    pub(super) fn load_preset(&mut self) {
        match fs::read_to_string("rand_explorer_preset.json")
            .map_err(|error| error.to_string())
            .and_then(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        {
            Ok(state) => {
                self.state = state;
                self.rebuild_primitive_samples();
                self.rebuild_distribution();
                self.status_message = "Preset loaded".to_owned();
            }
            Err(error) => self.status_message = format!("Load failed: {error}"),
        }
    }

    pub(super) fn export_distribution(&mut self) {
        let csv = self
            .state
            .distribution
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{index},{value}"))
            .collect::<Vec<_>>()
            .join("\n");
        match fs::write("rand_explorer_samples.csv", format!("index,value\n{csv}\n")) {
            Ok(()) => self.status_message = "Samples exported to rand_explorer_samples.csv".to_owned(),
            Err(error) => self.status_message = format!("Export failed: {error}"),
        }
    }

    pub(super) fn open_plotly_distribution(&self) {
        let values = self.state.distribution.values.clone();
        let x_values: Vec<f64> = (0..values.len()).map(|idx| idx as f64).collect();

        let mut plot = PlotlyPlot::new();
        plot.add_trace(Scatter::new(x_values, values).mode(plotly::common::Mode::Lines));
        plot.set_layout(
            plotly::Layout::new().title(plotly::common::Title::with_text("rand_explorer distribution")),
        );

        let path = std::env::temp_dir().join("rand_explorer_distribution_plot.html");
        plot.write_html(path.to_string_lossy().as_ref());

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd")
                .args(["/C", "start", "", path.to_string_lossy().as_ref()])
                .status();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("xdg-open").arg(&path).status();
        }
    }

    pub(super) fn rebuild_primitive_samples(&mut self) {
        let count = self.state.primitive_count;
        let mut rng = rand::rng();
        self.state.primitive_samples.u8 = (0..count).map(|_| rng.random::<u8>()).collect();
        self.state.primitive_samples.i8 = (0..count).map(|_| rng.random::<i8>()).collect();
        self.state.primitive_samples.u32 = (0..count).map(|_| rng.random::<u32>()).collect();
        self.state.primitive_samples.i32 = (0..count).map(|_| rng.random::<i32>()).collect();
        self.state.primitive_samples.u64 = (0..count).map(|_| rng.random::<u64>()).collect();
        self.state.primitive_samples.i64 = (0..count).map(|_| rng.random::<i64>()).collect();
        self.state.primitive_samples.u128 = (0..count).map(|_| rng.random::<u128>()).collect();
        self.state.primitive_samples.i128 = (0..count).map(|_| rng.random::<i128>()).collect();
        self.state.primitive_samples.f32 = (0..count).map(|_| rng.random::<f32>()).collect();
        self.state.primitive_samples.f64 = (0..count).map(|_| rng.random::<f64>()).collect();
        self.state.primitive_samples.bools = (0..count).map(|_| rng.random()).collect();
        self.state.primitive_samples.chars = (0..count)
            .map(|_| Self::random_visible_char(&mut rng))
            .collect();
        self.state.primitive_samples.arrows = random_arrow_sequence(&mut rng);
    }

    fn random_visible_char(rng: &mut impl Rng) -> char {
        const RANGES: &[(u32, u32)] = &[
            (0x21, 0x7E),
            (0x391, 0x3FF),
            (0x410, 0x44F),
            (0x590, 0x5FF),
            (0x600, 0x6FF),
            (0x4E00, 0x9FFF),
        ];
        let (start, end) = RANGES[rng.random_range(0..RANGES.len())];
        loop {
            if let Some(character) = char::from_u32(rng.random_range(start..=end))
                && !character.is_control()
                && !character.is_whitespace()
                && (start == 0x21 || character.is_alphanumeric())
            {
                return character;
            }
        }
    }

    pub(super) fn rebuild_range_samples(&mut self) {
        let bounds = &self.state.range_bounds;
        if bounds.start >= bounds.end {
            self.state.range_bounds.values.clear();
            return;
        }
        self.state.range_bounds.values = crate::state::sample_uniform_range(
            bounds.start,
            bounds.end,
            bounds.sample_count,
            bounds.inclusive,
        );
    }

    pub(super) fn distribution_pdf(&self, x: f64) -> Option<f64> {
        let dist = &self.state.distribution;
        match dist.kind {
            crate::state::DistributionKind::Standard => {
                Some((-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt())
            }
            crate::state::DistributionKind::Uniform => Some(if (0.0..=1.0).contains(&x) { 1.0 } else { 0.0 }),
            crate::state::DistributionKind::Bernoulli => None,
            crate::state::DistributionKind::Normal => {
                let sigma = dist.sigma.abs().max(f64::EPSILON);
                let z = (x - dist.mu) / sigma;
                Some((-0.5 * z * z).exp() / (sigma * (2.0 * std::f64::consts::PI).sqrt()))
            }
            crate::state::DistributionKind::Exponential => {
                let lambda = dist.lambda.max(f64::EPSILON);
                Some(if x >= 0.0 { lambda * (-lambda * x).exp() } else { 0.0 })
            }
            crate::state::DistributionKind::Poisson | crate::state::DistributionKind::WeightedIndex => None,
        }
    }

    pub(super) fn rebuild_distribution(&mut self) {
        let count = self.state.distribution.sample_size;
        let mut values = Vec::with_capacity(count);
        match self.state.distribution.kind {
            crate::state::DistributionKind::Standard => {
                let dist = Normal::new(0.0, 1.0).expect("standard Normal parameters");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            crate::state::DistributionKind::Uniform => {
                let dist = Uniform::new(0.0_f64, 1.0_f64).unwrap();
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            crate::state::DistributionKind::Bernoulli => {
                let probability = self.state.distribution.probability.clamp(0.0, 1.0);
                let dist = Bernoulli::new(probability).expect("clamped Bernoulli probability");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(if dist.sample(&mut rng) { 1.0 } else { 0.0 });
                }
            }
            crate::state::DistributionKind::Normal => {
                let sigma = self.state.distribution.sigma.abs().max(f64::EPSILON);
                let dist = Normal::new(self.state.distribution.mu, sigma)
                    .expect("positive Normal standard deviation");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            crate::state::DistributionKind::Exponential => {
                let lambda = self.state.distribution.lambda.max(f64::EPSILON);
                let dist = Exp::new(lambda).expect("positive Exponential rate");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            crate::state::DistributionKind::Poisson => {
                let lambda = self.state.distribution.lambda.max(0.0);
                let dist = Poisson::new(lambda).expect("non-negative Poisson rate");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            crate::state::DistributionKind::WeightedIndex => {
                let weights: Vec<f64> = self
                    .state
                    .distribution
                    .weights
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f64>().ok().filter(|weight| *weight >= 0.0))
                    .collect();
                if let Ok(dist) = WeightedIndex::new(&weights) {
                    let mut rng = rand::rng();
                    for _ in 0..count {
                        values.push(dist.sample(&mut rng) as f64);
                    }
                }
            }
        }
        if values.is_empty() {
            values = vec![0.0; count];
        }
        self.state.distribution.values = values;
        if let Some(first) = self.state.distribution.values.first() {
            self.state.distribution.min = self.state.distribution.values.iter().copied().fold(*first, f64::min);
            self.state.distribution.max = self.state.distribution.values.iter().copied().fold(*first, f64::max);
        }
        let n = self.state.distribution.values.len() as f64;
        let mean = self.state.distribution.values.iter().sum::<f64>() / n.max(1.0);
        let variance = self.state.distribution.values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n.max(1.0);
        self.state.distribution.mean = mean;
        self.state.distribution.variance = variance;
        self.state.distribution.min = self.state.distribution.values.iter().copied().fold(f64::INFINITY, f64::min);
        self.state.distribution.max = self.state.distribution.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }
}
