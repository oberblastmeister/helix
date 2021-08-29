pub enum Amount3<T> {
    One(T),
    Two(T, T),
    Three(T, T, T),
}

impl<T> Amount3<T> {
    fn push_left(self, t: T) -> Self {
        use Amount3::*;
        match self {
            One(a) => Two(t, a),
            Two(a, b) => Three(t, a, b),
            Three(_, _, _) => panic!(),
        }
    }

    fn split(self) -> (T, Option<Self>) {
        match self {
            Amount3::One(a) => (a, None),
            Amount3::Two(a, b) => (a, Some(Amount3::One(b))),
            Amount3::Three(a, b, c) => (a, Some(Amount3::Two(b, c))),
        }
    }

    fn next_window<I>(self, iter: &mut I) -> (T, Option<Self>)
    where
        I: Iterator<Item = T>,
    {
        use Amount3::*;

        match self {
            One(a) => (a, next_amount3(iter)),
            Two(a, b) => (a, next_amount2(iter).map(|a| a.push_left(b))),
            Three(a, b, c) => (a, iter.next().map(|i| Three(b, c, i))),
        }
    }
}

impl<T> From<(T,)> for Amount3<T> {
    fn from(t: (T,)) -> Self {
        Amount3::One(t.0)
    }
}

impl<T> From<(T, T)> for Amount3<T> {
    fn from(t: (T, T)) -> Self {
        Amount3::Two(t.0, t.1)
    }
}

impl<T> From<(T, T, T)> for Amount3<T> {
    fn from(t: (T, T, T)) -> Self {
        Amount3::Three(t.0, t.1, t.2)
    }
}

pub struct CoalesceBy3<I, F, G, T> {
    iter: I,
    last: Option<Amount3<T>>,
    f: F,
    g: G,
}

impl<I, F, G, T> CoalesceBy3<I, F, G, T> where I: Iterator // F: FnMut(I::Item, T, T) -> A,,,,,,,,,,,
{
}

impl<I, F, G, T, A> Iterator for CoalesceBy3<I, F, G, T>
where
    I: Iterator<Item = T>,
    F: FnMut(T, T, T) -> A,
    G: FnMut(T, T) -> A,
    A: Into<Amount3<T>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // this fuses the iterator
        let last = self.last.take()?;
        let self_last = &mut self.last;
        let self_iter = &mut self.iter;

        Some(match last {
            Amount3::One(a) => a,
            Amount3::Two(a, b) => {
            }
            Amount3::Two(a, b) => {
                let (t, new_last) = (self.g)(a, b).into().next_window(self_iter);
                *self_last = new_last;
                t
            }
            Amount3::Three(a, b, c) => {
                let (t, new_last) = (self.f)(a, b, c).into().next_window(self_iter);
                *self_last = new_last;
                t
            }
        })
    }
}

fn coelesce3<I, F, G, T>(mut iter: I, f: F, g: G) -> CoalesceBy3<I, F, G, T>
where
    I: Iterator<Item = T>,
{
    let last = next_amount3(&mut iter);

    CoalesceBy3 { iter, last, f, g }
}

fn next_amount2<I, T>(iter: &mut I) -> Option<Amount3<T>>
where
    I: Iterator<Item = T>,
{
    let a = iter.next()?;
    Some(match iter.next() {
        None => Amount3::One(a),
        Some(b) => Amount3::Two(a, b),
    })
}

fn next_amount3<I, T>(iter: &mut I) -> Option<Amount3<T>>
where
    I: Iterator<Item = T>,
{
    let a = iter.next()?;
    Some(match iter.next() {
        None => Amount3::One(a),
        Some(b) => match iter.next() {
            None => Amount3::Two(a, b),
            Some(c) => Amount3::Three(a, b, c),
        },
    })
}
