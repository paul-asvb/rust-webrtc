use anyhow::Result;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

pub async fn start_session() -> Result<String> {
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

    let peer_connection = api.new_peer_connection(config).await?;

    let offer = peer_connection.create_offer(None).await?;
    //let s = serde_json::to_string(&offer).unwrap();
    
    // Create a new RTCPeerConnection
    //let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // Create a tokio datachannel for concurrency
    //let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

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

    // // Set the remote SessionDescription
    // peer_connection.set_remote_description(offer).await?;

    // // Create an answer
    // let answer = peer_connection.create_answer(None).await?;

    // // Create channel that is blocked until ICE Gathering is complete
    // let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // // Sets the LocalDescription, and starts our UDP listeners
    // peer_connection.set_local_description(answer).await?;

    // // Block until ICE Gathering is complete, disabling trickle ICE
    // // we do this because we only can exchange one signaling message
    // // in a production application you should exchange ICE Candidates via OnICECandidate
    // let _ = gather_complete.recv().await;

    // // Output the answer in base64 so we can paste it in browser
    // if let Some(local_desc) = peer_connection.local_description().await {
    //     let json_str = serde_json::to_string(&local_desc)?;
    //     println!("{:?}", base64::encode(json_str.clone()));
    //     Ok(json_str)
    // } else {
    //     println!("generate local_description failed!");
    //     Ok(r#"generate local_description failed!"#.to_string())
    // }
    Ok("dsfg".to_string())
}
