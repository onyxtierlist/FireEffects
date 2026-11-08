use std::fmt;
use std::time::Duration;

use crate::{Error, Result};

/// An exact rational frame rate, such as `24/1`, `25/1`, or `24000/1001` (23.976 fps).
///
/// The rate is stored as a fraction rather than an `f64` on purpose: rates like
/// 23.976 and 29.97 are repeating decimals, so storing them as floating point
/// accumulates drift over a long timeline. Every conversion below uses integer
/// arithmetic and saturates instead of panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
    numerator: u32,
    denominator: u32,
}

impl FrameRate {
    /// 24 fps film rate.
    pub const FPS_24: Self = Self::from_parts(24, 1);
    /// 25 fps PAL rate.
    pub const FPS_25: Self = Self::from_parts(25, 1);
    /// 30 fps rate.
    pub const FPS_30: Self = Self::from_parts(30, 1);
    /// 50 fps rate.
    pub const FPS_50: Self = Self::from_parts(50, 1);
    /// 60 fps rate.
    pub const FPS_60: Self = Self::from_parts(60, 1);
    /// 23.976 fps (`24000/1001`), the NTSC film rate.
    pub const FPS_23_976: Self = Self::from_parts(24_000, 1_001);
    /// 29.97 fps (`30000/1001`), the NTSC video rate.
    pub const FPS_29_97: Self = Self::from_parts(30_000, 1_001);
    /// 59.94 fps (`60000/1001`), the NTSC high frame rate.
    pub const FPS_59_94: Self = Self::from_parts(60_000, 1_001);

    /// Creates a frame rate from a fraction, rejecting zero parts.
    ///
    /// ```
    /// # use fireeffects_core::FrameRate;
    /// let rate = FrameRate::new(24_000, 1_001)?;
    /// assert!(rate.is_ntsc());
    /// # Ok::<(), fireeffects_core::Error>(())
    /// ```
    pub const fn new(numerator: u32, denominator: u32) -> Result<Self> {
        if numerator == 0 {
            return Err(Error::ZeroFrameRateNumerator);
        }
        if denominator == 0 {
            return Err(Error::ZeroFrameRateDenominator);
        }
        Ok(Self::from_parts(numerator, denominator))
    }

    const fn from_parts(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// The number of frames per second in this rate's numerator.
    pub const fn numerator(&self) -> u32 {
        self.numerator
    }

    /// The divisor applied to [`Self::numerator`].
    pub const fn denominator(&self) -> u32 {
        self.denominator
    }

    /// The rate as a floating point number of frames per second.
    ///
    /// Use this for display only. Do not accumulate time with it; use
    /// [`Self::duration_for_frames`] and [`Self::frames_for_duration`] instead.
    pub fn fps(&self) -> f64 {
        f64::from(self.numerator) / f64::from(self.denominator)
    }

    /// Whether this is an NTSC rate based on a `1001` denominator.
    pub const fn is_ntsc(&self) -> bool {
        self.denominator == 1_001
    }

    /// The exact duration covered by `frames` frames, truncated to whole nanoseconds.
    ///
    /// Saturates at [`Duration::MAX`] instead of overflowing.
    pub fn duration_for_frames(&self, frames: u64) -> Duration {
        let nanos = u128::from(frames) * u128::from(self.denominator) * 1_000_000_000
            / u128::from(self.numerator);
        match u64::try_from(nanos) {
            Ok(nanos) => Duration::from_nanos(nanos),
            Err(_) => Duration::MAX,
        }
    }

    /// How many whole frames fit in `duration`, rounded down.
    ///
    /// The result is a floor, so a duration that ends mid-frame reports the number
    /// of complete frames before it.
    pub fn frames_for_duration(&self, duration: Duration) -> u64 {
        let frames = duration.as_nanos() * u128::from(self.numerator)
            / (u128::from(self.denominator) * 1_000_000_000);
        u64::try_from(frames).unwrap_or(u64::MAX)
    }
}

impl fmt::Display for FrameRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = format!("{:.3}", self.fps());
        let text = text.trim_end_matches('0').trim_end_matches('.');
        write!(f, "{text} fps")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_parts() {
        assert_eq!(FrameRate::new(0, 1), Err(Error::ZeroFrameRateNumerator));
        assert_eq!(FrameRate::new(24, 0), Err(Error::ZeroFrameRateDenominator));
        assert_eq!(FrameRate::new(0, 0), Err(Error::ZeroFrameRateNumerator));
        assert!(FrameRate::new(24, 1).is_ok());
    }

    #[test]
    fn exposes_its_parts() {
        let rate = FrameRate::new(30_000, 1_001).expect("valid rate");
        assert_eq!(rate.numerator(), 30_000);
        assert_eq!(rate.denominator(), 1_001);
    }

    #[test]
    fn reports_ntsc_rates() {
        assert!(FrameRate::FPS_23_976.is_ntsc());
        assert!(FrameRate::FPS_29_97.is_ntsc());
        assert!(FrameRate::FPS_59_94.is_ntsc());
        assert!(!FrameRate::FPS_24.is_ntsc());
        assert!(!FrameRate::FPS_25.is_ntsc());
    }

    #[test]
    fn converts_frame_counts_to_durations() {
        assert_eq!(FrameRate::FPS_25.duration_for_frames(0), Duration::ZERO);
        assert_eq!(
            FrameRate::FPS_25.duration_for_frames(25),
            Duration::from_secs(1)
        );
        assert_eq!(
            FrameRate::FPS_60.duration_for_frames(1),
            Duration::from_nanos(16_666_666)
        );
        assert_eq!(
            FrameRate::FPS_23_976.duration_for_frames(1),
            Duration::from_nanos(41_708_333)
        );
        assert_eq!(
            FrameRate::FPS_23_976.duration_for_frames(24_000),
            Duration::from_secs(1_001)
        );
    }

    #[test]
    fn converts_durations_to_whole_frames() {
        assert_eq!(
            FrameRate::FPS_25.frames_for_duration(Duration::from_secs(1)),
            25
        );
        assert_eq!(
            FrameRate::FPS_24.frames_for_duration(Duration::from_millis(1_500)),
            36
        );
        // A duration shorter than a single frame contains no complete frames.
        assert_eq!(
            FrameRate::FPS_24.frames_for_duration(Duration::from_millis(10)),
            0
        );
    }

    #[test]
    fn handles_long_timelines_exactly() {
        // One hour of 60 fps material is a common export length.
        let hour = Duration::from_secs(3_600);
        assert_eq!(FrameRate::FPS_60.frames_for_duration(hour), 216_000);
        assert_eq!(
            FrameRate::FPS_60.duration_for_frames(216_000),
            Duration::from_secs(3_600)
        );
    }

    #[test]
    fn saturates_instead_of_panicking() {
        assert_eq!(
            FrameRate::FPS_60.duration_for_frames(u64::MAX),
            Duration::MAX
        );
        assert_eq!(
            FrameRate::FPS_23_976.duration_for_frames(u64::MAX),
            Duration::MAX
        );
    }

    #[test]
    fn formats_rates_for_display() {
        assert_eq!(FrameRate::FPS_24.to_string(), "24 fps");
        assert_eq!(FrameRate::FPS_50.to_string(), "50 fps");
        assert_eq!(FrameRate::FPS_23_976.to_string(), "23.976 fps");
        assert_eq!(FrameRate::FPS_29_97.to_string(), "29.97 fps");
        assert_eq!(FrameRate::FPS_59_94.to_string(), "59.94 fps");
    }
}
