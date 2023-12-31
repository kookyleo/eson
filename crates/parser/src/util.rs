use crate::expr_token::ExprToken;

pub(crate) struct Iter<T> {
    inner: Vec<T>,
    cursor: usize,
}

impl From<Vec<ExprToken>> for Iter<ExprToken> {
    fn from(inner: Vec<ExprToken>) -> Self {
        Iter::new(inner)
    }
}

impl<T> Iter<T> {
    fn new(inner: Vec<T>) -> Self {
        Iter { inner, cursor: 0 }
    }

    pub fn take_next(&mut self) -> Option<T> {
        if self.cursor >= self.inner.len() {
            return None;
        }
        let item = self.inner.remove(self.cursor);
        Some(item)
    }

    pub(crate) fn next(&mut self) -> Option<&T> {
        let token = self.inner.get(self.cursor);
        self.cursor += 1;
        token
    }

    pub(crate) fn peek(&self) -> Option<&T> {
        if self.cursor >= self.inner.len() {
            return None;
        }
        Some(&self.inner[self.cursor])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut iter = Iter::new(vec![1, 2, 3]);
        assert_eq!(iter.peek(), Some(&1));
        assert_eq!(iter.peek(), Some(&1));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.peek(), Some(&2));
        assert_eq!(iter.peek(), Some(&2));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.peek(), Some(&3));
        assert_eq!(iter.peek(), Some(&3));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.next(), None);
    }
}