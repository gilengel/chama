use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::intersection::Intersection;
use crate::street::Street;


pub struct RoadSystem<'a> {
    intersections: Vec<Intersection>,
    streets: Vec<Street<'a>>
}

impl<'a> RoadSystem<'a> {
    pub fn new() -> RoadSystem<'a> {
        RoadSystem { 
            intersections: vec![
                Intersection { position : Vec2::new(30.0, 30.0)}
            ], 
            streets: vec![] 
        }
    }

    pub fn update(&self, commands: &mut Commands, materials: &mut ResMut<Assets<ColorMaterial>>, mut meshes: &mut ResMut<Assets<Mesh>>) {
        let shape = shapes::RegularPolygon {
            sides: 6,
            feature: shapes::RegularPolygonFeature::Radius(200.0),
            ..shapes::RegularPolygon::default()
        };

        // build the intersections
        for intersection in &self.intersections {
            commands
            .spawn_bundle(GeometryBuilder::build_as(
                &shape,
                ShapeColors::outlined(Color::TEAL, Color::BLACK),
                DrawMode::Outlined {
                    fill_options: FillOptions::default(),
                    outline_options: StrokeOptions::default().with_line_width(10.0),
                },
                Transform::from_translation(Vec3::new(intersection.position.x, intersection.position.y, 2.0)),
            ));
            //.with(Intersection{
            //    position: node.position
            //});
        }        
    }
}