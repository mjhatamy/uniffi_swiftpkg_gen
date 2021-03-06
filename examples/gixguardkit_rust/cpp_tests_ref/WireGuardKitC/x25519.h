#ifndef X25519_H
#define X25519_H

void curve25519_shared_secret(uint8_t shared_secret[32], const uint8_t private_key[32], const uint8_t public_key[32]);
void curve25519_derive_public_key(unsigned char public_key[32], const unsigned char private_key[32]);
void curve25519_generate_private_key(unsigned char private_key[32]);

#endif
