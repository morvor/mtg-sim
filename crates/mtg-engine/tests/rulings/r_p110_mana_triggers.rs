//! Rulings batch P110 — abilities that add mana but aren't mana abilities: triggered
//! abilities that don't trigger from a mana ability, abilities with a target, and loyalty
//! abilities use the stack and can be responded to (CR 605.1a, 605.1b, 605.5a, 606.3), and
//! their unspent mana empties as the step ends (CR 106.4, 500.5); mandatory triggered
//! abilities (CR 603.5).

use crate::r_p110_common::*;
use crate::r_s01_common::attack_with;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Makes the ability trigger (or activates it); returns its source.
type Setup = fn(&mut TestGame) -> ObjectId;

/// Casts the real creature `name` from P0's hand with lands and resolves the spell: its
/// enters ability triggers.
fn cast_and_resolve(t: &mut TestGame, name: &str) -> ObjectId {
    let spell = cast_new(t, P0, name, &[]);
    t.resolve();
    let id = t.g.current(spell);
    assert!(t.on_battlefield(id), "{name}");
    id
}

/// Puts `name` onto the battlefield and destroys it: its dies ability triggers.
fn dies(t: &mut TestGame, name: &str) -> ObjectId {
    let id = t.battlefield(P0, name);
    destroy(t, id);
    id
}

/// Puts the Saga `name` onto the battlefield with `lore - 1` lore counters and adds one:
/// chapter `lore` triggers.
fn chapter(t: &mut TestGame, name: &str, lore: u32) -> ObjectId {
    let saga = t.battlefield(P0, name);
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), lore - 1);
    add_lore(t, saga, 1);
    t.settle();
    saga
}

/// Each case: the card, how its ability is made to trigger, and the mana it adds.
fn cases() -> Vec<(&'static str, Setup, u32)> {
    vec![
        (
            "Akki Rockspeaker",
            |t| cast_and_resolve(t, "Akki Rockspeaker"),
            1,
        ),
        ("Priest of Gix", |t| cast_and_resolve(t, "Priest of Gix"), 3),
        (
            "Priest of Urabrask",
            |t| cast_and_resolve(t, "Priest of Urabrask"),
            3,
        ),
        (
            "Quirion Sentinel",
            |t| cast_and_resolve(t, "Quirion Sentinel"),
            1,
        ),
        (
            "Burning-Tree Emissary",
            |t| cast_and_resolve(t, "Burning-Tree Emissary"),
            2,
        ),
        // Its party is itself (a Wizard).
        (
            "Ardent Electromancer",
            |t| cast_and_resolve(t, "Ardent Electromancer"),
            1,
        ),
        (
            "Iridescent Tiger",
            |t| cast_and_resolve(t, "Iridescent Tiger"),
            5,
        ),
        (
            "Flamecache Gecko",
            |t| {
                cast_new(t, P0, "Shock", &[Entity::Player(P1)]);
                t.resolve_all();
                cast_and_resolve(t, "Flamecache Gecko")
            },
            2,
        ),
        ("Cathodion", |t| dies(t, "Cathodion"), 3),
        ("Myr Moonvessel", |t| dies(t, "Myr Moonvessel"), 1),
        ("Su-Chi", |t| dies(t, "Su-Chi"), 4),
        (
            "Rodeo Pyromancers",
            |t| {
                let r = t.battlefield(P0, "Rodeo Pyromancers");
                cast_new(t, P0, "Ornithopter", &[]);
                t.settle();
                r
            },
            2,
        ),
        (
            "Radha, Heir to Keld",
            |t| {
                let radha = t.battlefield(P0, "Radha, Heir to Keld");
                t.answer_yes(P0, true);
                attack_with(t, &[(radha, Entity::Player(P1))]);
                radha
            },
            2,
        ),
        (
            "Carpet of Flowers",
            |t| {
                let carpet = t.battlefield(P0, "Carpet of Flowers");
                t.lands(P1, "Island", 2);
                t.answer_targets(P0, &[Entity::Player(P1)]);
                t.answer_yes(P0, true);
                t.set_step(P0, Step::Draw);
                t.advance_to(P0, Step::PrecombatMain);
                t.settle();
                carpet
            },
            2,
        ),
        (
            "Summon: G.F. Ifrit",
            |t| chapter(t, "Summon: G.F. Ifrit", 3),
            1,
        ),
        ("The Bath Song", |t| chapter(t, "The Bath Song", 3), 2),
        (
            "Arni Slays the Troll",
            |t| {
                t.answer_targets(P0, &[]);
                chapter(t, "Arni Slays the Troll", 2)
            },
            1,
        ),
        // "At the beginning of your precombat main phase, add mana equal to enchanted
        // permanent's mana cost.", on a Grizzly Bears ({1}{G}).
        (
            "Elemental Resonance",
            |t| {
                let res = t.battlefield(P0, "Elemental Resonance");
                let b = t.battlefield(P0, "Grizzly Bears");
                assert!(t.g.attach(res, Entity::Object(b)));
                t.set_step(P0, Step::Upkeep);
                t.advance_to(P0, Step::PrecombatMain);
                t.settle();
                res
            },
            2,
        ),
        (
            "Bone Miser",
            |t| {
                // Cycling a land card (Forgotten Cave, cycling {R}) discards it.
                let miser = t.battlefield(P0, "Bone Miser");
                let cave = t.hand(P0, "Forgotten Cave");
                t.lands(P0, "Mountain", 1);
                cycle(t, cave);
                t.settle();
                miser
            },
            2,
        ),
    ]
}

