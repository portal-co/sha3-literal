use hash_literal_core::Digest;

hash_literal_core::literals!(sha3 => sha3::Sha3_256; nested: [
    ("sha3_literal", |b, s| (sha3::Sha3_256::digest(b).into_iter().collect(), s)),
    ("sha3_hex_literal", |b, s| (hash_literal_core::hex::encode(sha3::Sha3_256::digest(b)).into_bytes(), s)),
    ("sha3_512_literal", |b, s| (sha3::Sha3_512::digest(b).into_iter().collect(), s)),
    ("sha3_512_hex_literal", |b, s| (hash_literal_core::hex::encode(sha3::Sha3_512::digest(b)).into_bytes(), s)),
]);

hash_literal_core::literals!(sha3_512 => sha3::Sha3_512; nested: [
    ("sha3_literal", |b, s| (sha3::Sha3_256::digest(b).into_iter().collect(), s)),
    ("sha3_hex_literal", |b, s| (hash_literal_core::hex::encode(sha3::Sha3_256::digest(b)).into_bytes(), s)),
    ("sha3_512_literal", |b, s| (sha3::Sha3_512::digest(b).into_iter().collect(), s)),
    ("sha3_512_hex_literal", |b, s| (hash_literal_core::hex::encode(sha3::Sha3_512::digest(b)).into_bytes(), s)),
]);
