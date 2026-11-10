//! Nucleo do Witch (Rust Edition): ClickGUI sem nenhuma dependencia do Horizon.
//! Desenha por um `Canvas` e recebe mouse/scroll em pixels de GUI, entao da para testar com `cargo test`.

pub mod blocks;
pub mod fly;
pub mod math;
pub mod trigger;

pub const ACCENT: u32 = 0xFF087BFF;
const BG: u32 = 0xD900050D;
const PANEL: u32 = 0xE6090D16;
const ROW: u32 = 0xC9101724;
const ROW_HOVER: u32 = 0xE0182B45;
const OFF: u32 = 0xFF27364A;
const TEXT: u32 = 0xFFF1F6FF;
const DIM: u32 = 0xFF8294B0;

pub trait Canvas {
    fn fill(&mut self, x: f32, y: f32, w: f32, h: f32, argb: u32);
    fn text(&mut self, x: f32, y: f32, text: &str, rgb: u32, shadow: bool);
    fn text_width(&self, text: &str) -> f32;
    fn item_icon(&mut self, _name: &str, _x: f32, _y: f32) -> bool { false }
    /// Desenha a parte `src` da imagem da moldura em `dst`. false = sem imagem (usa retangulos).
    fn sprite(&mut self, _src: [f32; 4], _dst: [f32; 4]) -> bool {
        false
    }
}

/// Tamanho da imagem da moldura (assets/frame.png) em pixels.
pub const FRAME_SIZE: [f32; 2] = [640.0, 356.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Combat,
    Movement,
    Visuals,
    World,
    Utility,
    Hud,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Combat,
        Category::Movement,
        Category::Visuals,
        Category::World,
        Category::Utility,
        Category::Hud,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Category::Combat => "Combat",
            Category::Movement => "Movement",
            Category::Visuals => "Visuals",
            Category::World => "World",
            Category::Utility => "Utility",
            Category::Hud => "HUD",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: &'static str,
    pub category: Category,
    pub on: bool,
}

/// Modulos planejados. Por enquanto so ligam/desligam o estado: nenhum faz efeito no jogo ainda.
pub fn default_modules() -> Vec<Module> {
    use Category::*;
    let m = |name, category| Module { name, category, on: false };
    vec![
        m("Radar", Visuals),
        m("Player ESP", Visuals),
        m("Tracers", Visuals),
        m("ESP + Tracers", Visuals),
        m("Block ESP", Visuals),
        m("Fullbright", Visuals),
        m("Zoom", Visuals),
        m("FPS", Hud),
        m("Coordinates", Hud),
        m("Keystrokes", Hud),
        m("Module List", Hud),
        m("Fly", Movement),
        m("Auto Sprint", Movement),
        m("Trigger Bot", Combat),
        m("Auto Walk", Movement),
        m("Auto Respawn", Utility),
        m("Chat Macro", Utility),
        m("Item Counter", Utility),
        m("Waypoints", World),
        m("Block Info", World),
    ]
}

// Geometria medida na moldura (assets/frame.png), como fracao da janela.
const ASPECT: f32 = 775.0 / 1395.0;
const TABS_X: f32 = 0.0115;
const TABS_W: f32 = 0.179;
const TABS_Y: f32 = 0.1406;
const TAB_PITCH: f32 = 0.0856;
const TAB_H_F: f32 = 0.075;
const LIST_X: f32 = 0.2014;
const LIST_Y: f32 = 0.1148;
const LIST_W: f32 = 0.790;
const LIST_H: f32 = 0.871;
const ROW_H: f32 = 22.0;
const PAD: f32 = 6.0;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn has(&self, p: [f32; 2]) -> bool {
        p[0] >= self.x && p[0] < self.x + self.w && p[1] >= self.y && p[1] < self.y + self.h
    }
}

pub struct ClickGui {
    pub modules: Vec<Module>,
    pub category: Category,
    pub blocks: blocks::Selection,
    /// A grade de blocos (estilo inventario criativo) esta aberta no lugar da lista.
    pub picker: bool,
    picker_category: usize,
    scroll: f32,
}

impl Default for ClickGui {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickGui {
    pub fn new() -> Self {
        Self { modules: default_modules(), category: Category::Visuals, blocks: Default::default(), picker: false, picker_category: 0, scroll: 0.0 }
    }

