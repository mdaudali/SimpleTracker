struct PerformanceIndicators {
}

impl PerformanceIndicators {
    pub fn min(series: &[f64]) -> Option<f64> {
        PerformanceIndicators::get_extreme(series, |v, mv| v < mv)
    }

    pub fn max(series: &[f64]) -> Option<f64> {
        PerformanceIndicators::get_extreme(series, |v, mv| v > mv)
    }
    
    fn get_extreme(series: &[f64], comparator: fn(&f64, &f64) -> bool) -> Option<f64> {
        if series.len() == 0 {
            None
        } else {
            let mut min_value = &0f64;
            for value in series.iter() {
                if comparator(value, min_value) {
                    min_value = value;
                }
            }
            Some(*min_value)
        }
    }

    pub fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
        if n > series.len() {
            None
        }
        let series_iterator = series.iter()
        let mut window = series_iterator.take(n).sum();
        let mut index = 0;
        let sma = vec![window / n];
        for next_value in series_iterator {
            
        }

    }
}