use nalgebra::{ Vector3};
use std::f32::consts::PI;

use crate::utils::{
    geometry::compute_normals,
    web::random_f32,
    webgl::{Geometry, VertexAttrInfo, VertexAttrs},
};

const BOX_SIZE: f32 = 500.0;
const R: f32 = BOX_SIZE / 10.0;
const RESOLUTION: i32 = 100;
const SLICES: i32 = 100;
const STEP: f32 = BOX_SIZE / (RESOLUTION as f32);

macro_rules! ind {
    ($i:expr, $j:expr) => {
        $i * RESOLUTION + $j
    };
}

pub fn generate_terrain() -> Geometry {
    let mut delta = BOX_SIZE / 10.0;

    let mut positions = Vec::<Vector3<f32>>::new();
    for i in 0..RESOLUTION {
        for j in 0..RESOLUTION {
            positions.push(Vector3::new(
                STEP * (i as f32) - BOX_SIZE * 0.5,
                STEP * (j as f32) - BOX_SIZE * 0.5,
                0.0,
            ))
        }
    }

    for _ in 0..SLICES {
        delta *= 0.99;
        let p = Vector3::new(
            random_f32() * BOX_SIZE - BOX_SIZE * 0.5,
            random_f32() * BOX_SIZE - BOX_SIZE * 0.5,
            0.0,
        );
        let theta = random_f32() * PI * 2.0;
        let normal = Vector3::new(f32::cos(theta), f32::sin(theta), 0.0);
        for v in positions.iter_mut() {
            let dir = *v - p;
            let r = f32::sqrt(normal.dot(&dir));
            let g = if r < R {
                (1.0 - (r / R).powi(2)).powi(2)
            } else {
                0.0
            };
            v.z += if r > 0.0 { delta * g } else { -delta * g };
        }
    }

    let z_max = positions.iter().map(|v| v.z).reduce(f32::max).unwrap();
    let z_min = positions.iter().map(|v| v.z).reduce(f32::min).unwrap();
    for v in positions.iter_mut() {
        v.z = (v.z - z_min) / (z_max - z_min) * BOX_SIZE * 0.5;
    }

    let mut triangles = Vec::<(u32, u32, u32)>::new();

    for i in 0..RESOLUTION - 1 {
        for j in 0..RESOLUTION - 1 {
            triangles.push((ind!(i, j) as u32, ind!(i + 1, j) as u32, ind!(i, j + 1) as u32));
            triangles.push((ind!(i, j + 1) as u32, ind!(i + 1, j) as u32, ind!(i + 1, j + 1) as u32));
        }
    }

    spheroidal_weather_mut(&mut positions);

    let normals = compute_normals(&positions, &triangles);
    Geometry {
        triangles: triangles
            .iter()
            .flat_map(|&s| vec![s.0, s.1, s.2])
            .collect(),
        attributes: VertexAttrs {
            position: VertexAttrInfo {
                glsl_name: String::from("position"),
                size: 3,
                data: positions.iter().flat_map(|p| vec![p.x, p.y, p.z]).collect(),
            },
            normal: Some(VertexAttrInfo {
                glsl_name: String::from("normal"),
                size: 3,
                data: normals.iter().flat_map(|p| vec![p.x, p.y, p.z]).collect(),
            }),
            texcoord: Some(VertexAttrInfo {
                glsl_name: String::from("texCoord"),
                size: 2,
                data: positions
                    .iter()
                    .flat_map(|p| {
                        vec![
                            (p.x + BOX_SIZE * 0.5) / BOX_SIZE,
                            (BOX_SIZE * 0.5 - p.y) / BOX_SIZE,
                        ]
                    })
                    .collect(),
            }),
        },
    }
}

fn spheroidal_weather_mut(positions: &mut Vec<Vector3<f32>>) {
    for _ in 0..5 {
        let mut average: Vec<Vector3<f32>> =
            positions.iter().map(|_| Vector3::<f32>::zeros()).collect();
        for i in 0..RESOLUTION {
            for j in 0..RESOLUTION {
                average[ind!(i, j) as usize] = 0.25
                    * (positions[ind!(i32::max(0, i - 1), j) as usize]
                        + positions[ind!(i32::min(RESOLUTION - 1, i + 1), j) as usize]
                        + positions[ind!(i, i32::max(0, j - 1)) as usize]
                        + positions[ind!(i, i32::min(RESOLUTION - 1, j + 1)) as usize]);
            }
        }
        for i in 0..RESOLUTION {
            for j in 0..RESOLUTION {
                let p = &mut positions[ind!(i, j) as usize];
                let a = &average[ind!(i, j) as usize];
                *p =  0.5 * (*p + *a);
            }
        }
    }
}