use pharos::{Filter, Observable};
use rust_editor::{error, log, plugins::plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;
use wasm_bindgen_futures::spawn_local;


use futures::{
    io::WriteHalf,
    lock::Mutex,
    AsyncReadExt, AsyncWriteExt, StreamExt,
};
use ws_stream_wasm::{WsEvent, WsMeta, WsStreamIo};

use crate::map::map::Map;



type Writer = WriteHalf<async_io_stream::IoStream<WsStreamIo, Vec<u8>>>;

#[editor_plugin(specific_to=Map, description="Creates a connection to a remote application to stream made changes live.")]
pub struct Sync {
    
    #[option(
        label = "URL",
        description = "Remote applications url used to connect to",
        validator = r"(^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}
        (?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$)|127.0.0.1|localhost",
    )]
    url: String,

    #[option(
        label = "Port",
        description = "Remote applications port used to connect to. Make sure the port is accessible",
        validator = r"^([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$" 
    )]
    port: String,

    #[option(skip)]
    ws: Rc<Mutex<Option<Writer>>>,
}

impl Plugin<Map> for Sync {
    fn startup(&mut self, _: &mut App<Map>) -> Result<(), EditorError> {
        let ws = Rc::clone(&self.ws);

        let program = async move {
            let (mut _ws, _) = WsMeta::connect("ws://127.0.0.1:8765", None).await.unwrap();

            match WsMeta::connect("ws://127.0.0.1:8765", None).await {
                Ok((mut meta, stream)) => {
                    

       
                    let chain = async move {
                        let mut evts = meta.observe(Filter::Pointer(WsEvent::is_closed).into()).await.unwrap();
                        

                        while let Some(next) = evts.next().await {
                            match next {
                                WsEvent::Open => log!("Socket opened"),
                                WsEvent::Error => log!("Socket error"),
                                WsEvent::Closing => log!("Socket closing"),
                                WsEvent::Closed(e) => log!("Socked closed {:?}", e),
                                WsEvent::WsErr(e) => log!("WS Error {:?}", e)
                            }
                        }                        
                    };
                    spawn_local(chain);



                    let wsstream_io = stream.into_io();
                    let (mut sink, stream) = wsstream_io.split();

                    let handle_message = async move {
                        let mut data: Vec<u8> = vec![0; 1024];
                        let len = sink.read(&mut data).await.unwrap();

                        data.truncate(len);
                    };
                    spawn_local(handle_message);

                    let mut guard = ws.lock().await;
                    *guard = Some(stream);
                }
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
        };

        spawn_local(program);

        Ok(())
    }

    fn property_updated(&mut self, _: &str, _: &mut App<Map>) {
        let url = &self.url;
        let port = &self.port;

        log!("url={}, port={}", url, port)

    }
}

impl Sync {
    pub async fn send(&mut self, map: Map) {
        let ws: Rc<Mutex<Option<Writer>>> = self.ws.clone();

        let mut guard = ws.lock().await;
        let state: &mut Writer = guard.as_mut().unwrap();

        let data = serde_json::to_string(&map).unwrap();
        match state.write(data.as_bytes()).await {
            Ok(num_bytes_written) => log!("Written {} bytes", num_bytes_written),
            Err(e) => error!("{}", e),
        }
    }
}
