use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

mod intersection;
mod street;
mod road_system;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 8 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands) {
    commands
    .spawn((Graph, road_system::RoadSystem::new()));
}