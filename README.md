# ahash_collisions
A temporary repo for reproducing high collision rates when using ahash::RandomState.

To reproduce:

$ cargo run --release

The expected average sequence length should be 2.541.  However, with this seed
(the worst of 25 that I tested), we see average sequence length > 4.  This is
not good for swiss hash tables.
