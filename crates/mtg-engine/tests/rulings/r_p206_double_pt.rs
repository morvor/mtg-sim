//! Rulings batch P206 — doubling (and tripling) a creature's power and/or toughness
//! (CR 701.10a–c, 701.11): the creature gets +X/+0 (or +0/+X, or +X/+Y) where X is its
//! power as the spell or ability resolves; a negative value becomes more negative; the
//! amount is locked in.

use crate::r_p206_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s29_common::choose_modes;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[derive(Clone, Copy, PartialEq)]
enum What {
    Power,
    Toughness,
    Both,
}

/// One doubling ability: `setup` builds the board and returns the creature that will be
/// doubled; `fire` makes the ability happen and resolve.
struct Case {
    card: &'static str,
    players: usize,
    what: What,
    setup: fn(&mut TestGame) -> ObjectId,
    fire: fn(&mut TestGame, ObjectId),
}

fn giant(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Hill Giant")
}

fn spell(t: &mut TestGame, name: &str, subject: ObjectId) {
    cast_new(t, P0, name, &[Entity::Object(subject)]);
    t.resolve_all();
}

fn attack_alone(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
}

fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    rainbow_pool(t, p, 10);
    activate_containing(t, p, source, needle).expect("activation failed");
    t.resolve_all();
    empty_pool(t, p);
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            card: "Unleash Fury",
            players: 2,
            what: What::Power,
            setup: giant,
            fire: |t, s| spell(t, "Unleash Fury", s),
        },
        Case {
            card: "Bulk Up",
            players: 2,
            what: What::Power,
            setup: giant,
            fire: |t, s| spell(t, "Bulk Up", s),
        },
        Case {
            card: "Legion Leadership // Legion Stronghold",
            players: 2,
            what: What::Power,
            setup: giant,
            fire: |t, s| spell(t, "Legion Leadership // Legion Stronghold", s),
        },
        Case {
            card: "Double Trouble",
            players: 2,
            what: What::Power,
            setup: giant,
            fire: |t, _| {
                cast_new(t, P0, "Double Trouble", &[]);
                t.resolve_all();
            },
        },
        Case {
            card: "Choose Your Weapon",
            players: 2,
            what: What::Both,
            setup: giant,
            fire: |t, s| {
                choose_modes(t, P0, &[0]);
                spell(t, "Choose Your Weapon", s);
            },
        },
        Case {
            card: "Exponential Growth",
            players: 2,
            what: What::Power,
            setup: giant,
            fire: |t, s| {
                lands_for_cost(t, P0, "Exponential Growth");
                t.lands(P0, "Wastes", 2);
                let c = t.hand(P0, "Exponential Growth");
                t.cast(P0, c).x(1).target(s).go();
                t.resolve_all();
            },
        },
        Case {
            card: "Tifa's Limit Break",
            players: 2,
            what: What::Both,
            setup: giant,
            fire: |t, s| {
                choose_modes(t, P0, &[1]);
                t.lands(P0, "Wastes", 2);
                spell(t, "Tifa's Limit Break", s);
            },
        },
        Case {
            card: "Reckless Amplimancer",
            players: 2,
            what: What::Both,
            setup: |t| t.battlefield(P0, "Reckless Amplimancer"),
            fire: |t, s| activate(t, P0, s, "Double"),
        },
        Case {
            card: "Casey Jones, Asphalt Hooligan",
            players: 2,
            what: What::Power,
            setup: |t| t.battlefield(P0, "Casey Jones, Asphalt Hooligan"),
            fire: |t, s| activate(t, P0, s, "Double"),
        },
        Case {
            card: "Junk Jet",
            players: 2,
            what: What::Power,
            setup: |t| {
                let g = giant(t);
                attach_new(t, P0, "Junk Jet", g);
                t.battlefield(P0, "Ornithopter");
                g
            },
            fire: |t, _| {
                let jet = t.named_on_battlefield("Junk Jet")[0];
                activate(t, P0, jet, "Double");
            },
        },
        Case {
            card: "Zopandrel, Hunger Dominus",
            players: 2,
            what: What::Both,
            setup: |t| t.battlefield(P0, "Zopandrel, Hunger Dominus"),
            fire: |t, _| {
                t.advance_to(P0, Step::BeginningOfCombat);
                t.resolve_all();
            },
        },
        Case {
            card: "Grunn, the Lonely King",
            players: 2,
            what: What::Both,
            setup: |t| t.battlefield(P0, "Grunn, the Lonely King"),
            fire: attack_alone,
        },
        Case {
            card: "Nylea's Colossus",
            players: 2,
            what: What::Both,
            setup: |t| {
                t.battlefield(P0, "Nylea's Colossus");
                giant(t)
            },
            fire: |t, s| {
                t.answer_targets(P0, &[Entity::Object(s)]);
                t.enter(P0, "Ghostly Prison");
                t.resolve_all();
            },
        },
        Case {
            card: "Okaun, Eye of Chaos",
            players: 2,
            what: What::Both,
            setup: |t| t.battlefield(P0, "Okaun, Eye of Chaos"),
            fire: |t, _| {
                // Flip until a flip is lost: one win, then a loss.
                t.g.dice.loaded_coins.extend([true, false]);
                t.advance_to(P0, Step::BeginningOfCombat);
                t.resolve_all();
            },
        },
        Case {
            card: "Tifa Lockhart",
            players: 2,
            what: What::Power,
            setup: |t| t.battlefield(P0, "Tifa Lockhart"),
            fire: |t, _| {
                t.enter(P0, "Forest");
                t.resolve_all();
            },
        },
        Case {
            card: "Mightform Harmonizer",
            players: 2,
            what: What::Power,
            setup: |t| {
                t.battlefield(P0, "Mightform Harmonizer");
                giant(t)
            },
            fire: |t, s| {
                t.answer_targets(P0, &[Entity::Object(s)]);
                t.enter(P0, "Forest");
                t.resolve_all();
            },
        },
        Case {
            card: "Devilish Valet",
            players: 2,
            what: What::Power,
            setup: |t| t.battlefield(P0, "Devilish Valet"),
            fire: |t, _| {
                t.enter(P0, "Grizzly Bears");
                t.resolve_all();
            },
        },
        Case {
            card: "Death Kiss",
            players: 3,
            what: What::Power,
            setup: |t| {
                t.battlefield(P0, "Death Kiss");
                t.battlefield(P1, "Hill Giant")
            },
            fire: |t, s| {
                // P1's creature attacks P2, one of P0's opponents.
                t.set_step(P1, Step::BeginningOfCombat);
                attack_with(t, &[(s, Entity::Player(P2))]);
                t.resolve_all();
            },
        },
        Case {
            card: "Mr. Orfeo, the Boulder",
            players: 2,
            what: What::Power,
            setup: |t| {
                t.battlefield(P0, "Mr. Orfeo, the Boulder");
                giant(t)
            },
            fire: |t, s| {
                t.answer_targets(P0, &[Entity::Object(s)]);
                attack_alone(t, s);
            },
        },
        Case {
            card: "Two-Handed Axe // Sweeping Cleave",
            players: 2,
            what: What::Power,
            setup: |t| {
                let g = giant(t);
                attach_new(t, P0, "Two-Handed Axe // Sweeping Cleave", g);
                g
            },
            fire: attack_alone,
        },
        Case {
            card: "Thrakkus the Butcher",
            players: 2,
            what: What::Power,
            setup: |t| t.battlefield(P0, "Thrakkus the Butcher"),
            fire: attack_alone,
        },
        Case {
            card: "Rasaad yn Bashir",
            players: 2,
            what: What::Toughness,
            setup: |t| {
                t.g.initiative = Some(P0);
                t.battlefield(P0, "Rasaad yn Bashir")
            },
            fire: attack_alone,
        },
    ]
}

