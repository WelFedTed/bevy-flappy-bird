use bevy::prelude::*;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, test);
    }
}

fn test() {
    println!("SCOREPLUGIN WORKING");
}
