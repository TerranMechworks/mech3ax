use std::iter::FusedIterator;

pub trait EnumerateEx: Iterator + Sized {
    fn enumerate_one(self) -> EnumerateOne<Self>;
}

impl<T> EnumerateEx for T
where
    T: Iterator + Sized,
{
    fn enumerate_one(self) -> EnumerateOne<Self> {
        EnumerateOne::new(self)
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct EnumerateOne<I> {
    iter: I,
    count: usize,
}

impl<I> EnumerateOne<I> {
    pub fn new(iter: I) -> EnumerateOne<I> {
        EnumerateOne { iter, count: 1 }
    }
}

impl<I> Iterator for EnumerateOne<I>
where
    I: Iterator,
{
    type Item = (usize, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<(usize, <I as Iterator>::Item)> {
        let a = self.iter.next()?;
        let i = self.count;
        self.count += 1;
        Some((i, a))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<I> ExactSizeIterator for EnumerateOne<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I> FusedIterator for EnumerateOne<I> where I: FusedIterator {}
