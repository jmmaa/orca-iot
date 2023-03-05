#[derive(Debug)]
pub struct NoStartIndex;

#[derive(Debug)]
pub struct NoEndIndex;

#[derive(Debug)]
pub struct Slicer<'a, V, S = NoStartIndex> {
    // start state
    v: &'a [V],
    s: std::marker::PhantomData<S>,
    start: usize,
}

impl<'a, V> Slicer<'a, V, NoEndIndex> {
    pub fn to(&self, n: usize) -> &'a [V] {
        &self.v[self.start..=n]
    }

    pub fn to_end(&self) -> &'a [V] {
        &self.v[self.start..]
    }
}

impl<'a, V> Slicer<'a, V, NoStartIndex> {
    pub fn from(&self, n: usize) -> Slicer<'a, V, NoEndIndex> {
        Slicer {
            v: self.v,
            s: std::marker::PhantomData::<NoEndIndex>,
            start: n,
        }
    }

    pub fn from_after(&self, n: usize) -> Slicer<'a, V, NoEndIndex> {
        Slicer {
            v: self.v,
            s: std::marker::PhantomData::<NoEndIndex>,
            start: n + 1,
        }
    }

    pub fn up_to(&self, n: usize) -> &'a [V] {
        &self.v[..=n]
    }
}

impl<'a, V> Slicer<'a, V> {
    pub fn new(slice: &'a [V]) -> Slicer<'a, V, NoStartIndex> {
        Slicer {
            v: slice,
            s: std::marker::PhantomData::<NoStartIndex>,
            start: 0,
        }
    }
}
