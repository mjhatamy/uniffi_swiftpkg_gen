pub(crate) use crate::errors::*;
pub(crate) use crate::utils::constants::*;

#[allow(unused)]
pub fn key_eq(l_val: Option<&Vec<u8>>, r_val: Option<&Vec<u8>>) -> bool {
    let mut acc = 0_i16;
    if let Some(l) = l_val {
        if let Some(r) = r_val  {
            if l.len() != GXT_KEY_LEN || r.len() != GXT_KEY_LEN {
                return false;
            }
            for i in 0..GXT_KEY_LEN - 1 {
                acc |= (l[i] ^ r[i]) as i16;
            }
            return (1 & ((acc - 1) >> 8)) == 1;
        }
    }
    false
}

#[allow(unused)]
fn encode_base64(dest: &mut [u8; 4], src: [u8; 3]) {
    let input: [u8; 4] = [
        (src[0] >> 2) & 63,
        ((src[0] << 4) | (src[1] >> 4)) & 63,
        ((src[1] << 2) | (src[2] >> 6)) & 63,
        src[2] & 63,
    ];
    let mut _input: [i32; 4] = [0; 4];
    for i in 0..4 {
        let item = input[i] as i32;
        _input[i] = item + b'A' as i32 + (((25 - item) >> 8_i8) & 6_i32)
            - (((51 - item) >> 8_i8) & 75_i32)
            - (((61 - item) >> 8_i8) & 15_i32)
            + (((62 - item) >> 8_i8) & 3_i32);
    }
    let c = _input.iter().map(|f| (*f & 0xff) as u8);
    for (i, a) in c.enumerate() {
        dest[i] = a;
    }
}

#[allow(unused)]
pub fn key_to_base64(key: Option<&Vec<u8>>) -> Result<String, GixTunnelErrorKind> {
    let mut res: [u8; 4] = [0_u8; 4];
    let mut key_slice: [u8; 3] = [0_u8; 3];
    let key = match key {
        Some(x) => Ok(x),
        None => Err(GixTunnelErrorKind::NullInput),
    }?;
    match key.len() {
        GXT_KEY_LEN => {
            let mut base64 = vec![0_u8; GXT_KEY_LEN_BASE64 - 1];
            for i in 0..(GXT_KEY_LEN / 3) {
                key_slice.copy_from_slice(&key[(i * 3)..(i * 3 + 3)]);
                encode_base64(&mut res, key_slice);
                base64[(i * 4)..(i * 4 + 4)].copy_from_slice(&res);
            }
            key_slice = [key[10 * 3], key[10 * 3 + 1], 0];
            encode_base64(&mut res, key_slice);
            for index in 0..4 {
                base64[(10 * 4) + index] = res[index];
            }
            base64[(GXT_KEY_LEN_BASE64 - 2)] = b'=';
            Ok(String::from_utf8(base64).expect("Must succeed."))
        }
        _ => Err(GixTunnelErrorKind::InvalidInputLength),
    }
}

fn decode_base64(_src: [u8; 4]) -> i32 {
    let mut val: i32 = 0;
    let src: Vec<i32> = _src.iter().map(|f| *f as i32).collect();

    //let mut i = 0_i32;
    for (i, item) in src.iter().enumerate() {
        val |= (-1
            + (((((b'A' as i32 - 1) - item) & (item - (b'Z' as i32 + 1))) >> 8_u8) & (item - 64))
            + (((((b'a' as i32 - 1) - item) & (item - (b'z' as i32 + 1))) >> 8_u8) & (item - 70))
            + (((((b'0' as i32 - 1) - item) & (item - (b'9' as i32 + 1))) >> 8_u8) & (item + 5))
            + (((((b'+' as i32 - 1) - item) & (item - (b'+' as i32 + 1))) >> 8_u8) & 63_i32)
            + (((((b'/' as i32 - 1) - item) & (item - (b'/' as i32 + 1))) >> 8_u8) & 64_i32))
            << (18 - 6 * i);
    }
    val
}

