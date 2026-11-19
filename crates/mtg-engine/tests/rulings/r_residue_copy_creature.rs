//! Rulings residue of batches P020, P021 and P023 (creatures that copy, Clone exceptions,
//! copies entering with choices): what a copy copies. A copy copies exactly what was
//! printed on the original (its copiable values, CR 707.2) plus any copy effects that
//! applied to it (CR 707.3) and the copy's own exceptions (CR 707.9), and nothing else:
//! not whether it's tapped, its counters, what's attached to it, or the non-copy effects
//! that changed it. A copy of a token copies the original characteristics the effect that
//! created it gave it (CR 111.3, 707.2), and doesn't make the copy a token or stop it being
//! one (CR 111.1).
//!
//! The copiers are driven by [`copy_with`]: each card makes its copies of a given
//! creature in its own way (entering as a copy, becoming one, creating token copies).

#![allow(dead_code)]

use crate::r_p021_common::dressed_giant;
use crate::r_p023_common::{clone_of, cloned_angel, enter_copying};
use crate::r_s01_common::{attack_with, supported};
use crate::r_s18_common::lands_for;
use crate::r_s26_common::new_tokens;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

pub fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

/// The number of abilities the object has whose text contains `s`.
pub fn abilities_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(s))
        .count()
}

/// Casts `p`'s `spell` (paying `cost` with basic lands) with one target per entry of
/// `targets`, resolves everything, and returns the tokens `p` got.
pub fn cast_for_tokens(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    cost: &str,
    targets: &[Entity],
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, p, cost);
    let card = t.hand(p, spell);
    let before = t.g.battlefield.clone();
    let mut b = t.cast(p, card);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    t.resolve_all();
    new_tokens(t, p, &before)
}

/// Whose creature `card` copies in [`copy_with`]: P1's for the cards that copy an
/// opponent's creature, P0's otherwise.
pub fn copied_controller(card: &str) -> PlayerId {
    match card {
        "Benthic Anomaly" => P1,
        _ => P0,
    }
}