/// The P/T after doubling `(p, tough)` as `what` says.
fn doubled(what: What, (p, tough): (i32, i32)) -> (i32, i32) {
    match what {
        What::Power => (2 * p, tough),
        What::Toughness => (p, 2 * tough),
        What::Both => (2 * p, 2 * tough),
    }
}

fn run_case(c: &Case, negative: bool) {
    supported(c.card);
    let mut t = TestGame::new(c.players);
    let s = (c.setup)(&mut t);
    if negative {
        // Make its power -2.
        let (p, _) = t.pt(s);
        pump(&mut t, s, -(p + 2), 0);
        assert_eq!(t.pt(s).0, -2, "{}", c.card);
    }
    let before = t.pt(s);
    (c.fire)(&mut t, s);
    let after = doubled(c.what, before);
    assert_eq!(t.pt(s), after, "{}: doubling {before:?}", c.card);
    // The amount was locked in as the ability resolved: a later +1/+1 isn't doubled.
    pump(&mut t, s, 1, 1);
    assert_eq!(t.pt(s), (after.0 + 1, after.1 + 1), "{}", c.card);
}

#[test]
fn doubling_power_gives_plus_x_where_x_is_its_power_as_the_ability_resolves() {
    cr!("701.10a", "701.10b");
    ruling!(
        "Unleash Fury",
        "If an effect instructs you to \"double\" a creature's power, that creature gets +X/+0, where X is its power as that effect begins to apply. If its power is negative, instead it gets -X/-0 where X is how far below 0 its power is. The value of X won't change if another effect alters the creature's power later in the turn."
    );
    ruling!(
        "Zopandrel, Hunger Dominus",
        "If an effect instructs you to \"double\" a creature's power, that creature gets +X/+0, where X is its power as that effect begins to apply. Similarly, a creature whose toughness is doubled gets +0/+X, where X is its toughness as the effect begins to apply."
    );
    ruling!(
        "Grunn, the Lonely King",
        "If an effect instructs you to \"double\" a creature's power, that creature gets +X/+0, where X is its power as that effect begins to apply. The same is true for its toughness."
    );
    ruling!(
        "Choose Your Weapon",
        "If an effect instructs you to “double” a creature's power, that creature gets +X/+0, where X is its power as that effect begins to apply. The same is true for toughness."
    );
    ruling!(
        "Exponential Growth",
        "If an effect instructs you to “double” a creature’s power, that creature gets +X/+0, where X is its power as that effect begins to apply. (This is not the value of X you chose when you cast the spell)"
    );
    ruling!(
        "Nylea's Colossus",
        "If an effect instructs you to “double” a creature’s power, that creature gets +X/+0, where X is its power as that effect begins to apply. Toughness is doubled similarly."
    );
    ruling!(
        "Okaun, Eye of Chaos",
        "If an effect instructs you to “double” a creature’s power, that creature gets +X/+0, where X is its power. The same is true for its toughness."
    );
    ruling!(
        "Casey Jones, Asphalt Hooligan",
        "To double Casey Jones's power, he gets +X/+0, where X is his power when his last ability resolves."
    );
    ruling!(
        "Tifa Lockhart",
        "To double Tifa Lockhart's power, it gets +X/+0, where X is Tifa Lockhart's power when the landfall ability resolves."
    );
    ruling!(
        "Tifa's Limit Break",
        "To double a creature's power and toughness, that creature gets +X/+Y, where X is that creature's power and Y is that creature's toughness when Tifa's Limit Break resolves. To triple a creature's power, that creature gets +X/+Y, where X is twice that creature's power and Y is twice that creature's toughness when Tifa's Limit Break resolves."
    );
    ruling!(
        "Devilish Valet",
        "To double a creature's power means that creature gets +X/+0, where X is its power as the ability resolves."
    );
    ruling!(
        "Death Kiss",
        "To double a creature's power, it gets +X/+0, where X is its power as that ability resolves."
    );
    ruling!(
        "Double Trouble",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power as Double Trouble resolves."
    );
    ruling!(
        "Mr. Orfeo, the Boulder",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power as Mr. Orfeo's triggered ability resolves."
    );
    ruling!(
        "Two-Handed Axe // Sweeping Cleave",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power as the triggered ability resolves."
    );
    ruling!(
        "Bulk Up",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power when Bulk Up resolves."
    );
    ruling!(
        "Legion Leadership // Legion Stronghold",
        "To double a creature's power, that creature gets +X/+0, where X is that creature's power when Legion Leadership resolves."
    );
    ruling!(
        "Thrakkus the Butcher",
        "To double a creature's power, that creature gets +X/+0, where X is the power of that creature as the triggered ability resolves."
    );
    ruling!(
        "Rasaad yn Bashir",
        "To double a creature's toughness, that creature gets +0/+X, where X is that creature's toughness as the triggered ability resolves."
    );
    for c in cases() {
        run_case(&c, false);
    }
}

