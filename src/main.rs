use bevy::app::App;
use bevy::color::Color;
use bevy::DefaultPlugins;
use bevy::prelude::ClearColor;

const BACKGROUND_COLOR: Color = Color::BLACK;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .run();
}
