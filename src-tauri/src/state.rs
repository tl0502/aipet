use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct Hitbox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Hitbox {
    /// 判断屏幕物理坐标 (px, py) 是否落在 hitbox 内。
    /// 用 saturating_add 防止 self.x + self.w 在 i32 边界 overflow:
    /// window.rs 已限制 x/y/w/h ∈ i32 范围,但 x+w 这种复合值仍可能撑到 i32::MAX。
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px <= self.x.saturating_add(self.w)
            && py >= self.y
            && py <= self.y.saturating_add(self.h)
    }

    /// 向四周扩展 px 像素(用于 cursor_tracker 滞后区,HYSTERESIS_PX = 5)。
    /// 全 saturating 链:即便上下游传入边界值也不会 overflow / underflow,只会饱和到 i32::MIN / MAX。
    pub fn expand(&self, px: i32) -> Hitbox {
        let two_px = px.saturating_mul(2);
        Hitbox {
            x: self.x.saturating_sub(px),
            y: self.y.saturating_sub(px),
            w: self.w.saturating_add(two_px),
            h: self.h.saturating_add(two_px),
        }
    }
}

// 主进程全局状态容器。M1 D2:加 hitbox + is_dragging,供 cursor_tracker 与 window commands 共享。
#[derive(Default)]
pub struct AppState {
    pub pet_hitbox: Mutex<Option<Hitbox>>,
    pub is_dragging: Mutex<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hb(x: i32, y: i32, w: i32, h: i32) -> Hitbox {
        Hitbox { x, y, w, h }
    }

    // ===== contains 正常分支 =====

    #[test]
    fn contains_basic_inside() {
        let b = hb(100, 200, 50, 30);
        assert!(b.contains(120, 210));
        assert!(b.contains(100, 200), "左上角应命中(<= 是闭区间)");
        assert!(b.contains(150, 230), "右下角应命中");
    }

    #[test]
    fn contains_basic_outside() {
        let b = hb(100, 200, 50, 30);
        assert!(!b.contains(99, 210), "左外");
        assert!(!b.contains(151, 210), "右外");
        assert!(!b.contains(120, 199), "上外");
        assert!(!b.contains(120, 231), "下外");
    }

    // ===== contains i32 边界饱和 =====

    #[test]
    fn contains_saturates_when_x_plus_w_overflows() {
        // 老实现 self.x + self.w 在此处会 i32 overflow(debug panic / release wrap)
        // saturating_add 把上界饱和到 i32::MAX,行为可定义
        let b = hb(i32::MAX - 10, 0, 100, 100);
        assert!(b.contains(i32::MAX - 5, 50), "x 内部点应命中");
        assert!(b.contains(i32::MAX, 50), "i32::MAX 应命中(右界饱和)");
        assert!(!b.contains(i32::MAX - 11, 50), "< x 应不命中");
    }

    #[test]
    fn contains_saturates_when_y_plus_h_overflows() {
        let b = hb(0, i32::MAX - 10, 100, 100);
        assert!(b.contains(50, i32::MAX - 5));
        assert!(b.contains(50, i32::MAX));
        assert!(!b.contains(50, i32::MAX - 11));
    }

    // ===== expand 正常分支 =====

    #[test]
    fn expand_normal_case() {
        let b = hb(100, 200, 50, 30);
        let e = b.expand(5);
        assert_eq!(e.x, 95);
        assert_eq!(e.y, 195);
        assert_eq!(e.w, 60); // 50 + 2*5
        assert_eq!(e.h, 40); // 30 + 2*5
    }

    // ===== expand i32 边界饱和 =====

    #[test]
    fn expand_saturates_x_at_i32_min() {
        // x = i32::MIN + 2, sub 5 → 老实现 underflow,saturating 停在 i32::MIN
        let b = hb(i32::MIN + 2, 0, 10, 10);
        let e = b.expand(5);
        assert_eq!(e.x, i32::MIN, "x 饱和到 i32::MIN");
        assert_eq!(e.w, 20, "w 在小值不变");
    }

    #[test]
    fn expand_saturates_w_at_i32_max() {
        // w = i32::MAX - 5, add 10 → 老实现 overflow,saturating 停在 i32::MAX
        let b = hb(0, 0, i32::MAX - 5, i32::MAX - 5);
        let e = b.expand(5);
        assert_eq!(e.w, i32::MAX);
        assert_eq!(e.h, i32::MAX);
    }
}
