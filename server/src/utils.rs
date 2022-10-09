use anyhow::Result;

pub fn decode(s: &str) -> Result<String> {
    let b = base64::decode(s)?;
    let s = String::from_utf8(b)?;
    Ok(s)
}
pub fn encode(b: &str) -> String {
    base64::encode(b)
}
