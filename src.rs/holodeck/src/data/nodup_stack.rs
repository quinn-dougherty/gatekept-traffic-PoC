use std::cmp::Ordering;

#[derive(Debug)]
pub(crate) struct NoDupStack<T>
where
    T: PartialEq,
{
    stack: Vec<T>,
}

impl<T> NoDupStack<T>
where
    T: PartialEq,
{
    pub(crate) fn new() -> NoDupStack<T> {
        NoDupStack { stack: Vec::new() }
    }

    pub(crate) fn new_singleton(value: T) -> NoDupStack<T> {
        NoDupStack { stack: vec![value] }
    }

    pub(crate) fn push(&mut self, value: T) {
        if !self.stack.contains(&value) {
            self.stack.push(value);
        }
    }

    pub(crate) fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub(crate) fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub(crate) fn len(&self) -> usize {
        self.stack.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.stack.clear();
    }

    pub(crate) fn contains(&self, value: &T) -> bool {
        self.stack.contains(value)
    }

    pub(crate) fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.stack.sort_by(compare);
    }

    pub(crate) fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in iter {
            self.push(value);
        }
    }
}

impl<T> IntoIterator for NoDupStack<T>
where
    T: PartialEq,
{
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.stack.into_iter()
    }
}
