use crossbeam_channel::bounded;
use crossbeam_channel::{Sender, Receiver};
use rand::{Rng, SeedableRng, RngCore};
use rand_xorshift::XorShiftRng;

const BUFFER_SIZE: usize = 4096;
const BUFFER_AMOUNT: usize = 4;

fn main() {
    let (ascii_tx, ascii_rx) = bounded(BUFFER_AMOUNT);
    let (mut buf_tx, buf_rx) = bounded(BUFFER_AMOUNT);

    create_initial_buffers(&mut buf_tx);

    tokio_uring::start(async {
        tokio_uring::spawn(generate_ascii(buf_rx, ascii_tx));
        tokio_uring::start(output_ascii(ascii_rx, buf_tx));
    });
}

fn create_initial_buffers(buf_tx: &mut Sender<Vec<u8>>) {
    for _ in 0..BUFFER_AMOUNT {
        let buf = vec![0_u8; BUFFER_SIZE];
        buf_tx.send(buf).unwrap();
    }
}

async fn generate_ascii(buf_rx: Receiver<Vec<u8>>, ascii_tx: Sender<Vec<u8>>) {
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

        tokio::task::yield_now().await;
    }
}

async fn output_ascii(ascii_rx: Receiver<Vec<u8>>, buf_tx: Sender<Vec<u8>>) {
    let mut output = tokio_uring::fs::File::open("/dev/stdout").await.unwrap();
    while let Ok(buf) = ascii_rx.recv() {
        let written: usize = 0;

        while written < buf.len() {
            written += &output.write_at(&buf[written..], written as u64).await.0.unwrap()
        }

        buf_tx.send(buf).unwrap();
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
#[inline(never)]
fn u8_to_ascii(buffer: &mut[u8]) {
    buffer.iter_mut().for_each(|v| {
        *v = (*v % 93) + 32
    })
}
