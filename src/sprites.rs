use bevy::{prelude::*, render::render_resource::TextureUsages};
use bevy_rapier3d::prelude::*;

pub struct Sprite3dPlugin;

impl Plugin for Sprite3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_texture_filters_to_nearest)
            .add_system_to_stage(CoreStage::Last, animate_sprites)
            .add_system_to_stage(CoreStage::Last, align_billboards.after(animate_sprites))
            .add_system_to_stage(CoreStage::Last, project_blob_shadows);
    }
}

#[derive(Component)]
pub struct Animation {
    length: u8,
    speed: f32,
    rotates: bool,
    frame: u8,
    next_frame: f64,
}

impl Animation {
    pub fn new(length: u8, speed: f32, rotates: bool) -> Self {
        Animation {
            length,
            speed,
            rotates,
            ..default()
        }
    }
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
pub struct Billboard;

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
            let direction =
                ((-direction + 3.0 * std::f32::consts::FRAC_PI_8 + std::f32::consts::TAU)
                    / std::f32::consts::FRAC_PI_4) as u8
                    % 8;
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

#[derive(Component)]
pub struct BlobShadow;

fn project_blob_shadows(
    physics_context: Res<RapierContext>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut GlobalTransform, &Handle<StandardMaterial>), With<BlobShadow>>,
) {
    for (mut transform, material_handle) in query.iter_mut() {
        if !transform.is_changed() {
            continue;
        }
        if let Some((_entity, toi)) = physics_context.cast_ray(
            transform.translation,
            -Vec3::Y,
            1.0,
            true,
            QueryFilter::new().groups(InteractionGroups::new(0b0001, 0b0001)),
        ) {
            transform.translation.y -= toi;
            // Offset towards camera to avoid clipping through ground
            transform.translation += Vec3::ONE * 0.01;
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = Color::rgba(0.0, 0.0, 0.0, 1.0 - toi);
            }
        }
    }
}