/// `card` (P0's) makes its copies of `what` (a creature on the battlefield controlled by
/// [`copied_controller`]) and returns them: the permanent that entered as or became a
/// copy, or the token copies it created. Every choice picks `what`.
pub fn copy_with(t: &mut TestGame, card: &str, what: ObjectId) -> Vec<ObjectId> {
    supported(card);
    let what = t.g.current(what);
    match card {
        // "You may have this creature enter as a copy of ..."
        "Altered Ego"
        | "Sakashima of a Thousand Faces"
        | "Wall of Stolen Identity"
        | "Hulking Metamorph"
        | "Spark Double"
        | "Cursed Mirror"
        | "Pirated Copy"
        | "Auton Soldier" => {
            // Cursed Mirror asks "may" first.
            t.answer_yes(P0, true);
            let c = enter_copying(t, P0, card, what);
            t.resolve_all();
            vec![t.g.current(c)]
        }
        // "... with mana value less than or equal to the amount of mana spent to cast
        // this creature": cast with X = 5 (six mana).
        "Mockingbird" => {
            t.lands(P0, "Island", 6);
            let c = t.hand(P0, card);
            t.answer_choose(P0, &[obj(what)]);
            t.cast(P0, c).x(5).go();
            t.resolve_all();
            vec![t.g.current(c)]
        }
        // "At the beginning of combat on your turn, you may have Gogo become a copy of
        // another target creature you control until end of turn, except its name is Gogo,
        // Mysterious Mime."
        "Gogo, Mysterious Mime" => {
            let gogo = t.battlefield(P0, card);
            t.set_step(P0, Step::PrecombatMain);
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[obj(what)]);
            t.advance_to_step(Step::BeginningOfCombat);
            t.resolve_all();
            vec![gogo]
        }
        // "At the beginning of combat on your turn, this creature explores. Then you may
        // have it become a copy of another creature you control until end of turn." It
        // explores a land (no counter).
        "Deepfathom Echo" => {
            let echo = t.battlefield(P0, card);
            t.library_top(P0, "Island");
            t.set_step(P0, Step::PrecombatMain);
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(what)]);
            t.advance_to_step(Step::BeginningOfCombat);
            t.resolve_all();
            assert_eq!(t.counters(echo, counters::PLUS1), 0);
            vec![echo]
        }
        // "At the beginning of combat on your turn, you may pay {E}{E}. When you do, this
        // creature becomes a copy of another target creature you control, except ..."
        "Aurora Shifter" => {
            let s = t.battlefield(P0, card);
            t.g.add_counters(Entity::Player(P0), counters::ENERGY, 2, None);
            t.set_step(P0, Step::PrecombatMain);
            t.answer_yes(P0, true);
            t.answer_targets(P0, &[obj(what)]);
            t.advance_to_step(Step::BeginningOfCombat);
            t.resolve_all();
            vec![s]
        }
        // "Choose any number of target creatures you control. For each of them, create a
        // token that's a copy of that creature, except it has haste."
        "Twinflame" => cast_for_tokens(t, P0, card, "{1}{R}", &[obj(what)]),
        // "When you cast this spell, for each opponent, choose a creature that player
        // controls. Create a token that's a copy of one of those creatures, except ..."
        "Benthic Anomaly" => {
            lands_for(t, P0, "{6}{U}");
            let c = t.hand(P0, card);
            let before = t.g.battlefield.clone();
            t.answer_choose(P0, &[obj(what)]);
            t.cast(P0, c).go();
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        // "At the beginning of combat on your turn, create X tokens that are copies of
        // another target creature you control, where X is one plus the number of instant
        // and sorcery spells you've cast this turn."
        "Rionya, Fire Dancer" => {
            t.battlefield(P0, card);
            t.set_step(P0, Step::PrecombatMain);
            t.answer_targets(P0, &[obj(what)]);
            let before = t.g.battlefield.clone();
            t.advance_to_step(Step::BeginningOfCombat);
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        // Equipped to the creature: "the first time you would create one or more tokens
        // each turn, you may instead create that many tokens that are copies of equipped
        // creature." Raise the Alarm would create two Soldiers.
        "Mirrormind Crown" => {
            crate::r_s06_common::attach_new(t, P0, card, what);
            t.answer_yes(P0, true);
            cast_for_tokens(t, P0, "Raise the Alarm", "{1}{W}", &[])
        }
        // "Choose a nonbasic land type. Each land you control of that type becomes a copy
        // of target creature you control until end of turn and gains haste until end of
        // turn." An Azorius Guildgate, choosing Gate.
        "March from Velis Vel" => {
            let gate = t.battlefield(P0, "Azorius Guildgate");
            lands_for(t, P0, "{2}{U}");
            let m = t.hand(P0, card);
            t.cast(P0, m).target(obj(what)).go();
            t.answer(P0, DecisionKind::Option, Answer::Index(2));
            t.resolve_all();
            vec![gate]
        }
        // Chapter I: "Choose target nontoken creature you control that doesn't have the
        // same name as a token you control. Create a token that's a copy of it, except
        // ..."
        "The Apprentice's Folly" => {
            t.answer_targets(P0, &[obj(what)]);
            let before = t.g.battlefield.clone();
            t.enter(P0, card);
            t.settle();
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        // "When this creature enters from your hand, if it's attacking, create a token
        // that's a copy of another target attacking creature. The token enters tapped and
        // attacking."
        "Thousand-Faced Shadow" => {
            // Ninjutsu: `what` and a Grizzly Bears attack unblocked; the Bears are
            // returned and the Shadow enters attacking, its trigger copying `what`.
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.lands(P0, "Island", 4);
            let shadow = t.hand(P0, card);
            let at = Entity::Player(P1);
            crate::r_p021_common::untap(t, what);
            crate::r_s03_common::to_blockers(t, &[(bears, at), (what, at)], &[]);
            let before = t.g.battlefield.clone();
            ninjutsu(t, P0, shadow, bears);
            t.answer_targets(P0, &[obj(what)]);
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        // "Destroy target creature. If that creature dies this way, its controller creates
        // two tokens that are copies of that creature, except their power is half that
        // creature's power and their toughness is half that creature's toughness. Round
        // up each time."
        "Saw in Half" => cast_for_tokens(t, P0, card, "{2}{B}", &[obj(what)]),
        // "Whenever another nontoken creature you control dies, you may pay 2 life and
        // exile it. If you do, create a token that's a copy of that creature, except it's
        // 1/1 and has toxic 1."
        "Kinzu of the Bleak Coven" => {
            t.battlefield(P0, card);
            t.answer_yes(P0, true);
            let before = t.g.battlefield.clone();
            crate::r_s02_common::destroy(t, what);
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        // "For each player, choose friend or foe. Each friend creates a token that's a
        // copy of a creature they control. ..." P0 is a friend, P1 a foe.
        "Zndrsplt's Judgment" => {
            friend_or_foe(t, P0, true);
            friend_or_foe(t, P0, false);
            t.answer_choose(P0, &[obj(what)]);
            cast_for_tokens(t, P0, card, "{4}{U}", &[])
        }
        // Chapter III: "Create a token that's a copy of target creature, except it's a
        // legendary Alien named Prisoner Zero."
        "The Eleventh Hour" => {
            let saga = t.enter(P0, card);
            t.settle();
            t.resolve_all();
            t.g.add_counters(obj(t.g.current(saga)), counters::LORE, 1, None);
            t.g.flush_events();
            t.resolve_all();
            let before = t.g.battlefield.clone();
            t.answer_targets(P0, &[obj(what)]);
            t.g.add_counters(obj(t.g.current(saga)), counters::LORE, 1, None);
            t.g.flush_events();
            t.resolve_all();
            new_tokens(t, P0, &before)
        }
        other => panic!("no copy driver for {other}"),
    }
}

/// What a copy of a dressed-up permanent should look like: its printed name, colors and
/// subtype, as changed by the copy's exceptions.
pub struct Printed {
    pub name: &'static str,
    pub colors: ColorSet,
    pub subtype: &'static str,
}

/// Hill Giant: a red 3/3 Giant.
pub fn giant() -> Printed {
    Printed {
        name: "Hill Giant",
        colors: ColorSet::single(Color::Red),
        subtype: "Giant",
    }
}

/// Myr Enforcer: a colorless 7/7 Myr artifact creature.
pub fn enforcer() -> Printed {
    Printed {
        name: "Myr Enforcer",
        colors: ColorSet::NONE,
        subtype: "Myr",
    }
}

/// Asserts that `id` copies only the printed values `p` of a dressed-up permanent (see
/// [`dressed_giant`]): its name, colors and subtype, power and toughness `pt` (after the
/// copy's exceptions), no attachments, and (unless `counters_ok`) no counters.
pub fn assert_printed_copy(
    t: &TestGame,
    card: &str,
    id: ObjectId,
    p: &Printed,
    pt: (i32, i32),
    counters_ok: bool,
) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, p.name, "{card}");
    assert!(o.chars.has_subtype(p.subtype), "{card}: subtype");
    assert_eq!(o.chars.colors, p.colors, "{card}: colors");
    assert_eq!(t.pt(id), pt, "{card}: power and toughness");
    if !counters_ok {
        assert!(
            o.counters.values().all(|n| *n == 0),
            "{card}: counters {:?}",
            o.counters
        );
    }
    let attached: Vec<_> =
        t.g.attachments_of(Entity::Object(o.id))
            .into_iter()
            .filter(|a| t.obj_now(*a).chars.name != "Mirrormind Crown")
            .collect();
    assert!(attached.is_empty(), "{card}: attachments");
}

/// A dressed-up Hill Giant for `card` to copy (see [`dressed_giant`]: tapped, equipped,
/// two +1/+1 counters, +3/+3 and green-blue until end of turn).
pub fn dressed_giant_for(t: &mut TestGame, card: &str) -> ObjectId {
    dressed_giant(t, copied_controller(card))
}

/// Activates `ninja`'s ninjutsu ability, returning `returned` (an unblocked attacker).
pub fn ninjutsu(t: &mut TestGame, p: PlayerId, ninja: ObjectId, returned: ObjectId) {
    t.g.recompute();
    let index = t
        .obj(ninja)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with("Ninjutsu"))
        .expect("a ninjutsu ability");
    t.answer_choose(p, &[Entity::Object(returned)]);
    t.activate(p, ninja, index, &[]).expect("ninjutsu");
}

