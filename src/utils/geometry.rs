use nalgebra::Vector3;

pub fn compute_normals(
    positions: &Vec<Vector3<f32>>,
    triangles: &Vec<(u32, u32, u32)>,
) -> Vec<Vector3<f32>> {
    let mut normals: Vec<Vector3<f32>> =
        positions.iter().map(|_| Vector3::<f32>::zeros()).collect();
    for &(i, j, k) in triangles {
        let i = i as usize;
        let j = j as usize;
        let k = k as usize;
        let e1 = positions[j] - positions[i];
        let e2 = positions[k] - positions[i];
        let face_normal = e1.cross(&e2);
        normals[i] += face_normal;
        normals[j] += face_normal;
        normals[k] += face_normal;
    }
    for v in normals.iter_mut() {
        v.normalize_mut();
    }
    normals
}