#[test]
fn doubling_a_negative_power_gives_minus_x() {
    cr!("701.10c");
    ruling!(
        "Zopandrel, Hunger Dominus",
        "If a creature's power is less than 0 when it's doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given Bear Cub, a 2/2 creature, -4/-0 so that it's a -2/2 creature, doubling its power and toughness gives it -2/+2, and it becomes a -4/4 creature."
    );
    ruling!(
        "Grunn, the Lonely King",
        "If a creature's power is less than 0 when it's doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given Grunn -7/-0 so that it's a -2/5 creature, doubling its power and toughness gives it -2/+5, and it's a -4/10 until end of turn."
    );
    ruling!(
        "Reckless Amplimancer",
        "If a creature's power is less than 0 when it's doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given Reckless Amplimancer -4/-0 so that it's a -2/2 creature, doubling its power and toughness gives it -2/+2, and it becomes a -4/4 creature."
    );
    ruling!(
        "Junk Jet",
        "If a creature's power is less than 0 when it's doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, suppose you control Infesting Radroach, a 2/2 creature. If an effect has given Infesting Radroach -4/-0 so that it's a -2/2 creature, doubling its power gives it -2/-0, and it becomes a -4/2 creature."
    );
    ruling!(
        "Mightform Harmonizer",
        "If a creature’s power is less than 0 when it’s doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given Bear Cub, a 2/2 creature, -4/-0 so that it’s a -2/2 creature, doubling its power and toughness gives it -2/+2, and it becomes a -4/4 creature."
    );
    ruling!(
        "Nylea's Colossus",
        "If a creature’s power is less than 0 when it’s doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given Nylea’s Colossus -8/-0 so that it’s a -2/6 creature, doubling its power and toughness gives it -2/+6, and it will become a -4/12 until end of turn."
    );
    ruling!(
        "Exponential Growth",
        "If a creature’s power is less than 0 when it’s doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. For example, if an effect has given a 5/5 creature -7/-0 so that it’s a -2/5 creature, doubling its power makes it a -4/5 creature."
    );
    ruling!(
        "Okaun, Eye of Chaos",
        "If a creature’s power is less than 0 when it’s doubled, instead that creature gets -X/-0, where X is how much less than 0 its power is. The same is true for its toughness. For example, if an effect has given Okaun -7/-0 so that its power and toughness are -4/3, doubling its power and toughness gives it -4/+3 and it’s -8/6 until end of turn."
    );
    for c in cases().iter().filter(|c| {
        [
            "Zopandrel, Hunger Dominus",
            "Grunn, the Lonely King",
            "Reckless Amplimancer",
            "Junk Jet",
            "Mightform Harmonizer",
            "Nylea's Colossus",
            "Exponential Growth",
            "Okaun, Eye of Chaos",
        ]
        .contains(&c.card)
    }) {
        run_case(c, true);
    }
}

