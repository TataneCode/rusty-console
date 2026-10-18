use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Default,
    NameAsc,
    NameDesc,
}

impl SortOrder {
    pub fn next(self) -> Self {
        match self {
            SortOrder::Default => SortOrder::NameAsc,
            SortOrder::NameAsc => SortOrder::NameDesc,
            SortOrder::NameDesc => SortOrder::Default,
        }
    }

    pub fn is_default(self) -> bool {
        self == SortOrder::Default
    }
}

pub fn apply_name_sort<T>(items: &mut Vec<&T>, order: SortOrder, key: impl Fn(&T) -> &str) {
    match order {
        SortOrder::Default => {}
        SortOrder::NameAsc => items.sort_by_key(|item| key(item).to_lowercase()),
        SortOrder::NameDesc => items.sort_by_key(|item| Reverse(key(item).to_lowercase())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_cycles_through_all_orders() {
        assert_eq!(SortOrder::Default.next(), SortOrder::NameAsc);
        assert_eq!(SortOrder::NameAsc.next(), SortOrder::NameDesc);
        assert_eq!(SortOrder::NameDesc.next(), SortOrder::Default);
    }

    #[test]
    fn test_is_default() {
        assert!(SortOrder::Default.is_default());
        assert!(!SortOrder::NameAsc.is_default());
        assert!(!SortOrder::NameDesc.is_default());
    }

    #[test]
    fn test_apply_name_sort_ascending_and_descending() {
        let mut a = "banana".to_string();
        let mut b = "apple".to_string();
        let mut c = "Cherry".to_string();
        let mut items = vec![&a, &b, &c];
        apply_name_sort(&mut items, SortOrder::NameAsc, |s| s);
        assert_eq!(
            items.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            vec!["apple", "banana", "Cherry"]
        );

        let mut items = vec![&a, &b, &c];
        apply_name_sort(&mut items, SortOrder::NameDesc, |s| s);
        assert_eq!(
            items.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            vec!["Cherry", "banana", "apple"]
        );
    }

    #[test]
    fn test_apply_name_sort_default_keeps_original_order() {
        let mut a = "zeta".to_string();
        let mut b = "alpha".to_string();
        let mut items = vec![&a, &b];
        apply_name_sort(&mut items, SortOrder::Default, |s| s);
        assert_eq!(
            items.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            vec!["zeta", "alpha"]
        );
    }
}
