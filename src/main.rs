use ahash::RandomState;
use std::env;
use std::io::Write;

extern crate ahash;

#[inline(always)]
fn hash_u64(v: u64, hash_secret: u64) -> u64 {
    let v1 = (v + hash_secret) ^ v.rotate_left(32);
    let v2 = (v1 as u128).wrapping_mul(0x9d46_0858_ea81_ac79);
    v ^ (v2 as u64) ^ ((v2 >> 64) as u64)
}

struct Rand {
    seed: u64,
    hash_secret: u64,
    hasher: RandomState,
    use_ahash: bool,
}

impl Rand {
    fn rand64(&mut self) -> u64 {
        if self.use_ahash {
            self.seed = self.hasher.hash_one(self.seed);
        } else {
            self.seed = hash_u64(self.seed, self.hash_secret);
        }
        self.seed
    }
}

fn find_cycle_len(limit: usize, hash_secret: u64, use_ahash: bool) -> usize {
    let mut r = Rand {
        seed: 0u64,
        hash_secret: hash_secret,
        hasher: RandomState::with_seed(hash_secret as usize),
        use_ahash: use_ahash,
    };
    for loop_power in 0..64 {
        let target = r.rand64();
        for i in 0usize..(1usize << loop_power) {
            if r.rand64() == target {
                return i + 1;
            }
        }
        if 1usize << loop_power > limit {
            return 0;
        }
    }
    0
}

fn reduce(x: u32, n: u32) -> usize {
    (((x as u64) * (n as u64)) >> 32) as usize
}

fn dist_test(hash_secret: u64, use_ahash: bool) {
    let table_size = 1usize << 28;
    for tweak in 0..100 {
        let in_shift = hash_u64(hash_u64(tweak, 0x123), 0x456) & 0x1f;
        let out_shift = hash_u64(hash_u64(tweak, 0x789), 0xabc) & 0x1f;
        let hash_secret = hash_u64(hash_secret, tweak);
        let hasher = RandomState::with_seed(hash_secret as usize);
        let mut table: Vec<bool> = Vec::default();
        table.resize(table_size, false);
        for i in 0..table_size >> 1 {
            let mut h;
            if !use_ahash {
                h = reduce(
                    (hash_u64((i as u64) << in_shift, hash_secret) >> out_shift) as u32,
                    table_size as u32,
                );
            } else {
                h = ((hasher.hash_one(i << /* in_shift */ 30)) >> /* out_shift */ 0) as usize & (table_size - 1);
            }
            while table[h] {
                h += 1;
                if h == table.len() {
                    h = 0;
                }
            }
            table[h] = true;
        }
        let mut len = 0;
        let mut total_len = 0;
        let mut num_seq = 0;
        for i in 0..table_size {
            if table[i] {
                len += 1;
            } else if len != 0 {
                num_seq += 1;
                total_len += len;
                len = 0;
            }
        }
        println!(
            "tweak {}: Average sequence length = {}",
            tweak, total_len as f32 / num_seq as f32
        );
    }
}

fn usage() {
    println!("Usage: rng2 (-c|-ca-r|-ra|-t|-ta]-d|-da)
    -c:  Compute cycle length of Bill's hash function
    -ca: Compute cycle length of ahash's fastpath u64 fallback hasher
    -r:  Output raw bytes from Bill's hash function for use with dieharder
         cargo run --release -- -r | dieharder -a -g 200
    -ra: Output raw bytes from ahash's fastpath u64 falback hasher
    -t:  Compute 1 << 30 hashes in a loop, for use with time command
    -ta: Compute 1 << 30 ahash fastpath u64 fallback hashes in a loop
    -d:  Test Bill's hash function for collisions in a swiss hash table
    -da: Test ahash's fastpath u64 fallback  function for collisions\n"
    );
}

fn main() {
    let hash_secret = 0xe786_c22b_119c_1465u64;
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        usage();
        return;
    } else if args[1] == "-c" {
        for i in 0..10 {
            let res = find_cycle_len(0x10_0000_0000, hash_secret + i, false);
            if res != 0 {
                println!("sequence length =  {:#x} ({})", res, res);
            } else {
                println!("sequence too long");
            }
        }
    } else if args[1] == "-ca" {
        for i in 0..10 {
            let res = find_cycle_len(0x10_0000_0000, hash_secret + i, true);
            if res != 0 {
                println!("sequence length =  {:#x} ({})", res, res);
            } else {
                println!("sequence too long");
            }
        }
    } else if args[1] == "-r" {
        let mut stdout = std::io::stdout().lock();
        for i in 0..1u64 << 34 {
            let val = hash_u64(i, hash_secret).to_le_bytes();
            stdout.write_all(&val).unwrap();
        }
    } else if args[1] == "-t" {
        let mut total = 0;
        for i in 0..1 << 30 {
            let val = hash_u64(i, hash_secret);
            total += val;
        }
        println!("total = {}", total);
    } else if args[1] == "-ta" {
        let hasher = RandomState::with_seed(hash_secret as usize);
        let mut total = 0;
        for i in 0..1 << 30 {
            let val = hasher.hash_one(i);
            total += val;
        }
        println!("total = {}", total);
    } else if args[1] == "-ra" {
        let hasher = RandomState::with_seed(hash_secret as usize);
        let mut stdout = std::io::stdout().lock();
        for i in 0..1u64 << 34 {
            let val = hasher.hash_one(hasher.hash_one(i)).to_le_bytes();
            stdout.write_all(&val[0..4]).unwrap();
        }
    } else if args[1] == "-d" {
        dist_test(hash_secret, false);
    } else if args[1] == "-da" {
        dist_test(hash_secret, true);
    } else {
        usage();
    }
}
