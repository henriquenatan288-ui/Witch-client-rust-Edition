//! Trigger Bot: ataca quando a mira esta em cima de um alvo, no ritmo do cooldown da arma.

use crate::math::{V3, ray_aabb};

#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub id: i32,
    pub min: V3,
    pub max: V3,
}

/// O alvo mais perto ao longo do raio do olhar, dentro de `reach`.
pub fn pick(eye: V3, dir: V3, targets: &[Target], reach: f32) -> Option<i32> {
    targets
        .iter()
        .filter_map(|t| ray_aabb(eye, dir, t.min, t.max).filter(|d| *d <= reach).map(|d| (d, t.id)))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

/// Espera `cooldown` ticks entre ataques (0.9s de uma espada ~ 12 ticks) mais um pequeno atraso.
#[derive(Default, Debug)]
pub struct Timer {
    since: u32,
}

impl Timer {
    pub fn tick(&mut self, aiming: bool, cooldown: u32) -> bool {
        self.since = self.since.saturating_add(1);
        if aiming && self.since >= cooldown {
            self.since = 0;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: i32, z: f32) -> Target {
        Target { id, min: [-0.3, 0.0, z], max: [0.3, 1.8, z + 0.6] }
    }

    #[test]
    fn picks_nearest_in_reach() {
        let ts = [t(1, 5.0), t(2, 2.0)];
        assert_eq!(pick([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], &ts, 3.0), Some(2));
    }

    #[test]
    fn nothing_beyond_reach_or_off_aim() {
        assert_eq!(pick([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], &[t(1, 5.0)], 3.0), None);
        assert_eq!(pick([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], &[t(1, 2.0)], 3.0), None);
    }

    #[test]
    fn timer_respects_cooldown() {
        let mut tm = Timer::default();
        let hits = (0..24).filter(|_| tm.tick(true, 12)).count();
        assert_eq!(hits, 2);
        let mut tm = Timer::default();
        assert_eq!((0..24).filter(|_| tm.tick(false, 12)).count(), 0);
    }
}
