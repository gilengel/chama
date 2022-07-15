use pharos::{Filter, Observable};
use plugin_ribbon::model::ribbon_button::{RibbonButton, RibbonButtonType};
use plugin_ui_components::ComponentMessage;
use rust_editor::{error, log, plugin::Plugin, ui::app::EditorError};
use rust_macro::editor_plugin;

use futures::{io::WriteHalf, lock::Mutex, AsyncReadExt, AsyncWriteExt, StreamExt};
use wasm_bindgen_futures::spawn_local;
use ws_stream_wasm::{WsErr, WsEvent, WsMeta, WsStreamIo};

use crate::map::map::Map;

type Writer = WriteHalf<async_io_stream::IoStream<WsStreamIo, Vec<u8>>>;

#[editor_plugin(specific_to=Map, description="Creates a connection to a remote application to stream made changes live.")]
pub struct Sync {
    #[option(
        label = "URL",
        validator = r"(^(?:(?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])\.){3}
        (?:25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9][0-9]|[0-9])$)|127.0.0.1|localhost"
    )]
    url: String,

    #[option(
        label = "Port",
        validator = r"^([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$"
    )]
    port: String,

    #[option(skip)]
    ws: Rc<Mutex<Option<Writer>>>,

    #[option(skip)]
    connected: Rc<RefCell<Option<WsEvent>>>,

    #[option(skip)]
    connection_error: Rc<RefCell<Option<WsErr>>>,
}

impl Sync {
    pub fn connected(&self) -> bool {
        let value = self.connected.as_ref().borrow();
        if !value.is_some() {
            return false;
        }

        *value.as_ref().unwrap().borrow() == WsEvent::Open
    }
}

impl Plugin<Map> for Sync {
    fn startup(&mut self, editor: &mut App<Map>) -> Result<(), EditorError> {
        self.connected = Rc::new(RefCell::new(None));
        self.connection_error = Rc::new(RefCell::new(None));

        editor.plugin_mut(move |ribbon: &mut plugin_ribbon::RibbonPlugin<Map>| {
            let tab = ribbon.get_or_add_tab_mut("default", "Default").unwrap();
            let group = tab.get_or_add_group_mut("sync", "Remote Sync").unwrap();

            let ws = self.ws.clone();
            let connected_state = self.connected.clone();
            let connection_error = self.connection_error.clone();
            let connect_btn = RibbonButton::new(
                "cast",
                "cast",
                None,
                Some(RibbonButtonType::Toggle),
                move || {
                    log!("Clicked");
                    /*
                    move || {
                        EditorMessages::PluginMessage(
                            ComponentsPlugin::identifier(),
                            ComponentMessage::ShowSnackbar("Success :)", None, None),
                        )
                    }
                    */

                    connect(
                        ws.clone(),
                        connected_state.clone(),
                        connection_error.clone(),
                        || {
                            log!("SUCCESS :)")
                        },
                        || {
                            log!("ERROR :(")
                        },
                    );

                    EditorMessages::PluginMessage(
                        "ComponentsPlugin",
                        Box::new(ComponentMessage::ShowSnackbar("Success :)", None, None)),
                    )
                },
            );

            group.add_action(connect_btn);
        });

        Ok(())
    }

    fn property_updated(&mut self, _: &str, _: &mut App<Map>) {
        //let url = &self.url;
        //let port = &self.port;
    }
}

impl Sync {
    pub async fn send(&mut self, map: Map) {
        if !self.connected() {
            return;
        }

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

fn connect<F, T>(
    ws: Rc<Mutex<Option<Writer>>>,
    connected_state: Rc<RefCell<Option<WsEvent>>>,
    error_state: Rc<RefCell<Option<WsErr>>>,
    mut success_callback: F,
    mut error_callback: T,
) where
    F: FnMut() + 'static,
    T: FnMut() + 'static,
{
    let program = async move {
        match WsMeta::connect("ws://127.0.0.1:8765", None).await {
            Ok((mut meta, stream)) => {
                let chain = async move {
                    let mut evts = meta
                        .observe(Filter::Pointer(WsEvent::is_closed).into())
                        .await
                        .unwrap();

                    while let Some(next) = evts.next().await {
                        *connected_state.borrow_mut() = Some(next);
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

                success_callback();
            }
            Err(e) => {
                *error_state.borrow_mut() = Some(e);

                error_callback();
                return;
            }
        };
    };

    spawn_local(program);
}