/// The answer that picks friend (0) or foe (1) for one player in Zndrsplt's Judgment.
pub fn friend_or_foe(t: &mut TestGame, p: PlayerId, friend: bool) {
    t.answer(
        p,
        DecisionKind::Option,
        Answer::Index(if friend { 0 } else { 1 }),
    );
}

/// P0's attack (from the beginning of combat) with `attackers` at P1; leaves the game in
/// the declare attackers step with the triggers resolved.
pub fn attack_p1(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|id| (t.g.current(*id), Entity::Player(P1)))
        .collect();
    attack_with(t, &a);
    t.resolve_all();
}

/// A Clone controlled by `p` copying `p`'s Serra Angel.
pub fn angel_clone(t: &mut TestGame, p: PlayerId) -> ObjectId {
    if p == P0 {
        cloned_angel(t, P0)
    } else {
        let angel = t.battlefield(p, "Serra Angel");
        clone_of(t, p, angel)
    }
}

fn legendary(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.is_legendary()
}

fn is_artifact(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Artifact)
}

/// One case of [`printed_cases`]: the copier, what its copies look like (printed values
/// after its exceptions), their power and toughness, how many it makes, and a check of
/// its other exceptions.
struct PrintedCase {
    card: &'static str,
    printed: Printed,
    pt: (i32, i32),
    n: usize,
    check: fn(&TestGame, ObjectId) -> bool,
}

fn none(_: &TestGame, _: ObjectId) -> bool {
    true
}

