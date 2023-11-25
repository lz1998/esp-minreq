use embedded_io_async::Read;

pub trait BytesIter: Read {
    async fn next_byte(&mut self) -> Option<Result<u8, Self::Error>> {
        let mut byte = 0;
        match self.read(core::slice::from_mut(&mut byte)).await {
            Ok(0) => None,
            Ok(..) => Some(Ok(byte)),
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R: Read> BytesIter for R {}
