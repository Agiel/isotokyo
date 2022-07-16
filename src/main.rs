use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::{prelude::*, render::render_resource::TextureUsages};
// use rand::{thread_rng, Rng};

const MAP_SIZE: i32 = 128;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Isotokyo".into(),
            width: 960.,
            height: 540.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.125, 0.125, 0.125)))
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .add_startup_system(generate_map)
        .add_startup_system(setup_player)
        .add_system(set_texture_filters_to_nearest)
        .add_system(print_mesh_count)
        .add_system(update_crosshair)
        .add_system(look_at_crosshair.after(update_crosshair))
        .add_system_to_stage(CoreStage::Last, animate_sprites)
        .add_system_to_stage(CoreStage::Last, align_billboards.after(animate_sprites))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Animation {
    frame: u8,
    length: u8,
    rotates: bool,
    speed: f32,
    next_frame: f64,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            frame: 0,
            length: 1,
            speed: 0.0,
            rotates: true,
            next_frame: 0.0,
        }
    }
}

#[derive(Component)]
struct Billboard;

#[derive(Component)]
struct Crosshair;

fn update_crosshair(
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut query: Query<&mut Transform, (With<Crosshair>, Without<Camera>)>,
    cam_query: Query<(&Camera, &Transform)>,
) {
    let (camera, camera_transform) = cam_query.single();
    let mut crosshair_transform = query.single_mut();

    if let Some(ray) = Ray3d::from_screenspace(&windows, &images, &camera, &camera_transform) {
        if let Some(aim_point) = ray.intersect_y_plane(0.5) {
            crosshair_transform.translation = aim_point;
        }
    }
}

fn look_at_crosshair(
    mut query: Query<&mut Transform, With<Player>>,
    crosshair_query: Query<&Transform, (With<Crosshair>, Without<Player>)>,
) {
    let mut transform = query.single_mut();
    let crosshair_transform = crosshair_query.single();

    transform.look_at(crosshair_transform.translation, Vec3::Y);
}

fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // set up the camera
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 540.0 / 2.0 / 64.0;
    camera.transform = Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    // camera
    commands.spawn_bundle(camera);

    let texture_handle = asset_server.load("textures/props/sakura1.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mesh_handle = meshes.add(Mesh::from(shape::Quad {
        size: Vec2::new(1.5, 2.0),
        ..default()
    }));

    // props
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(1.5, 1.0, 1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(1.5, 1.0, -1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands
        .spawn_bundle(PbrBundle {
            mesh: mesh_handle.clone(),
            material: material_handle.clone(),
            transform: Transform::from_xyz(-1.5, 1.0, 1.5),
            ..Default::default()
        })
        .insert(Billboard);
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(-1.5, 0.5, -1.5),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 0.25,
            depth: 0.5,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.0, 0.7, 0.0).into()),
        transform: Transform::from_xyz(0., 0.5, 0.),
        ..Default::default()
    });
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Player
    let texture_handle = asset_server.load("textures/player/jinrai_walk.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let mut mesh = Mesh::from(shape::Quad {
        size: Vec2::new(1.0, 1.0),
        ..default()
    });
    let mut uvs = Vec::new();
    uvs.push([0.0, 0.125]);
    uvs.push([0.0, 0.0]);
    uvs.push([1.0, 0.0]);
    uvs.push([1.0, 0.125]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    let mesh_handle = meshes.add(mesh);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(4.0, 0.5, 4.0),
            ..default()
        })
        .insert(Player)
        // .insert(Wireframe)
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: mesh_handle,
                    material: material_handle,
                    ..default()
                })
                .insert(Billboard)
                .insert(Animation {
                    length: 8,
                    speed: 0.1,
                    rotates: true,
                    ..default()
                });
        });

    // Crosshair
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.05,
                ..default()
            })),
            material: materials.add(Color::WHITE.into()),
            ..default()
        })
        .insert(Crosshair);
}

fn generate_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture_handle = asset_server.load("textures/tiles/grass1.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mesh_handle = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    // plane
    for x in -MAP_SIZE..MAP_SIZE {
        for y in -MAP_SIZE..MAP_SIZE {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh_handle.clone(),
                material: material_handle.clone(),
                transform: Transform::from_xyz(x as f32, 0.0, y as f32),
                ..Default::default()
            });
        }
    }

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.05,
    });
}

