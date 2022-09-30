use azure_messaging_servicebus::prelude::*;
use rustdotenv::load;
use serde_json::{Value};
use std::time::Duration;
use std::env;
use std::process;
use std::io;
use std::sync::mpsc;
use std::sync::mpsc::{Sender};
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

struct Config {
    action: String,
    queue: String,
}

/// Handle parsing of command line arguments
impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
      if args.len() < 3 {
          return Err("Missing arguments. Usage: rustybus <send/receive/peek> <queue name>");
      }
      let action = args[1].clone();
      let queue = args[2].clone();
  
      Ok(Config { action, queue })
    }
}

#[tokio::main]
async fn main() {
    // Spawn a thread + channel to check for piped input
    // always spawned but only used for send
    let (tx, rx) = mpsc::channel::<String>();
    spawn_stdin_channel(tx);
    sleep(1000);

    // Check for piped input on receiver side of channel - only valid with send
    let mut message_to_send: String = "".to_string();
    loop{
        match rx.try_recv() {
            Ok(key) => { message_to_send.push_str(&key);},
            Err(TryRecvError::Empty) => { 
                if message_to_send.eq("") {
                    message_to_send.push_str("Empty"); 
                }
                break;
            },
            Err(TryRecvError::Disconnected) => { 
                if message_to_send.eq("") {
                    message_to_send.push_str("Empty"); 
                }
                break;
            },
        };
    };   
     
    // Save command line args for later use
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    
    // Load .env file
    load(None);

    // Load service bus client info from variables specified in .env file 
    let service_bus_namespace = std::env::var("AZURE_SERVICE_BUS_NAMESPACE")
      .expect("Service Bus Namespace not found in .env file");
    let policy_name = std::env::var("AZURE_POLICY_NAME")
      .expect("Policy name not found in .env file");
    let policy_key = std::env::var("AZURE_POLICY_KEY")
      .expect("Policy key not found in .env file");
    let http_client = azure_core::new_http_client();

    // Build service bus client
    let client = Client::new(
        http_client,
        service_bus_namespace,
        config.queue,
        policy_name,
        policy_key,
    )
    .expect("Failed to create client");

    // Call helper functions to handle send/receive/peek
    match config.action.as_str() {
        "send" => {send(client, message_to_send).await},
        "receive" => {receive(client).await},
        "peek" => {peek(client).await},
        _other => {println!("Invalid command")},
    }
}

/// Handle reading of input streamed via pipe for send command
fn spawn_stdin_channel(tx: Sender<String>) {
    thread::spawn(move || loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_n) => {
                let x = tx.send(buffer);
                if x.is_ok() {
                    continue;
                } else {
                    break;
                }
            }
            Err(error) => {println!("Error reading from stdin. Error: {}", error); break;}
        }
    });
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}

/// Handle sending messages
async fn send(client: Client, message_to_send: String) {
    // Remove newline chars from JSON string
    let message = message_to_send.replace("\n", "");

    client.send_message(&message)
        .await
        .expect("Failed to send message");
        // If the previous statement doesn't return an
        // error we can assume the send was successful.
        println!("Message sent!");
}

/// Handle receiving messages. 
async fn receive(client: Client) {
    let received_message = client
        .receive_and_delete_message()
        .await
        .expect("Failed to receive message");

    let value: Result<Value, serde_json::Error> = serde_json::from_str(&received_message);

    match value {
        Ok(message) => {println!("Message: {}", message);},
        Err(_error) => {println!("No messages in queue");},
    }
}

/// Handle peeking at messages
async fn peek(client: Client) {
    let three_seconds = Duration::new(3,0);
    let peek_message = client
    .peek_lock_message2(Some(three_seconds))
    .await
    .expect("Failed to receive message");

    let value: Result<Value, serde_json::Error> = serde_json::from_str(&peek_message.body());
    match value {
        Ok(message) => {
            println!("Peek message: {}", message);
            let unlock = peek_message.unlock_message().await;
            if unlock.is_err() {
                println!("Unable to unlock message");
            }
        },
        Err(_error) => {println!("No messages in queue");},
    }
}    