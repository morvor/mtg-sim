//! Mana payment (CR 601.2g–h, 106, 107.4, 118.3, 119.4): the automatic payer
//! (`mana_abilities::pay_mana`) against a brute-force search.
//!
//! [`check_board`] builds a small random board for a player — basic and snow lands, dual
//! and pain lands, mana creatures, Treasures, sources with spending restrictions, mana
//! already in the pool — and a spell with a small random cost (colored, colorless,
//! generic, hybrid, monocolored hybrid, Phyrexian, snow and X symbols). The oracle is
//! written from the rules, not from the engine's payer: each source is modeled by hand
//! from its Oracle text (what each of its mana abilities can add, the damage it deals,
//! the restriction on spending it), and the search tries every way of activating the
//! sources (each at most once, each choice of color) and every assignment of the mana to
//! the cost's symbols, with life for Phyrexian symbols (2 life each, CR 107.4f; only life
//! the player has, CR 119.4, after the damage the mana abilities dealt).
//!
//! * Whenever the search finds a payment, the spell must be among the player's legal
//!   actions and casting it must succeed.
//! * Every payment the engine makes must be legal: the sources it activated must be able
//!   to make the mana that was spent plus what's left in the pool, and the mana spent (with
//!   its snow status and restrictions) and the life paid must pay the cost.

use crate::ability::*;
use crate::card::CardDef;
use crate::decision::Action;
use crate::mana::ManaType;
use crate::object::*;
use crate::testing::{DecisionKind, TestGame, P0};
use crate::types::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;

/// A restriction on spending a unit of mana (from the source's Oracle text).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spend {
    Any,
    /// "Spend this mana only to cast a creature spell." (Ancient Ziggurat)
    CreatureSpell,
    /// "Spend this mana only to cast a multicolored spell." (Pillar of the Paruns)
    MulticoloredSpell,
    /// "Spend this mana only to cast artifact spells." (Mishra's Workshop)
    ArtifactSpell,
}

/// One unit of mana: its type, whether a snow source made it, and its restriction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Unit {
    ty: ManaType,
    snow: bool,
    spend: Spend,
}

/// One way to activate a source: the mana it adds and the damage it deals to its
/// controller as it does.
#[derive(Clone, Debug)]
struct Option_ {
    units: Vec<Unit>,
    damage: u32,
}

/// A source of mana, modeled from its Oracle text.
struct Model {
    card: &'static str,
    /// The token to create instead of a card.
    token: bool,
    options: Vec<Option_>,
}

const COLORS: [ManaType; 5] = [
    ManaType::W,
    ManaType::U,
    ManaType::B,
    ManaType::R,
    ManaType::G,
];

fn unit(ty: ManaType) -> Unit {
    Unit {
        ty,
        snow: false,
        spend: Spend::Any,
    }
}

fn one(ty: ManaType) -> Option_ {
    Option_ {
        units: vec![unit(ty)],
        damage: 0,
    }
}

fn any_color(spend: Spend) -> Vec<Option_> {
    COLORS
        .iter()
        .map(|c| Option_ {
            units: vec![Unit {
                ty: *c,
                snow: false,
                spend,
            }],
            damage: 0,
        })
        .collect()
}

