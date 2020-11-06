use tokio::stream::StreamExt;

use loxone::WebSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*
    let user = "admin";
    let password = "TdtuPMJjZTTutWetWMoPXy9V";
    let permission = 4;
    let uuid = "098802e1-02b4-603c-ffffeee000d80cfd";
    let info = "rust";
    */

    let cert = tokio::fs::read_to_string("public_key.pem").await?;
    let ws_url = "ws://172.16.3.59/ws/rfc6455".parse()?;

    let (mut ws, resp, rx, recv_loop) = WebSocket::connect(ws_url).await?;
    println!("WebSocket handshake has been successfully completed");
    println!("{:?}", resp);

    let recv_loop = tokio::spawn(recv_loop);
    println!("running recv loop on dedicated task");

    let reply = ws.key_exchange(&cert).await?;
    println!("exchanged session key: {} bytes", reply.len());

    let jwt: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&tokio::fs::read_to_string("token.json").await?)?;

    let reply = ws.authenticate(jwt["token"].as_str().unwrap()).await?;
    println!("authenticated: {}", serde_json::to_string(&reply)?);

    let reply = ws.get_loxapp3_timestamp().await?;
    println!("loxapp3 timestamp: {}", reply);

    let loxapp3: api::LoxoneApp3 = serde_json::from_str(&tokio::fs::read_to_string("loxapp3.json").await?)?;
    println!("loxapp3: {:#?}", loxapp3);

    let (initial_state, mut stream) = ws.enable_status_update(rx).await?;
    println!("got {} state events", initial_state.len());

    for event in initial_state {
        let x = match &event {
            api::Event::Value(uuid, _val) => loxapp3.find_uuid(&uuid),
            api::Event::Text(uuid, _uuid_icon, ref _val) => loxapp3.find_uuid(&uuid),
            _ => None
        };
        if let Some(y) = x {
            println!("found {} => {:?}", y, event);
        }
    }

    /*
    while let Some(event) = stream.next().await {
        println!("event: {:?}", event);
    }

    tokio::try_join!(recv_loop)?;
    */

    Ok(())
}