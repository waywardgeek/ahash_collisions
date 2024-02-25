use ahash::RandomState;

extern crate ahash;

fn main() {
        let k0 = 0xe786_c22b_119c_1479u64;
        let k1 = 0xec59_9ba1_0b7f_c0f2u64;
        let k2 = 0x8faa_23b7_08c7_b7a5;
        let k3 = 0x1abe_3cd3_050a_7b8c;
        let table_size = 1usize << 28;
        let mut table: Vec<bool> = Vec::default();
        table.resize(table_size, false);
        let hasher = RandomState::with_seeds(k0, k1, k2, k3);
        // Simulate a 50% load factor.
        for i in 0..table_size >> 1 {
            let mut h = hasher.hash_one(i as u64) as usize & (table_size - 1);
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
        println!( "Average sequence length = {}", total_len as f32 / num_seq as f32);
}
