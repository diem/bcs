> **Note to readers:** On December 1, 2020, the Libra Association was renamed to Diem Association. The project repos are in the process of being migrated. All projects will remain available for use here until the migration to a new GitHub Organization is complete.

## Binary Canonical Serialization (BCS)

BCS defines a deterministic means for translating a message or data structure into bytes
irrespective of platform, architecture, or programming language.

### Background

In Diem, participants pass around messages or data structures that often times need to be
signed by a prover and verified by one or more verifiers. Serialization in this context refers
to the process of converting a message into a byte array. Many serialization approaches support
loose standards such that two implementations can produce two different byte streams that would
represent the same, identical message. While for many applications, non-deterministic
serialization causes no issues, it does so for applications using serialization for
cryptographic purposes. For example, given a signature and a message, a verifier may not unable
to produce the same serialized byte array constructed by the prover when the prover signed the
message resulting in a non-verifiable message. In other words, to ensure message verifiability
when using non-deterministic serialization, participants must either retain the original
serialized bytes or risk losing the ability to verify messages. This creates a burden requiring
participants to maintain both a copy of the serialized bytes and the deserialized message often
leading to confusion about safety and correctness. While there exist a handful of existing
deterministic serialization formats, there is no obvious choice. To address this, we propose
Diem Canonical Serialization that defines a deterministic means for translating a message into
bytes and back again.

### Specification

BCS supports the following data types:

* Booleans
* Signed 8-bit, 16-bit, 32-bit, 64-bit, and 128-bit integers
* Unsigned 8-bit, 16-bit, 32-bit, 64-bit, and 128-bit integers
* Option
* Unit (an empty value)
* Fixed and variable length sequences
* UTF-8 Encoded Strings
* Tuples
* Structures (aka "structs")
* Externally tagged enumerations (aka "enums")
* Maps

### General structure

BCS is not a self-describing format and as such, in order to deserialize a message, one must
know the message type and layout ahead of time.

Unless specified, all numbers are stored in little endian, two's complement format.

### Recursion and Depth of BCS Data

Recursive data-structures (e.g. trees) are allowed. However, because of the possibility of stack
overflow during (de)serialization, the *container depth* of any valid BCS data cannot exceed the constant
`MAX_CONTAINER_DEPTH`. Formally, we define *container depth* as the number of structs and enums traversed
during (de)serialization.

This definition aims to minimize the number of operations while ensuring that
(de)serialization of a known BCS format cannot cause arbitrarily large stack allocations.

As an example, if `v1` and `v2` are values of depth `n1` and `n2`,
* a struct value `Foo { v1, v2 }` has depth `1 + max(n1, n2)`;
* an enum value `E::Foo { v1, v2 }` has depth `1 + max(n1, n2)`;
* a pair `(v1, v2)` has depth `max(n1, n2)`;
* the value `Some(v1)` has depth `n1`.

All string and integer values have depths `0`.

#### Booleans and Integers

|Type                       |Original data          |Hex representation |Serialized format  |
|---                        |---                    |---                |---                |
|Boolean                    |True / False           |0x01 / 0x00        |[01] / [00]        |
|8-bit signed integer       |-1                     |0xFF               |[FF]               |
|8-bit unsigned integer     |1                      |0x01               |[01]               |
|16-bit signed integer      |-4660                  |0xEDCC             |[CCED]             |
|16-bit unsigned integer    |4660                   |0x1234             |[3412]             |
|32-bit signed integer      |-305419896             |0xEDCBA988         |[88A9CBED]         |
|32-bit unsigned integer    |305419896              |0x12345678         |[78563412]         |
|64-bit signed integer      |-1311768467750121216   |0xEDCBA98754321100 |[0011325487A9CBED] |
|64-bit unsigned integer    |1311768467750121216    |0x12345678ABCDEF00 |[00EFCDAB78563412] |

#### ULEB128-Encoded Integers

