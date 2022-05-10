use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;


const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL).expect("listener failed to bind");
    server.set_nonblocking(true).expect("failed to initialize non-blocking");

    let mut client = vec![];
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket,addr)) = server.accept() {

            println!("client {} connected", addr);
            let tx = tx.clone();
            client.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || {
               let mut buff = vec![0 as u8, MSG_SIZE as u8];

                match socket.read_exact(&mut buff){
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("invalid utf8 message");

                        println!("{}: {:?}",addr, msg);
                        tx.send(msg).expect("failed to send message to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        panic!("closing connection with {}",addr);
                    }
                }

                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            client = client.into_iter().filter_map(|mut client|{
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE,0);
                client.write_all(&buff).map( |_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100))
}
