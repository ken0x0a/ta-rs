use std::fmt;

use crate::{errors::Result, NewWithPeriod};
use crate::{Close, High, Low, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::AverageTrueRange;

/// Normalized Average true range (NATR).
///
/// A technical analysis volatility indicator, originally developed by J. Welles Wilder.
/// The average true range is an N-day smoothed moving average of the true range values.
/// This implementation uses exponential moving average.
///
/// # Formula
///
/// NATR(period)<sub>t</sub> = EMA(period) of TR<sub>t</sub>
///
/// Where:
///
/// * _EMA(period)_ - [exponential moving average](struct.SimpleMovingAverage.html) with smoothing period
/// * _TR<sub>t</sub>_ - [true range](struct.TrueRange.html) for period _t_
///
/// # Parameters
///
/// * _period_ - smoothing period of EMA (integer greater than 0)
///
/// # Example
///
/// ```
/// extern crate ta;
/// #[macro_use] extern crate assert_approx_eq;
///
/// use ta::{Next, DataItem};
/// use ta::indicators::{ExponentialMovingAverage, NormalizedAverageTrueRange};
///
/// fn main() {
///     let data = vec![
///         // open, high, low, close, atr
///         (9.7   , 10.0, 9.0, 9.5  , 1.0),    // tr = high - low = 10.0 - 9.0 = 1.0
///         (9.9   , 10.4, 9.8, 10.2 , 0.95),   // tr = high - prev_close = 10.4 - 9.5 = 0.9
///         (10.1  , 10.7, 9.4, 9.7  , 1.125),  // tr = high - low = 10.7 - 9.4 = 1.3
///         (9.1   , 9.2 , 8.1, 8.4  , 1.3625), // tr = prev_close - low = 9.7 - 8.1 = 1.6
///     ];
///     let mut indicator = NormalizedAverageTrueRange::<ExponentialMovingAverage>::new(3).unwrap();
///
///     for (open, high, low, close, atr) in data {
///         let di = DataItem::builder()
///             .high(high)
///             .low(low)
///             .close(close)
///             .open(open)
///             .volume(1000.0)
///             .build().unwrap();
///         assert_approx_eq!(indicator.next(&di), atr * 100.0 / close);
///     }
/// }
#[doc(alias = "NATR")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct NormalizedAverageTrueRange<MA: Period + Reset + Next<f64> + NewWithPeriod> {
    atr: AverageTrueRange<MA>,
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> NormalizedAverageTrueRange<MA> {
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            atr: AverageTrueRange::<MA>::new(period)?,
        })
    }
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> Period for NormalizedAverageTrueRange<MA> {
    fn period(&self) -> usize {
        self.atr.period()
    }
}

impl<MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Next<f64>
    for NormalizedAverageTrueRange<MA>
{
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.atr.next(input) * 100.0 / input
    }
}

impl<T: High + Low + Close, MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Next<&T>
    for NormalizedAverageTrueRange<MA>
{
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.atr.next(input) * 100.0 / input.close()
    }
}

impl<MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Reset
    for NormalizedAverageTrueRange<MA>
{
    fn reset(&mut self) {
        self.atr.reset();
    }
}

impl<MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Default
    for NormalizedAverageTrueRange<MA>
{
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl<MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> fmt::Display
    for NormalizedAverageTrueRange<MA>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NATR({})", self.atr.period())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        indicators::{ExponentialMovingAverage, RunningMovingAverage, SimpleMovingAverage},
        test_helper::*,
    };

    // test_indicator!(NormalizedAverageTrueRange);

    #[test]
    fn test_new() {
        assert!(NormalizedAverageTrueRange::<SimpleMovingAverage>::new(0).is_err());
        assert!(NormalizedAverageTrueRange::<SimpleMovingAverage>::new(1).is_ok());
        assert!(NormalizedAverageTrueRange::<ExponentialMovingAverage>::new(0).is_err());
        assert!(NormalizedAverageTrueRange::<ExponentialMovingAverage>::new(1).is_ok());
        assert!(NormalizedAverageTrueRange::<RunningMovingAverage>::new(0).is_err());
        assert!(NormalizedAverageTrueRange::<RunningMovingAverage>::new(1).is_err());
        assert!(NormalizedAverageTrueRange::<RunningMovingAverage>::new(2).is_ok());
    }
    #[test]
    fn test_next() {
        let mut natr = NormalizedAverageTrueRange::<RunningMovingAverage>::new(3).unwrap();

        let bar1 = Bar::new().high(10).low(7.5).close(9);
        let bar2 = Bar::new().high(11).low(9).close(9.5);
        let bar3 = Bar::new().high(9).low(5).close(8);

        assert_eq!(natr.next(&bar1), 2.5 * 100. / 9.);
        assert_eq!(natr.next(&bar2), 2.25 * 100. / 9.5);
        assert_eq!(natr.next(&bar3), 37.5);
    }

    #[test]
    fn test_reset() {
        let mut natr = NormalizedAverageTrueRange::<RunningMovingAverage>::new(9).unwrap();

        let bar1 = Bar::new().high(10).low(7.5).close(9);
        let bar2 = Bar::new().high(11).low(9).close(9.5);

        natr.next(&bar1);
        natr.next(&bar2);

        natr.reset();
        let bar3 = Bar::new().high(60).low(15).close(51);
        assert_eq!(natr.next(&bar3), 45.0 * 100.0 / 51.0);
    }

    #[test]
    fn test_default() {
        NormalizedAverageTrueRange::<RunningMovingAverage>::default();
    }

    #[test]
    fn test_display() {
        let indicator = NormalizedAverageTrueRange::<RunningMovingAverage>::new(8).unwrap();
        assert_eq!(format!("{}", indicator), "NATR(8)");
    }
}
