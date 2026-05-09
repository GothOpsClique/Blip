use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct Message<'a> {
	pub msg: &'a String,
	pub channel: usize
}

pub async fn send_message<W>(stream: &mut W, message: Message<'_>) -> io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let bytes = message.msg.as_bytes();
    let len = bytes.len() as u32;

    stream.write_u32(len).await?;
    stream.write_all(bytes).await?;
    Ok(())
}

pub async fn read_message<R>(stream: &mut R) -> io::Result<Option<String>>
where
    R: AsyncRead + Unpin,
{
    let len = match stream.read_u32().await {
        Ok(len) => len,
        Err(_) => return Ok(None), // connection closed
    };

    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;

    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
}
