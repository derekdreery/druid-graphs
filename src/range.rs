/// Maintains invariants: `-∞ < min <= max < ∞`
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range {
    min: f64,
    max: f64,
}

impl Range {
    #[inline]
    pub fn new(min: f64, max: f64) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min <= max,
            "-∞ < {} <= {} < ∞",
            min,
            max
        );
        Range { min, max }
    }

    #[inline]
    pub fn as_tuple(self) -> (f64, f64) {
        (self.min, self.max)
    }

    #[inline]
    pub fn min(&self) -> f64 {
        self.min
    }

    #[inline]
    pub fn max(&self) -> f64 {
        self.max
    }

    #[inline]
    pub fn set_min(&mut self, min: f64) -> &mut Self {
        assert!(min.is_finite() && min <= self.max);
        self.min = min;
        self
    }

    #[inline]
    pub fn set_max(&mut self, max: f64) -> &mut Self {
        assert!(max.is_finite() && max >= self.min);
        self.max = max;
        self
    }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// Returns true if the range changed.
    pub fn extend_to(&mut self, val: f64) -> bool {
        // NaN will be ignored.
        if val < self.min {
            self.min = val;
            true
        } else if val > self.max {
            self.max = val;
            true
        } else {
            false
        }
    }
}

impl From<(f64, f64)> for Range {
    fn from((min, max): (f64, f64)) -> Self {
        Self::new(min, max)
    }
}

impl From<Range> for (f64, f64) {
    fn from(range: Range) -> (f64, f64) {
        (range.min(), range.max())
    }
}
