use orca_iot::utils::Slicer;

#[test]
fn test_slicer() {
    let slice = Slicer::new(&[1, 2, 3, 4, 5]);

    assert_eq!(slice.from(0).to_end(), &[1, 2, 3, 4, 5]);

    assert_eq!(slice.from(1).to_end(), &[2, 3, 4, 5]);

    assert_eq!(slice.from(2).to(4), &[3, 4, 5]);

    assert_eq!(slice.from_after(0).to_end(), &[2, 3, 4, 5])
}
