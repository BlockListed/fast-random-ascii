use std::io::Write;

use std::thread::{scope, yield_now};

use crossbeam_channel::bounded;
use crossbeam_channel::{Sender, Receiver};
use rand::{Rng, SeedableRng, RngCore};
use rand_xorshift::XorShiftRng;

const BUFFER_SIZE: usize = 1 << 16;
const BUFFER_AMOUNT: usize = 2;

fn main() {
    // Limit parallelism
    let cpu_amount = std::cmp::min(4, num_cpus::get());
    // Create buffers for each individual cpu.
    let buffer_amount = cpu_amount * BUFFER_AMOUNT;

    let (ascii_tx, ascii_rx) = bounded(buffer_amount);
    let (buf_tx, buf_rx) = bounded(buffer_amount);

    // Create initial buffers, which are all preallocated
    create_initial_buffers(&buf_tx, buffer_amount);

    scope(|s| {
        for _ in 0..cpu_amount-1 {
            s.spawn(|| generate_ascii(&buf_rx, &ascii_tx));
        }
        s.spawn(move || output_ascii(ascii_rx, buf_tx));
    });
}

fn create_initial_buffers(buf_tx: &Sender<Vec<u8>>, buffer_amount: usize) {
    for _ in 0..buffer_amount {
        let buf = vec![0_u8; BUFFER_SIZE];
        buf_tx.send(buf).unwrap();
    }
}

fn generate_ascii(buf_rx: &Receiver<Vec<u8>>, ascii_tx: &Sender<Vec<u8>>) {
    let mut generator = {
        let seed: u64 = rand::rngs::OsRng.gen();
        XorShiftRng::seed_from_u64(seed)
    };

    while let Ok(mut buf) = buf_rx.recv() {
    //loop {
        //let mut buf = vec![0_u8; BUFFER_SIZE];

        // Make sure our incoming vec is long enough
        make_vec_len(&mut buf, BUFFER_SIZE);

        generator.fill_bytes(&mut buf);

        u8_to_ascii(&mut buf);

        ascii_tx.send(buf).unwrap();
    }
}

fn output_ascii(ascii_rx: Receiver<Vec<u8>>, buf_tx: Sender<Vec<u8>>) {
    let mut output = std::io::stdout().lock();
    while let Ok(buf) = ascii_rx.recv() {
        output.write_all(&buf).unwrap();

        output.flush().unwrap();

        buf_tx.send(buf).unwrap();

        yield_now();
    }
}

fn make_vec_len<T: Default>(vec: &mut Vec<T>, len: usize) {
    if vec.len() >= len {
        return
    }

    let additional = len - vec.len();

    vec.reserve(additional);

    let free_space = vec.capacity() - vec.len();

    // Make sure vec length is equal to to capacity.
    vec.extend(
        std::iter::repeat_with(T::default)
            .take(free_space)
    );
}

// High speed, but some chars are more common
fn u8_to_ascii(buffer: &mut[u8]) {
    buffer.iter_mut().for_each(|v| {
        *v = (*v % 93) + 32
    })
}
