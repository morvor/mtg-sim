//! Rulings batch P169 — "whenever you cast a noncreature (or multicolored) spell"
//! triggers resolve before the spell that caused them, and resolve even if that spell is
//! countered (CR 603.3, 601.2i, 405.2).

use crate::r_p169_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The spell P0 casts to trigger the ability.
#[derive(Clone, Copy)]
enum Spell {
    /// Opt: a one-mana monocolored instant.
    Opt,
    /// Growth Spiral: a multicolored instant.
    Multicolored,
    /// Concentrate: four mana spent.
    FourMana,
    /// Stroke of Genius with X = 5 targeting P1: eight mana spent.
    EightMana,
}

fn cast(t: &mut TestGame, s: Spell) -> ObjectId {
    match s {
        Spell::Opt => cast_opt(t, P0),
        Spell::Multicolored => {
            mana(t, P0, ManaType::G, 1);
            mana(t, P0, ManaType::U, 1);
            let c = t.hand(P0, "Growth Spiral");
            t.cast(P0, c).go()
        }
        Spell::FourMana => {
            mana(t, P0, ManaType::U, 4);
            let c = t.hand(P0, "Concentrate");
            t.cast(P0, c).go()
        }
        Spell::EightMana => {
            mana(t, P0, ManaType::U, 8);
            let c = t.hand(P0, "Stroke of Genius");
            t.cast(P0, c).target(Entity::Player(P1)).x(5).go()
        }
    }
}

struct Case {
    card: &'static str,
    spell: Spell,
    /// Before the spell is cast (the source is given).
    prep: fn(&mut TestGame, ObjectId),
    /// After the spell is cast, before its triggers are put on the stack.
    answers: fn(&mut TestGame, ObjectId),
    /// After the spell is countered and the stack resolved.
    check: fn(&mut TestGame, ObjectId),
}

fn none(_: &mut TestGame, _: ObjectId) {}

/// The card's triggers go on the stack above the spell; the spell is countered; the
/// triggers still resolve (`check`).
fn run(c: &Case) {
    eprintln!("case: {}", c.card);
    supported(c.card);
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, c.card);
    (c.prep)(&mut t, src);
    let spell = cast(&mut t, c.spell);
    (c.answers)(&mut t, src);
    t.settle();
    let mine = stack_triggers_from(&t, src);
    assert!(!mine.is_empty(), "{}: no trigger", c.card);
    let at = t.g.stack.iter().position(|x| *x == spell).unwrap();
    let above: Vec<ObjectId> = t.g.stack[at + 1..].to_vec();
    assert_eq!(above, mine, "{}: the triggers aren't above the spell", c.card);
    counterspell(&mut t, spell);
    t.resolve_all();
    assert!(!resolved(&t, spell), "{}", c.card);
    (c.check)(&mut t, src);
}

