pub struct SPeekable<I, T> {
    iter: I,
    peeked: Option<Option<T>>,
}

impl<I, T> Iterator for SPeekable<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        }
    }
}

impl<I, T> SPeekable<I, T>
where
    I: Iterator<Item = T>,
{
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        let iter = &mut self.iter;
        self.peeked.get_or_insert_with(|| iter.next()).as_ref()
    }

    #[inline]
    pub fn set_peek(&mut self, t: Option<T>) {
        self.peeked = Some(t);
    }
}

pub trait IterExt: Iterator {
    fn speekable(self) -> SPeekable<Self, Self::Item>
    where
        Self: Sized,
    {
        SPeekable {
            iter: self,
            peeked: None,
        }
    }
}

impl<I> IterExt for I where I: Iterator {}
