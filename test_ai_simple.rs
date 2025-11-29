use quic_torrent_client_server::client::send_ai_query;
use quic_torrent_client_server::messages::{AiRequest, AiParameters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = "162.221.207.169";
    let port = 7001;
    
    println!("Testing AI request...");
    let response = send_ai_query(
        server,
        port,
        "What is artificial intelligence?",
        None,
        Some(AiParameters {
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: Some(0.9),
        }),
    ).await?;
    
    println!("AI Response received!");
    println!("Answer length: {}", response.answer.len());
    println!("Tokens: {:?}", response.metadata.and_then(|m| m.total_tokens));
    Ok(())
}
