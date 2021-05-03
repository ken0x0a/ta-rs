use std::fmt;

use crate::errors::{Result, TaError};
use crate::{Close, Next, Period, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Running moving average (RMA).
///
/// # Formula
///
/// ![RMA](https://wikimedia.org/api/rest_v1/media/math/render/svg/f37acfbaac29066a31956b811a503b9d304ac70d)
///
/// Where:
///
/// * _RMA<sub>t</sub>_ - value of simple moving average at a point of time _t_
/// * _period_ - number of periods (period)
/// * _p<sub>t</sub>_ - input value at a point of time _t_
///
/// # Parameters
///
/// * _period_ - number of periods (integer greater than 0)
///
/// # Example
///
/// ```
/// use ta::indicators::RunningMovingAverage;
/// use ta::Next;
///
/// let mut rma = RunningMovingAverage::new(3).unwrap();
/// assert_eq!(rma.next(&bar(4.0)), 0.0);
/// assert_eq!(rma.next(&bar(4.0)), 0.0);
/// assert_eq!(rma.next(&bar(7.0)), 5.0);
/// assert_eq!(rma.next(&bar(1.0)), 11.0 / 3.0);
/// ```
///
/// # Links
///
/// * [Running Moving Average, Wikipedia](https://en.wikipedia.org/wiki/Moving_average#Modified_moving_average)
///
#[doc(alias = "RMA")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RunningMovingAverage {
    period: usize,
    count: usize,
    prev: f64,
    sum: f64,
}

impl RunningMovingAverage {
    pub fn new(period: usize) -> Result<Self> {
        match period {
            0..=1 => Err(TaError::InvalidParameter),
            _ => Ok(Self {
                period,
                count: 0,
                prev: 0.0,
                sum: 0.0,
            }),
        }
    }
}

impl Period for RunningMovingAverage {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for RunningMovingAverage {
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.count += 1;

        let res = match self.count {
            i if i < self.period => {
                self.sum += input;
                0.0
            }
            i if i == self.period => (self.sum + input) / (self.period as f64),
            _ => (self.prev * (self.period - 1) as f64 + input) / (self.period as f64),
        };

        self.prev = res;
        res
    }
}

impl<T: Close> Next<&T> for RunningMovingAverage {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.next(input.close())
    }
}

impl Reset for RunningMovingAverage {
    fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
    }
}

impl Default for RunningMovingAverage {
    fn default() -> Self {
        Self::new(9).unwrap()
    }
}

impl fmt::Display for RunningMovingAverage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RMA({})", self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    test_indicator!(RunningMovingAverage);

    #[test]
    fn test_new() {
        assert!(RunningMovingAverage::new(0).is_err());
        assert!(RunningMovingAverage::new(1).is_err());
        assert!(RunningMovingAverage::new(2).is_ok());
    }

    #[test]
    fn test_next() {
        let mut rma = RunningMovingAverage::new(4).unwrap();
        assert_eq!(rma.next(4.0), 0.0);
        assert_eq!(rma.next(5.0), 0.0);
        assert_eq!(rma.next(6.0), 0.0);
        assert_eq!(rma.next(6.0), 5.25); // 21 / 4
        assert_eq!(rma.next(6.0), 5.4375); // (21 / 4 * 3 + 6) / 4
        assert_eq!(rma.next(6.0), 5.578125);
        assert_eq!(rma.next(2.0), 4.68359375);
    }

    #[test]
    fn test_next_with_bars() {
        fn bar(close: f64) -> Bar {
            Bar::new().close(close)
        }

        let mut rma = RunningMovingAverage::new(3).unwrap();
        assert_eq!(rma.next(&bar(4.0)), 0.0);
        assert_eq!(rma.next(&bar(4.0)), 0.0);
        assert_eq!(rma.next(&bar(7.0)), 5.0);
        assert_eq!(rma.next(&bar(1.0)), 11.0 / 3.0);
    }

    #[test]
    fn test_reset() {
        let mut rma = RunningMovingAverage::new(4).unwrap();
        assert_eq!(rma.next(4.0), 0.0);
        assert_eq!(rma.next(5.0), 0.0);
        assert_eq!(rma.next(6.0), 0.0);

        rma.reset();
        assert_eq!(rma.next(99.0), 0.0);
    }

    #[test]
    fn test_default() {
        RunningMovingAverage::default();
    }

    #[test]
    fn test_display() {
        let rma = RunningMovingAverage::new(5).unwrap();
        assert_eq!(format!("{}", rma), "RMA(5)");
    }
}
