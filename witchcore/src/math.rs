//! Matematica de camera: projecao mundo -> tela, raio contra caixa. Sem dependencias.

pub type V3 = [f32; 3];

/// Projeta um ponto (relativo a camera, em blocos) para pixels de GUI.
/// `view` e `proj` sao as matrizes do `NativeCamera` (column-major). None se estiver atras da camera.
pub fn project(view: &[f32; 16], proj: &[f32; 16], rel: V3, screen: [f32; 2]) -> Option<[f32; 2]> {
    let v = mul_vec(view, [rel[0], rel[1], rel[2], 1.0]);
    let c = mul_vec(proj, v);
    if c[3] <= 1e-4 {
        return None;
    }
    let (nx, ny) = (c[0] / c[3], c[1] / c[3]);
    if !nx.is_finite() || !ny.is_finite() || nx.abs() > 4.0 || ny.abs() > 4.0 {
        return None;
    }
    Some([(nx * 0.5 + 0.5) * screen[0], (1.0 - (ny * 0.5 + 0.5)) * screen[1]])
}

fn mul_vec(m: &[f32; 16], v: [f32; 4]) -> [f32; 4] {
    let mut o = [0.0; 4];
    for (r, out) in o.iter_mut().enumerate() {
        *out = m[r] * v[0] + m[4 + r] * v[1] + m[8 + r] * v[2] + m[12 + r] * v[3];
    }
    o
}

/// Direcao do olhar (Minecraft: yaw 0 olha para +Z, pitch positivo olha para baixo).
pub fn look_dir(yaw_deg: f32, pitch_deg: f32) -> V3 {
    let (y, p) = (yaw_deg.to_radians(), pitch_deg.to_radians());
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}

/// Distancia ate onde o raio entra na caixa, ou None.
pub fn ray_aabb(origin: V3, dir: V3, min: V3, max: V3) -> Option<f32> {
    let (mut t0, mut t1) = (0.0_f32, f32::MAX);
    for i in 0..3 {
        if dir[i].abs() < 1e-8 {
            if origin[i] < min[i] || origin[i] > max[i] {
                return None;
            }
        } else {
            let (a, b) = ((min[i] - origin[i]) / dir[i], (max[i] - origin[i]) / dir[i]);
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            t0 = t0.max(lo);
            t1 = t1.min(hi);
            if t0 > t1 {
                return None;
            }
        }
    }
    Some(t0)
}

/// Um segmento vira um quadrilatero fino (a API de HUD so desenha retangulos e quadrilateros).
pub fn line_quad(a: [f32; 2], b: [f32; 2], thickness: f32) -> [[f32; 2]; 4] {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l = (dx * dx + dy * dy).sqrt().max(1e-4);
    let (nx, ny) = (-dy / l * thickness * 0.5, dx / l * thickness * 0.5);
    [[a[0] + nx, a[1] + ny], [b[0] + nx, b[1] + ny], [b[0] - nx, b[1] - ny], [a[0] - nx, a[1] - ny]]
}

/// As 12 arestas de uma caixa.
pub fn box_edges(min: V3, max: V3) -> [(V3, V3); 12] {
    let c = |x: usize, y: usize, z: usize| [[min[0], max[0]][x], [min[1], max[1]][y], [min[2], max[2]][z]];
    [
        (c(0, 0, 0), c(1, 0, 0)), (c(0, 0, 1), c(1, 0, 1)), (c(0, 1, 0), c(1, 1, 0)), (c(0, 1, 1), c(1, 1, 1)),
        (c(0, 0, 0), c(0, 1, 0)), (c(1, 0, 0), c(1, 1, 0)), (c(0, 0, 1), c(0, 1, 1)), (c(1, 0, 1), c(1, 1, 1)),
        (c(0, 0, 0), c(0, 0, 1)), (c(1, 0, 0), c(1, 0, 1)), (c(0, 1, 0), c(0, 1, 1)), (c(1, 1, 0), c(1, 1, 1)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: [f32; 16] = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];

    #[test]
    fn look_dir_cardinals() {
        let d = look_dir(0.0, 0.0);
        assert!((d[2] - 1.0).abs() < 1e-6 && d[0].abs() < 1e-6);
        let d = look_dir(90.0, 0.0);
        assert!((d[0] + 1.0).abs() < 1e-6);
        let d = look_dir(0.0, 90.0);
        assert!((d[1] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn ray_hits_and_misses_box() {
        let hit = ray_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [-0.5, -0.5, 3.0], [0.5, 0.5, 4.0]);
        assert!((hit.unwrap() - 3.0).abs() < 1e-5);
        assert!(ray_aabb([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [2.0, -0.5, 3.0], [3.0, 0.5, 4.0]).is_none());
        assert!(ray_aabb([0.0, 0.0, 0.0], [0.0, 0.0, -1.0], [-0.5, -0.5, 3.0], [0.5, 0.5, 4.0]).is_none());
    }

    #[test]
    fn project_behind_camera_is_none() {
        // projecao de teste: w = -z (como uma perspectiva olhando para -Z)
        let mut p = ID;
        p[11] = -1.0;
        p[15] = 0.0;
        assert!(project(&ID, &p, [0.0, 0.0, 5.0], [100.0, 100.0]).is_none());
        let c = project(&ID, &p, [0.0, 0.0, -5.0], [100.0, 100.0]).unwrap();
        assert!((c[0] - 50.0).abs() < 1e-3 && (c[1] - 50.0).abs() < 1e-3);
    }

    #[test]
    fn line_quad_has_thickness() {
        let q = line_quad([0.0, 0.0], [10.0, 0.0], 2.0);
        assert!((q[0][1] - q[3][1]).abs() - 2.0 < 1e-5);
    }

    #[test]
    fn box_has_12_distinct_edges() {
        let e = box_edges([0.0; 3], [1.0; 3]);
        for (a, b) in e {
            let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
            assert!((d - 1.0).abs() < 1e-6);
        }
    }
}