The BCS format also uses the [ULEB128 encoding](https://en.wikipedia.org/wiki/LEB128) internally
to represent unsigned 32-bit integers in two cases where small values are usually expected:
(1) lengths of variable-length sequences and (2) tags of enum values (see the corresponding
sections below).

|Type                       |Original data          |Hex representation |Serialized format  |
|---                        |---                    |---                |---                |
|ULEB128-encoded u32-integer|2^0 = 1                |0x00000001         |[01]               |
|                           |2^7 = 128              |0x00000080         |[8001]             |
|                           |2^14 = 16384           |0x00004000         |[808001]           |
|                           |2^21 = 2097152         |0x00200000         |[80808001]         |
|                           |2^28 = 268435456       |0x10000000         |[8080808001]       |
|                           |9487                   |0x0000250f         |[8f4a]             |

In general, a ULEB128 encoding consists of a little-endian sequence of base-128 (7-bit)
digits. Each digit is completed into a byte by setting the highest bit to 1, except for the
last (highest-significance) digit whose highest bit is set to 0.

In BCS, the result of decoding ULEB128 bytes is required to fit into a 32-bit unsigned
integer and be in canonical form. For instance, the following values are rejected:
* `[808080808001]` (2^36) is too large.
* `[8080808010]` (2^33) is too large.
* `[8000]` is not a minimal encoding of 0.

#### Optional Data

Optional or nullable data either exists in its full representation or does not. BCS represents
this as a single byte representing the presence `0x01` or absence `0x00` of data. If the data
is present then the serialized form of that data follows. For example:

```rust
let some_data: Option<u8> = Some(8);
assert_eq!(to_bytes(&some_data)?, vec![1, 8]);

let no_data: Option<u8> = None;
assert_eq!(to_bytes(&no_data)?, vec![0]);
```

#### Fixed and Variable Length Sequences

Sequences can be made of up of any BCS supported types (even complex structures) but all
elements in the sequence must be of the same type. If the length of a sequence is fixed and
well known then BCS represents this as just the concatenation of the serialized form of each
individual element in the sequence. If the length of the sequence can be variable, then the
serialized sequence is length prefixed with a ULEB128-encoded unsigned integer indicating
the number of elements in the sequence. All variable length sequences must be
`MAX_SEQUENCE_LENGTH` elements long or less.

```rust
let fixed: [u16; 3] = [1, 2, 3];
assert_eq!(to_bytes(&fixed)?, vec![1, 0, 2, 0, 3, 0]);

let variable: Vec<u16> = vec![1, 2];
assert_eq!(to_bytes(&variable)?, vec![2, 1, 0, 2, 0]);

let large_variable_length: Vec<()> = vec![(); 9_487];
assert_eq!(to_bytes(&large_variable_length)?, vec![0x8f, 0x4a]);
```

#### Strings

Only valid UTF-8 Strings are supported. BCS serializes such strings as a variable length byte
sequence, i.e. length prefixed with a ULEB128-encoded unsigned integer followed by the byte
representation of the string.

```rust
// Note that this string has 10 characters but has a byte length of 24
let utf8_str = "çå∞≠¢õß∂ƒ∫";
let expecting = vec![
    24, 0xc3, 0xa7, 0xc3, 0xa5, 0xe2, 0x88, 0x9e, 0xe2, 0x89, 0xa0, 0xc2,
    0xa2, 0xc3, 0xb5, 0xc3, 0x9f, 0xe2, 0x88, 0x82, 0xc6, 0x92, 0xe2, 0x88, 0xab,
];
assert_eq!(to_bytes(&utf8_str)?, expecting);
```

#### Tuples

Tuples are typed composition of objects: `(Type0, Type1)`

Tuples are considered a fixed length sequence where each element in the sequence can be a
different type supported by BCS. Each element of a tuple is serialized in the order it is
defined within the tuple, i.e. [tuple.0, tuple.2].

```rust
let tuple = (-1i8, "diem");
let expecting = vec![0xFF, 4, b'd', b'i', b'e', b'm'];
assert_eq!(to_bytes(&tuple)?, expecting);
```


#### Structures

Structures are fixed length sequences consisting of fields with potentially different types.
Each field within a struct is serialized in the order specified by the canonical structure
definition. Structs can exist within other structs and as such, BCS recurses into each struct
and serializes them in order. There are no labels in the serialized format, the struct ordering
defines the organization within the serialization stream.

```rust
struct MyStruct {
    boolean: bool,
    bytes: Vec<u8>,
    label: String,
}

struct Wrapper {
    inner: MyStruct,
    name: String,
}

let s = MyStruct {
    boolean: true,
    bytes: vec![0xC0, 0xDE],
    label: "a".to_owned(),
};
let s_bytes = to_bytes(&s)?;
let mut expecting = vec![1, 2, 0xC0, 0xDE, 1, b'a'];
assert_eq!(s_bytes, expecting);

let w = Wrapper {
    inner: s,
    name: "b".to_owned(),
};
let w_bytes = to_bytes(&w)?;
assert!(w_bytes.starts_with(&s_bytes));

expecting.append(&mut vec![1, b'b']);
assert_eq!(w_bytes, expecting);
```

#### Externally Tagged Enumerations

An enumeration is typically represented as a type that can take one of potentially many
different variants. In BCS, each variant is mapped to a variant index, a ULEB128-encoded 32-bit unsigned
integer, followed by serialized data if the type has an associated value. An
associated type can be any BCS supported type. The variant index is determined based on the
ordering of the variants in the canonical enum definition, where the first variant has an index
of `0`, the second an index of `1`, etc.

```rust
enum E {
    Variant0(u16),
    Variant1(u8),
    Variant2(String),
}

let v0 = E::Variant0(8000);
let v1 = E::Variant1(255);
let v2 = E::Variant2("e".to_owned());

assert_eq!(to_bytes(&v0)?, vec![0, 0x40, 0x1F]);
assert_eq!(to_bytes(&v1)?, vec![1, 0xFF]);
assert_eq!(to_bytes(&v2)?, vec![2, 1, b'e']);
```

If you need to serialize a C-style enum, you should use a primitive integer type.

#### Maps (Key / Value Stores)

Maps are represented as a variable-length, sorted sequence of (Key, Value) tuples. Keys must be
unique and the tuples sorted by increasing lexicographical order on the BCS bytes of each key.
The representation is otherwise similar to that of a variable-length sequence. In particular,
it is preceded by the number of tuples, encoded in ULEB128.

```rust
let mut map = HashMap::new();
map.insert(b'e', b'f');
map.insert(b'a', b'b');
map.insert(b'c', b'd');

let expecting = vec![(b'a', b'b'), (b'c', b'd'), (b'e', b'f')];

assert_eq!(to_bytes(&map)?, to_bytes(&expecting)?);
```

### Backwards compatibility

Complex types dependent upon the specification in which they are used. BCS does not provide
direct provisions for versioning or backwards / forwards compatibility. A change in an objects
structure could prevent historical clients from understanding new clients and vice-versa.

## Contributing

See the [CONTRIBUTING](CONTRIBUTING.md) file for how to help out.

## License

This project is available under the terms of either the [Apache 2.0 license](LICENSE).

<!--
README.md is generated from README.tpl by cargo readme. To regenerate:

cargo install cargo-readme
cargo readme > README.md
-->