#[test]
fn tripling_with_tifas_limit_break() {
    cr!("701.11b");
    ruling!(
        "Tifa's Limit Break",
        "To double a creature's power and toughness, that creature gets +X/+Y, where X is that creature's power and Y is that creature's toughness when Tifa's Limit Break resolves. To triple a creature's power, that creature gets +X/+Y, where X is twice that creature's power and Y is twice that creature's toughness when Tifa's Limit Break resolves."
    );
    supported("Tifa's Limit Break");
    let mut t = TestGame::new(2);
    let g = giant(&mut t);
    choose_modes(&mut t, P0, &[2]);
    add_mana(&mut t, P0, ManaType::G, 1);
    t.lands(P0, "Wastes", 6);
    spell(&mut t, "Tifa's Limit Break", g);
    assert_eq!(t.pt(g), (9, 9));
}

#[test]
fn exponential_growth_doubles_sequentially() {
    cr!("701.10b");
    ruling!(
        "Exponential Growth",
        "Exponential Growth doubles a creature’s power sequentially, not all at the same time. For example, if X is 3 and you target a 2/2 creature, you will double its power once to 4, again to 8, and a third time to 16, making it a 16/2 creature until end of turn."
    );
    supported("Exponential Growth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Exponential Growth");
    t.lands(P0, "Wastes", 6);
    let c = t.hand(P0, "Exponential Growth");
    t.cast(P0, c).x(3).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (16, 2));
}

#[test]
fn junk_jet_doublings_apply_independently() {
    cr!("701.10b");
    ruling!(
        "Junk Jet",
        "If you \"double\" a creature's power more than once in a turn, each doubling effect applies independently. For example, suppose Junk Jet is attached to Infesting Radroach, a 2/2 creature. If you activate Junk Jet's second ability, Infesting Radroach will get +2/+0 when that ability resolves, making it a 4/2 creature. Doubling its power again will give it +4/+0, making it an 8/2 creature."
    );
    supported("Junk Jet");
    let mut t = TestGame::new(2);
    let roach = t.battlefield(P0, "Grizzly Bears");
    let jet = attach_new(&mut t, P0, "Junk Jet", roach);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    activate(&mut t, P0, jet, "Double");
    assert_eq!(t.pt(roach), (4, 2));
    activate(&mut t, P0, jet, "Double");
    assert_eq!(t.pt(roach), (8, 2));
}