/// Runs each case: `card` copies a dressed-up Hill Giant; each copy has only its printed
/// values (and the exceptions), is untapped, has no counters and nothing attached; the
/// Giant keeps its own dressing.
fn printed_cases(cases: &[PrintedCase]) {
    for c in cases {
        let mut t = TestGame::new(2);
        let giant = dressed_giant_for(&mut t, c.card);
        let copies = copy_with(&mut t, c.card, giant);
        assert_eq!(copies.len(), c.n, "{}: {}", c.card, t.dump_log());
        for id in copies {
            assert_printed_copy(&t, c.card, id, &c.printed, c.pt, false);
            assert!(!t.obj_now(id).tapped, "{}: tapped", c.card);
            assert!((c.check)(&t, id), "{}: exceptions", c.card);
        }
        if t.on_battlefield(giant) {
            assert_eq!(t.counters(giant, counters::PLUS1), 2, "{}", c.card);
        }
    }
}

#[test]
fn copies_copy_only_the_printed_creature() {
    cr!("707.2", "707.9a", "707.9b", "614.1c", "611.2c");
    ruling!(
        "Altered Ego",
        "Altered Ego copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "Cursed Mirror",
        "Cursed Mirror copies exactly what was printed on the original creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras or Equipment attached to it, or any non-copy effects that have changed its types, color, power and toughness, and so on."
    );
    ruling!(
        "Deepfathom Echo",
        "Deepfathom Echo copies exactly what is printed on the chosen creature (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "Gogo, Mysterious Mime",
        "Gogo copies exactly what was printed on the original creature, with the listed exception (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or any Auras or Equipment attached to it, or any non-copy effects that have changed its types, color, power and toughness, and so on."
    );
    ruling!(
        "Hulking Metamorph",
        "Except for also being an artifact creature and having a different power and toughness, Hulking Metamorph copies exactly what was printed on the original permanent and nothing more (unless that permanent is itself copying something or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, or any noncopy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "The Eleventh Hour",
        "Except for its name and types, the token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or it is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, and so on."
    );
    ruling!(
        "Saw in Half",
        "Except for power and toughness, the tokens copy exactly what was printed on the original creature and nothing else (unless that creature was copying something else or it's a token; see below). They don't copy whether that creature was tapped or untapped, whether it had any counters on it or Auras and Equipment attached to it, and so on."
    );
    ruling!(
        "Kinzu of the Bleak Coven",
        "Except for the listed exceptions, the token copies exactly what was printed on the original creature and nothing else (unless that creature is copying something else; see below). It doesn't copy whether that creature was tapped or untapped, whether it had any counters on it or Auras or Equipment attached to it, or any non-copy effects that had changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "The Apprentice's Folly",
        "Except for the listed exceptions, the token copies exactly what was printed on the original creature and nothing else (unless that permanent is itself copying something else; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Zndrsplt's Judgment",
        "Each token copies exactly what is printed on the original creature and nothing else (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    let gogo = Printed {
        name: "Gogo, Mysterious Mime",
        ..giant()
    };
    let zero = Printed {
        name: "Prisoner Zero",
        subtype: "Alien",
        ..giant()
    };
    printed_cases(&[
        PrintedCase {
            card: "Altered Ego",
            printed: giant(),
            pt: (3, 3),
            n: 1,
            check: none,
        },
        PrintedCase {
            card: "Cursed Mirror",
            printed: giant(),
            pt: (3, 3),
            n: 1,
            check: |t, id| has_kw(t, id, KeywordKind::Haste),
        },
        PrintedCase {
            card: "Deepfathom Echo",
            printed: giant(),
            pt: (3, 3),
            n: 1,
            check: none,
        },
        // A 3/3 named Gogo, plus Gogo's own +2/+0 (a non-copy effect that began after).
        PrintedCase {
            card: "Gogo, Mysterious Mime",
            printed: gogo,
            pt: (5, 3),
            n: 1,
            check: |t, id| has_kw(t, id, KeywordKind::Haste),
        },
        PrintedCase {
            card: "Hulking Metamorph",
            printed: giant(),
            pt: (7, 7),
            n: 1,
            check: |t, id| is_artifact(t, id),
        },
        // A legendary Alien named Prisoner Zero: it loses the Giant subtype.
        PrintedCase {
            card: "The Eleventh Hour",
            printed: zero,
            pt: (3, 3),
            n: 1,
            check: |t, id| legendary(t, id) && !has_subtype(t, id, "Giant"),
        },
        // Half the dressed-up Giant's 10/8, rounded up: the exception uses its power and
        // toughness as it last existed, but the copy is of its printed values.
        PrintedCase {
            card: "Saw in Half",
            printed: giant(),
            pt: (5, 4),
            n: 2,
            check: none,
        },
        PrintedCase {
            card: "Kinzu of the Bleak Coven",
            printed: giant(),
            pt: (1, 1),
            n: 1,
            check: |t, id| has_kw(t, id, KeywordKind::Toxic),
        },
        PrintedCase {
            card: "The Apprentice's Folly",
            printed: giant(),
            pt: (3, 3),
            n: 1,
            check: |t, id| has_subtype(t, id, "Reflection") && has_kw(t, id, KeywordKind::Haste),
        },
        PrintedCase {
            card: "Zndrsplt's Judgment",
            printed: giant(),
            pt: (3, 3),
            n: 1,
            check: none,
        },
    ]);
}

