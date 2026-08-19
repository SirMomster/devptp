use crate::{
    error::Result,
    helpers::{receive_json, send_json},
    protocol::{Request, Response, STREAM_NAME},
};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericNamespaced,
};

use tokio::io::BufReader;

pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Self {}
    }

    async fn send_ipc_command(&self, request: Request) -> Result<Response> {
        let name = STREAM_NAME.to_ns_name::<GenericNamespaced>()?;
        let stream = Stream::connect(name).await?;

        let (read_half, mut write_half) = tokio::io::split(stream);
        let mut reader = BufReader::new(read_half);

        send_json(&mut write_half, &request).await?;

        Ok(receive_json::<_, Response>(&mut reader).await?.unwrap())
    }

    pub async fn send_expose(&self, local_port: u16) -> Result<()> {
        let response = self
            .send_ipc_command(Request::ExposePort { local_port })
            .await?;
        self.handle_response(response)?;
        Ok(())
    }

    pub async fn send_connect(&self, ticket: String) -> Result<()> {
        let response = self.send_ipc_command(Request::Connect { ticket }).await?;
        self.handle_response(response)?;
        Ok(())
    }

    pub async fn send_start_serving(&self) -> Result<()> {
        let response = self.send_ipc_command(Request::StartServing).await?;
        self.handle_response(response)?;
        Ok(())
    }

    pub async fn send_ping(&self) -> Result<()> {
        let response = self.send_ipc_command(Request::Ping).await?;
        self.handle_response(response)?;
        Ok(())
    }

    fn handle_response(&self, response: Response) -> Result<()> {
        match response {
            Response::Pong => println!("pong"),
            Response::Ok => println!("Ok"),
            Response::Status { running } => println!("Running is {}", running),
            Response::Error { message } => println!("Error: {}", message),
            Response::ServingStarted { ticket } => println!("Ticket: {}", ticket),
        };
        Ok(())
    }
}
