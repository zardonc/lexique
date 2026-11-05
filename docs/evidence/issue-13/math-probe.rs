// Inputs are the two first divergent operations in fsrs 6.6.2's model.rs.
#[unsafe(no_mangle)]
pub extern "C" fn math(operation: u32, x: f32, y: f32) -> u32 {
    match operation {
        0 => x.exp().to_bits(),
        _ => x.powf(y).to_bits(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    for (operation, x, y) in [
        (0, 0.014672648161649704_f32, 0.0_f32),
        (1, 837.2720947265625_f32, -0.16660000383853912_f32),
    ] {
        let result = math(operation, std::hint::black_box(x), std::hint::black_box(y));
        println!("{result}");
    }
}
