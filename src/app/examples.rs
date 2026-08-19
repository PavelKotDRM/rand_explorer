#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Primitives,
    Ranges,
    Distributions,
    Sequences,
    Engines,
    System,
}

pub const RAND_COOKBOOK_EXAMPLE: &str = r#"// import commonly used items from the prelude:
use rand::prelude::*;

fn main() {
    // We can use random() immediately. It can produce values of many common types:
    let x: u8 = rand::random();
    println!("{}", x);

    if rand::random() { // generates a boolean
        println!("Heads!");
    }

    // If we want to be a bit more explicit (and a little more efficient) we can
    // make a handle to the thread-local generator:
    let mut rng = rand::rng();
    if rng.random() { // random bool
        let x: f64 = rng.random(); // random number in range [0, 1)
        let y = rng.random_range(-10.0..10.0);
        println!("x is: {}", x);
        println!("y is: {}", y);
    }

    println!("Dice roll: {}", rng.random_range(1..=6));
    println!("Number from 0 to 9: {}", rng.random_range(0..10));

    // Sometimes it's useful to use distributions directly:
    let distr = rand::distr::Uniform::new_inclusive(1, 100).unwrap();
    let mut nums = [0i32; 3];
    for x in &mut nums {
        *x = rng.sample(distr);
    }
    println!("Some numbers: {:?}", nums);

    // We can also interact with iterators and slices:
    let arrows_iter = "➡⬈⬆⬉⬅⬋⬇⬊".chars();
    println!("Lets go in this direction: {}", arrows_iter.choose(&mut rng).unwrap());
    let mut nums = [1, 2, 3, 4, 5];
    nums.shuffle(&mut rng);
    println!("I shuffled my {:?}", nums);
}"#;

pub const RANGE_EXAMPLE: &str = r#"let mut rng = rand::rng();
let value = rng.random_range(-10.0..10.0);
let inclusive = rng.random_range(1..=6);
println!("value: {value}, dice: {inclusive}");"#;

pub const DISTRIBUTION_EXAMPLE: &str = r#"use rand::distr::{Distribution, Uniform};
use rand_distr::Normal;

let mut rng = rand::rng();
let uniform = Uniform::new(0.0, 1.0).unwrap();
let normal = Normal::new(0.0, 1.0).unwrap();
let sample = uniform.sample(&mut rng);
let gaussian = normal.sample(&mut rng);
println!("{sample} {gaussian}");"#;

pub const SEQUENCE_EXAMPLE: &str = r#"use rand::seq::{IndexedRandom, SliceRandom};

let mut rng = rand::rng();
let mut values = ["A", "B", "C", "D"];
let chosen = values.choose(&mut rng);
let many = values.sample(&mut rng, 2);
values.shuffle(&mut rng);
println!("{chosen:?} {many:?} {values:?}");"#;

pub const ENGINE_EXAMPLE: &str = r#"use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

let seed = 42;
let mut first = ChaCha20Rng::seed_from_u64(seed);
let mut second = ChaCha20Rng::seed_from_u64(seed);
assert_eq!(first.random::<u64>(), second.random::<u64>());"#;

pub const SYSTEM_EXAMPLE: &str = r#"println!("package: {}", env!("CARGO_PKG_VERSION"));
println!("target: {}", std::env::consts::ARCH);
println!("renderer: Glow / OpenGL");"#;

pub fn code_example(active_tab: ActiveTab) -> (&'static str, &'static str) {
    match active_tab {
        ActiveTab::Primitives => ("Primitive generation", RAND_COOKBOOK_EXAMPLE),
        ActiveTab::Ranges => ("Range generation", RANGE_EXAMPLE),
        ActiveTab::Distributions => ("Distribution sampling", DISTRIBUTION_EXAMPLE),
        ActiveTab::Sequences => ("Sequence operations", SEQUENCE_EXAMPLE),
        ActiveTab::Engines => ("Deterministic RNG", ENGINE_EXAMPLE),
        ActiveTab::System => ("Build information", SYSTEM_EXAMPLE),
    }
}
