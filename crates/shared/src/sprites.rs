use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, BoxedFuture, LoadContext},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    utils::HashMap,
};
use bevy_xpbd_3d::plugins::spatial_query::{SpatialQuery, SpatialQueryFilter};
use serde::{Deserialize, Serialize};

use crate::{physics::Layer, MainCamera};

pub struct Sprite3dPlugin;

impl Plugin for Sprite3dPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(AnimationSetLoader)
            .init_asset::<AnimationSet>()
            .init_asset_loader::<AnimationSetLoader>()
            .add_systems(
                PostUpdate,
                (check_sequence, rotate_sprites, animate_sprites).chain(),
            )
            .add_systems(Last, (align_billboards, project_blob_shadows));
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
    direction: u8,
    next_frame: f64,
}

impl Animator {
    pub fn new(animation_handle: Handle<AnimationSet>) -> Self {
        Self {
            animation_handle,
            frame: 0,
            direction: 0,
            next_frame: 0.0,
        }
    }
}

#[derive(Component, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Sequence {
    None,
    Idle,
    Walk,
    Jump,
}

#[derive(Asset, Deref, DerefMut, Serialize, Deserialize, TypeUuid, TypePath)]
#[uuid = "2b1255e1-6bb8-4295-93ee-6be7ebe405d0"]
pub struct AnimationSet(HashMap<Sequence, Animation>);

#[derive(Default)]
pub struct AnimationSetLoader;

impl AssetLoader for AnimationSetLoader {
    type Asset = AnimationSet;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let animation_set = AnimationSet(ron::de::from_bytes(&bytes)?);
            Ok(animation_set)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim"]
    }
}

#[derive(Component)]
pub struct Billboard;

fn check_sequence(
    animation_sets: Res<Assets<AnimationSet>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Animator, &mut Sequence, &Handle<StandardMaterial>), Changed<Sequence>>,
) {
    for (mut animator, mut sequence, material_handle) in &mut query {
        if let Some(animation_set) = animation_sets.get(&animator.animation_handle) {
            if !animation_set.contains_key(sequence.as_ref()) {
                *sequence = Sequence::Idle;
            }
            animator.frame = 0;
            animator.next_frame = 0.0;
            if let Some(material) = materials.get_mut(material_handle) {
                let animation = animation_set.get(sequence.as_ref()).unwrap();
                material.base_color_texture = Some(asset_server.load(&animation.texture));
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

fn rotate_sprites(
    animation_sets: Res<Assets<AnimationSet>>,
    mut query: Query<(&mut Animator, &Sequence, &Parent)>,
    p_query: Query<&Transform, Changed<Transform>>,
) {
    for (mut animator, sequence, parent) in query.iter_mut() {
        if let (Some(animation), Ok(transform)) = (
            get_animation(&animation_sets, &animator.animation_handle, sequence),
            p_query.get(parent.get()),
        ) {
            animator.direction = if animation.rotates {
                let (direction, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
                ((-direction + 3.0 * std::f32::consts::FRAC_PI_8 + std::f32::consts::TAU)
                    / std::f32::consts::FRAC_PI_4) as u8
                    % 8
            } else {
                0
            }
        }
    }
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
    )>,
) {
    for (mesh_handle, material_handle, mut animator, sequence) in query.iter_mut() {
        if let Some(animation) =
            get_animation(&animation_sets, &animator.animation_handle, sequence)
        {
            if animation.speed > 0.0 && time.elapsed_seconds_f64() > animator.next_frame {
                if animator.next_frame == 0.0 {
                    animator.next_frame = time.elapsed_seconds_f64();
                } else {
                    animator.frame = (animator.frame + 1) % animation.length;
                }
                animator.next_frame += animation.speed as f64
            }

            let frame = animator.frame + animator.direction * animation.length;

            if let Some(texture) = get_texture(&materials, material_handle, &textures) {
                let texture_size = texture.size();
                let size_x = animation.size.0 / texture_size.x as f32;
                let size_y = animation.size.1 / texture_size.y as f32;
                let offset_x = (frame % animation.length) as f32 * size_x;
                let offset_y = (frame / animation.length) as f32 * size_y;
                // info!("frame: {}, size_x: {}, size_y: {}", frame, size_x, size_y);

                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    let uvs = vec![
                        [0.0 + offset_x, size_y + offset_y],
                        [0.0 + offset_x, 0.0 + offset_y],
                        [size_x + offset_x, 0.0 + offset_y],
                        [size_x + offset_x, size_y + offset_y],
                    ];
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
        let translation = transform.translation();
        *transform = GlobalTransform::from(
            Transform::from_translation(translation)
                .looking_at(translation + cam_transform.forward(), Vec3::Y),
        );
    }
}

#[derive(Component)]
pub struct BlobShadow;

fn project_blob_shadows(
    spatial_query: SpatialQuery,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut GlobalTransform, &Handle<StandardMaterial>), With<BlobShadow>>,
) {
    for (mut transform, material_handle) in query.iter_mut() {
        if !transform.is_changed() {
            continue;
        }
        if let Some(hit) = spatial_query.cast_ray(
            transform.translation(),
            -Vec3::Y,
            1.0,
            true,
            SpatialQueryFilter::new().with_masks([Layer::Ground]),
        ) {
            let mut translation = transform.translation();
            translation.y -= hit.time_of_impact;
            // Offset towards camera to avoid clipping through ground
            translation += Vec3::ONE * 0.01;
            *transform = GlobalTransform::from(Transform::from_translation(translation));
            if let Some(material) = materials.get_mut(material_handle) {
                material.base_color = Color::rgba(0.0, 0.0, 0.0, 1.0 - hit.time_of_impact);
            }
        }
    }
}
