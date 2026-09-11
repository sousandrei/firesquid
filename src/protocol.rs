use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{error::Error, runtime::VmMetrics, state::Vm};

pub const VERSION: u16 = 1;
const MAX_FRAME_SIZE: u32 = 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub version: u16,
    pub request_id: u64,
    pub operation: Operation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Operation {
    Status,
    VmList,
    VmCreate {
        name: String,
        kernel_path: String,
        rootfs_path: String,
        vcpus: u32,
        memory_mib: u32,
    },
    VmStart {
        id: String,
    },
    VmStop {
        id: String,
    },
    VmKill {
        id: String,
    },
    VmInspect {
        id: String,
    },
    VmDelete {
        id: String,
    },
    VmLogs {
        id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub version: u16,
    pub request_id: u64,
    pub result: Result<ResponseBody, ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseBody {
    Status { service: String, vms: Vec<VmInfo> },
    VmList(Vec<Vm>),
    Vm(VmInfo),
    Logs(String),
    Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VmInfo {
    pub vm: Vm,
    pub metrics: VmMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub message: String,
}

impl Response {
    pub fn ok(request_id: u64, body: ResponseBody) -> Self {
        Self {
            version: VERSION,
            request_id,
            result: Ok(body),
        }
    }

    pub fn error(request_id: u64, error: &Error) -> Self {
        Self {
            version: VERSION,
            request_id,
            result: Err(ResponseError {
                message: error.to_string(),
            }),
        }
    }
}

pub async fn read_frame<R>(reader: &mut R) -> Result<Option<Request>, Error>
where
    R: AsyncRead + Unpin,
{
    let mut length = [0; 4];
    match reader.read_exact(&mut length).await {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error.into()),
    }

    let length = u32::from_be_bytes(length);
    if length == 0 || length > MAX_FRAME_SIZE {
        return Err(Error::Protocol(format!("invalid frame size: {length}")));
    }

    let mut payload = vec![0; length as usize];
    reader.read_exact(&mut payload).await?;
    Ok(Some(postcard::from_bytes(&payload)?))
}

pub async fn write_response<W>(writer: &mut W, response: &Response) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
{
    let payload = postcard::to_allocvec(response)?;
    let length = u32::try_from(payload.len())
        .map_err(|_| Error::Protocol("response is too large".to_owned()))?;
    if length > MAX_FRAME_SIZE {
        return Err(Error::Protocol("response is too large".to_owned()));
    }

    writer.write_all(&length.to_be_bytes()).await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio::io::duplex;

    use super::{Operation, Request, Response, ResponseBody, VERSION, read_frame, write_response};

    #[tokio::test]
    async fn request_round_trip_uses_versioned_envelope() {
        let request = Request {
            version: VERSION,
            request_id: 7,
            operation: Operation::VmList,
        };
        let payload = postcard::to_allocvec(&request).unwrap();
        let (mut writer, mut reader) = duplex(1024);
        tokio::io::AsyncWriteExt::write_all(&mut writer, &(payload.len() as u32).to_be_bytes())
            .await
            .unwrap();
        tokio::io::AsyncWriteExt::write_all(&mut writer, &payload)
            .await
            .unwrap();

        let decoded = read_frame(&mut reader).await.unwrap().unwrap();
        assert_eq!(decoded.request_id, 7);
        assert!(matches!(decoded.operation, Operation::VmList));
    }

    #[tokio::test]
    async fn response_round_trip_preserves_payload() {
        let response = Response::ok(9, ResponseBody::Empty);
        let (mut writer, mut reader) = duplex(1024);
        write_response(&mut writer, &response).await.unwrap();

        let mut length = [0; 4];
        tokio::io::AsyncReadExt::read_exact(&mut reader, &mut length)
            .await
            .unwrap();
        let mut payload = vec![0; u32::from_be_bytes(length) as usize];
        tokio::io::AsyncReadExt::read_exact(&mut reader, &mut payload)
            .await
            .unwrap();
        let decoded: Response = postcard::from_bytes(&payload).unwrap();
        assert_eq!(decoded.request_id, 9);
    }
}
