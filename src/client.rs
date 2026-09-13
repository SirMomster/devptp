use crate::{
    error::{Error, Result},
    helpers::{receive_json, send_json},
    protocol::{Request, Response, ipc_socket_path},
};
use interprocess::local_socket::{
    GenericFilePath, ToFsName,
    tokio::{Stream, prelude::*},
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::BufReader;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Client;

impl Client {
    pub fn new() -> Self {
        Self
    }

    async fn send_ipc_command(&self, method: &str, params: Value) -> Result<Response> {
        let name = ipc_socket_path().to_fs_name::<GenericFilePath>()?;
        let stream = Stream::connect(name).await?;
        let (read_half, mut write_half) = tokio::io::split(stream);
        let mut reader = BufReader::new(read_half);
        let request = Request {
            id: Some(json!(NEXT_ID.fetch_add(1, Ordering::Relaxed))),
            method: method.to_string(),
            params,
        };
        send_json(&mut write_half, &request).await?;
        receive_json::<_, Response>(&mut reader)
            .await?
            .ok_or_else(|| Error::Custom("IPC server closed the connection".into()))
    }

    async fn command(&self, method: &str, params: Value) -> Result<()> {
        let response = self.send_ipc_command(method, params).await?;
        println!(
            "{}",
            serde_json::to_string(&response).map_err(|e| Error::Custom(e.to_string()))?
        );
        Ok(())
    }

    pub async fn send_expose(&self, local_port: u16) -> Result<()> {
        self.command("expose_port", json!({"local_port": local_port}))
            .await
    }
    pub async fn send_connect(&self, ticket: String) -> Result<()> {
        self.command("connect", json!({"ticket": ticket})).await
    }
    pub async fn send_start_serving(&self) -> Result<()> {
        self.command("start_serving", json!({})).await
    }
    pub async fn send_ping(&self) -> Result<()> {
        self.command("ping", json!({})).await
    }
    pub async fn send_status(&self) -> Result<()> {
        self.command("status", json!({})).await
    }
    pub async fn send_disconnect(&self, port: Option<u16>) -> Result<()> {
        self.command("disconnect", json!({"port": port})).await
    }
    pub async fn send_list_forwarded_ports(&self) -> Result<()> {
        self.command("list_forwarded_ports", json!({})).await
    }
    pub async fn send_shutdown(&self) -> Result<()> {
        self.command("shutdown", json!({})).await
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