#[allow(unused)]
pub fn key_from_base64(base64: Option<String>) -> Result<Vec<u8>, GixTunnelErrorKind> {
    match base64 {
        Some(base64_str) => {
            let x = match base64_str {
                x if x.len() == (GXT_KEY_LEN_BASE64 - 1) => match x {
                    y if y.as_bytes()[GXT_KEY_LEN_BASE64 - 2] == b'=' => Ok(y),
                    _ => Err(GixTunnelErrorKind::InvalidInput),
                },
                x if x.len() < GXT_KEY_LEN_BASE64 - 1 => {
                    Err(GixTunnelErrorKind::InvalidInputLength)
                }
                _ => Err(GixTunnelErrorKind::InvalidInputLength),
            }?;
            let _x = x.as_bytes();

            let mut key = vec![0_u8; GXT_KEY_LEN]; //&mut [u8; GXT_KEY_LEN]
            let mut ret = 0_u8;
            let mut base_code2: [u8; 4] = [0; 4];
            for i in 0..(GXT_KEY_LEN / 3) {
                base_code2.copy_from_slice(&_x[i * 4..(i * 4 + 4)]);
                let val = decode_base64(base_code2);
                ret |= (val >> 31) as u8;
                key[i * 3] = ((val >> 16) & 0xff) as u8;
                key[i * 3 + 1] = ((val >> 8) & 0xff) as u8;
                key[i * 3 + 2] = (val & 0xff) as u8;
            }
            base_code2.copy_from_slice(&[_x[10 * 4], _x[10 * 4 + 1], _x[10 * 4 + 2], b'A']);
            let val = decode_base64(base_code2);
            ret |= ((val >> 31) as u8) | (val & 0xff) as u8;
            key[10 * 3] = ((val >> 16) & 0xff) as u8;
            key[10 * 3 + 1] = ((val >> 8) & 0xff) as u8;

            if 1 & ((ret as i32 - 1) >> 8) == 1 {
                Ok(key)
            } else {
                Err(GixTunnelErrorKind::Failed)
            }
        }
        None => Err(GixTunnelErrorKind::NullInput),
    }
}

#[allow(unused)]
pub fn key_to_hex(key: Option<&Vec<u8>>) -> Result<Vec<u8>, GixTunnelErrorKind> {
    let mut hex = vec![0_u8; GXT_KEY_LEN_HEX - 1];
    match key {
        Some(key) if key.len() == GXT_KEY_LEN => {
            for (i, item) in key.iter().enumerate() {
                let p: u8 = if *item > 159 { 87 } else { 48 };
                let p2: u8 = if (*item & 0xf) > 9 { 87 } else { 48 };
                //hex[i * 2] = 87 + (item >> 4) +  ((((item >> 4) - 10)  >> 8) & !38);
                hex[i * 2] = p + (item >> 4);
                //hex[i * 2 + 1] = 87 + (item & 0xf) +  ((((item & 0xf) - 10)  >> 8) & !38);
                hex[i * 2 + 1] = p2 + (item & 0xf);
            }
            Ok(hex)
        }
        Some(key) => {
            println!(
                "Input key is too short or longer. len: {}\nkey: {:?}",
                key.len(),
                key
            );
            Err(GixTunnelErrorKind::InvalidInputLength)
        }
        None => {
            println!("Failed");
            Err(GixTunnelErrorKind::NullInput)
        }
    }
}

