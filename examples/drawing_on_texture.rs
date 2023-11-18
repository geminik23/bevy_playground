use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

// constant
pub const WINDOW_SIZE: (f32, f32) = (480.0, 480.0);
pub const CANVAS_SIZE: (u32, u32) = (64, 64);

// TODO
// - [x] display the window
// - [x] create the texture
// - [x] mouse event : Interaction & MouseButton
// - [x] CanvasEvent
// - [x] drawing dot / line

// issue : scattered dots on canvas...
// - draw line [x]
// - lower the canvas_size [x]
// - big dots around the point ?

//
fn line_points(mut xy1: Vec2, mut xy2: Vec2) -> Vec<Vec2> {
    // check the slope
    let swap = (xy2.y - xy1.y).abs() > (xy2.x - xy1.x).abs();
    if swap {
        xy1 = xy1.yx();
        xy2 = xy2.yx();
    }

    if xy1.x > xy2.x {
        let temp = xy1;
        xy1 = xy2;
        xy2 = temp;
    }

    let mut result = Vec::new();

    let mut x = xy1.x;
    let mut y = xy1.y;
    let dx = xy2.x - xy1.x;
    let dy = xy2.y - xy1.y;
    let mut d = 2.0 * dy - dx; // discriminator

    while x <= xy2.x {
        result.push(if swap {
            Vec2::new(y, x)
        } else {
            Vec2::new(x, y)
        });
        x = x + 1.0;
        if d <= 0.0 {
            d = d + 2.0 * dy;
        } else {
            d = d + 2.0 * (dy - dx);
            y = y + 1.0;
        }
    }
    //
    result
}

#[derive(Event)]
enum CanvasEvent {
    DrawAt(Vec2, f32),
    Clear,
}

#[derive(Component)]
pub struct Canvas;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // camera settings
    let camera = Camera2dBundle::default();
    commands.spawn(camera);

    // texture to draw
    let image = Image::new_fill(
        Extent3d {
            width: CANVAS_SIZE.0,
            height: CANVAS_SIZE.1,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
    );
    // info!("{}", image.data.len()); // output : width*height*4 (bytes)

    let image = asset_server.add(image);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ImageBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    image: UiImage::new(image),
                    ..default()
                })
                .insert(Canvas)
                .insert(Interaction::None);
        });
}

fn on_canvas_event(
    mut event_reader: EventReader<CanvasEvent>,
    mut assets: ResMut<Assets<Image>>,
    mut canvas: Query<&UiImage, With<Canvas>>,
) {
    let image = canvas.get_single_mut().unwrap();
    let image = image.texture.clone();
    let image = assets.get_mut(image.id());
    if image.is_none() {
        return;
    }
    let image = image.unwrap();
    // image.data[

    for event in event_reader.read() {
        match event {
            CanvasEvent::DrawAt(pos, intensity) => {
                let intensity = if *intensity > 1.0 { 1.0 } else { *intensity };

                if pos.x < 0.0 || pos.x > 1.0 {
                    return;
                }
                if pos.y < 0.0 || pos.y > 1.0 {
                    return;
                }
                let x = (CANVAS_SIZE.0 as f32 * pos.x + 0.5) as u32;
                let y = (CANVAS_SIZE.1 as f32 * pos.y + 0.5) as u32;
                let x = if x > CANVAS_SIZE.0 - 1 {
                    CANVAS_SIZE.0
                } else {
                    x
                };
                let y = if y > CANVAS_SIZE.1 - 1 {
                    CANVAS_SIZE.1
                } else {
                    y
                };

                let offset = (y * CANVAS_SIZE.1 + x) as usize;
                let val = (255.0 * intensity) as u8;
                image.data[offset * 4] = val;
                image.data[offset * 4 + 1] = val;
                image.data[offset * 4 + 2] = val;
                image.data[offset * 4 + 3] = 255;
            }
            CanvasEvent::Clear => {
                for i in 0..image.data.len() / 4 {
                    image.data[i * 4] = 0;
                    image.data[i * 4 + 1] = 0;
                    image.data[i * 4 + 2] = 0;
                    image.data[i * 4 + 3] = 255;
                }
            }
        }
    }
}

fn draw_on_mouse_move(
    mut cursor_reader: EventReader<CursorMoved>,
    mut last_point: Local<Option<Vec2>>,
    mouse_button: Res<Input<MouseButton>>,
    mut event_writer: EventWriter<CanvasEvent>,
    canvas: Query<(&Node, &Interaction, &GlobalTransform), With<Canvas>>,
) {
    // Style for size, Interaction to detect drawing, GlobalTransform for global position.
    //
    if mouse_button.just_released(MouseButton::Left) {
        *last_point = None;
        // send the 'Clear' event
        event_writer.send(CanvasEvent::Clear);
    }
    for (node, interaction, transform) in canvas.iter() {
        match interaction {
            Interaction::Pressed => {
                for cursor in cursor_reader.read() {
                    let size = node.size();
                    let xy = cursor.position.xy();
                    let trans = transform.translation().xy();

                    // info!("Node size : {:?}", size);
                    // info!("Global Translation : {:?}", trans);
                    // info!("Move position : {:?}", xy);

                    let mut points = vec![];

                    if let Some(last_point) = *last_point {
                        points = line_points(last_point, xy);
                    }
                    *last_point = Some(Vec2::new(xy.x, xy.y));

                    // normalized position [0-1]
                    for xy in points.into_iter() {
                        let norm_x = (xy.x - trans.x + size.x / 2.0) / size.x;
                        let norm_y = (xy.y - trans.y + size.y / 2.0) / size.y;

                        event_writer.send(CanvasEvent::DrawAt(Vec2::new(norm_x, norm_y), 1.0));
                    }
                }
            }
            _ => {}
        }
    }
    //
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Drawing on canvas".into(),
                    resolution: WINDOW_SIZE.into(),
                    resizable: false,
                    ..Default::default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),))
        .add_event::<CanvasEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (on_canvas_event, draw_on_mouse_move))
        .run();
}
