use {
    alloc::{
        format,
        string::{String, ToString},
        vec::Vec
    },
    any_ascii::any_ascii,
    base64::{DecodeSliceError, EncodeSliceError, Engine as _, engine::general_purpose},
    heck::ToSnakeCase
};

pub fn strip_html(source: &str) -> String {
    let mut result = String::new();
    let mut inside = false;

    for c in source.chars() {
        if c == '<' {
            inside = true;
            continue;
        }

        if c == '>' {
            inside = false;
            continue;
        }

        if !inside {
            result.push(c);
        }
    }

    result
}

#[inline]
pub fn to_capitalize(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str()
    }
}

#[inline]
pub fn to_slug(s: &str) -> String {
    any_ascii(s).to_snake_case()
}

pub trait StringExt {
    #[inline]
    fn to_capitalize(&self) -> String
    where
        Self: AsRef<str>
    {
        to_capitalize(self.as_ref())
    }
}

impl StringExt for String {}

impl StringExt for &str {}

pub fn base64_encode(
    data: impl AsRef<[u8]>,
    pad: bool
) -> Result<String, EncodeSliceError> {
    let mut buf = alloc::vec![0; data.as_ref().len() * 4 / 3 + 4];

    let len = if pad {
        general_purpose::STANDARD.encode_slice(data, &mut buf)?
    } else {
        general_purpose::STANDARD_NO_PAD.encode_slice(data, &mut buf)?
    };

    buf.truncate(len);

    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn base64_decode(str: &str, pad: bool) -> Result<String, DecodeSliceError> {
    let mut buf = Vec::default();
    buf.resize(str.len(), 0);

    let len = if pad {
        general_purpose::STANDARD.decode_slice(str, &mut buf)?
    } else {
        general_purpose::STANDARD_NO_PAD.decode_slice(str, &mut buf)?
    };

    buf.truncate(len);

    Ok(String::from_utf8_lossy(&buf).to_string())
}

#[inline]
pub fn md5(value: &[u8]) -> String {
    format!("{:x}", md5::compute(value))
}
