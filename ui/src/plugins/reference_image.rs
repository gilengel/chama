use geo::Coordinate;
use rust_editor::{input::keyboard::Key, plugins::plugin::Plugin};
use rust_macro::editor_plugin;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{DragEvent, ImageBitmap, Url};
use yew::{classes, function_component, Properties};

use crate::map::map::Map;

#[derive(PartialEq)]
enum State {
    Idle,
    MouseDown,
}

impl Default for State {
    fn default() -> Self {
        State::Idle
    }
}

#[function_component]
fn ImageComponent(props: &ImageProps) -> Html {
    let mut classes = vec!["reference"];
    if props.selected {
        classes.push("reference-selected");
    }

    html! {
        <div class={classes!(classes)} style={format!("width:{}px; height:{}px; left:{}px; top:{}px; z-index:{}", props.size[0], props.size[1], props.position[0], props.position[1], props.z_index)}>
            <img src={props.content.clone()} alt="MUUUUUUUUUUUUUUUUUU" />
        </div>
    }
}

struct ImageData {
    content: String,
    position: [i32; 2],
    size: [u32; 2],
    rotation: f64,
    selected: bool,
    mouse_offset: [i32; 2],
    z_index: i32,
}

#[derive(Properties, PartialEq)]
struct ImageProps {
    content: String,
    size: [u32; 2],
    rotation: f64,
    position: [i32; 2],
    z_index: i32,
    selected: bool,
}

#[editor_plugin(skip, specific_to=Map)]
pub struct ReferenceImage {
    #[option(skip)]
    images: Rc<RefCell<Vec<ImageData>>>,

    #[option(skip)]
    drag_start: Coordinate<f64>,

    #[option(skip)]
    drag_state: State,
}

impl Plugin<Map> for ReferenceImage {
    fn drop(&mut self, event: DragEvent) {
        let images = Rc::clone(&self.images);

        let highest_z_index = self.images.as_ref().borrow().len() as i32;

        spawn_local(async move {
            let images = images.clone();

            if let Some(transfer) = event.data_transfer() {
                let items = transfer.items();
                for i in 0..items.length() {
                    let item = items.get(i).unwrap();

                    if &item.kind()[..] == "file" {
                        let item = item.get_as_file().unwrap().unwrap();

                        let window = web_sys::window().expect("no global `window` exists");

                        let future =
                            JsFuture::from(window.create_image_bitmap_with_blob(&item).unwrap());

                        let resolved = future.await;

                        if let Ok(e) = resolved {
                            let url = Url::create_object_url_with_blob(&item);

                            let content: String = url.unwrap(); //
                            let bitmap: ImageBitmap = e.dyn_into().unwrap();
                            let size = [bitmap.width(), bitmap.height()];

                            let mut images = images.as_ref().borrow_mut();
                            images.push(ImageData {
                                content,
                                position: [
                                    event.client_x() - (size[0] / 2) as i32,
                                    event.client_y() - (size[1] / 2) as i32,
                                ],
                                size,
                                rotation: 0.,
                                selected: false,
                                mouse_offset: [0, 0],
                                z_index: highest_z_index,
                            });
                        }
                    }
                }
            }
        });
    }

    fn editor_elements(&mut self, _: &Context<App<Map>>, _: &App<Map>) -> Vec<Html> {
        let images = self.images.as_ref().borrow();

        let mut elements: Vec<Html> = Vec::with_capacity(images.len());

        for image in images.iter() {
            elements.push(html! {
                <ImageComponent content={image.content.clone()} size={image.size} rotation={image.rotation} position={image.position} selected={image.selected} z_index={image.z_index} />
            });
        }

        elements
    }

    fn key_up(&mut self, key: Key, _: &mut App<Map>) {
        match key {
            Key::Delete => {
                self.images.as_ref().borrow_mut().pop();
            }
            Key::PageUp => {
                let mut images = self.images.as_ref().borrow_mut();
                let images_len = images.len() as i32;

                images
                    .iter_mut()
                    .filter(|image| image.selected)
                    .for_each(|image| {
                        if image.z_index < images_len - 1 {
                            image.z_index += 1
                        }
                    })
            }
            Key::PageDown => {
                let mut images = self.images.as_ref().borrow_mut();
                images
                    .iter_mut()
                    .filter(|image| image.selected)
                    .for_each(|image| {
                        if image.z_index > 0 {
                            image.z_index -= 1
                        }
                    })
            }
            _ => {}
        };
    }

    fn mouse_down(&mut self, mouse_pos: Coordinate<f64>, _: u32, _: &App<Map>) {
        self.drag_state = State::MouseDown;

        fn within_bounds(pos: Coordinate<f64>, start: [i32; 2], size: [u32; 2]) -> bool {
            pos.x >= start[0] as f64
                && pos.x <= start[0] as f64 + size[0] as f64
                && pos.y >= start[1] as f64
                && pos.y <= start[1] as f64 + size[1] as f64
        }

        let mut images = self.images.as_ref().borrow_mut();
        let images_len = images.len() as i32;
        images.iter_mut().for_each(|image| image.selected = false);

        let image_selected = images
            .iter()
            .filter(|image| {
                within_bounds(mouse_pos, image.position, image.size)
                    && image.z_index != images_len - 1
            })
            .max_by(|x, y| x.z_index.cmp(&y.z_index))
            .is_some();

        if image_selected {
            images
                .iter_mut()
                .filter(|image| image.z_index > 0)
                .for_each(|image| image.z_index -= 1);
        }

        match images
            .iter_mut()
            .filter(|image| within_bounds(mouse_pos, image.position, image.size))
            .max_by(|x, y| x.z_index.cmp(&y.z_index))
        {
            Some(image) => {
                image.selected = true;
                image.z_index = images_len as i32 - 1;
                image.mouse_offset = [
                    mouse_pos.x as i32 - image.position[0],
                    mouse_pos.y as i32 - image.position[1],
                ];
            }
            None => {
                images.iter_mut().for_each(|image| {
                    image.selected = false;
                    image.mouse_offset = [0, 0]
                });
            }
        }

        self.drag_start = mouse_pos;
    }

    fn mouse_move(
        &mut self,
        mouse_pos: Coordinate<f64>,
        _mouse_movement: Coordinate<f64>,
        _: &mut App<Map>,
    ) {
        if self.drag_state == State::Idle {
            return;
        }

        let mut images = self.images.as_ref().borrow_mut();
        images
            .iter_mut()
            .filter(|image| image.selected)
            .for_each(|image| {
                image.position = [
                    mouse_pos.x as i32 - image.mouse_offset[0],
                    mouse_pos.y as i32 - image.mouse_offset[1],
                ]
            })
    }

    fn mouse_up(&mut self, _mouse_pos: Coordinate<f64>, _: u32, _: &mut App<Map>) {
        self.drag_state = State::Idle;
    }
}
