#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

#include "WireGuardKitC/key.h"
#include "WireGuardKitC/x25519.h"

const char *wg_private_keys_arr[5]= {
        "ADdgjBTmzc7FCXBFxgD5Pz3UXal7TDqlE95IjJXs9kI=",
        "6L5h4vbCEVRLgxZ/znpNUD+0m/Fa+DtY4eV4DAh4pEA=",
        "2FzSWHf/Cib3Knl8VuIy5skedNVgXOsr/BKdW4gnul8=",
        "uBDEHfrtdLPUTDI4UH4Obl8znsAHEPxHl6OL2bE1wXg=",
        "6NPKNjP8o2TovHam9JMSIuuXoWyll88f4UMNgF37o1k="
};

static inline void print_val(const uint8_t *src, int len, const char * func_name, const char * label) {
    printf("\n ::%s:: %s: ", func_name, label);
    for(int i=0; i < len; i++) {
        printf("%0.2x", src[i]);
    }
    printf("\n");
}
static inline void print_bytes(const uint8_t *src, int len, const char * func_name, const char * label) {
    printf("::%s:: %s Bytes: [", func_name, label);
    for(int i=0; i < len; i++) {
        printf("%#0.2x,", src[i]);
    }
    printf("] \n");
}

static inline void print_chars(const uint8_t *src, int len, const char * func_name, const char * label) {
    printf("::%s:: %s Chars: [", func_name, label);
    for(int i=0; i < len; i++) {
        printf("'%c',", src[i]);
    }
    printf("] \n");
}

static inline void print_integer(const uint8_t *src, int len, const char * func_name, const char * label) {
    printf("\n ::%s:: %s Integer: [", func_name, label);
    for(int i=0; i < len; i++) {
        printf("'%d',", src[i]);
    }
    printf("] \n");
}

void make_key_to_hex_and_inverse_tests() {
    uint8_t input_key[WG_KEY_LEN];
    char hex_result_out[WG_KEY_LEN_HEX];
    uint8_t key_result_out[WG_KEY_LEN];
    for(int i=0; i < 1; i++) {
        if(key_from_base64(input_key, wg_private_keys_arr[i])) {
            key_to_hex(hex_result_out, input_key);
            print_bytes(input_key, WG_KEY_LEN, "key_to_hex", "input_key");
            //print_chars((uint8_t *)hex_result_out, WG_KEY_LEN_HEX, "key_to_hex", "hex_result_out");
            printf("HEX: %s\n", hex_result_out);
            bool ret = key_from_hex(key_result_out, hex_result_out);
            print_bytes(key_result_out, WG_KEY_LEN, "key_from_hex", "key_result_out");
            printf("\nret:%d\n", ret);
        }

    }
}

void make_key_from_base64_tests() {
    uint8_t key[WG_KEY_LEN];

    for(int i=0; i < 5; i++) {
        printf("base64 string: %s\n", wg_private_keys_arr[i]);
        print_bytes((uint8_t *)wg_private_keys_arr[i], WG_KEY_LEN_BASE64, "key_from_base64", "base64");
        if(key_from_base64(key, wg_private_keys_arr[i])) {
            print_bytes(key, WG_KEY_LEN, "key_from_base64", "key");
            printf("\n");
        } else {
            printf("Failed.\n");
        }
    }
}

extern uint8_t test_print1();

void print_named_lines(uint8_t *b, int total_len, const char *name) {
    printf("%s: ", name);
    for(int i=0; i < total_len; i++) {
        printf("%#02x", b[i]);
        if(i>0) printf(":");
    }
    printf("\n");
}

void print_rust_arr(uint8_t b[32], int total_len ) {
    printf("[");
    for(int i=0; i < total_len; i++) {
        if(i>0 && i < total_len) printf(",");
        printf("%#.02x", (unsigned int) b[i]);
    }
    printf("]");
}

void print_rust_arr_var(uint8_t b[16][32], int arr_size, const char *variableName) {
    printf("%s: [\n", variableName);
    for(int i=0; i < arr_size; i++) {
        printf("\t");
        print_rust_arr(b[i], 32);
        if(i < arr_size) printf(",");
        printf("\n");
    }
    printf("],\n");
}

int main() {
    printf("\"Hello, World!\"\n");

//    make_key_from_base64_tests();
//
    // printf("\n\nstarting next test data for key_to_hex :\n\n");
    // make_key_to_hex_and_inverse_tests();

//static void curve25519_shared_secret(uint8_t shared_secret[32], const uint8_t private_key[32], const uint8_t public_key[32])
    uint8_t shared_secret[32];
    uint8_t private_key[32];
    uint8_t public_key[32];

    uint8_t shared_secret_arr[16][32];
    uint8_t private_key_arr[16][32];
    uint8_t public_key_arr[16][32];
    uint8_t public_key_calc_arr[16][32];

    for(int i=0; i < 16; i++) {
        srand (time(NULL) + rand());

        for(int j=0; j < 32; j++) {
            private_key[j] = rand() % 255;
            public_key[j] = rand() % 255;
        }

        memset(shared_secret, 0, 32);
        curve25519_shared_secret(shared_secret, private_key, public_key);
        print_rust_arr(shared_secret, 32);
        printf("\n");

        memcpy((*shared_secret_arr) + (i * (32 * sizeof(uint8_t))), shared_secret, 32 * sizeof(uint8_t));
        memcpy(private_key_arr[i], private_key, 32 * sizeof(uint8_t));
        memcpy(public_key_arr[i], public_key, 32 * sizeof(uint8_t));
    }

    print_rust_arr_var(private_key_arr, 16, "private_key_arr");
    print_rust_arr_var(public_key_arr, 16, "public_key_arr");
    print_rust_arr_var(shared_secret_arr, 16, "shared_secret_arr");

    char public_key_out[32];
    for(int i=0; i < 16; i++) {
        curve25519_derive_public_key(public_key_out, private_key_arr[i]);
        memcpy(public_key_calc_arr[i], public_key_out, 32 * sizeof(uint8_t));
    }
    print_rust_arr_var(public_key_calc_arr, 16, "public_key_calc_arr");
    
    curve25519_generate_private_key(public_key_out);
    print_rust_arr(public_key_out, 32);
    printf("\n");
    //public_key_calc_arr

    // curve25519_shared_secret(shared_secret, private_key, public_key);

    // printf("shared_secret: ");
    // for(int i=0; i < 32; i++) {
    //     printf(":%02x", shared_secret[i]);
    // }
    // printf("\n");

    //test_print1();
    return 0;
}
