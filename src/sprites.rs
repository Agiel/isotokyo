use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::TextureUsages,
    utils::HashMap,
};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::MainCamera;

pub struct Sprite3dPlugin;

impl Plugin for Sprite3dPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset_loader::<AnimationSetLoader>()
            .add_asset::<AnimationSet>()
            .add_system(set_texture_filters_to_nearest)
            .add_system_to_stage(CoreStage::PostUpdate, check_sequence)
            .add_system_to_stage(CoreStage::PostUpdate, animate_sprites.after(check_sequence))
            .add_system_to_stage(CoreStage::Last, align_billboards)
            .add_system_to_stage(CoreStage::Last, project_blob_shadows);
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Animation {
    texture: String,
    offset: (f32, f32),
    size: (f32, f32),
    length: u8,
    speed: f32,
    rotates: bool,
}

#[derive(Component)]
pub struct Animator {
    animation_handle: Handle<AnimationSet>,
    frame: u8,
    next_frame: f64,
}

impl Animator {
    pub fn new(animation_handle: Handle<AnimationSet>) -> Self {
        Self {
            animation_handle,
            frame: 0,
            next_frame: 0.0,
        }
    }
}

#[derive(Component, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sequence {
    Idle,
    Walk,
    Jump,
}

#[derive(Deref, DerefMut, Serialize, Deserialize, TypeUuid)]
#[uuid = "2b1255e1-6bb8-4295-93ee-6be7ebe405d0"]
pub struct AnimationSet(HashMap<Sequence, Animation>);

#[derive(Default)]
pub struct AnimationSetLoader;

impl AssetLoader for AnimationSetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let animation_set = AnimationSet(ron::de::from_bytes(bytes)?);
            load_context.set_default_asset(LoadedAsset::new(animation_set));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
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

fn check_sequence(
    animation_sets: Res<Assets<AnimationSet>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Animator, &mut Sequence, &Handle<StandardMaterial>), Changed<Sequence>>,
) {
    for (mut animator, mut sequence, material_handle) in query.iter_mut() {
        if let Some(animation_set) = animation_sets.get(&animator.animation_handle) {
            if !animation_set.contains_key(&sequence) {
                *sequence = Sequence::Idle;
            }
            animator.frame = 0;
            animator.next_frame = 0.0;
            if let Some(mut material) = materials.get_mut(material_handle) {
                let animation = animation_set.get(&sequence).unwrap();
                material.base_color_texture = Some(asset_server.load(animation.texture.as_str()));
            }
        }
    }
}

fn get_animation<'a>(
    animation_sets: &'a Res<Assets<AnimationSet>>,
    animation_handle: &Handle<AnimationSet>,
    sequence: &Sequence,
) -> Option<&'a Animation> {
    animation_sets.get(animation_handle)?.get(sequence)
}

fn get_texture<'a>(
    materials: &'a Res<Assets<StandardMaterial>>,
    material_handle: &Handle<StandardMaterial>,
    textures: &'a Res<Assets<Image>>,
) -> Option<&'a Image> {
    let texture_handle = materials
        .get(material_handle)?
        .base_color_texture
        .as_ref()?;
    textures.get(texture_handle)
}

fn animate_sprites(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    animation_sets: Res<Assets<AnimationSet>>,
    materials: Res<Assets<StandardMaterial>>,
    textures: Res<Assets<Image>>,
    mut query: Query<(
        &Handle<Mesh>,
        &Handle<StandardMaterial>,
        &mut Animator,
        &Sequence,
        &GlobalTransform,
    )>,
) {
    for (mesh_handle, material_handle, mut animator, sequence, transform) in query.iter_mut() {
        if let Some(animation) =
            get_animation(&animation_sets, &animator.animation_handle, &sequence)
        {
            if animation.speed > 0.0 && time.seconds_since_startup() > animator.next_frame {
                if animator.next_frame == 0.0 {
                    animator.next_frame = time.seconds_since_startup();
                } else {
                    animator.frame = (animator.frame + 1) % animation.length;
                }
                animator.next_frame += animation.speed as f64
            }

            let mut frame = animator.frame;
            if animation.rotates {
                let (direction, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
                let direction =
                    ((-direction + 3.0 * std::f32::consts::FRAC_PI_8 + std::f32::consts::TAU)
                        / std::f32::consts::FRAC_PI_4) as u8
                        % 8;
                frame = frame + direction * animation.length;
            }

            if let Some(texture) = get_texture(&materials, &material_handle, &textures) {
                let texture_size = texture.size();
                let size_x = animation.size.0 / texture_size.x;
                let size_y = animation.size.1 / texture_size.y;
                let offset_x = (frame % animation.length) as f32 * size_x;
                let offset_y = (frame / animation.length) as f32 * size_y;
                // info!("frame: {}, size_x: {}, size_y: {}", frame, size_x, size_y);

                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    let mut uvs = Vec::new();
                    uvs.push([0.0 + offset_x, size_y + offset_y]);
                    uvs.push([0.0 + offset_x, 0.0 + offset_y]);
                    uvs.push([size_x + offset_x, 0.0 + offset_y]);
                    uvs.push([size_x + offset_x, size_y + offset_y]);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                }
            } else {
                info!("Texture not loaded");
            }
        }
    }
}

fn align_billboards(
    mut query: Query<&mut GlobalTransform, (With<Billboard>, Without<MainCamera>)>,
    cam_query: Query<&GlobalTransform, With<MainCamera>>,
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
