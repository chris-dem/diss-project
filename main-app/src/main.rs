use bevy::prelude::*;

fn hello_world() {
    println!("Hello world!");
}

fn main() {
    App::new().add_systems(Update, hello_world).run();
}
