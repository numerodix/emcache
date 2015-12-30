pub fn eq_f64(x: f64, y: f64, error: f64) -> bool {
    (x - y).abs() < error
}
