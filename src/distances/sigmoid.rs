/// Computes the sigmoid function.
///
/// Expects a value between 0 and 1, resize it to -4 to 4 and applies the sigmoid function.
pub fn sigmoid(v: f32) -> f32 {
    let v = v * 8.0 - 4.0;
    1.0 / (1.0 + (-v).exp())
}