// --- Copying a token: its original characteristics ----------------------------------------

/// One case of [`token_cases`]: the copier, whether its copies are tokens, the copies'
/// name (`None`: Wolf), color (`None`: green), power and toughness, how many there are,
/// and a check of the copier's other exceptions.
struct TokenCase {
    card: &'static str,
    token: bool,
    name: Option<&'static str>,
    colorless: bool,
    pt: (i32, i32),
    n: usize,
    check: fn(&TestGame, ObjectId) -> bool,
}

/// Runs each case: `card` copies a dressed-up 2/2 green Wolf token (tapped and untapped
/// again, two +1/+1 counters, +3/+3 and green-blue until end of turn: a 7/7); each copy
/// has the Wolf's original characteristics as its creating effect stated them, with the
/// copier's exceptions, and is a token only if the copier made a token.
fn token_cases(cases: &[TokenCase]) {
    for c in cases {
        let mut t = TestGame::new(2);
        let wolf = crate::r_p023_common::dressed_wolf(&mut t, copied_controller(c.card));
        let copies = copy_with(&mut t, c.card, wolf);
        assert_eq!(copies.len(), c.n, "{}: {}", c.card, t.dump_log());
        for id in copies {
            let o = t.obj_now(id);
            assert_eq!(o.chars.name, c.name.unwrap_or("Wolf"), "{}", c.card);
            // (Benthic Anomaly's token is "a colorless Eldrazi creature".)
            assert_eq!(
                o.chars.has_subtype("Wolf"),
                !c.colorless,
                "{}: subtype",
                c.card
            );
            let colors = if c.colorless {
                ColorSet::NONE
            } else {
                ColorSet::single(Color::Green)
            };
            assert_eq!(o.chars.colors, colors, "{}: colors", c.card);
            assert_eq!(t.pt(id), c.pt, "{}: power and toughness", c.card);
            assert_eq!(o.is_token(), c.token, "{}: token", c.card);
            assert!((c.check)(&t, id), "{}: exceptions", c.card);
        }
    }
}

fn no_counters(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).counters.values().all(|n| *n == 0)
}

#[test]
fn copies_of_a_token_copy_its_original_characteristics() {
    cr!("707.2", "111.3", "111.1", "707.9a", "707.9b");
    ruling!(
        "Altered Ego",
        "If the chosen creature is a token, Altered Ego copies the original characteristics of that token as stated by the effect that created that token. Altered Ego isn't a token."
    );
    ruling!(
        "Sakashima of a Thousand Faces",
        "If the chosen creature is a token, Sakashima copies the original characteristics of that token as stated by the effect that put that token onto the battlefield. Sakashima doesn't become a token in this case."
    );
    ruling!(
        "Wall of Stolen Identity",
        "If the chosen creature is a token, Wall of Stolen Identity copies the original characteristics of that token as stated by the effect that created the token. Wall of Stolen Identity doesn’t become a token in this case."
    );
    ruling!(
        "Hulking Metamorph",
        "If the chosen permanent is a token, Hulking Metamorph copies the original characteristics of that token as stated by the effect that put the token onto the battlefield, with the listed exceptions. Hulking Metamorph does not become a token."
    );
    ruling!(
        "Spark Double",
        "If the chosen permanent is a token, Spark Double copies the original characteristics of that token as stated by the effect that created the token. Spark Double doesn’t become a token in this case."
    );
    ruling!(
        "Deepfathom Echo",
        "If the copied creature is a token, Deepfathom Echo copies the original characteristics of that token as stated by the effect that created that token. Deepfathom Echo doesn't become a token (although it might already be one)."
    );
    ruling!(
        "Gogo, Mysterious Mime",
        "If the copied creature is a token, Gogo copies the original characteristics of that token as stated by the effect that created the token, with the listed exception."
    );
    ruling!(
        "Mockingbird",
        "If the copied creature is a token, Mockingbird copies the original characteristics of that token as stated by the effect that created that token, with the noted exceptions."
    );
    ruling!(
        "Thousand-Faced Shadow",
        "If the copied creature is a token, the token created by Thousand-Faced Shadow copies the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    ruling!(
        "Twinflame",
        "If the copied creature is a token, the token created by Twinflame copies the original characteristics of that token as stated by the effect that put that token onto the battlefield."
    );
    ruling!(
        "Benthic Anomaly",
        "If the copied creature is itself a token, the token created by Benthic Anomaly's ability copies the original characteristics of that token as stated by the effect that created it, with the listed exception."
    );
    ruling!(
        "Rionya, Fire Dancer",
        "If the copied creature is itself a token, the tokens created by Rionya copy the original characteristics of that token as stated by the effect that created it."
    );
    let haste: fn(&TestGame, ObjectId) -> bool = |t, id| has_kw(t, id, KeywordKind::Haste);
    let card = |card: &'static str,
                token: bool,
                pt: (i32, i32),
                check: fn(&TestGame, ObjectId) -> bool| TokenCase {
        card,
        token,
        name: None,
        colorless: false,
        pt,
        n: 1,
        check,
    };
    token_cases(&[
        card("Altered Ego", false, (2, 2), no_counters),
        card("Sakashima of a Thousand Faces", false, (2, 2), |t, id| {
            abilities_with(t, id, "legend rule") == 1
        }),
        card("Wall of Stolen Identity", false, (2, 2), |t, id| {
            has_subtype(t, id, "Wall") && has_kw(t, id, KeywordKind::Defender)
        }),
        card("Hulking Metamorph", false, (7, 7), |t, id| {
            is_artifact(t, id)
        }),
        // Its exception: an additional +1/+1 counter.
        card("Spark Double", false, (3, 3), |t, id| {
            t.counters(id, counters::PLUS1) == 1
        }),
        card("Deepfathom Echo", false, (2, 2), no_counters),
        // Named Gogo; with Gogo's +2/+0.
        TokenCase {
            name: Some("Gogo, Mysterious Mime"),
            ..card("Gogo, Mysterious Mime", false, (4, 2), haste)
        },
        card("Mockingbird", false, (2, 2), |t, id| {
            has_subtype(t, id, "Bird") && has_kw(t, id, KeywordKind::Flying)
        }),
        card("Thousand-Faced Shadow", true, (2, 2), |t, id| {
            t.obj_now(id).tapped
        }),
        card("Twinflame", true, (2, 2), haste),
        // The total power and toughness of the chosen creatures (the 7/7 Wolf as it is);
        // a colorless Eldrazi.
        TokenCase {
            colorless: true,
            ..card("Benthic Anomaly", true, (7, 7), |t, id| {
                has_subtype(t, id, "Eldrazi")
            })
        },
        card("Rionya, Fire Dancer", true, (2, 2), haste),
    ]);
}