/// The sources boards are made of, with their models.
fn catalog() -> Vec<Model> {
    use ManaType::*;
    let basic = |card, ty| Model {
        card,
        token: false,
        options: vec![one(ty)],
    };
    let snow = |card, ty| Model {
        card,
        token: false,
        options: vec![Option_ {
            units: vec![Unit {
                ty,
                snow: true,
                spend: Spend::Any,
            }],
            damage: 0,
        }],
    };
    let pain = |card, a, b| Model {
        card,
        token: false,
        options: vec![
            one(C),
            Option_ {
                units: vec![unit(a)],
                damage: 1,
            },
            Option_ {
                units: vec![unit(b)],
                damage: 1,
            },
        ],
    };
    vec![
        basic("Plains", W),
        basic("Island", U),
        basic("Swamp", B),
        basic("Mountain", R),
        basic("Forest", G),
        basic("Wastes", C),
        snow("Snow-Covered Forest", G),
        snow("Snow-Covered Island", U),
        snow("Snow-Covered Swamp", B),
        Model {
            card: "Taiga",
            token: false,
            options: vec![one(R), one(G)],
        },
        Model {
            card: "Underground Sea",
            token: false,
            options: vec![one(U), one(B)],
        },
        pain("Karplusan Forest", R, G),
        pain("Underground River", U, B),
        // "Whenever City of Brass becomes tapped, it deals 1 damage to you": a triggered
        // ability, which doesn't resolve during the payment.
        Model {
            card: "City of Brass",
            token: false,
            options: any_color(Spend::Any),
        },
        basic("Llanowar Elves", G),
        Model {
            card: "Birds of Paradise",
            token: false,
            options: any_color(Spend::Any),
        },
        Model {
            card: "Treasure",
            token: true,
            options: any_color(Spend::Any),
        },
        Model {
            card: "Ancient Ziggurat",
            token: false,
            options: any_color(Spend::CreatureSpell),
        },
        Model {
            card: "Pillar of the Paruns",
            token: false,
            options: any_color(Spend::MulticoloredSpell),
        },
        Model {
            card: "Sol Ring",
            token: false,
            options: vec![Option_ {
                units: vec![unit(C), unit(C)],
                damage: 0,
            }],
        },
        Model {
            card: "Mishra's Workshop",
            token: false,
            options: vec![Option_ {
                units: vec![
                    Unit {
                        ty: C,
                        snow: false,
                        spend: Spend::ArtifactSpell,
                    };
                    3
                ],
                damage: 0,
            }],
        },
    ]
}

/// A mana symbol of the cost, as the oracle sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sym {
    Colored(ManaType),
    Generic(u32),
    Hybrid(ManaType, ManaType),
    /// {2/W}
    TwoHybrid(ManaType),
    /// {W/P}
    Phyrexian(ManaType),
    Snow,
    X,
}

fn letter(t: ManaType) -> char {
    match t {
        ManaType::W => 'W',
        ManaType::U => 'U',
        ManaType::B => 'B',
        ManaType::R => 'R',
        ManaType::G => 'G',
        ManaType::C => 'C',
    }
}

fn sym_text(s: Sym) -> String {
    match s {
        Sym::Colored(t) => format!("{{{}}}", letter(t)),
        Sym::Generic(n) => format!("{{{n}}}"),
        Sym::Hybrid(a, b) => format!("{{{}/{}}}", letter(a), letter(b)),
        Sym::TwoHybrid(c) => format!("{{2/{}}}", letter(c)),
        Sym::Phyrexian(c) => format!("{{{}/P}}", letter(c)),
        Sym::Snow => "{S}".into(),
        Sym::X => "{X}".into(),
    }
}

/// What the spell is, for spending restrictions.
#[derive(Clone, Copy, Debug)]
struct SpellKind {
    creature: bool,
    artifact: bool,
    multicolored: bool,
}

fn may_spend(u: &Unit, k: SpellKind) -> bool {
    match u.spend {
        Spend::Any => true,
        Spend::CreatureSpell => k.creature,
        Spend::MulticoloredSpell => k.multicolored,
        Spend::ArtifactSpell => k.artifact,
    }
}

