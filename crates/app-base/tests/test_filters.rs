use app_base::filters::{base64_decode, base64_encode};

#[test]
fn test_base64() {
    let str = "Hello World!";

    for pad in [true, false] {
        let encode_str = base64_encode(&str, pad).unwrap();
        dbg!(&encode_str);

        let decode_str = base64_decode(&encode_str, pad).unwrap();
        dbg!(&decode_str);

        assert_eq!(str, &decode_str);
    }
}