#[test]
fn spell_cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "603.3b", "601.2i");
    ruling!(
        "Mage Tower Referee",
        "Mage Tower Referee's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Rampaging Monument",
        "Rampaging Monument's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Hero of Precinct One",
        "The triggered ability of Hero of Precinct One resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Pyroconvergence",
        "The triggered ability will resolve and Pyroconvergence will deal damage before that spell resolves."
    );
    ruling!(
        "Black Waltz No. 3",
        "Black Waltz No. 3's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Blazing Bomb",
        "Blazing Bomb's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Burning Prophet",
        "Burning Prophet’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Cathar's Companion",
        "Cathar’s Companion’s triggered ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Deeproot Champion",
        "Deeproot Champion’s ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Ultros, Obnoxious Octopus",
        "Each of Ultros's abilities resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Estinien Varlineau",
        "Estinien Varlineau's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Firebrand Archer",
        "Firebrand Archer’s ability will resolve before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Firespitter Whelp",
        "Firespitter Whelp's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Flux Channeler",
        "Flux Channeler's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Hermes, Overseer of Elpis",
        "Hermes's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Hraesvelgr of the First Brood",
        "Hraesvelgr's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    ruling!(
        "Kindlespark Duo",
        "Kindlespark Duo's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Magmaroth",
        "Magmaroth’s second ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Mizzium Tank",
        "Mizzium Tank's triggered ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    let cases = [
        Case {
            card: "Mage Tower Referee",
            spell: Spell::Multicolored,
            prep: none,
            answers: none,
            check: |t, s| assert_eq!(t.counters(s, counters::PLUS1), 1),
        },
        Case {
            card: "Rampaging Monument",
            spell: Spell::Multicolored,
            // The three counters it enters with keep the 0/0 alive.
            prep: |t, s| put_counters(t, s, counters::PLUS1, 3),
            answers: none,
            check: |t, s| assert_eq!(t.counters(s, counters::PLUS1), 4),
        },
        Case {
            card: "Hero of Precinct One",
            spell: Spell::Multicolored,
            prep: none,
            answers: none,
            check: |t, _| assert_eq!(creature_tokens(t, P0), 1),
        },
        Case {
            card: "Pyroconvergence",
            spell: Spell::Multicolored,
            prep: none,
            answers: |t, _| {
                t.answer_targets(P0, &[Entity::Player(P1)]);
            },
            check: |t, _| assert_eq!(t.life(P1), 18),
        },
        Case {
            card: "Black Waltz No. 3",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, _| assert_eq!(t.life(P1), 18),
        },
        Case {
            card: "Blazing Bomb",
            spell: Spell::FourMana,
            prep: none,
            answers: none,
            check: |t, s| assert_eq!(t.counters(s, counters::PLUS1), 1),
        },
        Case {
            card: "Burning Prophet",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, s| assert_eq!(t.pt(s), (2, 3)),
        },
        Case {
            card: "Cathar's Companion",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, s| assert!(has_kw(t, s, KeywordKind::Indestructible)),
        },
        Case {
            card: "Deeproot Champion",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, s| assert_eq!(t.counters(s, counters::PLUS1), 1),
        },
        Case {
            card: "Ultros, Obnoxious Octopus",
            spell: Spell::EightMana,
            prep: |t, _| {
                t.battlefield(P1, "Grizzly Bears");
            },
            answers: |t, _| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                t.answer_targets(P0, &[obj(bears)]);
            },
            check: |t, s| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                assert!(t.obj_now(bears).tapped);
                assert_eq!(t.counters(bears, counters::STUN), 1);
                assert_eq!(t.counters(s, counters::PLUS1), 8);
            },
        },
        Case {
            card: "Estinien Varlineau",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, s| {
                assert_eq!(t.counters(s, counters::PLUS1), 1);
                assert!(has_kw(t, s, KeywordKind::Flying));
            },
        },
        Case {
            card: "Firebrand Archer",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
        Case {
            card: "Firespitter Whelp",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
        Case {
            card: "Flux Channeler",
            spell: Spell::Opt,
            prep: |t, _| {
                let bears = t.battlefield(P0, "Grizzly Bears");
                put_counters(t, bears, counters::PLUS1, 1);
            },
            answers: |t, _| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                t.answer_choose(P0, &[obj(bears)]);
            },
            check: |t, _| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                assert_eq!(t.counters(bears, counters::PLUS1), 2);
            },
        },
        Case {
            card: "Hermes, Overseer of Elpis",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, _| assert_eq!(crate::r_s01_common::with_subtype(t, P0, "Bird").len(), 1),
        },
        Case {
            card: "Hraesvelgr of the First Brood",
            spell: Spell::Opt,
            prep: |t, _| {
                t.battlefield(P0, "Grizzly Bears");
                t.battlefield(P1, "Hill Giant");
            },
            answers: |t, _| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                t.answer_targets(P0, &[obj(bears)]);
            },
            check: |t, _| {
                let bears = t.named_on_battlefield("Grizzly Bears")[0];
                let giant = t.named_on_battlefield("Hill Giant")[0];
                assert_eq!(t.pt(bears), (3, 2));
                attack_with(t, &[(bears, Entity::Player(P1))]);
                assert!(!legal_blocks(t, P1, &[(giant, bears)]));
            },
        },
        Case {
            card: "Kindlespark Duo",
            spell: Spell::Opt,
            prep: |t, s| {
                t.g.tap(s);
            },
            answers: none,
            check: |t, s| assert!(!t.obj_now(s).tapped),
        },
        Case {
            card: "Magmaroth",
            spell: Spell::Opt,
            prep: |t, s| put_counters(t, s, counters::MINUS1, 1),
            answers: none,
            check: |t, s| {
                assert_eq!(t.counters(s, counters::MINUS1), 0);
                assert_eq!(t.pt(s), (5, 5));
            },
        },
        Case {
            card: "Mizzium Tank",
            spell: Spell::Opt,
            prep: none,
            answers: none,
            check: |t, s| {
                assert!(t.obj_now(s).is(CardType::Creature));
                assert_eq!(t.pt(s), (4, 3));
            },
        },
    ];
    for c in &cases {
        run(c);
    }
}

#[test]
fn kykar_spirit_is_created_after_the_spells_targets_are_chosen() {
    cr!("601.2c", "603.3");
    ruling!(
        "Kykar, Wind's Fury",
        "Kykar's triggered ability will resolve before the spell that caused it to trigger but after targets have been chosen for that spell, so the Spirit that will be created can't be a target of that spell. Kykar's ability resolves even if that spell is countered."
    );
    supported("Kykar, Wind's Fury");
    let mut t = TestGame::new(2);
    let kykar = t.battlefield(P0, "Kykar, Wind's Fury");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(bears).go();
    t.settle();
    // No Spirit existed as the target was chosen.
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    assert!(offered
        .iter()
        .flatten()
        .all(|e| !matches!(e, Entity::Object(o) if t.obj_now(*o).is_token())));
    assert_eq!(targets_of(&t, spell), vec![obj(bears)]);
    assert_eq!(stack_triggers_from(&t, kykar).len(), 1);
    counterspell(&mut t, spell);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Spirit").len(), 1);
    assert!(t.on_battlefield(bears));
}
