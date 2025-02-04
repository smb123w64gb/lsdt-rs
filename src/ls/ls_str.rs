use crc32fast::Hasher;

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new_with_initial(0xFFFFFFFF);
    hasher.update(bytes);
    hasher.finalize()
}