    pub fn enabled(&self, name: &str) -> bool {
        self.modules.iter().any(|m| m.name == name && m.on)
    }

    fn window(screen: [f32; 2]) -> Rect {
        let mut w = 480.0_f32.min(screen[0] - 10.0).max(200.0);
        let mut h = w * ASPECT;
        if h > screen[1] - 10.0 {
            h = (screen[1] - 10.0).max(110.0);
            w = h / ASPECT;
        }
        Rect { x: ((screen[0] - w) / 2.0).floor(), y: ((screen[1] - h) / 2.0).floor(), w, h }
    }

    fn tab_rect(win: Rect, i: usize) -> Rect {
        Rect {
            x: win.x + win.w * TABS_X,
            y: win.y + win.h * (TABS_Y + i as f32 * TAB_PITCH),
            w: win.w * TABS_W,
            h: win.h * TAB_H_F,
        }
    }

    fn list_area(win: Rect) -> Rect {
        Rect { x: win.x + win.w * LIST_X, y: win.y + win.h * LIST_Y, w: win.w * LIST_W, h: win.h * LIST_H }
    }

    const CELL: f32 = 40.0;
    const CELL_H: f32 = 30.0;
    const BACK_H: f32 = 20.0;
    const PICKER_TAB_H: f32 = 18.0;
    const PICKER_TABS: [&'static str; 5] = ["Minerios", "Utilidade", "Construcao", "Redstone", "Outros"];
    fn picker_indices(category: usize) -> Vec<usize> {
        blocks::BLOCKS.iter().enumerate().filter_map(|(i, (name, _, _))|
            (blocks::category_of(name) == category).then_some(i)).collect()
    }
    fn picker_tab_rect(area: Rect, i: usize) -> Rect {
        let gap = 3.0;
        let w = (area.w - PAD * 2.0 - gap * 4.0) / 5.0;
        Rect { x: area.x + PAD + i as f32 * (w + gap), y: area.y + PAD + Self::BACK_H, w, h: Self::PICKER_TAB_H }
    }

    fn cols(area: Rect) -> usize {
        (((area.w - PAD * 2.0) / Self::CELL).floor() as usize).max(1)
    }

    fn cell_rect(area: Rect, scroll: f32, i: usize) -> Rect {
        let cols = Self::cols(area);
        Rect {
            x: area.x + PAD + (i % cols) as f32 * Self::CELL,
            y: area.y + PAD + Self::BACK_H + Self::PICKER_TAB_H + 4.0 + (i / cols) as f32 * Self::CELL_H - scroll,
            w: Self::CELL - 3.0,
            h: Self::CELL_H - 3.0,
        }
    }

    /// Botao "Blocos" na linha do Block ESP (toque funciona no celular, sem botao direito).
    fn blocks_btn(row: Rect) -> Rect {
        Rect { x: row.x + row.w - 82.0, y: row.y + 3.0, w: 44.0, h: row.h - 6.0 }
    }

    fn back_rect(area: Rect) -> Rect {
        Rect { x: area.x + PAD, y: area.y + PAD, w: area.w - PAD * 2.0, h: Self::BACK_H - 3.0 }
    }

    fn visible(&self) -> Vec<usize> {
        (0..self.modules.len()).filter(|&i| self.modules[i].category == self.category).collect()
    }

    fn max_scroll(&self, area: Rect) -> f32 {
        if self.picker {
            let rows = Self::picker_indices(self.picker_category).len().div_ceil(Self::cols(area)) as f32;
            return (rows * Self::CELL_H + Self::BACK_H + Self::PICKER_TAB_H + 4.0 + PAD * 2.0 - area.h).max(0.0);
        }
        (self.visible().len() as f32 * ROW_H + PAD * 2.0 - area.h).max(0.0)
    }

    fn row_rect(area: Rect, scroll: f32, n: usize) -> Rect {
        Rect { x: area.x + PAD, y: area.y + PAD + n as f32 * ROW_H - scroll, w: area.w - PAD * 2.0, h: ROW_H - 4.0 }
    }

    pub fn draw(&self, c: &mut dyn Canvas, mouse: [f32; 2], screen: [f32; 2]) {
        let win = Self::window(screen);
        let skinned = c.sprite([0.0, 0.0, FRAME_SIZE[0], FRAME_SIZE[1]], [win.x, win.y, win.w, win.h]);
        if !skinned {
            c.fill(win.x, win.y, win.w, win.h, BG);
            c.fill(win.x, win.y, win.w, 2.0, ACCENT);
            c.text(win.x + 8.0, win.y + 6.0, "Witch Client", TEXT, true);
            let l = Self::list_area(win);
            c.fill(win.x, win.y + win.h * LIST_Y, l.x - win.x, win.h * LIST_H, PANEL);
        }
        for (i, cat) in Category::ALL.iter().enumerate() {
            let r = Self::tab_rect(win, i);
            let sel = *cat == self.category;
            if sel {
                if !skinned {
                    c.fill(r.x, r.y, r.w, r.h, ROW_HOVER);
                }
                c.fill(r.x, r.y + 2.0, 2.0, r.h - 4.0, ACCENT);
            } else {
                if r.has(mouse) {
                    c.fill(r.x, r.y, r.w, r.h, ROW);
                } else if i >= 4 {
                    c.fill(r.x, r.y, r.w, r.h, 0xFF0C0C12); // a moldura so traz 4 abas: as outras seguem o mesmo estilo
                }
            }
            c.text(r.x + 8.0, r.y + (r.h - 8.0) / 2.0, cat.name(), if sel { TEXT } else { DIM }, false);
        }

        let area = Self::list_area(win);
        if self.picker {
            self.draw_picker(c, mouse, area);
            return;
        }
        let vis = self.visible();
        if vis.is_empty() {
            c.text(area.x + 10.0, area.y + 10.0, "Nenhum modulo aqui ainda", DIM, false);
        }
        for (n, &i) in vis.iter().enumerate() {
            let r = Self::row_rect(area, self.scroll, n);
            // so desenha linhas inteiras dentro da area (sem scissor no Canvas)
            if r.y < area.y || r.y + r.h > area.y + area.h {
                continue;
            }
            let m = &self.modules[i];
            c.fill(r.x, r.y, r.w, r.h, if r.has(mouse) { ROW_HOVER } else { ROW });
            c.text(r.x + 8.0, r.y + 7.0, m.name, TEXT, false);
            if m.name == "Block ESP" {
                let b = Self::blocks_btn(r);
                c.fill(b.x, b.y, b.w, b.h, if b.has(mouse) { ACCENT } else { OFF });
                let w = c.text_width("Blocos");
                c.text(b.x + (b.w - w) / 2.0, b.y + 4.0, "Blocos", TEXT, false);
            }
            let (px, py) = (r.x + r.w - 30.0, r.y + 5.0);
            c.fill(px, py, 22.0, 12.0, if m.on { ACCENT } else { OFF });
            let kx = if m.on { px + 12.0 } else { px + 2.0 };
            c.fill(kx, py + 2.0, 8.0, 8.0, TEXT);
        }
        if self.max_scroll(area) > 0.0 {
            let total = vis.len() as f32 * ROW_H + PAD * 2.0;
            let bh = (area.h * area.h / total).max(12.0);
            let by = area.y + (area.h - bh) * (self.scroll / self.max_scroll(area));
            c.fill(win.x + win.w - 3.0, by, 2.0, bh, ACCENT);
        }
    }

    fn draw_picker(&self, c: &mut dyn Canvas, mouse: [f32; 2], area: Rect) {
        let b = Self::back_rect(area);
        c.fill(b.x, b.y, b.w, b.h, if b.has(mouse) { ROW_HOVER } else { ROW });
        c.text(b.x + 8.0, b.y + 6.0, "< Voltar  |  toque nos blocos para marcar", TEXT, false);
        for (tab, label) in Self::PICKER_TABS.iter().enumerate() {
            let tr = Self::picker_tab_rect(area, tab);
            let selected = tab == self.picker_category;
            c.fill(tr.x, tr.y, tr.w, tr.h, if selected { ACCENT } else if tr.has(mouse) { ROW_HOVER } else { ROW });
            let tw = c.text_width(label);
            c.text(tr.x + (tr.w - tw) / 2.0, tr.y + 5.0, label, if selected { TEXT } else { DIM }, false);
        }
        for (cell_i, block_i) in Self::picker_indices(self.picker_category).iter().copied().enumerate() {
            let (name, label, color) = &blocks::BLOCKS[block_i];
            let i = cell_i;
            let r = Self::cell_rect(area, self.scroll, i);
            if r.y < area.y + Self::BACK_H || r.y + r.h > area.y + area.h {
                continue;
            }
            let on = self.blocks.has(name);
            c.fill(r.x, r.y, r.w, r.h, if r.has(mouse) { ROW_HOVER } else { ROW });
            if on {
                c.fill(r.x, r.y, r.w, 2.0, *color);
                c.fill(r.x, r.y + r.h - 2.0, r.w, 2.0, *color);
            }
            let icon_drawn = c.item_icon(name, r.x + 8.0, r.y + 2.0);
            let w = c.text_width(label);
            c.text(r.x + (r.w - w) / 2.0, r.y + if icon_drawn { 20.0 } else { 10.0 }, label, if on { *color } else { DIM }, false);
        }
    }

    fn click_picker(&mut self, mouse: [f32; 2], area: Rect, button: i32) {
        if button == 1 || Self::back_rect(area).has(mouse) {
            self.picker = false;
            self.scroll = 0.0;
            return;
        }
        for (cell_i, block_i) in Self::picker_indices(self.picker_category).iter().copied().enumerate() {
            let (name, _, _) = &blocks::BLOCKS[block_i];
            let r = Self::cell_rect(area, self.scroll, cell_i);
            if r.y >= area.y + Self::BACK_H && r.y + r.h <= area.y + area.h && r.has(mouse) {
                self.blocks.toggle(name);
                return;
            }
        }
    }

    /// Clique esquerdo. Devolve true se a GUI usou o clique (dentro da janela).
    pub fn click(&mut self, mouse: [f32; 2], screen: [f32; 2], button: i32) -> bool {
        let win = Self::window(screen);
        if !win.has(mouse) {
            return false;
        }
        if button > 1 {
            return true;
        }
        if button == 1 {
            let area = Self::list_area(win);
            if self.picker {
                self.click_picker(mouse, area, 1);
            } else if area.has(mouse) {
                // botao direito em "Block ESP" abre a grade de blocos
                for (n, i) in self.visible().into_iter().enumerate() {
                    let r = Self::row_rect(area, self.scroll, n);
                    if self.modules[i].name == "Block ESP" && r.has(mouse) {
                        self.picker = true;
                        self.scroll = 0.0;
                    }
                }
            }
            return true;
        }
        for (i, cat) in Category::ALL.iter().enumerate() {
            let r = Self::tab_rect(win, i);
            if r.has(mouse) {
                if self.category != *cat || self.picker {
                    self.category = *cat;
                    self.picker = false;
                    self.scroll = 0.0;
                }
                return true;
            }
        }
        let area = Self::list_area(win);
        if self.picker {
            if area.has(mouse) {
                for tab in 0..Self::PICKER_TABS.len() {
                    if Self::picker_tab_rect(area, tab).has(mouse) {
                        self.picker_category = tab;
                        self.scroll = 0.0;
                        return true;
                    }
                }
                self.click_picker(mouse, area, 0);
            }
            return true;
        }
        if area.has(mouse) {
            for (n, i) in self.visible().into_iter().enumerate() {
                let r = Self::row_rect(area, self.scroll, n);
                if r.y >= area.y && r.y + r.h <= area.y + area.h && r.has(mouse) {
                    if self.modules[i].name == "Block ESP" && Self::blocks_btn(r).has(mouse) {
                        self.picker = true;
                        self.scroll = 0.0;
                    } else {
                        self.modules[i].on = !self.modules[i].on;
                    }
                    break;
                }
            }
        }
        true
    }

    /// Roda do mouse / arrastar com o dedo: `dy` positivo rola para cima (como a roda do mouse).
    pub fn scroll(&mut self, mouse: [f32; 2], screen: [f32; 2], dy: f32) -> bool {
        let win = Self::window(screen);
        if !win.has(mouse) {
            return false;
        }
        let area = Self::list_area(win);
        self.scroll = (self.scroll - dy * ROW_H).clamp(0.0, self.max_scroll(area));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Null;
    impl Canvas for Null {
        fn fill(&mut self, _: f32, _: f32, _: f32, _: f32, _: u32) {}
        fn text(&mut self, _: f32, _: f32, _: &str, _: u32, _: bool) {}
        fn text_width(&self, t: &str) -> f32 {
            t.len() as f32 * 6.0
        }
    }

    const SCREEN: [f32; 2] = [427.0, 240.0];

    fn tab_center(i: usize) -> [f32; 2] {
        let r = ClickGui::tab_rect(ClickGui::window(SCREEN), i);
        [r.x + r.w / 2.0, r.y + r.h / 2.0]
    }

    fn row_center(g: &ClickGui, n: usize) -> [f32; 2] {
        let win = ClickGui::window(SCREEN);
        let r = ClickGui::row_rect(ClickGui::list_area(win), g.scroll, n);
        [r.x + 20.0, r.y + 5.0]
    }

    #[test]
    fn click_tab_changes_category() {
        let mut g = ClickGui::new();
        assert!(g.click(tab_center(1), SCREEN, 0));
        assert_eq!(g.category, Category::Movement);
    }

    #[test]
    fn click_row_toggles_module() {
        let mut g = ClickGui::new();
        g.category = Category::Hud;
        assert!(!g.enabled("FPS"));
        g.click(row_center(&g, 0), SCREEN, 0);
        assert!(g.enabled("FPS"));
        g.click(row_center(&g, 0), SCREEN, 0);
        assert!(!g.enabled("FPS"));
    }

    #[test]
    fn click_outside_is_not_used() {
        let mut g = ClickGui::new();
        assert!(!g.click([1.0, 1.0], SCREEN, 0));
    }

    #[test]
    fn right_click_does_not_toggle() {
        let mut g = ClickGui::new();
        g.category = Category::Hud;
        g.click(row_center(&g, 0), SCREEN, 1);
        assert!(!g.enabled("FPS"));
    }

    #[test]
    fn scroll_is_clamped() {
        let mut g = ClickGui::new();
        let win = ClickGui::window(SCREEN);
        let inside = [win.x + win.w * 0.5, win.y + win.h * 0.5];
        g.scroll(inside, SCREEN, 50.0);
        assert_eq!(g.scroll, 0.0);
        g.scroll(inside, SCREEN, -50.0);
        assert!(g.scroll <= g.max_scroll(ClickGui::list_area(win)));
    }

    #[test]
    fn blocks_button_opens_grid_and_select_works() {
        let mut g = ClickGui::new();
        g.category = Category::Visuals;
        let idx = g.visible().iter().position(|&i| g.modules[i].name == "Block ESP").unwrap();
        let r = ClickGui::row_rect(ClickGui::list_area(ClickGui::window(SCREEN)), g.scroll, idx);
        let b = ClickGui::blocks_btn(r);
        g.click([b.x + 3.0, b.y + 3.0], SCREEN, 0);
        assert!(g.picker && !g.enabled("Block ESP"));
        let win = ClickGui::window(SCREEN);
        let area = ClickGui::list_area(win);
        let r = ClickGui::cell_rect(area, g.scroll, 0);
        g.click([r.x + 5.0, r.y + 5.0], SCREEN, 0);
        assert!(g.blocks.has(blocks::BLOCKS[0].0));
        g.click([r.x + 5.0, r.y + 5.0], SCREEN, 1); // direito fecha
        assert!(!g.picker);
    }

    #[test]
    fn picker_draws_all_cells() {
        let mut g = ClickGui::new();
        g.picker = true;
        g.draw(&mut Null, [0.0, 0.0], SCREEN);
    }

    #[test]
    fn small_screen_window_fits() {
        let w = ClickGui::window([240.0, 140.0]);
        assert!(w.x >= 0.0 && w.y >= 0.0 && w.x + w.w <= 240.0 && w.y + w.h <= 140.0);
    }

    #[test]
    fn draw_runs_for_every_category() {
        let mut g = ClickGui::new();
        for c in Category::ALL {
            g.category = c;
            g.draw(&mut Null, [0.0, 0.0], SCREEN);
        }
    }
}
