use homotopy_core::monotone::MonotoneIterator;

#[test]
fn monotone_sequences() {
    let empty = MonotoneIterator::new(false, vec![]);
    assert_eq!(empty.collect::<Vec<_>>(), [vec![].into()],);

    let iterator0_1_2 = MonotoneIterator::new(false, vec![0..2, 0..2]);
    assert_eq!(
        iterator0_1_2.collect::<Vec<_>>(),
        [vec![0, 0].into(), vec![0, 1].into(), vec![1, 1].into()]
    );

    let strict_iterator0_1_2 = MonotoneIterator::new(true, vec![0..2, 0..2]);
    assert_eq!(
        strict_iterator0_1_2.collect::<Vec<_>>(),
        [vec![0, 1].into()]
    );

    let iterator0_3_3 = MonotoneIterator::new(false, vec![0..4, 0..4, 0..4]);
    assert_eq!(
        iterator0_3_3.collect::<Vec<_>>(),
        [
            vec![0, 0, 0].into(),
            vec![0, 0, 1].into(),
            vec![0, 0, 2].into(),
            vec![0, 0, 3].into(),
            vec![0, 1, 1].into(),
            vec![0, 1, 2].into(),
            vec![0, 1, 3].into(),
            vec![0, 2, 2].into(),
            vec![0, 2, 3].into(),
            vec![0, 3, 3].into(),
            vec![1, 1, 1].into(),
            vec![1, 1, 2].into(),
            vec![1, 1, 3].into(),
            vec![1, 2, 2].into(),
            vec![1, 2, 3].into(),
            vec![1, 3, 3].into(),
            vec![2, 2, 2].into(),
            vec![2, 2, 3].into(),
            vec![2, 3, 3].into(),
            vec![3, 3, 3].into(),
        ]
    );
    let strict_iterator0_3_3 = MonotoneIterator::new(true, vec![0..4, 0..4, 0..4]);
    assert_eq!(
        strict_iterator0_3_3.collect::<Vec<_>>(),
        [
            vec![0, 1, 2].into(),
            vec![0, 1, 3].into(),
            vec![0, 2, 3].into(),
            vec![1, 2, 3].into()
        ]
    );

    let iterator1_3_3 = MonotoneIterator::new(false, vec![1..4, 0..4, 1..4]);
    assert_eq!(
        iterator1_3_3.collect::<Vec<_>>(),
        [
            vec![1, 1, 1].into(),
            vec![1, 1, 2].into(),
            vec![1, 1, 3].into(),
            vec![1, 2, 2].into(),
            vec![1, 2, 3].into(),
            vec![1, 3, 3].into(),
            vec![2, 2, 2].into(),
            vec![2, 2, 3].into(),
            vec![2, 3, 3].into(),
            vec![3, 3, 3].into(),
        ]
    );
    let strict_iterator1_3_3 = MonotoneIterator::new(true, vec![1..4, 0..4, 1..4]);
    assert_eq!(
        strict_iterator1_3_3.collect::<Vec<_>>(),
        [vec![1, 2, 3].into()]
    );

    // unsatisfiable constraints
    let invalid_ms = MonotoneIterator::new(false, vec![1..2, 0..1]);
    assert_eq!(invalid_ms.collect::<Vec<_>>(), []);
}
