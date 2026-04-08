use super::messages::{SyncRequest, SyncResponse};
use libp2p::request_response::Codec;
use async_trait::async_trait;
use std::io;
use futures::{AsyncRead, AsyncWrite};
use futures::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct RvcProtocol;

impl AsRef<str> for RvcProtocol {
    fn as_ref(&self) -> &str {
        "/rvc/1.0.0"
    }
}

#[derive(Clone)]
pub struct RvcCodec;

impl Default for RvcCodec {
    fn default() -> Self {
        eprintln!("[Codec] INITIALIZED (Length-Prefixed Mode)");
        Self
    }
}

#[async_trait]
impl Codec for RvcCodec {
    type Protocol = RvcProtocol;
    type Request = SyncRequest;
    type Response = SyncResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        eprintln!("[Codec] Reading request...");
        let mut len_buf = [0u8; 4];
        if let Err(e) = io.read_exact(&mut len_buf).await {
            eprintln!("[Codec] Failed to read request length: {:?}", e);
            return Err(e);
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        
        if len > 10 * 1024 * 1024 { // 10MB limit
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Request too large"));
        }

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        eprintln!("[Codec] Successfully read request ({} bytes)", len);
        
        serde_json::from_slice(&buf).map_err(|e| {
            eprintln!("[Codec] JSON Decode Error (Request): {:?}", e);
            io::Error::new(io::ErrorKind::InvalidData, e)
        })
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        eprintln!("[Codec] Reading response...");
        let mut len_buf = [0u8; 4];
        if let Err(e) = io.read_exact(&mut len_buf).await {
            eprintln!("[Codec] Failed to read response length: {:?}", e);
            return Err(e);
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 10 * 1024 * 1024 { // 10MB limit
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Response too large"));
        }

        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf).await?;
        eprintln!("[Codec] Successfully read response ({} bytes)", len);

        serde_json::from_slice(&buf).map_err(|e| {
            eprintln!("[Codec] JSON Decode Error (Response): {:?}", e);
            io::Error::new(io::ErrorKind::InvalidData, e)
        })
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let len = buf.len() as u32;
        
        eprintln!("[Codec] Writing request ({} bytes)", len);
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&buf).await?;
        io.flush().await?;

        // Force flush and wait for Windows TCP
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&res).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let len = buf.len() as u32;

        eprintln!("[Codec] Writing response ({} bytes)", len);
        io.write_all(&len.to_be_bytes()).await?;
        io.write_all(&buf).await?;
        io.flush().await?;

        // Force flush and wait for Windows TCP
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        io.close().await?;
        Ok(())
    }
}
