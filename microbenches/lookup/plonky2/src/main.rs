pub mod lookup;
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
fn main() {

    println!("Plonky2 Lookup Bench!");
    let n: [u16;3] = [16383, 32767, 65535]; // (2^k-1 for k=14,15,16)
    let k: [usize; 5] = [1000, 10000, 100000, 1000000, 10000000];
    for i in 0..n.len(){
        for j in 0..k.len(){
            lookup::run_lookup(n[i], k[j]);
        }
    }
}