/// Whether `units` (each usable at most once) and up to `life` life pay `cost` with `x`
/// for X, paying exactly `phyrexian_life` Phyrexian symbols with life if given (CR
/// 107.4f). Brute force over assignments.
fn pays(
    units: &[Unit],
    cost: &[Sym],
    x: u32,
    k: SpellKind,
    life: u32,
    phyrexian_life: Option<u32>,
) -> bool {
    // Slots: each one unit (or, for {2/C} and Phyrexian symbols, alternatives).
    #[derive(Clone, Copy)]
    enum Slot {
        One(fn(&Unit, ManaType, ManaType) -> bool, ManaType, ManaType),
        TwoHybrid(ManaType),
        Phyrexian(ManaType),
    }
    let mut slots: Vec<Slot> = Vec::new();
    fn is_type(u: &Unit, a: ManaType, _: ManaType) -> bool {
        u.ty == a
    }
    fn either(u: &Unit, a: ManaType, b: ManaType) -> bool {
        u.ty == a || u.ty == b
    }
    fn anything(_: &Unit, _: ManaType, _: ManaType) -> bool {
        true
    }
    fn snowy(u: &Unit, _: ManaType, _: ManaType) -> bool {
        u.snow
    }
    let w = ManaType::W;
    for s in cost {
        match *s {
            Sym::Colored(t) => slots.push(Slot::One(is_type, t, t)),
            Sym::Hybrid(a, b) => slots.push(Slot::One(either, a, b)),
            Sym::Snow => slots.push(Slot::One(snowy, w, w)),
            Sym::TwoHybrid(c) => slots.push(Slot::TwoHybrid(c)),
            Sym::Phyrexian(c) => slots.push(Slot::Phyrexian(c)),
            Sym::Generic(n) => slots.extend((0..n).map(|_| Slot::One(anything, w, w))),
            Sym::X => slots.extend((0..x).map(|_| Slot::One(anything, w, w))),
        }
    }
    let usable: Vec<Unit> = units.iter().copied().filter(|u| may_spend(u, k)).collect();
    fn go(
        slots: &[Slot],
        i: usize,
        units: &[Unit],
        used: &mut Vec<bool>,
        life_left: u32,
        phy_left: Option<u32>,
    ) -> bool {
        let Some(slot) = slots.get(i) else {
            return phy_left.is_none_or(|n| n == 0);
        };
        let take = |used: &mut Vec<bool>,
                    f: &dyn Fn(&Unit) -> bool,
                    next: &mut dyn FnMut(&mut Vec<bool>) -> bool| {
            for j in 0..units.len() {
                if !used[j] && f(&units[j]) {
                    used[j] = true;
                    if next(used) {
                        used[j] = false;
                        return true;
                    }
                    used[j] = false;
                }
            }
            false
        };
        match *slot {
            Slot::One(f, a, b) => take(used, &|u| f(u, a, b), &mut |used| {
                go(slots, i + 1, units, used, life_left, phy_left)
            }),
            Slot::TwoHybrid(c) => {
                take(used, &|u| u.ty == c, &mut |used| {
                    go(slots, i + 1, units, used, life_left, phy_left)
                }) || take(used, &|_| true, &mut |used| {
                    take(used, &|_| true, &mut |used| {
                        go(slots, i + 1, units, used, life_left, phy_left)
                    })
                })
            }
            Slot::Phyrexian(c) => {
                let mana = take(used, &|u| u.ty == c, &mut |used| {
                    go(slots, i + 1, units, used, life_left, phy_left)
                });
                mana || (life_left >= 2
                    && phy_left.is_none_or(|n| n > 0)
                    && go(
                        slots,
                        i + 1,
                        units,
                        used,
                        life_left - 2,
                        phy_left.map(|n| n - 1),
                    ))
            }
        }
    }
    let mut used = vec![false; usable.len()];
    go(&slots, 0, &usable, &mut used, life, phyrexian_life)
}

/// One generated case.
#[derive(Clone, Debug)]
pub struct Case {
    pub seed: u64,
    sources: Vec<usize>,
    pool: Vec<ManaType>,
    cost: Vec<Sym>,
    x: u32,
    kind: SpellKind,
    life: i32,
}

impl Case {
    pub fn describe(&self) -> String {
        let cat = catalog();
        let names: Vec<&str> = self.sources.iter().map(|i| cat[*i].card).collect();
        format!(
            "board [{}], pool [{}], life {}, a {}{}{} spell costing {} with X={}",
            names.join(", "),
            self.pool.iter().map(|t| letter(*t)).collect::<String>(),
            self.life,
            if self.kind.creature { "creature " } else { "" },
            if self.kind.artifact { "artifact " } else { "" },
            if self.kind.multicolored {
                "(multicolored)"
            } else {
                ""
            },
            self.cost.iter().map(|s| sym_text(*s)).collect::<String>(),
            self.x
        )
    }
}

