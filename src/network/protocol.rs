use libp2p::request_response::Codec;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
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

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequest {
    GetRefs,
    GetObjects(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    Refs(HashMap<String, String>),
    Objects(Vec<(String, Vec<u8>)>),
}

#[derive(Clone, Default)]
pub struct RvcCodec;

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
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 1_000_000 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "request too large"));
        }

        let mut buf = vec![0; len];
        io.read_exact(&mut buf).await?;

        bincode::deserialize(&buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "decode request"))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut len_buf = [0u8; 4];
        io.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 1_000_000 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "response too large"));
        }

        let mut buf = vec![0; len];
        io.read_exact(&mut buf).await?;

        bincode::deserialize(&buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "decode response"))
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
        let data = bincode::serialize(&req)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "encode request"))?;

        let len = (data.len() as u32).to_be_bytes();
        io.write_all(&len).await?;
        io.write_all(&data).await?;
        io.close().await
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
        let data = bincode::serialize(&res)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "encode response"))?;

        let len = (data.len() as u32).to_be_bytes();
        io.write_all(&len).await?;
        io.write_all(&data).await?;
        io.close().await
    }
}