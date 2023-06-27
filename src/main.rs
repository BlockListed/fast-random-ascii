use rand::{Rng, SeedableRng, RngCore};
use rand_xorshift::XorShiftRng;

use glommio::{LocalExecutor, spawn_local, yield_if_needed};
use glommio::channels::local_channel::{LocalReceiver, LocalSender, new_bounded};
use glommio::io::{BufferedFile, StreamWriterBuilder};

use futures_lite::AsyncWriteExt;

const BUFFER_SIZE: usize = 4096;
const BUFFER_AMOUNT: usize = 4;

fn main() {
    let (ascii_tx, ascii_rx) = new_bounded(BUFFER_AMOUNT);
    let (mut buf_tx, buf_rx) = new_bounded(BUFFER_AMOUNT);

    LocalExecutor::default()
        .run(async {
            create_initial_buffers(&mut buf_tx).await;

            spawn_local(generate_ascii(buf_rx, ascii_tx)).detach();
            output_ascii(ascii_rx, buf_tx).await;
        });
}

async fn create_initial_buffers(buf_tx: &mut LocalSender<Vec<u8>>) {
    for _ in 0..BUFFER_AMOUNT {
        let buf = vec![0_u8; BUFFER_SIZE];
        buf_tx.send(buf).await.unwrap();
    }
}

async fn generate_ascii(buf_rx: LocalReceiver<Vec<u8>>, ascii_tx: LocalSender<Vec<u8>>) {
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

        yield_if_needed().await;
    }
}

async fn output_ascii(ascii_rx: LocalReceiver<Vec<u8>>, buf_tx: LocalSender<Vec<u8>>) {
    let file = BufferedFile::open("/dev/stdout").await.unwrap();
    let mut output = StreamWriterBuilder::new(file).with_buffer_size(BUFFER_SIZE).build();

    while let Some(buf) = ascii_rx.recv().await {
        output.write_all(&buf).await.unwrap();

        buf_tx.send(buf).await.unwrap();

        yield_if_needed().await;
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
