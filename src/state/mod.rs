use rand::{rng, RngExt, SeedableRng};
use rand_chacha::{ChaCha8Rng, ChaCha20Rng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineKind {
    ThreadRng,
    StdRng,
    SmallRng,
    ChaCha8,
    ChaCha20,
    Pcg64,
}

impl EngineKind {
    pub fn all() -> &'static [EngineKind] {
        &[
            EngineKind::ThreadRng,
            EngineKind::StdRng,
            EngineKind::SmallRng,
            EngineKind::ChaCha8,
            EngineKind::ChaCha20,
            EngineKind::Pcg64,
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum DistributionKind {
    #[default]
    Normal,
    Standard,
    Uniform,
    Bernoulli,
    Exponential,
    Poisson,
    WeightedIndex,
}

impl DistributionKind {
    pub fn all() -> &'static [DistributionKind] {
        &[
            DistributionKind::Standard,
            DistributionKind::Uniform,
            DistributionKind::Bernoulli,
            DistributionKind::Normal,
            DistributionKind::Exponential,
            DistributionKind::Poisson,
            DistributionKind::WeightedIndex,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimitiveSample {
    pub u8: Vec<u8>,
    pub i8: Vec<i8>,
    pub u32: Vec<u32>,
    pub i32: Vec<i32>,
    pub u64: Vec<u64>,
    pub i64: Vec<i64>,
    pub u128: Vec<u128>,
    pub i128: Vec<i128>,
    pub f32: Vec<f32>,
    pub f64: Vec<f64>,
    pub bools: Vec<bool>,
    pub chars: Vec<char>,
    pub arrows: String,
    pub coin_heads: usize,
    pub coin_tails: usize,
    pub dice_counts: [usize; 6],
}

impl Default for PrimitiveSample {
    fn default() -> Self {
        Self {
            u8: Vec::new(),
            i8: Vec::new(),
            u32: Vec::new(),
            i32: Vec::new(),
            u64: Vec::new(),
            i64: Vec::new(),
            u128: Vec::new(),
            i128: Vec::new(),
            f32: Vec::new(),
            f64: Vec::new(),
            bools: Vec::new(),
            chars: Vec::new(),
            arrows: String::new(),
            coin_heads: 0,
            coin_tails: 0,
            dice_counts: [0; 6],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RangeBounds {
    pub start: f64,
    pub end: f64,
    pub inclusive: bool,
    pub sample_count: usize,
    pub stream_ms: u64,
    pub live_enabled: bool,
    pub values: Vec<f64>,
}

impl Default for RangeBounds {
    fn default() -> Self {
        Self {
            start: 0.0,
            end: 10.0,
            inclusive: false,
            sample_count: 64,
            stream_ms: 250,
            live_enabled: false,
            values: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DistState {
    pub kind: DistributionKind,
    pub sample_size: usize,
    pub mean: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub values: Vec<f64>,
    pub mu: f64,
    pub sigma: f64,
    pub lambda: f64,
    pub probability: f64,
    pub weights: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SequenceState {
    pub items: Vec<String>,
    pub choose_count: usize,
    pub password_len: usize,
    pub alphabet: String,
    pub shuffle_step: usize,
    pub chosen_item: Option<String>,
    pub chosen_items: Vec<String>,
    pub generated_password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineBenchmark {
    pub seed: u64,
    pub total_values: usize,
    pub selected: EngineKind,
    pub throughput_mbps: f64,
    pub elapsed_ms: f64,
    pub sample: Vec<u64>,
    pub stream_a: Vec<u64>,
    pub stream_b: Vec<u64>,
    pub streams_match: bool,
}

impl Default for EngineBenchmark {
    fn default() -> Self {
        Self {
            seed: 123456789_u64,
            total_values: 1_000_000,
            selected: EngineKind::StdRng,
            throughput_mbps: 0.0,
            elapsed_ms: 0.0,
            sample: Vec::new(),
            stream_a: Vec::new(),
            stream_b: Vec::new(),
            streams_match: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppState {
    pub seed: u64,
    pub engine: EngineKind,
    pub dark_mode: bool,
    pub primitive_count: usize,
    pub primitive_stream_ms: u64,
    pub primitive_stream_enabled: bool,
    pub primitive_samples: PrimitiveSample,
    pub range_bounds: RangeBounds,
    pub distribution: DistState,
    pub sequence: SequenceState,
    pub benchmark: EngineBenchmark,
}

impl Default for DistState {
    fn default() -> Self {
        Self {
            kind: DistributionKind::Normal,
            sample_size: 1000,
            mean: 0.0,
            variance: 1.0,
            min: 0.0,
            max: 0.0,
            values: Vec::new(),
            mu: 0.0,
            sigma: 1.0,
            lambda: 2.0,
            probability: 0.5,
            weights: "1,2,3,4".to_string(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            seed: 42,
            engine: EngineKind::StdRng,
            dark_mode: true,
            primitive_count: 32,
            primitive_stream_ms: 250,
            primitive_stream_enabled: false,
            primitive_samples: PrimitiveSample::default(),
            range_bounds: RangeBounds::default(),
            distribution: DistState::default(),
            sequence: SequenceState {
                items: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                choose_count: 2,
                password_len: 12,
                alphabet: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string(),
                shuffle_step: 0,
                chosen_item: None,
                chosen_items: Vec::new(),
                generated_password: String::new(),
            },
            benchmark: EngineBenchmark::default(),
        }
    }
}

pub fn generate_seed() -> u64 {
    rng().random()
}

pub fn reseed_for_kind(kind: EngineKind, seed: u64) -> Box<dyn rand::Rng + 'static> {
    match kind {
        EngineKind::ThreadRng => Box::new(rand::rngs::ThreadRng::default()),
        EngineKind::StdRng => Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
        EngineKind::SmallRng => Box::new(rand::rngs::SmallRng::seed_from_u64(seed)),
        EngineKind::ChaCha8 => Box::new(ChaCha8Rng::seed_from_u64(seed)),
        EngineKind::ChaCha20 => Box::new(ChaCha20Rng::seed_from_u64(seed)),
        EngineKind::Pcg64 => Box::new(Pcg64::seed_from_u64(seed)),
    }
}

pub fn sample_uniform_range(start: f64, end: f64, count: usize, inclusive: bool) -> Vec<f64> {
    let mut rng = rand::rng();
    let mut values = Vec::with_capacity(count);
    let lo = start.min(end);
    let hi = start.max(end);
    for _ in 0..count {
        let value = if inclusive {
            rng.random_range(lo..=hi)
        } else {
            rng.random_range(lo..hi)
        };
        values.push(value);
    }
    values
}

pub fn sample_bool_sequence(count: usize, p: f64) -> Vec<bool> {
    let mut rng = rand::rng();
    let mut v = Vec::with_capacity(count);
    for _ in 0..count {
        v.push(rng.random_bool(p));
    }
    v
}