#[allow(unused)]
pub fn key_from_hex(hex_str: Option<Vec<u8>>) -> Result<Vec<u8>, GixTunnelErrorKind> {
    match hex_str {
        Some(hex) if hex.len() == GXT_KEY_LEN_HEX - 1 => {
            let mut out = vec![0_u8; GXT_KEY_LEN];
            let mut c1: i16;
            let mut c2: i16;
            let mut c_acc1: i16;
            let mut c_acc2: i16;
            let mut key_val: i16;
            let mut c_num0_0: i16;
            let mut c_num0_1: i16;
            let mut c_alpha0_0: i16;
            let mut c_alpha0_1: i16;
            let mut ret: i16 = 0;

            let calc_acc = |val: i16| match val {
                48..=58 => val - 48,
                65..=70 => val - 55,
                97..=102 => val - 87,
                _ => 0,
            };

            let calc_c_alpha0 = |val: i16| match val {
                65..=70 => 255,
                97..=102 => 255,
                _ => 0,
            };

            for i in (0..GXT_KEY_LEN_HEX - 1).step_by(2) {
                c1 = hex[i] as i16;
                c2 = hex[i + 1] as i16;
                c_num0_0 = if c1 > 48 && c1 < 58 { 0xff } else { 0 };
                c_num0_1 = if c2 > 48 && c2 < 58 { 0xff } else { 0 };
                c_alpha0_0 = calc_c_alpha0(c1);
                c_alpha0_1 = calc_c_alpha0(c2);

                ret |= ((c_num0_0 | c_alpha0_0) - 1) >> 8;
                ret |= ((c_num0_1 | c_alpha0_1) - 1) >> 8;
                c_acc1 = calc_acc(c1) * 16;
                c_acc2 = calc_acc(c2);
                key_val = c_acc1 | c_acc2;

                out[i / 2] = key_val as u8;
            }
            if (1 & ((ret - 1) >> 8)) == 1 {
                Ok(out)
            } else {
                Err(GixTunnelErrorKind::Failed)
            }
        }
        Some(hex) if hex.len() != GXT_KEY_LEN_HEX - 1 => {
            println!(" hex len is short {}", hex.len());
            Err(GixTunnelErrorKind::InvalidInputLength)
        }
        None => Err(GixTunnelErrorKind::NullInput),
        _ => Err(GixTunnelErrorKind::NullInput),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_from_base64_test() {
        let base64 = [
            "ADdgjBTmzc7FCXBFxgD5Pz3UXal7TDqlE95IjJXs9kI=",
            "6L5h4vbCEVRLgxZ/znpNUD+0m/Fa+DtY4eV4DAh4pEA=",
            "2FzSWHf/Cib3Knl8VuIy5skedNVgXOsr/BKdW4gnul8=",
            "uBDEHfrtdLPUTDI4UH4Obl8znsAHEPxHl6OL2bE1wXg=",
            "6NPKNjP8o2TovHam9JMSIuuXoWyll88f4UMNgF37o1k=",
        ];

        let expected_key_result: Vec<Vec<u8>> = vec![
            vec![
                00_u8, 0x37, 0x60, 0x8c, 0x14, 0xe6, 0xcd, 0xce, 0xc5, 0x09, 0x70, 0x45, 0xc6, 00,
                0xf9, 0x3f, 0x3d, 0xd4, 0x5d, 0xa9, 0x7b, 0x4c, 0x3a, 0xa5, 0x13, 0xde, 0x48, 0x8c,
                0x95, 0xec, 0xf6, 0x42,
            ],
            vec![
                0xe8, 0xbe, 0x61, 0xe2, 0xf6, 0xc2, 0x11, 0x54, 0x4b, 0x83, 0x16, 0x7f, 0xce, 0x7a,
                0x4d, 0x50, 0x3f, 0xb4, 0x9b, 0xf1, 0x5a, 0xf8, 0x3b, 0x58, 0xe1, 0xe5, 0x78, 0x0c,
                0x08, 0x78, 0xa4, 0x40,
            ],
            vec![
                0xd8, 0x5c, 0xd2, 0x58, 0x77, 0xff, 0x0a, 0x26, 0xf7, 0x2a, 0x79, 0x7c, 0x56, 0xe2,
                0x32, 0xe6, 0xc9, 0x1e, 0x74, 0xd5, 0x60, 0x5c, 0xeb, 0x2b, 0xfc, 0x12, 0x9d, 0x5b,
                0x88, 0x27, 0xba, 0x5f,
            ],
            vec![
                0xb8, 0x10, 0xc4, 0x1d, 0xfa, 0xed, 0x74, 0xb3, 0xd4, 0x4c, 0x32, 0x38, 0x50, 0x7e,
                0x0e, 0x6e, 0x5f, 0x33, 0x9e, 0xc0, 0x07, 0x10, 0xfc, 0x47, 0x97, 0xa3, 0x8b, 0xd9,
                0xb1, 0x35, 0xc1, 0x78,
            ],
            vec![
                0xe8, 0xd3, 0xca, 0x36, 0x33, 0xfc, 0xa3, 0x64, 0xe8, 0xbc, 0x76, 0xa6, 0xf4, 0x93,
                0x12, 0x22, 0xeb, 0x97, 0xa1, 0x6c, 0xa5, 0x97, 0xcf, 0x1f, 0xe1, 0x43, 0x0d, 0x80,
                0x5d, 0xfb, 0xa3, 0x59,
            ],
        ];

        for (i, item_base64) in base64.iter().enumerate() {
            let res = key_from_base64(Option::Some(item_base64.to_string())).unwrap();
            assert_eq!(expected_key_result[i], res);
        }
    }

    #[test]
    fn test_encode_base64() {
        let mut dest: [u8; 4] = [0_u8; 4];
        let src: [u8; 3] = [0x15, 0xb7, 0x1c];

        let expected_result_dest: [u8; 4] = [0x46_u8, 0x62_u8, 0x63_u8, 0x63_u8];
        encode_base64(&mut dest, src);
        //println!("encode_base64 result. dest: {:02X?}", dest.map(|f| f as u8));

        assert_eq!(expected_result_dest, dest, "key_from_base64");
        println!("Test::> key_from_base64 passed.");
    }

    #[test]
    fn test_key_to_base64() {
        let keys: Vec<Vec<u8>> = vec![
            vec![
                00, 0x37, 0x60, 0x8c, 0x14, 0xe6, 0xcd, 0xce, 0xc5, 0x09, 0x70, 0x45, 0xc6, 00,
                0xf9, 0x3f, 0x3d, 0xd4, 0x5d, 0xa9, 0x7b, 0x4c, 0x3a, 0xa5, 0x13, 0xde, 0x48, 0x8c,
                0x95, 0xec, 0xf6, 0x42,
            ],
            vec![
                0xe8, 0xbe, 0x61, 0xe2, 0xf6, 0xc2, 0x11, 0x54, 0x4b, 0x83, 0x16, 0x7f, 0xce, 0x7a,
                0x4d, 0x50, 0x3f, 0xb4, 0x9b, 0xf1, 0x5a, 0xf8, 0x3b, 0x58, 0xe1, 0xe5, 0x78, 0x0c,
                0x08, 0x78, 0xa4, 0x40,
            ],
            vec![
                0xd8, 0x5c, 0xd2, 0x58, 0x77, 0xff, 0x0a, 0x26, 0xf7, 0x2a, 0x79, 0x7c, 0x56, 0xe2,
                0x32, 0xe6, 0xc9, 0x1e, 0x74, 0xd5, 0x60, 0x5c, 0xeb, 0x2b, 0xfc, 0x12, 0x9d, 0x5b,
                0x88, 0x27, 0xba, 0x5f,
            ],
            vec![
                0xb8, 0x10, 0xc4, 0x1d, 0xfa, 0xed, 0x74, 0xb3, 0xd4, 0x4c, 0x32, 0x38, 0x50, 0x7e,
                0x0e, 0x6e, 0x5f, 0x33, 0x9e, 0xc0, 0x07, 0x10, 0xfc, 0x47, 0x97, 0xa3, 0x8b, 0xd9,
                0xb1, 0x35, 0xc1, 0x78,
            ],
            vec![
                0xe8, 0xd3, 0xca, 0x36, 0x33, 0xfc, 0xa3, 0x64, 0xe8, 0xbc, 0x76, 0xa6, 0xf4, 0x93,
                0x12, 0x22, 0xeb, 0x97, 0xa1, 0x6c, 0xa5, 0x97, 0xcf, 0x1f, 0xe1, 0x43, 0x0d, 0x80,
                0x5d, 0xfb, 0xa3, 0x59,
            ],
        ];
        //let mut base64: [char; GXT_KEY_LEN_BASE64] = [0 as char; GXT_KEY_LEN_BASE64];

        // At the end of the string, '\0' causes compatibility with null ending C chars.
        let expected_result_base64 = [
            "ADdgjBTmzc7FCXBFxgD5Pz3UXal7TDqlE95IjJXs9kI=",
            "6L5h4vbCEVRLgxZ/znpNUD+0m/Fa+DtY4eV4DAh4pEA=",
            "2FzSWHf/Cib3Knl8VuIy5skedNVgXOsr/BKdW4gnul8=",
            "uBDEHfrtdLPUTDI4UH4Obl8znsAHEPxHl6OL2bE1wXg=",
            "6NPKNjP8o2TovHam9JMSIuuXoWyll88f4UMNgF37o1k=",
        ];
        // ]
        // .iter()
        // .map(|f| f.chars().collect::<Vec<char>>())
        // .collect::<Vec<Vec<char>>>();

        for (i, key) in keys.iter().enumerate() {
            let base64 = key_to_base64(Some(key)).expect("--");
            //println!("Test::> key_to_base64 passed item {} expected_result_base64 len: {} - base64 len {}.", i, expected_result_base64[i].len(), base64.len() );
            assert_eq!(expected_result_base64[i], base64);
        }
    }

    #[test]
    fn test_key_to_hex() {
        //let keys: [[u8; GXT_KEY_LEN]; 5] = [
        let keys: Vec<[u8; GXT_KEY_LEN]> = vec![
            [
                0x0, 0x37, 0x60, 0x8c, 0x14, 0xe6, 0xcd, 0xce, 0xc5, 0x09, 0x70, 0x45, 0xc6, 00,
                0xf9, 0x3f, 0x3d, 0xd4, 0x5d, 0xa9, 0x7b, 0x4c, 0x3a, 0xa5, 0x13, 0xde, 0x48, 0x8c,
                0x95, 0xec, 0xf6, 0x42,
            ],
            [
                0xe8, 0xbe, 0x61, 0xe2, 0xf6, 0xc2, 0x11, 0x54, 0x4b, 0x83, 0x16, 0x7f, 0xce, 0x7a,
                0x4d, 0x50, 0x3f, 0xb4, 0x9b, 0xf1, 0x5a, 0xf8, 0x3b, 0x58, 0xe1, 0xe5, 0x78, 0x0c,
                0x08, 0x78, 0xa4, 0x40,
            ],
            [
                0xd8, 0x5c, 0xd2, 0x58, 0x77, 0xff, 0x0a, 0x26, 0xf7, 0x2a, 0x79, 0x7c, 0x56, 0xe2,
                0x32, 0xe6, 0xc9, 0x1e, 0x74, 0xd5, 0x60, 0x5c, 0xeb, 0x2b, 0xfc, 0x12, 0x9d, 0x5b,
                0x88, 0x27, 0xba, 0x5f,
            ],
            [
                0xb8, 0x10, 0xc4, 0x1d, 0xfa, 0xed, 0x74, 0xb3, 0xd4, 0x4c, 0x32, 0x38, 0x50, 0x7e,
                0x0e, 0x6e, 0x5f, 0x33, 0x9e, 0xc0, 0x07, 0x10, 0xfc, 0x47, 0x97, 0xa3, 0x8b, 0xd9,
                0xb1, 0x35, 0xc1, 0x78,
            ],
            [
                0xe8, 0xd3, 0xca, 0x36, 0x33, 0xfc, 0xa3, 0x64, 0xe8, 0xbc, 0x76, 0xa6, 0xf4, 0x93,
                0x12, 0x22, 0xeb, 0x97, 0xa1, 0x6c, 0xa5, 0x97, 0xcf, 0x1f, 0xe1, 0x43, 0x0d, 0x80,
                0x5d, 0xfb, 0xa3, 0x59,
            ],
        ];

        let expected_result_hex_str: Vec<&str> = vec![
            "0037608c14e6cdcec5097045c600f93f3dd45da97b4c3aa513de488c95ecf642",
            "e8be61e2f6c211544b83167fce7a4d503fb49bf15af83b58e1e5780c0878a440",
            "d85cd25877ff0a26f72a797c56e232e6c91e74d5605ceb2bfc129d5b8827ba5f",
            "b810c41dfaed74b3d44c3238507e0e6e5f339ec00710fc4797a38bd9b135c178",
            "e8d3ca3633fca364e8bc76a6f4931222eb97a16ca597cf1fe1430d805dfba359",
        ];

        for (i, key) in keys.iter().enumerate() {
            let hex = key_to_hex(Some(&key.to_vec())).expect("Correct Result");
            let hex_string = String::from_utf8(hex).unwrap();
            println!("hex_string 1 : {:?}", hex_string);
            assert_eq!(
                expected_result_hex_str[i], hex_string,
                "key_to_hex Test failed at index: {}",
                i
            );
        }

        // Test hex_to_key function
        for (i, key) in expected_result_hex_str.iter().enumerate() {
            let p: Vec<u8> = key.chars().map(|f| f as u8).collect();
            let res = key_from_hex(Some(p)).unwrap();
            assert_eq!(keys[i].to_vec(), res, "Test failed at index: {}", i);
        }
    }
}


//include!(concat!(env!("OUT_DIR"), "/gix_guard.uniffi.rs"));