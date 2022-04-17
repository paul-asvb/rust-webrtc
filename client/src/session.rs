use anyhow::Result;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::interceptor::twcc::receiver::Receiver;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

pub async fn start_session() -> Result<()> {
    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();
    // Register default codecs
    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // Create a tokio datachannel for concurrency
    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            match s {
                RTCPeerConnectionState::Unspecified => {
                    println!("RTCPeerConnectionState::Unspecified")
                }
                RTCPeerConnectionState::New => println!("RTCPeerConnectionState::New"),
                RTCPeerConnectionState::Connecting => {
                    println!("RTCPeerConnectionState::Connecting")
                }
                RTCPeerConnectionState::Connected => {
                    println!("RTCPeerConnectionState::Connected")
                }
                RTCPeerConnectionState::Disconnected => {
                    println!("RTCPeerConnectionState::Disconnected")
                }
                RTCPeerConnectionState::Failed => println!("RTCPeerConnectionState::Failed"),
                RTCPeerConnectionState::Closed => println!("RTCPeerConnectionState::Unspecified"),
            }

            Box::pin(async {})
        }))
        .await;

    Ok(())
}