/// The random case of `seed`.
pub fn case(seed: u64) -> Case {
    use ManaType::*;
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x9a7_3e47);
    let cat = catalog();
    let n_sources = rng.gen_range(0..=5);
    let sources: Vec<usize> = (0..n_sources)
        .map(|_| rng.gen_range(0..cat.len()))
        .collect();
    let pool: Vec<ManaType> = if rng.gen_bool(0.2) {
        (0..rng.gen_range(1..=2))
            .map(|_| *[W, U, B, R, G, C].choose(&mut rng).unwrap())
            .collect()
    } else {
        vec![]
    };
    let mut cost = Vec::new();
    let n_syms = rng.gen_range(1..=4);
    for _ in 0..n_syms {
        let c = *COLORS.choose(&mut rng).unwrap();
        let d = *COLORS.choose(&mut rng).unwrap();
        let s = match rng.gen_range(0..10) {
            0 | 1 => Sym::Colored(c),
            2 => Sym::Colored(C),
            3 => Sym::Generic(rng.gen_range(1..=2)),
            4 if c != d => Sym::Hybrid(c, d),
            5 => Sym::TwoHybrid(c),
            6 => Sym::Phyrexian(c),
            7 => Sym::Snow,
            8 if !cost.contains(&Sym::X) => Sym::X,
            _ => Sym::Colored(c),
        };
        cost.push(s);
    }
    // Generic mana first, as costs are written (CR 107.4: the order doesn't matter to the
    // payment).
    cost.sort_by_key(|s| match s {
        Sym::X => 0,
        Sym::Generic(_) => 1,
        _ => 2,
    });
    // Merge generic symbols.
    let total: u32 = cost
        .iter()
        .map(|s| if let Sym::Generic(n) = s { *n } else { 0 })
        .sum();
    cost.retain(|s| !matches!(s, Sym::Generic(_)));
    if total > 0 {
        let at = cost.iter().position(|s| *s != Sym::X).unwrap_or(cost.len());
        cost.insert(at, Sym::Generic(total));
    }
    let x = if cost.contains(&Sym::X) {
        rng.gen_range(0..=2)
    } else {
        0
    };
    let mut colors = std::collections::BTreeSet::new();
    for s in &cost {
        match s {
            Sym::Colored(t) if *t != C => {
                colors.insert(letter(*t));
            }
            Sym::Hybrid(a, b) => {
                colors.insert(letter(*a));
                colors.insert(letter(*b));
            }
            Sym::TwoHybrid(t) | Sym::Phyrexian(t) => {
                colors.insert(letter(*t));
            }
            _ => {}
        }
    }
    let kind = SpellKind {
        creature: rng.gen_bool(0.5),
        artifact: rng.gen_bool(0.3),
        multicolored: colors.len() >= 2,
    };
    let life = *[1, 2, 3, 4, 20].choose(&mut rng).unwrap();
    Case {
        seed,
        sources,
        pool,
        cost,
        x,
        kind,
        life,
    }
}

/// Whether the oracle finds a payment for the case.
fn oracle(c: &Case) -> bool {
    let cat = catalog();
    let models: Vec<&Model> = c.sources.iter().map(|i| &cat[*i]).collect();
    let pool: Vec<Unit> = c.pool.iter().map(|t| unit(*t)).collect();
    // Each source: not activated, or one of its options.
    fn rec(models: &[&Model], i: usize, units: &mut Vec<Unit>, damage: u32, c: &Case) -> bool {
        if i == models.len() {
            let life = (c.life.max(0) as u32).saturating_sub(damage);
            return pays(units, &c.cost, c.x, c.kind, life, None);
        }
        if rec(models, i + 1, units, damage, c) {
            return true;
        }
        for o in &models[i].options {
            let n = units.len();
            units.extend(o.units.iter().copied());
            let ok = rec(models, i + 1, units, damage + o.damage, c);
            units.truncate(n);
            if ok {
                return true;
            }
        }
        false
    }
    let mut units = pool;
    rec(&models, 0, &mut units, 0, c)
}

