use std::{collections::VecDeque, io::IoSlice};

use bytes::Buf;

pub struct BufVec<B> {
    remaining: usize,
    bufs: VecDeque<B>,
}

impl<B: Buf> BufVec<B> {
    pub fn new() -> Self {
        BufVec {
            remaining: 0,
            bufs: VecDeque::new(),
        }
    }

    pub fn push<T>(&mut self, value: T)
    where
        T: Into<B>,
    {
        let buf = value.into();
        if buf.remaining() > 0 {
            self.remaining += buf.remaining();
            self.bufs.push_back(buf)
        }
    }
}

impl<B: Buf> Buf for BufVec<B> {
    fn remaining(&self) -> usize {
        self.remaining
    }

    fn chunk(&self) -> &[u8] {
        if let Some(buf) = self.bufs.front() {
            buf.chunk()
        } else {
            &[]
        }
    }

    fn advance(&mut self, mut cnt: usize) {
        self.remaining -= cnt;
        while cnt > 0 {
            let mut should_pop = false;
            if let Some(buf) = self.bufs.front_mut() {
                let rem = buf.remaining();
                let adv = ::std::cmp::min(cnt, rem);
                buf.advance(adv);
                cnt -= adv;
                if !buf.has_remaining() {
                    should_pop = true;
                }
            }
            if should_pop {
                self.bufs.pop_front();
            }
        }
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        let mut dst_idx = 0;
        let mut buf_idx = 0;
        while dst_idx < dst.len() {
            if let Some(buf) = self.bufs.get(buf_idx) {
                dst_idx += buf.chunks_vectored(&mut dst[dst_idx..]);
                buf_idx += 1;
            } else {
                break;
            }
        }
        dst_idx
    }

}

#[cfg(test)]
mod tests {
    use std::io::IoSlice;

    // use std::io::Cursor;
    use bytes::{Buf, Bytes};

    use super::BufVec;

    quickcheck! {
        fn push_and_collect(chunks: Vec<Vec<u8>>) -> bool {
            let mut bv = BufVec::<Bytes>::new();
            let mut bytes = Vec::new();
            for chunk in chunks {
                bytes.extend_from_slice(&chunk);
                bv.push(Bytes::from(chunk));
            }

            let mut collected = Vec::new();
            while bv.has_remaining() {
                let chunk = bv.chunk();
                collected.extend_from_slice(chunk);
                bv.advance(chunk.len());
            }

            bytes == collected
        }
    }

    quickcheck! {
        fn push_and_concat_iovec(chunks: Vec<Vec<u8>>, lens: Vec<u8>) -> bool {
            let mut bv = BufVec::<Bytes>::new();
            let mut bytes = Vec::new();
            for chunk in chunks {
                bytes.extend_from_slice(&chunk);
                bv.push(Bytes::from(chunk));
            }

            let mut total = 0;
            let mut collected = Vec::new();
            for len in lens {
                let mut num = 0;
                {
                    let mut vecs = vec![IoSlice::new(&[0u8]); len as usize];
                    let n = bv.chunks_vectored(&mut vecs);
                    for vec in &vecs[..n] {
                        num += vec.len();
                        collected.extend_from_slice(vec);
                    }
                }
                {
                    bv.advance(num);
                    total += num;
                }
            }

            bytes[..total] == collected[..]
        }
    }
}
