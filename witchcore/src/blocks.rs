//! Block ESP: lista de blocos (grade) e varredura em volta do jogador.

use crate::math::V3;
use std::collections::BTreeSet;

/// (nome do bloco, rotulo curto, cor ARGB da caixa). Lista curada dos blocos que mais se procura.
pub const BLOCKS: &[(&str, &str, u32)] = &[
    ("minecraft:diamond_ore", "Dia", 0xFF4AEDD9),
    ("minecraft:deepslate_diamond_ore", "DDi", 0xFF4AEDD9),
    ("minecraft:ancient_debris", "Deb", 0xFF8B5A3C),
    ("minecraft:emerald_ore", "Eme", 0xFF17DD62),
    ("minecraft:deepslate_emerald_ore", "DEm", 0xFF17DD62),
    ("minecraft:gold_ore", "Gol", 0xFFFCEE4B),
    ("minecraft:deepslate_gold_ore", "DGo", 0xFFFCEE4B),
    ("minecraft:iron_ore", "Iro", 0xFFD8AF93),
    ("minecraft:deepslate_iron_ore", "DIr", 0xFFD8AF93),
    ("minecraft:copper_ore", "Cop", 0xFFE77C56),
    ("minecraft:deepslate_copper_ore", "DCo", 0xFFE77C56),
    ("minecraft:coal_ore", "Coa", 0xFF6B6B6B),
    ("minecraft:deepslate_coal_ore", "DCa", 0xFF6B6B6B),
    ("minecraft:redstone_ore", "Red", 0xFFFF2A2A),
    ("minecraft:deepslate_redstone_ore", "DRe", 0xFFFF2A2A),
    ("minecraft:lapis_ore", "Lap", 0xFF2A52E0),
    ("minecraft:deepslate_lapis_ore", "DLa", 0xFF2A52E0),
    ("minecraft:nether_gold_ore", "NGo", 0xFFFCEE4B),
    ("minecraft:nether_quartz_ore", "Qrz", 0xFFEDE6DA),
    ("minecraft:spawner", "Spw", 0xFFB06BFF),
    ("minecraft:chest", "Cht", 0xFFE0A030),
    ("minecraft:trapped_chest", "TCh", 0xFFFF6A00),
    ("minecraft:ender_chest", "ECh", 0xFF9A3CFF),
    ("minecraft:barrel", "Bar", 0xFFC08040),
    ("minecraft:shulker_box", "Shu", 0xFFC080FF),
    ("minecraft:obsidian", "Obs", 0xFF3A2A5A),
    ("minecraft:crying_obsidian", "COb", 0xFF7A2AD0),
    ("minecraft:respawn_anchor", "Anc", 0xFFB02AFF),
    ("minecraft:end_portal_frame", "EPF", 0xFF70FF9A),
    ("minecraft:tnt", "TNT", 0xFFFF3030),
    ("minecraft:bedrock", "Bed", 0xFF404040),
    ("minecraft:budding_amethyst", "Ame", 0xFFC79CFF),
];

pub fn category_of(name: &str) -> usize {
    if name.contains("_ore") || name == "minecraft:ancient_debris" || name == "minecraft:budding_amethyst" { 0 }
    else if matches!(name, "minecraft:spawner" | "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:ender_chest" | "minecraft:barrel" | "minecraft:shulker_box") { 1 }
    else if matches!(name, "minecraft:obsidian" | "minecraft:crying_obsidian" | "minecraft:bedrock" | "minecraft:end_portal_frame" | "minecraft:respawn_anchor") { 2 }
    else if name == "minecraft:tnt" { 3 } else { 4 }
}

pub fn color_of(name: &str) -> u32 {
    BLOCKS.iter().find(|b| b.0 == name).map(|b| b.2).unwrap_or(0xFFFFFFFF)
}

#[derive(Default, Clone, Debug)]
pub struct Selection {
    names: BTreeSet<&'static str>,
}

impl Selection {
    pub fn toggle(&mut self, name: &'static str) {
        if !self.names.remove(name) {
            self.names.insert(name);
        }
    }
    pub fn has(&self, name: &str) -> bool {
        self.names.contains(name)
    }
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Origem dos blocos (no jogo: os chunks do cliente). `name_at` devolve o nome ou None se ar/nao carregado.
pub trait World {
    fn name_at(&self, pos: [i32; 3]) -> Option<String>;
}

/// Varre um cubo de raio `r` em volta de `center`, devolvendo no maximo `cap` blocos selecionados.
pub fn scan(world: &dyn World, center: [i32; 3], r: i32, sel: &Selection, cap: usize) -> Vec<([i32; 3], u32)> {
    let mut out = Vec::new();
    if sel.is_empty() {
        return out;
    }
    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                let p = [center[0] + dx, center[1] + dy, center[2] + dz];
                if let Some(n) = world.name_at(p)
                    && sel.has(&n)
                {
                    out.push((p, color_of(&n)));
                    if out.len() >= cap {
                        return out;
                    }
                }
            }
        }
    }
    out
}

pub fn box_of(p: [i32; 3]) -> (V3, V3) {
    ([p[0] as f32, p[1] as f32, p[2] as f32], [p[0] as f32 + 1.0, p[1] as f32 + 1.0, p[2] as f32 + 1.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(Vec<([i32; 3], &'static str)>);
    impl World for Fake {
        fn name_at(&self, pos: [i32; 3]) -> Option<String> {
            self.0.iter().find(|(p, _)| *p == pos).map(|(_, n)| n.to_string())
        }
    }

    #[test]
    fn scan_finds_only_selected_in_radius() {
        let w = Fake(vec![([1, 0, 0], "minecraft:diamond_ore"), ([2, 0, 0], "minecraft:coal_ore"), ([9, 0, 0], "minecraft:diamond_ore")]);
        let mut s = Selection::default();
        s.toggle("minecraft:diamond_ore");
        let r = scan(&w, [0, 0, 0], 4, &s, 100);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].0, [1, 0, 0]);
    }

    #[test]
    fn empty_selection_scans_nothing_and_cap_holds() {
        let w = Fake(vec![([1, 0, 0], "minecraft:diamond_ore"), ([0, 1, 0], "minecraft:diamond_ore")]);
        assert!(scan(&w, [0; 3], 3, &Selection::default(), 10).is_empty());
        let mut s = Selection::default();
        s.toggle("minecraft:diamond_ore");
        assert_eq!(scan(&w, [0; 3], 3, &s, 1).len(), 1);
    }

    #[test]
    fn toggle_twice_unselects() {
        let mut s = Selection::default();
        s.toggle("minecraft:chest");
        assert!(s.has("minecraft:chest"));
        s.toggle("minecraft:chest");
        assert!(!s.has("minecraft:chest"));
    }

    #[test]
    fn every_block_has_unique_name_and_label() {
        let mut n = BTreeSet::new();
        let mut l = BTreeSet::new();
        for b in BLOCKS {
            assert!(n.insert(b.0) && l.insert(b.1), "{}", b.0);
        }
    }
}
