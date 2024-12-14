use std::sync::mpsc;
use std::thread;
use std::io::{self, Write};

fn main() {
    let (tx3, rx3) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx1, rx1) = mpsc::channel();

    let serv3_handle = thread::spawn(move || serv3(rx3));
    let serv2_handle = thread::spawn(move || serv2(rx2, tx3));
    let serv1_handle = thread::spawn(move || serv1(rx1, tx2));

    loop {
        print!("Enter a message (or 'all_done' to quit): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input. Try again.");
            continue;
        }

        let trimmed_input = input.trim();
        if trimmed_input == "all_done" {
            let _ = tx1.send(Message::Halt);
            break;
        }

        match parse_message(trimmed_input) {
            Ok(message) => {
                if tx1.send(message).is_err() {
                    eprintln!("Error sending message to serv1.");
                    break;
                }
            }
            Err(_) => eprintln!("Invalid message format. Try again."),
        }
    }

    let _ = serv1_handle.join();
    let _ = serv2_handle.join();
    let _ = serv3_handle.join();
}

#[derive(Debug)]
enum Message {
    Add(i32, i32),
    Sub(i32, i32),
    Halt,
    Other(String),
}

fn parse_message(input: &str) -> Result<Message, ()> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    match tokens.as_slice() {
        ["add", x, y] => Ok(Message::Add(x.parse().map_err(|_| ())?, y.parse().map_err(|_| ())?)),
        ["sub", x, y] => Ok(Message::Sub(x.parse().map_err(|_| ())?, y.parse().map_err(|_| ())?)),
        _ => Ok(Message::Other(input.to_string())),
    }
}

fn serv1(rx: mpsc::Receiver<Message>, tx_next: mpsc::Sender<Message>) {
    for message in rx {
        match message {
            Message::Add(x, y) => println!("(serv1) Add {} + {} = {}", x, y, x + y),
            Message::Sub(x, y) => println!("(serv1) Subtract {} - {} = {}", x, y, x - y),
            Message::Halt => {
                println!("(serv1) Halting...");
                let _ = tx_next.send(Message::Halt);
                break;
            }
            Message::Other(data) => {
                println!("(serv1) Unhandled message: {}", data);
                let _ = tx_next.send(Message::Other(data));
            }
        }
    }
}

fn serv2(rx: mpsc::Receiver<Message>, tx_next: mpsc::Sender<Message>) {
    for message in rx {
        match message {
            Message::Halt => {
                println!("(serv2) Halting...");
                let _ = tx_next.send(Message::Halt);
                break;
            }
            Message::Other(data) => {
                println!("(serv2) Unhandled message: {}", data);
                let _ = tx_next.send(Message::Other(data));
            }
            other => {
                println!("(serv2) Forwarding unhandled message: {:?}", other);
                let _ = tx_next.send(other);
            }
        }
    }
}

fn serv3(rx: mpsc::Receiver<Message>) {
    let mut unhandled_count = 0;
    for message in rx {
        match message {
            Message::Halt => {
                println!(
                    "(serv3) Halting... Total unhandled messages: {}",
                    unhandled_count
                );
                break;
            }
            Message::Other(data) => {
                println!("(serv3) Not handled: {}", data);
                unhandled_count += 1;
            }
            other => {
                println!("(serv3) Not handled: {:?}", other);
                unhandled_count += 1;
            }
        }
    }
}
