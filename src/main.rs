use rand::{Rng, SeedableRng, RngCore};
use rand_xorshift::XorShiftRng;

use tokio::sync::mpsc::{channel, Sender, Receiver};
use tokio::io::stdout;
use tokio::io::AsyncWriteExt;
use tokio::task::yield_now;

const BUFFER_SIZE: usize = 4096;
const BUFFER_AMOUNT: usize = 4;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (ascii_tx, ascii_rx) = channel(BUFFER_AMOUNT);
    let (mut buf_tx, buf_rx) = channel(BUFFER_AMOUNT);

    create_initial_buffers(&mut buf_tx).await;
    tokio::spawn(generate_ascii(buf_rx, ascii_tx));
    output_ascii(ascii_rx, buf_tx).await;
}

async fn create_initial_buffers(buf_tx: &mut Sender<Vec<u8>>) {
    for _ in 0..BUFFER_AMOUNT {
        let buf = vec![0_u8; BUFFER_SIZE];
        buf_tx.send(buf).await.unwrap();
    }
}

async fn generate_ascii(mut buf_rx: Receiver<Vec<u8>>, ascii_tx: Sender<Vec<u8>>) {
    let mut generator = {
        let seed: u64 = rand::rngs::OsRng.gen();
        XorShiftRng::seed_from_u64(seed)
    };

    while let Some(mut buf) = buf_rx.recv().await {
    //loop {
        //let mut buf = vec![0_u8; BUFFER_SIZE];

        // Make sure our incoming vec is long enough
        make_vec_len(&mut buf, BUFFER_SIZE);

        generator.fill_bytes(&mut buf);

        u8_to_ascii(&mut buf);

        ascii_tx.send(buf).await.unwrap();

        yield_now().await;
    }
}

async fn output_ascii(mut ascii_rx: Receiver<Vec<u8>>, buf_tx: Sender<Vec<u8>>) {
    let mut output = stdout();

    while let Some(buf) = ascii_rx.recv().await {
        output.write_all(&buf).await.unwrap();

        output.flush().await.unwrap();

        buf_tx.send(buf).await.unwrap();

        yield_now().await;
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
