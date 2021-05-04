use std::fmt;

use crate::errors::Result;
use crate::indicators::TrueRange;
use crate::{Close, High, Low, NewWithPeriod, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Average true range (ATR).
///
/// A technical analysis volatility indicator, originally developed by J. Welles Wilder.
/// The average true range is an N-day smoothed moving average of the true range values.
/// This implementation uses exponential moving average.
///
/// # Formula
///
/// ATR(period)<sub>t</sub> = EMA(period) of TR<sub>t</sub>
///
/// Where:
///
/// * _EMA(period)_ - [exponential moving average](struct.ExponentialMovingAverage.html) with smoothing period
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
/// use ta::indicators::{AverageTrueRange, ExponentialMovingAverage};
///
/// fn main() {
///     let data = vec![
///         // open, high, low, close, atr
///         (9.7   , 10.0, 9.0, 9.5  , 1.0),    // tr = high - low = 10.0 - 9.0 = 1.0
///         (9.9   , 10.4, 9.8, 10.2 , 0.95),   // tr = high - prev_close = 10.4 - 9.5 = 0.9
///         (10.1  , 10.7, 9.4, 9.7  , 1.125),  // tr = high - low = 10.7 - 9.4 = 1.3
///         (9.1   , 9.2 , 8.1, 8.4  , 1.3625), // tr = prev_close - low = 9.7 - 8.1 = 1.6
///     ];
///     let mut indicator = AverageTrueRange::<ExponentialMovingAverage>::new(3).unwrap();
///
///     for (open, high, low, close, atr) in data {
///         let di = DataItem::builder()
///             .high(high)
///             .low(low)
///             .close(close)
///             .open(open)
///             .volume(1000.0)
///             .build().unwrap();
///         assert_approx_eq!(indicator.next(&di), atr);
///     }
/// }
#[doc(alias = "ATR")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct AverageTrueRange<MA: Period + Reset + Next<f64> + NewWithPeriod> {
    true_range: TrueRange,
    ma: MA,
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> AverageTrueRange<MA> {
    pub fn new(period: usize) -> Result<Self> {
        Ok(Self {
            true_range: TrueRange::new(),
            ma: MA::with_period(period)?,
        })
    }
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> Period for AverageTrueRange<MA> {
    fn period(&self) -> usize {
        self.ma.period()
    }
}

impl<MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Next<f64>
    for AverageTrueRange<MA>
{
    type Output = f64;

    fn next(&mut self, input: f64) -> Self::Output {
        self.ma.next(self.true_range.next(input))
    }
}

impl<T: High + Low + Close, MA: Period + Reset + Next<f64, Output = f64> + NewWithPeriod> Next<&T>
    for AverageTrueRange<MA>
{
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        self.ma.next(self.true_range.next(input))
    }
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> Reset for AverageTrueRange<MA> {
    fn reset(&mut self) {
        self.true_range.reset();
        self.ma.reset();
    }
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> Default for AverageTrueRange<MA> {
    fn default() -> Self {
        Self::new(14).unwrap()
    }
}

impl<MA: Period + Reset + Next<f64> + NewWithPeriod> fmt::Display for AverageTrueRange<MA> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ATR({})", self.ma.period())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::*;
    use crate::test_helper::*;

    // test_indicator!(AverageTrueRange::<ExponentialMovingAverage>);

    #[test]
    fn test_new() {
        assert!(AverageTrueRange::<ExponentialMovingAverage>::new(0).is_err());
        assert!(AverageTrueRange::<ExponentialMovingAverage>::new(1).is_ok());
    }
    #[test]
    fn test_next() {
        let mut atr = AverageTrueRange::<ExponentialMovingAverage>::new(3).unwrap();

        let bar1 = Bar::new().high(10).low(7.5).close(9);
        let bar2 = Bar::new().high(11).low(9).close(9.5);
        let bar3 = Bar::new().high(9).low(5).close(8);

        assert_eq!(atr.next(&bar1), 2.5);
        assert_eq!(atr.next(&bar2), 2.25);
        assert_eq!(atr.next(&bar3), 3.375);
    }

    #[test]
    fn test_reset() {
        let mut atr = AverageTrueRange::<ExponentialMovingAverage>::new(9).unwrap();

        let bar1 = Bar::new().high(10).low(7.5).close(9);
        let bar2 = Bar::new().high(11).low(9).close(9.5);

        atr.next(&bar1);
        atr.next(&bar2);

        atr.reset();
        let bar3 = Bar::new().high(60).low(15).close(51);
        assert_eq!(atr.next(&bar3), 45.0);
    }

    #[test]
    fn test_default() {
        AverageTrueRange::<ExponentialMovingAverage>::default();
    }

    #[test]
    fn test_display() {
        let indicator = AverageTrueRange::<ExponentialMovingAverage>::new(8).unwrap();
        assert_eq!(format!("{}", indicator), "ATR(8)");
    }

    #[test]
    fn test_result() {
        use crate::{DataItem, Next};
        // let mut indicator = AverageTrueRange::<ExponentialMovingAverage>::new(8).unwrap();
        // let mut indicator = AverageTrueRange::<SimpleMovingAverage>::new(8).unwrap();
        let mut indicator = AverageTrueRange::<RunningMovingAverage>::new(14).unwrap();
        let data = get_data();
        for dt in data {
            let di = DataItem::builder()
                .high(dt.high)
                .low(dt.low)
                .close(dt.close)
                .open(dt.low)
                .volume(1000.0)
                .build()
                .unwrap();
            let val = indicator.next(&di);
            // eprintln!("{}", val);
            assert_eq!(val, dt.atr);
        }
        // assert_eq!(true, false);
    }

    struct HLC {
        high: f64,
        low: f64,
        close: f64,
        atr: f64,
    }

    impl HLC {
        fn new(high: f64, low: f64, close: f64, atr: f64) -> Self {
            Self {
                high,
                low,
                close,
                atr,
            }
        }
    }

    fn get_data() -> Vec<HLC> {
        let data = [
            [48438.01, 44100.00, 46313.13, 4338.010000000002],
            [48476.05, 44926.77, 46102.63, 3943.645000000004],
            [46678.00, 43002.05, 45163.36, 3854.4133333333352],
            [49800.00, 44972.49, 49592.70, 4097.687500000002],
            [50221.77, 47058.06, 48450.57, 3910.892000000001],
            [52681.51, 48110.66, 50373.75, 4020.8850000000007],
            [51800.00, 47500.00, 48385.77, 4060.758571428572],
            [49470.00, 46320.00, 48770.57, 3946.9137500000006],
            [49253.63, 47090.00, 48908.21, 3748.771111111111],
            [51480.00, 48908.21, 50995.27, 3631.0730000000003],
            [52450.00, 49310.00, 52429.43, 3586.4300000000003],
            [54950.00, 51880.00, 54917.78, 3543.394166666667],
            [57500.92, 53061.00, 55947.98, 3612.3576923076926],
            [58283.00, 54356.44, 57844.52, 3634.800714285714],
            [58186.33, 55074.30, 57291.03, 3597.4599489795914],
            [61950.00, 56136.00, 61267.41, 3755.7842383381917],
            [61798.00, 59005.00, 59037.12, 3687.0139355997494],
            [60740.00, 54517.28, 55646.61, 3868.1357973426243],
            [56988.00, 53335.00, 56954.19, 3852.768954675294],
            [59078.79, 54154.93, 58959.00, 3929.275457912773],
            [60510.00, 57072.00, 57700.01, 3894.1843537761465],
            [59550.00, 56301.27, 58089.11, 3848.080471363565],
            [59948.00, 57881.00, 58145.61, 3720.860437694739],
            [58638.70, 55403.55, 57415.18, 3686.166835002258],
            [58500.00, 53635.81, 54129.56, 3770.311346787811],
            [55890.68, 52981.40, 54400.00, 3708.809107731539],
            [57234.79, 51666.00, 52311.54, 3841.664885750715],
            [53255.20, 50452.08, 51325.21, 3767.483108197092],
            [55123.00, 51260.00, 55030.00, 3774.3057433258714],
            [56658.76, 54001.00, 55828.00, 3694.5524759454524],
            [56588.31, 54694.91, 55805.08, 3565.898727663634],
            [58490.11, 54931.15, 57677.64, 3565.403104259088],
            [59560.00, 57110.00, 58794.16, 3485.7314539548674],
            [59920.00, 56812.00, 58807.24, 3458.750635815234],
            [59592.22, 58002.00, 58794.01, 3325.284161828432],
            [60397.85, 58490.03, 59031.36, 3224.0367216978298],
            [59925.00, 56900.00, 57134.65, 3209.819813005128],
            [58553.00, 56517.23, 58234.61, 3125.95911207619],
            [59385.00, 56808.94, 59243.39, 3086.680604070748],
            [59626.18, 57500.00, 58059.99, 3018.0734180656946],
            [58720.00, 55536.07, 56018.18, 3029.920316775288],
            [58200.00, 55759.13, 58122.26, 2987.845294148482],
            [58950.00, 57700.00, 58176.03, 2863.7134874235903],
            [61800.00, 57943.95, 59832.43, 2934.5946668933343],
            [60816.88, 59304.02, 60090.20, 2833.0421906866677],
            [61449.91, 59500.00, 59911.18, 2769.9613199233345],
            [63850.00, 59873.01, 63649.71, 2856.1776542145244],
            [64986.11, 61404.00, 63019.10, 2908.029964627773],
            [63905.83, 62157.89, 63232.55, 2825.1663957257892],
            [63600.00, 60100.00, 61424.67, 2873.36879603109],
            [62610.00, 59620.00, 60090.14, 2881.6995963145832],
            [60450.00, 50050.00, 56140.05, 3418.7210537206843],
            [57532.70, 54200.00, 55634.39, 3412.5766927406353],
            [57060.00, 53330.82, 56410.56, 3435.191214687733],
            [56756.78, 53610.00, 53800.00, 3414.590413638609],
            [55470.51, 50534.34, 51714.61, 3523.27466980728],
            [52152.00, 47546.16, 51122.62, 3600.6007648210457],
            [51183.51, 48690.00, 50092.40, 3521.522853048114],
            [50577.90, 46930.43, 49063.94, 3530.519077830392],
            [54364.89, 48763.00, 53984.33, 3678.474143699649],
            [55500.00, 53225.14, 55006.25, 3578.215990578246],
            [56574.13, 53852.00, 54861.77, 3517.066991251228],
            [55222.02, 52350.00, 53558.80, 3470.992206161854],
            [57935.00, 53043.62, 57684.16, 3572.448477150293],
            [58488.17, 57051.23, 57836.34, 3419.912157353843],
            [57959.97, 56100.00, 56630.33, 3308.4877175428546],
            [59056.59, 56559.80, 58889.97, 3250.5093091469357],
        ];

        data.iter()
            .map(|v| HLC::new(v[0], v[1], v[2], v[3]))
            .collect::<Vec<_>>()
    }
}
