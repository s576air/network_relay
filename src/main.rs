use std::{fs::File, io::{self, Read, Write}, net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream}, thread, time::Duration};

struct Connection<T>(T);

type Listener = Connection<TcpListener>;

impl Connection<TcpListener> {
    fn new(ip: &str) -> Self {
        println!("ip 주소 '{}'로 TcpListener을 생성합니다.", ip);
        Self(TcpListener::bind(ip).unwrap())
    }

    fn get_stream(&mut self) -> Option<TcpStream> {
        match self.0.accept() {
            Ok((s, _)) => Some(s),
            Err(_) => None
        }
    }

    fn name(&self) -> &str {
        "TcpListener"
    }
}

type Stream = Connection<SocketAddr>;

impl Connection<SocketAddr> {
    fn new(ip: &str) -> Self {
        println!("TcpSteam 생성을 위해 ip 주소 '{}'을 등록합니다.", ip);
        let (ip, port) = ip.split_once(':').unwrap();
        let ip: Ipv4Addr = ip.parse().unwrap();
        let port: u16 = port.parse().unwrap();
        let socket_addr = SocketAddr::V4(SocketAddrV4::new(ip, port));
        Self(socket_addr)
    }

    fn get_stream(&mut self) -> Option<TcpStream> {
        TcpStream::connect_timeout(&self.0, Duration::from_secs(2)).ok()
    }
    
    fn name(&self) -> &str {
        "TcpStream"
    }
}

fn main() -> io::Result<()> {
    let mut s = String::new();
    {
        File::open("config.txt").unwrap().read_to_string(&mut s).unwrap();
    }
    let (mut listen_addr, mut server_addr) = s.split_once(',').unwrap();
    listen_addr = listen_addr.trim();
    server_addr = server_addr.trim();

    let mut client_conn = Stream::new(listen_addr);
    let mut server_conn = Listener::new(server_addr);

    let mut client = client_conn.get_stream();
    let mut server = server_conn.get_stream();
    loop {
        match &mut client {
            Some(stream) => if stream.write(&[]).is_err() { client = client_conn.get_stream() },
            None => client = client_conn.get_stream(),
        }
        match &mut server {
            Some(stream) => if stream.write(&[]).is_err() { server = server_conn.get_stream() },
            None => server = server_conn.get_stream(),
        }
        println!("1. {}, {}의 연결 여부: {}", client_conn.name(), listen_addr, client.is_some());
        println!("2. {}, {}의 연결 여부: {}", server_conn.name(), server_addr, server.is_some());

        if client.is_some() && server.is_some() {
            println!("데이터 중계를 시작합니다.");
            let mut client1 = client.take().unwrap();
            let mut server1 = server.take().unwrap();
            
            let Ok(mut client2) = client1.try_clone() else { break };
            let Ok(mut server2) = server1.try_clone() else { break };

            let client_to_server_thread = thread::spawn(move || {
                let _ = io::copy(&mut client2, &mut server1);
                let _ = client2.shutdown(Shutdown::Both);
                let _ = server1.shutdown(Shutdown::Both);
            });
            
            let server_to_client_thread = thread::spawn(move || {
                let _ = io::copy(&mut server2, &mut client1);
                let _ = server2.shutdown(Shutdown::Both);
                let _ = client1.shutdown(Shutdown::Both);
            });

            client_to_server_thread.join().unwrap();
            server_to_client_thread.join().unwrap();

            client = None;
            server = None;
            println!("데이터 중계가 종료되었습니다.");
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