#[test]
fn cursed_mirror_copying_a_token_isnt_a_token_and_a_token_mirror_stays_one() {
    cr!("707.2", "111.1", "614.1c");
    ruling!(
        "Cursed Mirror",
        "If the copied creature is a token, Cursed Mirror copies the original characteristics of that token as stated by the effect that created the token. Copying a token doesn't make Cursed Mirror a token. Similarly, if Cursed Mirror itself is a token, copying a nontoken permanent doesn't make it stop being a token."
    );
    // The first part: as in the table above.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::dressed_wolf(&mut t, P0);
    let m = copy_with(&mut t, "Cursed Mirror", wolf)[0];
    assert_eq!(name_of(&t, m), "Wolf");
    assert_eq!(t.pt(m), (2, 2));
    assert!(!t.obj_now(m).is_token());
    // A token copy of Cursed Mirror (Saheeli Rai's −2: "Create a token that's a copy of
    // target artifact or creature you control, except it's an artifact in addition to its
    // other types.") enters as a copy of a nontoken Hill Giant: it's still a token.
    supported("Saheeli Rai");
    let mut t = TestGame::new(2);
    let mirror = t.battlefield(P0, "Cursed Mirror");
    let giant = t.battlefield(P0, "Hill Giant");
    let saheeli = t.battlefield(P0, "Saheeli Rai");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(mirror)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    crate::r_s06_common::activate_containing(&mut t, P0, saheeli, "Create a token")
        .expect("Saheeli's −2");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert!(t.obj_now(toks[0]).is_token());
    assert!(has_kw(&t, toks[0], KeywordKind::Haste));
    // The later copy effect overwrote the first one's exception (CR 613.2a, 707.9b).
    assert!(!is_artifact(&t, toks[0]));
}

// --- Copying a copy: whatever it copied ---------------------------------------------------

/// One case of [`copy_of_copy_cases`]: the copier, the copies' name (`None`: Serra Angel),
/// power and toughness, how
/// many there are, whether they're colorless (else white), and a check of the copier's
/// other exceptions.
struct CopyCase {
    card: &'static str,
    name: Option<&'static str>,
    pt: (i32, i32),
    n: usize,
    colorless: bool,
    check: fn(&TestGame, ObjectId) -> bool,
}

