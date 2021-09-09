use bevy::prelude::*;
use bevy_svg::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "complex_one_color".to_string(),
            width: 400.0,
            height: 400.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_svg::SvgPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(SvgBundle::build(SvgBundleConfig {
        svg:      asset_server.load("asteroid_field.svg"),
        position: Vec3::new(0.0, 0.0, 0.0),
        scale:    Vec2::new(3.0, 3.0),
    }));
}
