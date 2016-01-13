pub fn eq_f64(x: f64, y: f64, error: f64) -> bool {
    (x - y).abs() < error
}


#[cfg(test)]
mod tests {
    use super::eq_f64;


    #[test]
    fn test_eq_f64() {
        // Actual equality (need to supply a >0 error though)
        assert!(eq_f64(1.1, 1.1, 0.000000001));

        // The second is smaller
        assert!(eq_f64(1.1, 1.0, 1.000000001));

        // The second is greater
        assert!(eq_f64(1.1, 1.2, 1.000000001));

    }
}
