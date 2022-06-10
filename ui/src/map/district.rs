use geo::{prelude::{Contains}, Coordinate, LineString, Polygon};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rust_editor::{
    gizmo::Id,
    interactive_element::{InteractiveElement, InteractiveElementState},
    style::{InteractiveElementStyle, Style},
};
use rust_editor::{gizmo::SetId, renderer::PrimitiveRenderer};
use rust_macro::ElementId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d; // TODO

use super::house::generate_houses_from_polygon;

#[derive(Serialize, Deserialize, Clone)]
pub struct House {
    pub polygon: Polygon<f64>,

    #[serde(skip_serializing)]
    pub line_styles: Vec<Style>,

    #[serde(skip_serializing)]
    pub style: Style,
}

#[derive(Serialize, Deserialize, ElementId, Clone)]
pub struct District {
    pub(crate) id: Uuid,
    pub(crate) polygon: Polygon<f64>,

    #[serde(skip_serializing)]
    pub(crate) style: InteractiveElementStyle,

    #[serde(skip_serializing)]
    pub(crate) state: InteractiveElementState,

    #[serde(skip_serializing)]
    pub(crate) minimum_house_side: f64,

    #[serde(skip_serializing)]
    pub(crate) seed: <ChaCha8Rng as SeedableRng>::Seed,

    pub(crate) houses: Vec<House>,
}

impl Default for District {
    fn default() -> Self {
        District {
            id: Uuid::new_v4(),
            polygon: Polygon::new(LineString::from(vec![Coordinate { x: 0., y: 0. }]), vec![]),
            style: InteractiveElementStyle {
                normal: Style {
                    border_width: 0,
                    border_color: "#000000".to_string(),
                    background_color: "rgba(30, 136, 229, 0.4)".to_string(),
                },
                hover: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "#1e88e5".to_string(),
                },
                selected: Style {
                    border_width: 0,
                    border_color: "".to_string(),
                    background_color: "hsl(0, 100%, 50%)".to_string(),
                },
            },
            state: InteractiveElementState::Normal,
            minimum_house_side: 500.0,
            houses: Vec::new(),
            seed: Default::default(),
        }
    }
}

impl InteractiveElement for District {
    fn set_state(&mut self, new_state: InteractiveElementState) {
        self.state = new_state;
    }

    fn style(&self) -> &Style {
        match self.state {
            InteractiveElementState::Normal => return &self.style.normal,
            InteractiveElementState::Hover => return &self.style.hover,
            InteractiveElementState::Selected => return &self.style.selected,
        }
    }

    fn state(&self) -> InteractiveElementState {
        self.state.clone()
    }
}

impl District {
    pub fn is_point_on_district(&self, point: &Coordinate<f64>) -> bool {
        self.polygon.contains(point)
    }

    pub fn polygon(&self) -> &Polygon<f64> {
        &self.polygon
    }

    pub fn update_houses(&mut self) {
        self.houses =
            generate_houses_from_polygon(&self.polygon, self.minimum_house_side, self.seed);
    }

    pub fn render(&self, context: &CanvasRenderingContext2d) -> Result<(), JsValue> {

        /*
        self.polygon.render(&Style {
            border_width: 4,
            border_color: "#FFFFFF".to_string(),
            background_color: "#FFFFFF".to_string(),
        }, context)?;

        let convex_hull = self.polygon.convex_hull();
        convex_hull.render(&Style {
            border_width: 4,
            border_color: "#FFFFFF".to_string(),
            background_color: "rgb(0, 0, 255, 0.3)".to_string(),
        }, context)?;

        let mut min_area = f64::MAX;
        let mut min_angle = 0.;

        let mut min_poly: Polygon<f64> = convex_hull.bounding_rect().unwrap().into();
        for (i, line) in convex_hull.exterior().lines().enumerate() {



            let angle = (-1. * f64::atan2(line.end.y - line.start.y, line.end.x - line.start.x)) * 180.0 / PI;

            //let ne = convex_hull.clone().rotate_around_point(45., convex_hull.centroid().unwrap());
            let mut bbox: Polygon<f64> = convex_hull.bounding_rect().unwrap().into();
            bbox = convex_hull.rotate_around_point(angle, bbox.centroid().unwrap()).bounding_rect().unwrap().into();
            bbox = bbox.rotate_around_point(angle * -1., bbox.centroid().unwrap());
            let area = bbox.signed_area();

            
            if area < min_area {
                min_area = area;
                min_angle = angle;
                min_poly = bbox.clone();
            }
            

            


            bbox.render(&Style {
                border_width: 1,
                border_color: "rgb(255, 255, 255, 0.1)".to_string(),
                background_color: "rgb(0, 0, 255, 0.1)".to_string(),
            }, context)?;


        }

        min_poly.render(&Style {
            border_width: 1,
            border_color: "rgb(255, 255, 255, 1.0)".to_string(),
            background_color: "rgb(0, 0, 255, 0.1)".to_string(),
        }, context)?;
        */



        
        context.save();

        for p in &self.houses {
            p.polygon.render(&p.style, context)?;

            for (line, style) in p.polygon.exterior().lines().zip(p.line_styles.iter()) {
                line.render(style, context)?;
            }
        }

        context.restore();
        

        Ok(())
    }
}