/// Cycles the card `card` in P0's hand.
pub fn cycle(t: &mut TestGame, card: ObjectId) {
    t.g.turn.priority = Some(P0);
    t.g.recompute();
    let action = t
        .g
        .legal_actions(P0)
        .into_iter()
        .find(|a| {
            matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == card)
        })
        .expect("cycling");
    let mtg_engine::decision::Action::Activate { source, ability } = action else {
        unreachable!()
    };
    t.g.activate_ability(P0, source, ability).expect("cycled");
    t.g.flush_events();
}

/// The cards whose ability isn't optional: "You get the mana whether you want it or not."
const MANDATORY: [&str; 8] = [
    "Akki Rockspeaker",
    "Cathodion",
    "Elemental Resonance",
    "Myr Moonvessel",
    "Priest of Gix",
    "Priest of Urabrask",
    "Quirion Sentinel",
    "Su-Chi",
];

#[test]
fn mana_adding_triggered_abilities_use_the_stack() {
    cr!("605.1b", "605.5a", "603.3", "117.7", "106.4", "500.5");
    ruling!(
        "Akki Rockspeaker",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Cathodion",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Myr Moonvessel",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Priest of Gix",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Priest of Urabrask",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Quirion Sentinel",
        "The triggered ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Burning-Tree Emissary",
        "Burning-Tree Emissary's ability isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Ardent Electromancer",
        "Ardent Electromancer’s ability uses the stack and players may respond to it. It isn’t a mana ability because the event that causes it to trigger isn’t a mana ability."
    );
    ruling!(
        "Iridescent Tiger",
        "Iridescent Tiger’s ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Flamecache Gecko",
        "Flamecache Gecko’s first ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Su-Chi",
        "You get the mana whether you want it or not. If you don’t spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Akki Rockspeaker",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Cathodion",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Elemental Resonance",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Myr Moonvessel",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Priest of Gix",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Priest of Urabrask",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Quirion Sentinel",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    ruling!(
        "Rodeo Pyromancers",
        "Rodeo Pyromancers’s ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Radha, Heir to Keld",
        "Radha's triggered ability is not a mana ability because it doesn't trigger from another mana ability. It goes on the stack and it can be responded to."
    );
    ruling!("Carpet of Flowers", "The ability is not a mana ability.");
    ruling!(
        "Summon: G.F. Ifrit",
        "Summon: G.F. Ifrit's third and fourth chapter abilities aren't mana abilities. They use the stack and can be responded to."
    );
    ruling!(
        "The Bath Song",
        "The Bath Song's final chapter ability is not a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Arni Slays the Troll",
        "The chapter II ability isn’t a mana ability. It uses the stack and can be responded to."
    );
    for (card, setup, amount) in cases() {
        supported(card);
        let mut t = TestGame::new(2);
        let from = t.asked().len();
        let src = setup(&mut t);
        assert!(ability_on_stack(&t, src), "{card}: on the stack");
        assert_eq!(pool_total(&t, P0), 0, "{card}: no mana yet");
        // P1 responds: Shock resolves first, before any mana is added.
        let life = t.life(P0);
        cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
        t.resolve();
        assert_eq!(t.life(P0), life - 2, "{card}");
        assert_eq!(pool_total(&t, P0), 0, "{card}");
        t.resolve_all();
        assert_eq!(
            pool_total(&t, P0),
            amount,
            "{card}: mana added on resolution"
        );
        if MANDATORY.contains(&card) {
            assert!(
                t.asked()[from..]
                    .iter()
                    .all(|(_, d)| !matches!(d, Decision::YesNo { .. })),
                "{card}: not optional"
            );
        }
        // Unspent, the mana empties from the pool as the step ends.
        let next = match t.g.turn.step {
            Step::PrecombatMain => Step::BeginningOfCombat,
            Step::DeclareAttackers => Step::DeclareBlockers,
            s => panic!("{card}: {s:?}"),
        };
        t.advance_to(P0, next);
        assert_eq!(pool_total(&t, P0), 0, "{card}");
    }
}

#[test]
fn bone_misers_mana_cant_pay_for_the_discard_cost_that_made_it() {
    cr!("605.1b", "601.2g", "602.2b");
    ruling!(
        "Bone Miser",
        "This creature’s second ability isn’t a mana ability. It uses the stack and can be responded to. Notably, this means that you can’t pay a cost of “{2}, Discard a card” by discarding a land card and using the {B}{B} produced to pay for the {2}."
    );
    ruling!(
        "Bone Miser",
        "This creature’s abilities are triggered abilities, not activated abilities. They don’t allow you to discard a card whenever you want; rather, you need some other way of discarding a card (such as cycling)."
    );
    supported("Bone Miser");
    supported("Flamecache Gecko");
    // Flamecache Gecko: "{1}{R}, Discard a card: Draw a card." With only {R} available,
    // discarding a land can't pay for the {1}.
    let mut t = TestGame::new(2);
    let miser = t.battlefield(P0, "Bone Miser");
    assert!(!can_activate(&mut t, P0, miser), "no activated abilities");
    let gecko = t.battlefield(P0, "Flamecache Gecko");
    let land = t.hand(P0, "Forest");
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_choose(P0, &[obj(land)]);
    assert!(act(&mut t, P0, gecko, "Discard", &[]).is_err());
    assert_eq!(t.zone(land), mtg_engine::object::Zone::Hand(P0));
    // With {1}{R}, it can: the {B}{B} comes after, from the stack.
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(land)]);
    act(&mut t, P0, gecko, "Discard", &[]).unwrap();
    t.settle();
    assert_eq!(pool_total(&t, P0), 0);
    assert!(ability_on_stack(&t, miser));
    t.resolve();
    assert_eq!(pool_of(&t, P0, ManaType::B), 2);
}

