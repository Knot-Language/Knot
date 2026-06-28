#ifndef KNOT_H
#define KNOT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// --- integer types ---
typedef int8_t   knot_i8;
typedef int16_t  knot_i16;
typedef int32_t  knot_i32;
typedef int64_t  knot_i64;
typedef uint8_t  knot_u8;
typedef uint16_t knot_u16;
typedef uint32_t knot_u32;
typedef uint64_t knot_u64;

// --- float types ---
typedef float  knot_f32;
typedef double knot_f64;

// --- character ---
typedef char knot_char;

// --- boolean ---
typedef int32_t knot_bool;
#define KNOT_TRUE  1
#define KNOT_FALSE 0

// --- string (Array[Char]) ---
// Knot Array[Char] is passed as a pointer to the first byte.
// For null-terminated C strings use const char* directly.
// For length-aware strings pair with an explicit knot_i64 len parameter.

#endif
