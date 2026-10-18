pub mod rules;

use rand::{RngExt, rngs::SmallRng};

/// The different types of each symbol
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchType {
    /// The main trunk of the tree
    Trunk,
    /// A branch that grows to the left
    ShootLeft,
    /// A branch that grows to the right
    ShootRight,
    /// A branch that is dying
    Dying,
    /// A branch that is dead
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Segment {
    /// Whether this segment was randomly chosen to be rendered bold.
    pub bold: bool,
    pub glyph: &'static str,
    pub pos: Position,
    pub branch_type: BranchType,
}

pub struct TreeParams {
    pub life: i32,
    pub multiplier: i32,
    pub max_x: u16,
    pub max_y: u16,
    pub tree_bottom: u16,
}

struct Counters {
    shoot_counter: i32,
    tree_bottom: u16,
    max_x: u16,
    max_y: u16,
}

fn in_bounds(pos: Position, max_x: u16, max_y: u16) -> bool {
    u16::try_from(pos.x).is_ok_and(|x| x < max_x) && u16::try_from(pos.y).is_ok_and(|y| y < max_y)
}

pub fn grow_tree(params: &TreeParams, rng: &mut SmallRng) -> Vec<Segment> {
    let mut counters = Counters {
        // Initialize shoot counter to a random value
        shoot_counter: rng.random_range(1..=3),
        tree_bottom: params.tree_bottom,
        max_x: params.max_x,
        max_y: params.max_y,
    };

    let mut tree = Vec::new();

    branch(
        params,
        &mut counters,
        &mut tree,
        rng,
        Position {
            x: i32::from(params.max_x / 2),
            y: i32::from(params.tree_bottom),
        },
        BranchType::Trunk,
        params.life,
    );

    tree
}

fn choose_bold(branch_type: BranchType, rng: &mut SmallRng) -> bool {
    match branch_type {
        BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight => {
            rng.random_ratio(1, 2)
        }
        BranchType::Dying => rng.random_ratio(1, 10),
        BranchType::Dead => rng.random_ratio(1, 3),
    }
}

fn branch(
    params: &TreeParams,
    counters: &mut Counters,
    tree: &mut Vec<Segment>,
    rng: &mut SmallRng,
    mut pos: Position,
    branch_type: BranchType,
    mut life: i32,
) {
    let mut shoot_cooldown = params.multiplier;
    let (max_x, max_y) = (counters.max_x, counters.max_y);

    while life > 0 {
        life -= 1;
        let age = params.life - life;

        let (dx, mut dy) = rules::set_deltas(branch_type, life, age, params.multiplier, rng);

        if dy > 0 && pos.y >= i32::from(counters.tree_bottom) {
            dy -= 1;
        }

        if !in_bounds(pos, max_x, max_y) {
            break;
        }

        if life < 3 {
            branch(params, counters, tree, rng, pos, BranchType::Dead, life);
        } else {
            match branch_type {
                BranchType::Trunk | BranchType::ShootLeft | BranchType::ShootRight
                    if life < params.multiplier.saturating_add(2) =>
                {
                    branch(params, counters, tree, rng, pos, BranchType::Dying, life);
                }
                BranchType::Trunk
                    if rng.random_ratio(1, 3) || life % params.multiplier.max(1) == 0 =>
                {
                    if rng.random_ratio(1, 8) && life > 7 {
                        shoot_cooldown = params.multiplier.saturating_mul(2);
                        let random_life = life.saturating_add(rng.random_range(-2..3));
                        branch(
                            params,
                            counters,
                            tree,
                            rng,
                            pos,
                            BranchType::Trunk,
                            random_life,
                        );
                    } else if shoot_cooldown <= 0 {
                        shoot_cooldown = params.multiplier.saturating_mul(2);
                        counters.shoot_counter += 1;
                        let shoot_direction = if counters.shoot_counter % 2 == 0 {
                            BranchType::ShootLeft
                        } else {
                            BranchType::ShootRight
                        };
                        let shoot_life = life.saturating_add(params.multiplier);
                        branch(
                            params,
                            counters,
                            tree,
                            rng,
                            pos,
                            shoot_direction,
                            shoot_life,
                        );
                    }
                }
                _ => {}
            }
        }

        shoot_cooldown -= 1;
        pos.x += dx;
        pos.y += dy;

        if !in_bounds(pos, max_x, max_y) {
            continue;
        }

        tree.push(Segment {
            pos,
            bold: choose_bold(branch_type, rng),
            glyph: rules::choose_glyph(branch_type, life, dx, dy),
            branch_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn seeded_growth_is_reproducible() {
        let params = TreeParams {
            life: 32,
            multiplier: 3,
            max_x: 80,
            max_y: 24,
            tree_bottom: 19,
        };

        let mut first_rng = SmallRng::seed_from_u64(42);
        let mut second_rng = SmallRng::seed_from_u64(42);
        let first = grow_tree(&params, &mut first_rng);
        let second = grow_tree(&params, &mut second_rng);

        assert!(
            first.len() >= 10,
            "expected a non-trivial tree, got {} segments",
            first.len()
        );
        let distinct: std::collections::BTreeSet<_> =
            first.iter().map(|s| (s.pos.x, s.pos.y)).collect();
        assert!(distinct.len() >= 10);
        assert_eq!(first.len(), second.len());

        for (a, b) in first.iter().zip(second.iter()) {
            assert_eq!(a.pos, b.pos);
            assert_eq!(a.glyph, b.glyph);
            assert_eq!(a.bold, b.bold);
            assert_eq!(a.branch_type, b.branch_type);
        }
    }
}
