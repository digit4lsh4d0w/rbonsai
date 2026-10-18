use rand::{RngExt, rngs::SmallRng};

use super::BranchType;

pub fn set_deltas(
    branch_type: BranchType,
    life: i32,
    age: i32,
    multiplier: i32,
    rng: &mut SmallRng,
) -> (i32, i32) {
    match branch_type {
        BranchType::Trunk => {
            if age <= 2 || life < 4 {
                (rng.random_range(-1..=1), 0)
            } else if age < multiplier.saturating_mul(3) {
                let dy = if age % (multiplier / 2).max(1) == 0 {
                    -1
                } else {
                    0
                };
                let dx = match rng.random_range(0..10) {
                    0 => -2,
                    1..=3 => -1,
                    4..=5 => 0,
                    6..=8 => 1,
                    _ => 2,
                };
                (dx, dy)
            } else {
                let dy = if rng.random_ratio(7, 10) { -1 } else { 0 };
                (rng.random_range(-1..=1), dy)
            }
        }
        BranchType::ShootLeft => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=7 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..10) {
                0..=1 => -2,
                2..=5 => -1,
                6..=8 => 0,
                _ => 1,
            };
            (dx, dy)
        }
        BranchType::ShootRight => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=7 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..10) {
                0..=1 => 2,
                2..=5 => 1,
                6..=8 => 0,
                _ => -1,
            };
            (dx, dy)
        }
        BranchType::Dying => {
            let dy = match rng.random_range(0..10) {
                0..=1 => -1,
                2..=8 => 0,
                _ => 1,
            };
            let dx = match rng.random_range(0..15) {
                0 => -3,
                1..=2 => -2,
                3..=5 => -1,
                6..=8 => 0,
                9..=11 => 1,
                12..=13 => 2,
                _ => 3,
            };
            (dx, dy)
        }
        BranchType::Dead => {
            let dy = match rng.random_range(0..10) {
                0..=2 => -1,
                3..=6 => 0,
                _ => 1,
            };
            (rng.random_range(-1..=1), dy)
        }
    }
}

pub fn choose_glyph(branch_type: BranchType, life: i32, dx: i32, dy: i32) -> &'static str {
    if life < 4 {
        return "&";
    }

    match branch_type {
        BranchType::Trunk => match (dx, dy) {
            (0, 0) => "/~",
            _ if dx < 0 => "\\|",
            (0, _) => "/|\\",
            _ if dx > 0 => "|/",
            _ => "?", // Fallback
        },
        BranchType::ShootLeft => match (dx, dy) {
            _ if dy > 0 => "\\",
            (0, 0) => "\\_",
            _ if dx < 0 => "\\|",
            (0, _) => "/|",
            _ if dx > 0 => "/",
            _ => "?", // Fallback
        },
        BranchType::ShootRight => match (dx, dy) {
            _ if dy > 0 => "/",
            (0, 0) => "_/",
            _ if dx < 0 => "\\|",
            (0, _) => "/|",
            _ if dx > 0 => "/",
            _ => "?", // Fallback
        },
        BranchType::Dying | BranchType::Dead => "&",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn young_branches_are_always_leaves() {
        assert_eq!(choose_glyph(BranchType::Trunk, 3, 0, 0), "&");
        assert_eq!(choose_glyph(BranchType::Dying, 3, 1, 1), "&");
    }

    #[test]
    fn trunk_glyph_follows_direction() {
        assert_eq!(choose_glyph(BranchType::Trunk, 10, 0, 0), "/~");
        assert_eq!(choose_glyph(BranchType::Trunk, 10, -1, 0), "\\|");
        assert_eq!(choose_glyph(BranchType::Trunk, 10, 1, 0), "|/");
    }
}
