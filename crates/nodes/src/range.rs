#[derive(Debug, Clone)]
pub(crate) struct RangeI32 {
    start: i32,
    curr: i32,
    stop: i32,
    step: i32,
}

impl RangeI32 {
    pub(crate) fn new(start: i32, stop: i32, step: i32) -> Self {
        let modulo = (start - stop) % step;
        Self {
            start,
            curr: start,
            stop: stop + modulo,
            step,
        }
    }

    pub(crate) fn len(&self) -> i32 {
        (self.stop - self.start) / self.step
    }
}

impl Iterator for RangeI32 {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.curr != self.stop {
            Some(self.curr)
        } else {
            None
        };
        self.curr += self.step;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_range() {
        let range = RangeI32::new(0, 10, 1);
        let vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(range.len(), vec.len() as i32);
        assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);
        let range = RangeI32::new(0, 100, 10);
        let vec = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90];
        assert_eq!(range.len(), vec.len() as i32);
        assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);
    }

    #[test]
    fn negative_range() {
        let range = RangeI32::new(10, 0, -1);
        let vec = vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        assert_eq!(range.len(), vec.len() as i32);
        assert_eq!(range.into_iter().collect::<Vec<_>>(), vec);
    }
}
