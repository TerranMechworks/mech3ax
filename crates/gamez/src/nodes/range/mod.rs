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
mod tests;
