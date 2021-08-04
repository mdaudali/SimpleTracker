use crate::price::Price;
use crate::percentage::Percentage;
#[derive(Debug, PartialEq)]
pub struct PerformanceIndicators {
    pub min: Option<Price>,
    pub max: Option<Price>,
    pub n_window_sma: Option<Vec<Price>>,
    pub percentage_change: Option<Percentage>,
    pub abs_change: Option<Price>
}

pub fn min(series: &[f64]) -> Option<f64> {
    get_extreme(series, |v, mv| v < mv)
}

pub fn max(series: &[f64]) -> Option<f64> {
    get_extreme(series, |v, mv| v > mv)
}

fn get_extreme(series: &[f64], comparator: fn(&f64, &f64) -> bool) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        let mut min_value = series.first()?;
        for value in series.iter() {
            if comparator(value, min_value) {
                min_value = value;
            }
        }
        Some(*min_value)
    }
}

pub fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if n > series.len() || n == 0 {
        return None;
    }
    let mut window: f64 = series.iter().take(n).sum();
    let mut index = 0;
    let window_size: f64 = n as f64;
    let mut sma = vec![window / window_size];
    let mut series_iterator = series.iter();
    for _ in 0..n {
        series_iterator.next();
    }
    for next_value in series_iterator {
        window += next_value - series[index];
        index += 1;
        sma.push(window / window_size);
    }
    Some(sma)
}

pub fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
    if series.len() < 2 {
        return None;
    }

    let first = series.first()?;
    let second = series.last()?;

    Some(((second / first) * 100_f64, second - first))
}

impl PerformanceIndicators {
    pub fn create(window: usize, series: &[f64]) -> PerformanceIndicators {
        let (percentage_change, abs_change) = match price_diff(series) {
            Some((percentage_change, abs_change)) => (Some(Percentage(percentage_change)), Some(Price(abs_change))),
            None => (None, None)
        };

        PerformanceIndicators {
            min: min(series).map(Price),
            max: max(series).map(Price),
            n_window_sma: n_window_sma(window, series).map(|vec| vec.into_iter().map(Price).collect()),
            percentage_change,
            abs_change
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn min_returns_none_on_empty_list() {
        assert_eq!(min(&[]), None);
    }  

    #[test]
    fn min_returns_minimum_value_on_non_empty_list() {
        assert_eq!(min(&[3f64, 2f64, 14f64]), Some(2f64));
    }

    #[test]
    fn max_returns_none_on_empty_list() {
        assert_eq!(max(&[]), None);
    }

    #[test]
    fn max_returns_max_value_on_non_empty_list() {
        assert_eq!(max(&[3f64, 1f64, 14f64, 2f64]), Some(14f64));
    }

    #[test]
    fn n_window_sma_returns_none_if_n_is_0() {
        assert_eq!(n_window_sma(0, &[1f64]), None);
    }

    #[test]
    fn n_window_sma_returns_none_if_n_is_greater_than_series() {
        assert_eq!(n_window_sma(15, &[1f64]), None)
    }

    #[test]
    fn n_window_sma_returns_correct_simple_moving_average() {
        let series = [15f64, 13f64, 2f64, 11f64];
        let expected_sma = vec![14f64, 7.5f64, 6.5f64];
        assert_eq!(n_window_sma(2, &series), Some(expected_sma));
    }

    #[test]
    fn price_diff_returns_none_if_series_is_smaller_than_2() {
        assert_eq!(price_diff(&[1f64]), None);
    }

    #[test]
    fn price_diff_returns_correct_abs_and_percentage_diff_on_positive_change() {
        let series = [16f64, 3f64, 32f64];
        let expected = (200f64, 16f64);
        assert_eq!(price_diff(&series), Some(expected));
    }

    #[test]
    fn price_diff_returns_correct_abs_and_percentage_diff_on_negative_change() {
        let series = [16f64, 3f64, 0f64];
        let expected = (0f64, -16f64);
        assert_eq!(price_diff(&series), Some(expected));
    }

    #[test]
    fn create_performance_indicators_returns_correct_performance_indicator() {
        let series = [15f64, 13f64, 2f64, 7.5f64];
        let expected = PerformanceIndicators {
            min: Some(Price(2f64)),
            max: Some(Price(15f64)),
            n_window_sma: Some(vec![Price(14f64), Price(7.5f64), Price(4.75f64)]),
            percentage_change: Some(Percentage(50f64)),
            abs_change: Some(Price(-7.5f64))
        };
        assert_eq!(PerformanceIndicators::create(2, &series), expected);
    }
}