/// Runs each case: `card` copies a Clone that entered as a copy of Serra Angel; each copy
/// is a Serra Angel (a white Angel with flying and vigilance), with the copier's
/// exceptions, not a Clone.
fn copy_of_copy_cases(cases: &[CopyCase]) {
    for c in cases {
        let mut t = TestGame::new(2);
        let clone = angel_clone(&mut t, copied_controller(c.card));
        let copies = copy_with(&mut t, c.card, clone);
        assert_eq!(copies.len(), c.n, "{}: {}", c.card, t.dump_log());
        for id in copies {
            let o = t.obj_now(id);
            assert_eq!(o.chars.name, c.name.unwrap_or("Serra Angel"), "{}", c.card);
            // (Benthic Anomaly's token is "a colorless Eldrazi creature": its subtype
            // replaces Angel.)
            assert_eq!(o.chars.has_subtype("Angel"), !c.colorless, "{}", c.card);
            assert!(!o.chars.has_subtype("Shapeshifter"), "{}", c.card);
            let colors = if c.colorless {
                ColorSet::NONE
            } else {
                ColorSet::single(Color::White)
            };
            assert_eq!(o.chars.colors, colors, "{}: colors", c.card);
            assert!(
                o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Vigilance),
                "{}: keywords",
                c.card
            );
            assert_eq!(t.pt(id), c.pt, "{}: power and toughness", c.card);
            assert!((c.check)(&t, id), "{}: exceptions", c.card);
        }
    }
}

#[test]
fn copies_of_a_copy_copy_what_it_copied() {
    cr!("707.3", "707.2", "707.9a", "707.9b", "614.1c");
    ruling!(
        "Altered Ego",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Altered Ego), then your Altered Ego enters as whatever the chosen creature copied."
    );
    ruling!(
        "Wall of Stolen Identity",
        "If the chosen creature is copying something else (for example, if the chosen creature is another Wall of Stolen Identity), then Wall of Stolen Identity enters the battlefield as whatever the chosen creature copied."
    );
    ruling!(
        "Sakashima of a Thousand Faces",
        "If the chosen creature is copying something else, then Sakashima enters the battlefield as whatever the chosen creature copied."
    );
    ruling!(
        "Hulking Metamorph",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is a Clone), then your Hulking Metamorph enters the battlefield as whatever the chosen permanent copied, with the stated exceptions."
    );
    ruling!(
        "Spark Double",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is another Spark Double), then Spark Double enters the battlefield as whatever the chosen permanent copied."
    );
    ruling!(
        "Rionya, Fire Dancer",
        "If the copied creature is copying something else, the tokens enter the battlefield as whatever that creature is copying."
    );
    ruling!(
        "Aurora Shifter",
        "If the copied creature is copying something else, then Aurora Shifter becomes a copy of whatever that creature copied (with the listed exception)."
    );
    ruling!(
        "Cursed Mirror",
        "If the copied creature is copying something else, then Cursed Mirror enters the battlefield as a copy of whatever that creature is copying."
    );
    ruling!(
        "Gogo, Mysterious Mime",
        "If the copied creature is copying something else, then Gogo becomes a copy of whatever that creature is copying, with the listed exception."
    );
    ruling!(
        "Mockingbird",
        "If the copied creature is copying something else, then Mockingbird enters as whatever that creature copied, with the noted exceptions."
    );
    ruling!(
        "March from Velis Vel",
        "If the copied creature is copying something else, then the lands become copies of whatever that creature copied."
    );
    ruling!(
        "Benthic Anomaly",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied, with the listed exceptions."
    );
    ruling!(
        "The Apprentice's Folly",
        "If the copied creature is copying something else, then the token enters the battlefield as whatever that creature copied, with the stated exceptions."
    );
    ruling!(
        "Mirrormind Crown",
        "If the copied creature is copying something else, then the tokens enter as whatever that creature copied."
    );
    ruling!(
        "Deepfathom Echo",
        "If Deepfathom Echo becomes a copy of a creature that is copying something else, it becomes a copy of whatever that creature is a copy of."
    );
    let haste: fn(&TestGame, ObjectId) -> bool = |t, id| has_kw(t, id, KeywordKind::Haste);
    let card =
        |card: &'static str, pt: (i32, i32), check: fn(&TestGame, ObjectId) -> bool| CopyCase {
            card,
            name: None,
            pt,
            n: 1,
            colorless: false,
            check,
        };
    copy_of_copy_cases(&[
        card("Altered Ego", (4, 4), none),
        card("Wall of Stolen Identity", (4, 4), |t, id| {
            has_subtype(t, id, "Wall") && has_kw(t, id, KeywordKind::Defender)
        }),
        card("Sakashima of a Thousand Faces", (4, 4), |t, id| {
            abilities_with(t, id, "legend rule") == 1
        }),
        card("Hulking Metamorph", (7, 7), |t, id| is_artifact(t, id)),
        card("Spark Double", (5, 5), |t, id| {
            t.counters(id, counters::PLUS1) == 1
        }),
        card("Rionya, Fire Dancer", (4, 4), haste),
        // "except it has this ability and "Whenever this creature deals combat damage to a
        // player, you get that many {E}.""
        card("Aurora Shifter", (4, 4), |t, id| {
            abilities_with(t, id, "pay {E}{E}") == 1 && abilities_with(t, id, "that many {E}") >= 1
        }),
        card("Cursed Mirror", (4, 4), haste),
        // Named Gogo; with Gogo's +2/+0.
        CopyCase {
            name: Some("Gogo, Mysterious Mime"),
            ..card("Gogo, Mysterious Mime", (6, 4), haste)
        },
        card("Mockingbird", (4, 4), |t, id| has_subtype(t, id, "Bird")),
        card("March from Velis Vel", (4, 4), |t, id| {
            has_kw(t, id, KeywordKind::Haste) && t.obj_now(id).is(CardType::Creature)
        }),
        CopyCase {
            colorless: true,
            ..card("Benthic Anomaly", (4, 4), |t, id| {
                has_subtype(t, id, "Eldrazi")
            })
        },
        card("The Apprentice's Folly", (4, 4), |t, id| {
            has_subtype(t, id, "Reflection") && has_kw(t, id, KeywordKind::Haste)
        }),
        CopyCase {
            n: 2,
            ..card("Mirrormind Crown", (4, 4), none)
        },
        card("Deepfathom Echo", (4, 4), none),
    ]);
}

