pub fn unpack_string(data: &[u8]) -> Option<(String, &[u8])> {
  let (len_chunk, rest) = data.split_at(4);
  let len = match len_chunk.try_into().ok()
    .map(u32::from_le_bytes) {
      None => return None,
      Some(u) => u as usize
    };

  if rest.len() < len {
    return None
  }
  if len == 0 {
    return Some(("".to_string(), rest))
  }

  let (str_chunk, rest) = rest.split_at(len);
  let result = match str_chunk.try_into().ok()
    .map(String::from_utf8) {
      None => return None,
      Some(r) => match r{
        Ok(r) => r,
        _ => return None,
      }
    };
  
  return Some((result, rest));
}