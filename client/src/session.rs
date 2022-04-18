use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::interceptor::twcc::receiver::Receiver;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::math_rand_alpha;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

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

    // Register data channel creation handling
    peer_connection
    .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let d_label = d.label().to_owned();
        let d_id = d.id();
        println!("New DataChannel {} {}", d_label, d_id);

        // Register channel opening handling
        Box::pin(async move {
            let d2 = Arc::clone(&d);
            let d_label2 = d_label.clone();
            let d_id2 = d_id;
            d.on_open(Box::new(move || {
                println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d_label2, d_id2);

                Box::pin(async move {
                    let mut result = Result::<usize>::Ok(0);
                    while result.is_ok() {
                        let timeout = tokio::time::sleep(Duration::from_secs(5));
                        tokio::pin!(timeout);

                        tokio::select! {
                            _ = timeout.as_mut() =>{
                                let message = math_rand_alpha(15);
                                println!("Sending '{}'", message);
                                result = d2.send_text(message).await.map_err(Into::into);
                            }
                        };
                    }
                })
            })).await;

            // Register text message handling
            d.on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
                Box::pin(async {})
            })).await;
        })
    }))
    .await;

    let pc = api.new_peer_connection(RTCConfiguration::default()).await?;

    let offer = pc.create_offer(None).await?;

    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        println!("{:?}", base64::encode(json_str));
    } else {
        println!("generate local_description failed!");
    }

    Ok(())
}