/// Changes `id` for good in the ways a copy doesn't copy (it lasts into the next turn):
/// equips it with Bonesplitter (for P0), puts two +1/+1 counters on it, gives it +3/+3 and
/// makes it green-blue.
fn dress_for_good(t: &mut TestGame, id: ObjectId, equip: bool) {
    use mtg_engine::ability::*;
    if equip {
        crate::r_s06_common::attach_new(t, P0, "Bonesplitter", id);
    }
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, 2, None);
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::ModifyPT(Value::c(3), Value::c(3)),
                Modification::SetColors(
                    ColorSet::single(Color::Green).union(ColorSet::single(Color::Blue)),
                ),
            ],
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.settle();
}

/// P0's Endless Evil ("At the beginning of your upkeep, create a token that's a copy of
/// enchanted creature, except the token is 1/1.") on `what`; P0's next upkeep comes.
/// Returns the tokens.
fn endless_evil(t: &mut TestGame, what: ObjectId) -> Vec<ObjectId> {
    supported("Endless Evil");
    crate::r_s06_common::attach_new(t, P0, "Endless Evil", what);
    let before = t.g.battlefield.clone();
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn endless_evil_copies_the_printed_creature_or_the_tokens_original_characteristics() {
    cr!("707.2", "707.9b", "111.3");
    ruling!(
        "Endless Evil",
        "Except for power and toughness, the token copies exactly what was printed on the original creature and nothing else (unless that permanent is copying something else or it is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, and so on."
    );
    ruling!(
        "Endless Evil",
        "If the copied creature is a token, the new token that's created copies the original characteristics of that token, with the exceptions noted above."
    );
    // A dressed-up Hill Giant (equipped, two +1/+1 counters, +3/+3 and green-blue, and
    // the Aura): a 1/1 red Hill Giant.
    let mut t = TestGame::new(2);
    let hill = t.battlefield(P0, "Hill Giant");
    dress_for_good(&mut t, hill, true);
    let toks = endless_evil(&mut t, hill);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(hill), (10, 8));
    assert_printed_copy(&t, "Endless Evil", toks[0], &giant(), (1, 1), false);
    // A dressed-up Wolf token: a 1/1 green Wolf token.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::wolf_token(&mut t, P0);
    dress_for_good(&mut t, wolf, false);
    let toks = endless_evil(&mut t, wolf);
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert_eq!(o.chars.name, "Wolf");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(o.is_token() && no_counters(&t, toks[0]));
    assert_eq!(t.pt(toks[0]), (1, 1));
}

#[test]
fn mirrormind_crown_copies_a_tokens_original_characteristics() {
    cr!("707.2", "111.3", "614.1a");
    ruling!(
        "Mirrormind Crown",
        "If the copied creature is a token, the tokens that are created copy the original characteristics of that token as stated by the effect that created that token."
    );
    // The Wolf token was created on an earlier turn (so the Crown's "first time you would
    // create one or more tokens each turn" is still to come), then dressed up.
    let mut t = TestGame::new(2);
    let wolf = crate::r_p023_common::wolf_token(&mut t, P0);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    crate::r_s26_common::dress_up(&mut t, wolf);
    let toks = copy_with(&mut t, "Mirrormind Crown", wolf);
    assert_eq!(toks.len(), 2, "{}", t.dump_log());
    for tok in toks {
        let o = t.obj_now(tok);
        assert_eq!(o.chars.name, "Wolf");
        assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
        assert!(o.is_token() && no_counters(&t, tok) && !o.tapped);
        assert_eq!(t.pt(tok), (2, 2));
    }
}
