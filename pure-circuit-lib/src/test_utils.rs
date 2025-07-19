use itertools::Itertools;
use proptest::prelude::*;
use std::fmt::Debug;
use strum::IntoEnumIterator;

pub(crate) fn enum_strategy<T: IntoEnumIterator + Debug + Clone + 'static>()
-> impl Strategy<Value = T> {
    prop::sample::select(T::iter().collect_vec())
}
