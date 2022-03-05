pub fn scale(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub fn round_precision(value: f64) -> f64 {
    ((value * 10000.0) as f64).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale() {
        let start = scale(0.0, 0.0, 1.0, 4.0, 8.0);
        assert!((start - 4.0).abs() < 0.1);

        let middle = scale(0.5, 0.0, 1.0, 4.0, 8.0);
        assert!((middle - 6.0).abs() < 0.1);

        let end = scale(1.0, 0.0, 1.0, 4.0, 8.0);
        assert!((end - 8.0).abs() < 0.1);

        let inverted = scale(0.25, 0.0, 1.0, 8.0, 4.0);
        assert!((inverted - 7.0).abs() < 0.1);
    }

    #[test]
    fn test_round_precision() {
        let rounded = round_precision(1.235567774);
        assert!(rounded == 1.2356);
    }
}
