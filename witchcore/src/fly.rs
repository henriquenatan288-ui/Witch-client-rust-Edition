//! Fly por empurroes: o Horizon so deixa o client SOMAR velocidade (`native_push_player`) e nao le a
//! velocidade, entao estimamos pela diferenca de posicao e empurramos ate a velocidade desejada.

use crate::math::V3;

#[derive(Clone, Copy, Default, Debug)]
pub struct FlyInput {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub yaw_deg: f32,
}

const GRAVITY: f32 = 0.08;
const MAX_STEP: f32 = 0.35;

/// Empurrao a aplicar neste tick. `vel` e a velocidade estimada (blocos/tick), `speed` a desejada.
pub fn push(inp: &FlyInput, vel: V3, speed: f32) -> V3 {
    let f = (inp.forward as i8 - inp.back as i8) as f32;
    let s = (inp.right as i8 - inp.left as i8) as f32;
    let y = inp.yaw_deg.to_radians();
    let (mut wx, mut wz) = (-y.sin() * f - y.cos() * s, y.cos() * f - y.sin() * s);
    let l = (wx * wx + wz * wz).sqrt();
    if l > 1e-6 {
        wx = wx / l * speed;
        wz = wz / l * speed;
    }
    let wy = ((inp.up as i8 - inp.down as i8) as f32) * speed;
    let clamp = |v: f32| v.clamp(-MAX_STEP, MAX_STEP);
    [clamp(wx - vel[0]), clamp(wy - vel[1] + GRAVITY), clamp(wz - vel[2])]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regra de movimento do jogo no ar: soma o empurrao, gravidade e arrasto.
    fn step(v: V3, p: V3) -> V3 {
        let (x, y, z) = (v[0] + p[0], v[1] + p[1], v[2] + p[2]);
        [x * 0.91, (y - GRAVITY) * 0.98, z * 0.91]
    }

    #[test]
    fn hovers_without_input() {
        let mut v = [0.0, -0.5, 0.0];
        for _ in 0..40 {
            v = step(v, push(&FlyInput::default(), v, 0.5));
        }
        assert!(v[1].abs() < 0.1, "vy = {}", v[1]);
    }

    #[test]
    fn climbs_with_up() {
        let mut v = [0.0; 3];
        let inp = FlyInput { up: true, ..Default::default() };
        for _ in 0..40 {
            v = step(v, push(&inp, v, 0.5));
        }
        assert!(v[1] > 0.25, "vy = {}", v[1]);
    }

    #[test]
    fn moves_forward_along_yaw() {
        let mut v = [0.0; 3];
        let inp = FlyInput { forward: true, yaw_deg: 90.0, ..Default::default() };
        for _ in 0..40 {
            v = step(v, push(&inp, v, 0.5));
        }
        assert!(v[0] < -0.2 && v[2].abs() < 0.1, "v = {v:?}");
    }
}
