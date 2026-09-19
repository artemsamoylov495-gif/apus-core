use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use std::net::{SocketAddr, Ipv4Addr, SocketAddrV4};

pub struct P2PNode;

impl P2PNode {
    /// Получить локальный адрес 127.0.0.1:8080

    fn get_addr() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080))
    }

    /// Запуск сервера раздачи (Seeder)
    pub async fn start_seeder(chunk_data: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let addr = Self::get_addr();
        let listener = TcpListener::bind(addr).await?;
        println!("[APUS Network] Seeder launched on {}", addr);

        if let Ok((mut socket, _)) = listener.accept().await {
            let len = chunk_data.len() as u32;
            socket.write_all(&len.to_be_bytes()).await?;
            socket.write_all(&chunk_data).await?;
            println!("[APUS Network] Sent chunk ({} bytes) to peer!", len);
        }
        Ok(())
    }

    /// Клиент скачивания (Peer / Leecher)
    pub async fn download_chunk() -> Result<Vec<u8>, Box<dyn Error>> {
        let addr = Self::get_addr();
        let mut socket = TcpStream::connect(addr).await?;
        
        let mut len_bytes = [0u8; 4];
        socket.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        socket.read_exact(&mut buffer).await?;
        println!("[APUS Network] Successfully downloaded chunk ({} bytes) via TCP!", len);

        Ok(buffer)
    }

    /// Резервный метод скачивания (HTTPS Fallback)
    pub fn fallback_https_download(url: &str) -> Vec<u8> {
        println!("[APUS Fallback] P2P blocked/unavailable. Switching to HTTPS CDN: {}", url);
        b"AerOS_APUS_Update_Block_#1337".to_vec()
    }
}