fn animate_sprites(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Handle<Mesh>, &mut Animation, &GlobalTransform)>,
) {
    for (mesh_handle, mut animation, transform) in query.iter_mut() {
        if animation.speed > 0.0 && time.seconds_since_startup() > animation.next_frame {
            animation.frame = (animation.frame + 1) % animation.length;
            if animation.next_frame == 0.0 {
                animation.next_frame = time.seconds_since_startup();
            }
            animation.next_frame += animation.speed as f64
        }
 
        let mut frame = animation.frame;       
        if animation.rotates {
            let (direction, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            let direction = ((-direction + 3.0 * std::f32::consts::FRAC_PI_8 + std::f32::consts::TAU) / std::f32::consts::FRAC_PI_4) as u8 % 8;
            frame = frame + direction * animation.length;
        }
    
        let offset_x = (frame % animation.length) as f32 * 0.125;
        let offset_y = (frame / animation.length) as f32 * 0.125;
        
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            let mut uvs = Vec::new();
            uvs.push([0.0 + offset_x, 0.125 + offset_y]);
            uvs.push([0.0 + offset_x, 0.0 + offset_y]);
            uvs.push([0.125 + offset_x, 0.0 + offset_y]);
            uvs.push([0.125 + offset_x, 0.125 + offset_y]);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        }        
    }
}

fn align_billboards(
    mut query: Query<&mut GlobalTransform, (With<Billboard>, Without<Camera>)>,
    cam_query: Query<&GlobalTransform, With<Camera>>,
) {
    let cam_transform = cam_query.single();
    for mut transform in query.iter_mut() {
        let translation = transform.translation;
        transform.look_at(translation + cam_transform.forward(), Vec3::Y);
    }
}

fn print_mesh_count(
    time: Res<Time>,
    mut timer: Local<PrintingTimer>,
    sprites: Query<(&Handle<Mesh>, &ComputedVisibility)>,
) {
    timer.tick(time.delta());

    if timer.just_finished() {
        info!(
            "Meshes: {} - Visible Meshes {}",
            sprites.iter().len(),
            sprites.iter().filter(|(_, cv)| cv.is_visible).count(),
        );
    }
}

#[derive(Deref, DerefMut)]
struct PrintingTimer(Timer);

impl Default for PrintingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, true))
    }
}

/// A 3D ray, with an origin and direction. The direction is guaranteed to be normalized.
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Ray3d {
    pub(crate) origin: Vec3,
    pub(crate) direction: Vec3,
}

impl Ray3d {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray3d { origin, direction }
    }

    pub fn from_screenspace(
        windows: &Res<Windows>,
        images: &Res<Assets<Image>>,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> Option<Self> {
        let view = camera_transform.compute_matrix();
        let screen_size = match camera.target.get_logical_size(windows, images) {
            Some(s) => s,
            None => {
                error!(
                    "Unable to get screen size for RenderTarget {:?}",
                    camera.target
                );
                return None;
            }
        };

        let window = windows.get_primary().unwrap();
        let cursor_position = match window.cursor_position() {
            Some(c) => c,
            None => return None,
        };

        let projection = camera.projection_matrix;

        // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
        let cursor_ndc = (cursor_position / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
        let ndc_to_world: Mat4 = view * projection.inverse();
        let world_to_ndc = projection * view;
        let is_orthographic = projection.w_axis[3] == 1.0;

        // Compute the cursor position at the near plane. The bevy camera looks at -Z.
        let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera.near).z;
        let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

        // Compute the ray's direction depending on the projection used.
        let ray_direction = match is_orthographic {
            true => view.transform_vector3(-Vec3::Z), // All screenspace rays are parallel in ortho
            false => cursor_pos_near - camera_transform.translation, // Direction from camera to cursor
        };

        Some(Ray3d::new(cursor_pos_near, ray_direction))
    }

    pub fn intersect_y_plane(&self, y_offset: f32) -> Option<Vec3> {
        let plane_normal = Vec3::Y;
        let plane_origin = Vec3::new(0.0, y_offset, 0.0);
        let denominator = self.direction.dot(plane_normal);
        if denominator.abs() > f32::EPSILON {
            let point_to_point = plane_origin - self.origin;
            let intersect_dist = plane_normal.dot(point_to_point) / denominator;
            let intersect_position = self.direction * intersect_dist + self.origin;
            Some(intersect_position)
        } else {
            None
        }
    }
}