/// What checking a case found.
#[derive(Clone, Debug, Default)]
pub struct Outcome {
    /// The oracle found a payment.
    pub payable: bool,
    /// The engine cast the spell.
    pub cast: bool,
    /// The spell was among the legal actions.
    pub listed: bool,
}

fn spell_def(c: &Case) -> CardDef {
    let cost_text: String = c.cost.iter().map(|s| sym_text(*s)).collect();
    let mana_cost = crate::mana::ManaCost::parse(&cost_text).expect("a mana cost");
    let mut types = CardTypeSet::default();
    if c.kind.creature {
        types.insert(CardType::Creature);
    }
    if c.kind.artifact {
        types.insert(CardType::Artifact);
    }
    if !c.kind.creature && !c.kind.artifact {
        types.insert(CardType::Sorcery);
    }
    CardDef::custom(Characteristics {
        name: "Payment Test".into(),
        colors: mana_cost.colors(),
        mana_cost: Some(mana_cost),
        card_types: types,
        power: c.kind.creature.then_some(1),
        toughness: c.kind.creature.then_some(1),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// Checks the case of `seed`: `Err` describes a violation.
pub fn check(seed: u64) -> Result<Outcome, String> {
    let c = case(seed);
    let fail = |what: String| Err(format!("{what}: {} (seed {seed})", c.describe()));
    let cat = catalog();
    let mut t = TestGame::new(2);
    t.g.logging = false;
    t.g.players[0].life = c.life;
    let mut source_ids: Vec<(ObjectId, usize)> = Vec::new();
    for i in &c.sources {
        let m = &cat[*i];
        let id = if m.token {
            let before = t.g.objects.len();
            let mut ctx = crate::eval::Ctx::new(None, P0);
            t.g.exec(
                &Effect::CreateToken {
                    spec: crate::tokens::predefined(m.card).expect("a predefined token"),
                    count: Value::Const(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                },
                &mut ctx,
            );
            let Some(id) =
                t.g.battlefield
                    .iter()
                    .copied()
                    .find(|id| id.0 as usize >= before)
            else {
                return fail("couldn't create the token".into());
            };
            t.g.objects[id.0 as usize].summoning_sick = false;
            id
        } else {
            t.battlefield(P0, m.card)
        };
        source_ids.push((id, *i));
    }
    for ty in &c.pool {
        t.g.players[0].mana_pool.add_type(*ty, 1);
    }
    t.g.recompute();
    let card = t.custom(P0, spell_def(&c), Zone::Hand(P0));
    let payable = oracle(&c);
    let listed =
        t.g.legal_actions(P0)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: x, .. } if *x == card));
    let tapped_before: Vec<bool> = source_ids
        .iter()
        .map(|(id, _)| t.g.obj(*id).tapped)
        .collect();
    let life_before = t.g.player(P0).life;
    t.answer(
        P0,
        DecisionKind::X,
        crate::decision::Answer::Number(c.x as i64),
    );
    let r = t.cast(P0, card).try_go();
    let mut out = Outcome {
        payable,
        cast: r.is_ok(),
        listed,
    };
    // With X = 0 only: whether it's listed doesn't depend on the X chosen later.
    if payable && !listed && c.x == 0 {
        return fail("the oracle finds a payment but the spell isn't a legal action".into());
    }
    let spell = match r {
        Ok(s) => s,
        Err(e) => {
            if payable {
                return fail(format!(
                    "the oracle finds a payment but casting failed ({e:?})"
                ));
            }
            return Ok(out);
        }
    };
    if !payable {
        return fail("the engine paid a cost the oracle can't pay".into());
    }
    // The payment must be legal: some activation of the sources it used makes the mana
    // spent plus what's left in the pool, and what was spent pays the cost.
    let si = t.g.obj(spell).stack.as_deref().cloned();
    let Some(si) = si else {
        return fail("the spell isn't on the stack".into());
    };
    let spent: Vec<ManaType> = si.cast.mana_spent.clone();
    let left: Vec<ManaType> = t.g.player(P0).mana_pool.mana.iter().map(|m| m.ty).collect();
    let used: Vec<usize> = source_ids
        .iter()
        .enumerate()
        .filter(|(k, (id, _))| {
            !t.g.is_live(*id)
                || t.g.obj(*id).zone != Zone::Battlefield
                || (t.g.obj(*id).tapped && !tapped_before[*k])
        })
        .map(|(_, (_, m))| *m)
        .collect();
    let life_paid = (life_before - t.g.player(P0).life).max(0) as u32;
    let legal = payment_is_legal(&c, &used, &spent, &left, life_paid);
    if !legal {
        return fail(format!(
            "the engine's payment isn't legal: used [{}], spent {:?}, left in pool {:?}, life lost {life_paid}",
            used.iter().map(|i| cat[*i].card).collect::<Vec<_>>().join(", "),
            spent,
            left
        ));
    }
    out.cast = true;
    Ok(out)
}

/// Whether some activation of the `used` sources (each once) makes, with the starting
/// pool, exactly the mana `spent` plus the mana `left`, such that the spent units pay the
/// cost with the life lost beyond the sources' damage paying Phyrexian symbols.
fn payment_is_legal(
    c: &Case,
    used: &[usize],
    spent: &[ManaType],
    left: &[ManaType],
    life_lost: u32,
) -> bool {
    let cat = catalog();
    let mut want: Vec<ManaType> = spent.iter().chain(left).copied().collect();
    want.sort_by_key(|t| letter(*t));
    fn rec(
        cat: &[Model],
        used: &[usize],
        i: usize,
        units: &mut Vec<Unit>,
        damage: u32,
        f: &dyn Fn(&[Unit], u32) -> bool,
    ) -> bool {
        if i == used.len() {
            return f(units, damage);
        }
        for o in &cat[used[i]].options {
            let n = units.len();
            units.extend(o.units.iter().copied());
            let ok = rec(cat, used, i + 1, units, damage + o.damage, f);
            units.truncate(n);
            if ok {
                return true;
            }
        }
        false
    }
    let check = |units: &[Unit], damage: u32| -> bool {
        let mut have: Vec<ManaType> = units.iter().map(|u| u.ty).collect();
        have.sort_by_key(|t| letter(*t));
        if have != want || life_lost < damage || (life_lost - damage) % 2 != 0 {
            return false;
        }
        let phy = (life_lost - damage) / 2;
        // Choose which units were spent: every subset with the spent types.
        let n = units.len();
        let mut spent_sorted: Vec<ManaType> = spent.to_vec();
        spent_sorted.sort_by_key(|t| letter(*t));
        (0u32..(1 << n)).any(|mask| {
            let sub: Vec<Unit> = (0..n)
                .filter(|j| mask & (1 << j) != 0)
                .map(|j| units[j])
                .collect();
            let mut tys: Vec<ManaType> = sub.iter().map(|u| u.ty).collect();
            tys.sort_by_key(|t| letter(*t));
            tys == spent_sorted
                && sub.len() == spent.len()
                && pays(&sub, &c.cost, c.x, c.kind, 2 * phy, Some(phy))
                && sub.iter().all(|u| may_spend(u, c.kind))
        })
    };
    let mut units: Vec<Unit> = c.pool.iter().map(|t| unit(*t)).collect();
    rec(&cat, used, 0, &mut units, 0, &check)
}

/// Checks `n` cases from `seed` on: the number held, and the violations.
pub fn check_many(seed: u64, n: u64) -> (u64, u64, Vec<String>) {
    let mut held = 0;
    let mut payable = 0;
    let mut violations = Vec::new();
    for i in 0..n {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(i);
        match std::panic::catch_unwind(|| check(s)) {
            Ok(Ok(o)) => {
                held += 1;
                payable += o.payable as u64;
            }
            Ok(Err(e)) => violations.push(e),
            Err(_) => violations.push(format!("panic: {} (seed {s})", case(s).describe())),
        }
    }
    (held, payable, violations)
}
