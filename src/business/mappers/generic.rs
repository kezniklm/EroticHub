pub trait ToMappedList<T> {
    fn to_mapped_list<F>(self, mapper: F) -> Vec<T>
    where
        F: Fn(Self::Item) -> T,
        Self: Sized,
        Self: IntoIterator;
}

impl<S, T, I> ToMappedList<T> for I
where
    I: IntoIterator<Item = S>,
{
    fn to_mapped_list<F>(self, mapper: F) -> Vec<T>
    where
        F: Fn(S) -> T,
    {
        self.into_iter().map(mapper).collect()
    }
}
