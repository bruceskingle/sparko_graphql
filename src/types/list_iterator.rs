pub struct ListIterator<T> {
    inner: ListIteratorInner<T>,
}

enum ListIteratorInner<T> {
    Values(std::vec::IntoIter<T>),
    OptionalValues(std::iter::Flatten<std::vec::IntoIter<Option<T>>>),
}

impl<T> Iterator for ListIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            ListIteratorInner::Values(iter) => iter.next(),
            ListIteratorInner::OptionalValues(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            ListIteratorInner::Values(iter) => iter.size_hint(),
            ListIteratorInner::OptionalValues(iter) => iter.size_hint(),
        }
    }
}

impl<T> From<Vec<T>> for ListIterator<T> {
    fn from(values: Vec<T>) -> Self {
        Self {
            inner: ListIteratorInner::Values(values.into_iter()),
        }
    }
}

impl<T> From<Vec<Option<T>>> for ListIterator<T> {
    fn from(values: Vec<Option<T>>) -> Self {
        Self {
            inner: ListIteratorInner::OptionalValues(values.into_iter().flatten()),
        }
    }
}

impl<T> From<Option<Vec<T>>> for ListIterator<T> {
    fn from(values: Option<Vec<T>>) -> Self {
        Self::from(values.unwrap_or_default())
    }
}

impl<T> From<Option<Vec<Option<T>>>> for ListIterator<T> {
    fn from(values: Option<Vec<Option<T>>>) -> Self {
        Self::from(values.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::ListIterator;

    #[test]
    fn iterates_values() {
        let values: Vec<_> = ListIterator::from(vec![1, 2, 3]).collect();

        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn skips_missing_elements() {
        let values: Vec<i32> = ListIterator::from(vec![Some(1), None, Some(3)]).collect();

        assert_eq!(values, vec![1, 3]);
    }

    #[test]
    fn treats_missing_list_as_empty() {
        let values: Vec<i32> = ListIterator::from(None::<Vec<i32>>).collect();

        assert!(values.is_empty());
    }

    #[test]
    fn skips_missing_elements_in_present_list() {
        let values: Vec<i32> =
            ListIterator::from(Some(vec![Some(1), None, Some(3)])).collect();

        assert_eq!(values, vec![1, 3]);
    }

    #[test]
    fn treats_missing_list_of_optional_elements_as_empty() {
        let values: Vec<i32> =
            ListIterator::from(None::<Vec<Option<i32>>>).collect();

        assert!(values.is_empty());
    }
}