#[test]
fn bone_miser_trigger_from_a_cost_resolves_before_the_spell() {
    cr!("601.2c", "601.2h", "603.3", "608.2c");
    ruling!(
        "Bone Miser",
        "If you discard a card as a cost to cast a spell or activate an ability, this creature’s triggered ability resolves before that spell or ability but after you’ve chosen targets for it. If you discard a card while a spell or ability is resolving, the triggered ability resolves after you’ve finished resolving that spell or ability."
    );
    supported("Lightning Axe");
    // Lightning Axe: "As an additional cost to cast this spell, discard a card or pay {5}.
    // Lightning Axe deals 5 damage to target creature." Discarding Grizzly Bears: the
    // Zombie comes first; the Axe's target was chosen before.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bone Miser");
    let target = t.battlefield(P1, "Hill Giant");
    let b = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(b)]);
    let axe = cast_new(&mut t, P0, "Lightning Axe", &[obj(target)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(crate::r_s25_common::targets_of(&t, axe), vec![obj(target)]);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(t.on_battlefield(target), "the Axe hasn't resolved");
    t.resolve();
    assert!(!t.on_battlefield(target));
    // Mind Rot makes P0 discard two creature cards as it resolves: both Zombies come
    // after it has finished resolving.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bone Miser");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Hill Giant");
    let rot = cast_new(&mut t, P0, "Mind Rot", &[Entity::Player(P0)]);
    t.g.resolve_top();
    assert!(!t.g.stack.contains(&rot));
    assert_eq!(t.hand_size(P0), 0);
    assert!(tokens(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn not_optional_abilities() {
    cr!("603.5", "603.3d");
    ruling!("Eladamri's Vineyard", "This ability is not optional.");
    ruling!("Magus of the Vineyard", "This ability is not optional.");
    ruling!("Grave Peril", "This ability is not optional.");
    ruling!("Magister Sphinx", "This ability is not optional.");
    // The Vineyards: no choice is offered and each player gets {G}{G} in their first
    // main phase.
    for card in ["Eladamri's Vineyard", "Magus of the Vineyard"] {
        supported(card);
        for p in [P0, P1] {
            let mut t = TestGame::new(2);
            t.battlefield(P0, card);
            t.set_step(p, Step::Draw);
            let from = t.asked().len();
            t.advance_to(p, Step::PrecombatMain);
            t.resolve();
            assert!(t.asked()[from..]
                .iter()
                .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
            assert_eq!(pool_of(&t, p, ManaType::G), 2, "{card}, {p:?}");
        }
    }
    // Grave Peril: even P0's own nonblack creature is destroyed, and the Peril sacrificed.
    supported("Grave Peril");
    let mut t = TestGame::new(2);
    let peril = t.battlefield(P0, "Grave Peril");
    let b = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(peril));
    assert!(!t.on_battlefield(b));
    // Magister Sphinx: a player must be targeted, even P0 itself.
    supported("Magister Sphinx");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.enter(P0, "Magister Sphinx");
    t.resolve_all();
    assert_eq!(t.life(P0), 10);
}
