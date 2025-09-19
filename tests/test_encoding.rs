use encoding_rs;

#[test]
fn test_decode() {
    let content = "Hello\nWorld\nTest".as_bytes();
    let encoding = encoding_rs::Encoding::for_label(b"utf-8").unwrap();
    let (decoded, _, has_error) = encoding.decode(content);
    assert_eq!(decoded, "Hello\nWorld\nTest");
    assert!(!has_error);
}

#[test]
fn test_stream_decode() {
    let encoding = encoding_rs::Encoding::for_label(b"utf-8").unwrap();
    let mut decoder = encoding.new_decoder();

    let content = "Hello\nWorld\nTest".as_bytes();
    let mut buffer = String::with_capacity(128);
    let (result, read, had_error) = decoder.decode_to_string(content, &mut buffer, true);
    println!("result: {:?}, read: {}, had_error: {}", result, read, had_error);
    assert_eq!(buffer, "Hello\nWorld\nTest");
    assert!(!had_error);
}

#[test]
fn test_stream_gbk_decode() {
    let encoding = encoding_rs::Encoding::for_label(b"gbk").unwrap();
    let data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xCA, 0xC0, 0xBD, 0xE7];

    let mut decoder = encoding.new_decoder();
    let mut buffer = String::with_capacity(128);
    let (result, read, had_error) = decoder.decode_to_string(&data[..3], &mut buffer, false);
    println!("result: {:?}, read: {}, had_error: {}", result, read, had_error);
    assert_eq!(buffer, "你");
    assert_eq!(result, encoding_rs::CoderResult::InputEmpty);
    assert_eq!(read, 3);
    assert!(!had_error);

    let (result, read, had_error) = decoder.decode_to_string(&data[3..], &mut buffer, true);
    println!("result: {:?}, read: {}, had_error: {}", result, read, had_error);
    assert_eq!(buffer, "你好\n世界");
    assert!(!had_error);
    assert_eq!(result, encoding_rs::CoderResult::InputEmpty);
    assert_eq!(read, 6);
}

#[test]
fn test_stream_gbk_decode_error() {
    let encoding = encoding_rs::Encoding::for_label(b"gbk").unwrap();
    let data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xCA, 0xC0, 0xBD, 0xE7];

    let mut decoder = encoding.new_decoder();
    let mut buffer = String::with_capacity(128);
    let (result, read, had_error) = decoder.decode_to_string(&data[..3], &mut buffer, true);
    println!("result: {:?}, read: {}, had_error: {}", result, read, had_error);
    assert_eq!(buffer, "你�");
    assert_eq!(result, encoding_rs::CoderResult::InputEmpty);
    assert_eq!(read, 3);
    assert!(had_error);

    let mut decoder = encoding.new_decoder();
    let mut buffer = String::with_capacity(128);
    let (result, read) = decoder.decode_to_string_without_replacement(&data[..3], &mut buffer, true);
    println!("result: {:?}, read: {}", result, read);
    assert_eq!(buffer, "你");
    assert_eq!(result, encoding_rs::DecoderResult::Malformed(1, 0));
    assert_eq!(read, 3);
}

#[test]
fn test_detect() {
    let data = vec![0xEF, 0xBB, 0xBF, 0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xCA, 0xC0, 0xBD, 0xE7];
    let (_, encoding, _) = encoding_rs::UTF_16LE.decode(&data[..]);
    assert_eq!(encoding.name(), "UTF-8");
}
