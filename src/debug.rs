use crate::terrain::ChunkGenerator;
use bevy::diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::core::Timer;
use std::time::Duration;

pub struct DebugPlugin {
    pub wait_duration: Duration,
}

pub struct DebugState {
    timer: Timer,
    message: String,
}

impl Default for DebugPlugin {
    fn default() -> Self {
        DebugPlugin {
            wait_duration: Duration::from_secs(1),
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_resource(DebugState {
                timer: Timer::new(self.wait_duration, true),
                message: String::from(""),
            });

        app.add_startup_system(Self::setup_debug_ui.system())
            .add_system(Self::update_debug.system())
            .add_system_to_stage(stage::POST_UPDATE, Self::print_diagnostics_system.system());
    }
}

impl DebugPlugin {
    pub fn setup_debug_ui(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
        commands
            // ui camera
            .spawn(UiCameraComponents::default())
            // root node
            .spawn(NodeComponents {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    justify_content: JustifyContent::FlexEnd,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|parent| {
                parent
                    // right vertical fill
                    .spawn(NodeComponents {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Px(100.0)),
                            position_type: PositionType::Absolute,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        material: materials.add(Color::rgba(0.08, 0.06, 0.07, 0.4).into()),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextComponents {
                            style: Style {
                                margin: Rect::all(Val::Px(5.0)),
                                ..Default::default()
                            },
                            text: Text {
                                value: "Text Example".to_string(),
                                font: asset_server.load("assets/fonts/FiraSans-Bold.ttf").unwrap(),
                                style: TextStyle {
                                    font_size: 30.0,
                                    color: Color::WHITE,
                                },
                            },
                            ..Default::default()
                        });
                    });
            });
    }

    fn update_debug(
        generator: Res<ChunkGenerator>,
        state: Res<DebugState>,
        mut query: Query<&mut Text>,
    ) {
        for mut text in &mut query.iter() {
            text.value = format!(
                "UV Scale: {}\nScale: {}\n Bias: {}. {}",
                generator.uscale, generator.scale, generator.bias, state.message
            );
        }
    }

    fn print_diagnostic(diagnostic: &Diagnostic) -> String {
        if let Some(value) = diagnostic.value() {
            if let Some(average) = diagnostic.average() {
                format!("{}: {} (avg {:.6})", diagnostic.name, value, average)
            } else {
                format!("{}: {}", diagnostic.name, value)
            }
        } else {
            format!("{}: No value", diagnostic.name)
        }
    }

    pub fn print_diagnostics_system(
        mut state: ResMut<DebugState>,
        time: Res<Time>,
        diagnostics: Res<Diagnostics>,
    ) {
        state.timer.tick(time.delta_seconds);
        if state.timer.finished {
            state.message = diagnostics
                .iter()
                .map(|diagnostic| Self::print_diagnostic(diagnostic))
                .collect::<Vec<_>>()
                .join(";");

            state.timer.reset();
        }
    }
}
