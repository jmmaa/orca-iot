use orca_iot::utils;

#[test]
fn test_split_marker() {
    let marker = b'$';
    let data = &[1, 0, 2, 3, 4, 0, 5, marker, 6, 0, 7, 8, 9, 0, 10]
        .iter()
        .filter_map(|&b| if b != 0 { Some(b) } else { None })
        .collect::<Vec<u8>>();

    let (bytes, excess) = utils::split_buffer(data, marker);

    assert_eq!(&[1, 2, 3, 4, 5], bytes);

    assert_eq!(&[6, 7, 8, 9, 10], excess)
}
