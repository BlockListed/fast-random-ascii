use std::io::Write;

use std::cmp::min;
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
    // This allows buffer reuse and makes allocations extremely unlikely.
    // They can however still happen, if a buffer with a capacity less than
    // `BUFFER_SIZE` is given to the channel.
    let (buf_tx, buf_rx) = bounded(buffer_amount);

    // Create initial buffers, which are all preallocated.
    create_initial_buffers(&buf_tx, buffer_amount);

    scope(|s| {
        // Reserve one cpu core for outputting our results.
        for _ in 0..min(cpu_amount - 1,1) {
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
    // This is seeded from the OsRng to increase randomness.
    let mut generator = {
        let seed: u64 = rand::rngs::OsRng.gen();
        XorShiftRng::seed_from_u64(seed)
    };

    while let Ok(mut buf) = buf_rx.recv() {
        // Make sure our incoming vec is long enough.
        make_vec_len(&mut buf, BUFFER_SIZE);

        generator.fill_bytes(&mut buf);

        u8_to_ascii(&mut buf);

        ascii_tx.send(buf).unwrap();

        // Make our CPU more happy.
        // This shouldn't kill performance,
        // thanks to our large buffer sizes.
        yield_now();
    }
}

fn output_ascii(ascii_rx: Receiver<Vec<u8>>, buf_tx: Sender<Vec<u8>>) {
    // Lock stdout to remove Mutex overhead.
    let mut output = std::io::stdout().lock();

    while let Ok(buf) = ascii_rx.recv() {
        output.write_all(&buf).unwrap();

        // Return our buffer to the generators.
        buf_tx.send(buf).unwrap();

        // I have no idea, if this actually improves niceness,
        // since we are probably cooperatively yielding
        // with our syscall in `write_all`.
        yield_now();
    }
}

fn make_vec_len<T: Default>(vec: &mut Vec<T>, len: usize) {
    // This improves inlining and makes this (hopefully)
    // just a comparison and a function call, if the
    // comparison fails.
    #[cold]
    fn extend_vec<T: Default>(vec: &mut Vec<T>, len: usize) {
        let additional = len - vec.len();

        vec.reserve(additional);

        let free_space = vec.capacity() - vec.len();

        // Make sure vec length is equal to capacity.
        vec.extend(
            std::iter::repeat_with(T::default)
                .take(free_space)
        );
    }

    if vec.len() >= len {
        return;
    } else {
        extend_vec(vec, len);
    }
}

// High speed, but some chars are more common .
// Also, very vectorization- and inlining-friendly.
// WARNING: first 69 characters are generated 33.3% more often
// than other characters.
fn u8_to_ascii(buffer: &mut[u8]) {
    buffer.iter_mut().for_each(|v| {
        *v = (*v % 93) + 32
    })
}
