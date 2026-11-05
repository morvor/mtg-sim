//! Rulings batch P027 — a permanent that may enter as a copy (CR 707.9, 614.1c, 614.12)
//! can copy nothing: it enters with its own printed characteristics, and none of the copy
//! effect's exceptions apply (no added abilities, types or counters, CR 707.9). A 0/0
//! one is put into its owner's graveyard the next time state-based actions are checked
//! (CR 704.5f) unless something raises its toughness.

use crate::r_p027_common::*;
use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// What a creature that can copy needs to have something to copy.
#[derive(Clone, Copy)]
enum Offer {
    /// A Grizzly Bears controlled by P0 and one controlled by P1.
    Bears,
    /// An Expedition Envoy (a 2/1 Human Scout Ally) controlled by P0.
    Ally,
    /// A Grizzly Bears that died this turn (a creature card in P0's graveyard put there
    /// from the battlefield this turn).
    DeadBears,
}

fn offer(t: &mut TestGame, o: Offer) -> Vec<ObjectId> {
    match o {
        Offer::Bears => vec![
            t.battlefield(P1, "Grizzly Bears"),
            t.battlefield(P0, "Grizzly Bears"),
        ],
        Offer::Ally => vec![t.battlefield(P0, "Expedition Envoy")],
        Offer::DeadBears => {
            let b = t.battlefield(P0, "Grizzly Bears");
            kill(t, b);
            assert!(t.in_graveyard(P0, "Grizzly Bears"));
            vec![t.g.current(b)]
        }
    }
}

/// `name` enters for P0 (with Glorious Anthem on the battlefield when `anthem`) while
/// something it could copy is available, and P0 chooses to copy nothing. Asserts the
/// choice was offered, optional, and included the candidate. Returns the game and the
/// card (followed across its entry).
fn enter_copying_nothing(name: &str, o: Offer, anthem: bool) -> (TestGame, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    if anthem {
        t.battlefield(P0, "Glorious Anthem");
    }
    let cand = offer(&mut t, o);
    let from = t.asked().len();
    t.answer_choose(P0, &[]);
    let c = t.enter(P0, name);
    t.settle();
    let offered = choices_since(&t, from);
    assert!(
        offered
            .iter()
            .any(|(cs, min)| *min == 0 && cand.iter().any(|c| cs.contains(&obj(*c)))),
        "{name}: no optional choice offered: {offered:?}"
    );
    (t, c)
}

/// The cases of the 0/0 creatures that copy nothing: the card, what it could copy, its
/// printed creature types, and text only the copy effect's exception would give it.
const ZEROS: &[(&str, Offer, &[&str], Option<&str>)] = &[
    (
        "Evil Twin",
        Offer::Bears,
        &["Shapeshifter"],
        Some("Destroy target creature"),
    ),
    (
        "Gigantoplasm",
        Offer::Bears,
        &["Shapeshifter"],
        Some("base power and toughness X/X"),
    ),
    (
        "Jwari Shapeshifter",
        Offer::Ally,
        &["Shapeshifter", "Ally"],
        None,
    ),
    (
        "Malleable Impostor",
        Offer::Bears,
        &["Faerie", "Shapeshifter"],
        None,
    ),
    (
        "Mercurial Pretender",
        Offer::Bears,
        &["Shapeshifter"],
        Some("Return this creature"),
    ),
    (
        "Progenitor Mimic",
        Offer::Bears,
        &["Shapeshifter"],
        Some("create a token"),
    ),
    (
        "Sakashima's Student",
        Offer::Bears,
        &["Human", "Ninja"],
        None,
    ),
    ("Stunt Double", Offer::Bears, &["Shapeshifter"], None),
    ("Synth Infiltrator", Offer::Bears, &["Synth"], None),
    (
        "Dack's Duplicate",
        Offer::Bears,
        &["Shapeshifter"],
        Some("Dethrone"),
    ),
    (
        "Pirated Copy",
        Offer::Bears,
        &["Shapeshifter", "Pirate"],
        Some("you draw a card"),
    ),
    (
        "Activated Sleeper",
        Offer::DeadBears,
        &["Phyrexian", "Shapeshifter"],
        None,
    ),
    ("Body Double", Offer::DeadBears, &["Shapeshifter"], None),
];

#[test]
fn zero_zero_creatures_that_copy_nothing() {
    cr!("707.9", "614.12", "704.5f", "613.4c");
    ruling!("Evil Twin", "You can choose not to copy anything. In that case, Evil Twin enters the battlefield as a 0/0 creature, and is probably put into the graveyard immediately.");
    ruling!("Gigantoplasm", "You can choose not to copy anything. In that case, Gigantoplasm enters the battlefield as a 0/0 Shapeshifter creature, and is probably put into the graveyard immediately.");
    ruling!("Jwari Shapeshifter", "You can choose not to copy anything. In that case, Jwari Shapeshifter enters as a 0/0 creature, and is probably put into the graveyard immediately.");
    ruling!("Malleable Impostor", "You can choose not to copy anything. In that case, Malleable Impostor simply enters the battlefield as a 0/0 creature and is probably put into your graveyard immediately, unless something else is increasing its toughness to keep it alive.");
    ruling!("Mercurial Pretender", "You can choose not to copy anything. In that case, Mercurial Pretender enters the battlefield as a 0/0 Shapeshifter creature, and is probably put into the graveyard immediately.");
    ruling!("Progenitor Mimic", "You can choose not to copy anything. In that case, Progenitor Mimic enters the battlefield as a 0/0 Shapeshifter creature and is probably put into the graveyard immediately.");
    ruling!("Sakashima's Student", "You can choose not to copy anything. In that case, Sakashima's Student enters as a 0/0 creature, and is probably put into the graveyard immediately.");
    ruling!("Stunt Double", "You can choose not to copy anything. In that case, Stunt Double enters the battlefield as a 0/0 creature, and is put into the graveyard immediately (unless something else is raising its toughness).");
    ruling!("Synth Infiltrator", "You can choose not to have Synth Infiltrator enter the battlefield as a copy of another creature. If you do, it will probably die due to having 0 toughness.");
    ruling!("Dack's Duplicate", "You can choose to not copy anything. In that case, Dack's Duplicate enters the battlefield as a 0/0 Shapeshifter creature, and will probably die almost immediately, when state-based actions are next performed.");
    ruling!("Pirated Copy", "You can choose to not copy anything. In that case, Pirated Copy enters the battlefield as a 0/0 Shapeshifter Pirate creature, and will probably die almost immediately, when state-based actions are next performed.");
    ruling!("Activated Sleeper", "You don't have to choose a card to copy. If you don't, Activated Sleeper enters the battlefield as a 0/0 creature and is probably put into your graveyard immediately, unless something is increasing its toughness to keep it alive.");
    ruling!("Body Double", "You don't have to choose a card to copy. If you don't, Body Double enters as a 0/0 creature and is probably put into your graveyard immediately, unless something is increasing its toughness to keep it alive.");
    for &(name, o, types, exception) in ZEROS {
        // Without help, it's put into the graveyard as state-based actions are checked.
        let (t, c) = enter_copying_nothing(name, o, false);
        assert!(!t.on_battlefield(c), "{name} survived as a 0/0");
        assert!(t.in_graveyard(P0, name), "{name} isn't in the graveyard");
        // With Glorious Anthem it survives as a 1/1 with its own printed characteristics,
        // and none of the copy effect's exceptions.
        let (t, c) = enter_copying_nothing(name, o, true);
        assert!(t.on_battlefield(c), "{name} died with Glorious Anthem");
        assert_eq!(name_of(&t, c), name);
        assert_eq!(t.pt(c), (1, 1), "{name}");
        for ty in types {
            assert!(has_subtype(&t, c, ty), "{name} isn't a {ty}");
        }
        assert!(!has_subtype(&t, c, "Bear"), "{name}");
        // Only its printed abilities (the same as the card's in a hand).
        let mut t = t;
        let printed = t.hand(P1, name);
        let n = t.obj_now(printed).chars.abilities.len();
        assert_eq!(t.obj_now(c).chars.abilities.len(), n, "{name}: abilities");
        if let Some(text) = exception {
            let granted = t
                .obj_now(c)
                .chars
                .abilities
                .iter()
                .filter(|a| a.text.contains(text) && !a.text.contains("enter as a copy"));
            assert_eq!(granted.count(), 0, "{name} has the exception");
        }
        assert_eq!(t.counters(c, counters::PLUS1), 0, "{name}");
    }
}

#[test]
fn dacks_duplicate_copying_nothing_has_no_haste_or_dethrone() {
    cr!("707.9", "707.9a", "614.12");
    ruling!("Dack's Duplicate", "You can choose to not copy anything. In that case, Dack's Duplicate enters the battlefield as a 0/0 Shapeshifter creature, and will probably die almost immediately, when state-based actions are next performed.");
    let (mut t, c) = enter_copying_nothing("Dack's Duplicate", Offer::Bears, true);
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(c).has_keyword(KeywordKind::Dethrone));
    // A token copy of it enters as a copy of Dack's Duplicate (the token's copy effect),
    // and its copied ability may make it a copy of something else (CR 614.12): copying
    // nothing, it's a Dack's Duplicate with none of that ability's exceptions either.
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    let before = crate::r_s01_common::tokens(&t, P0);
    t.answer_choose(P0, &[]);
    t.cast(P0, cc).target(obj(c)).go();
    t.resolve_all();
    let toks: Vec<ObjectId> = crate::r_s01_common::tokens(&t, P0)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect();
    assert_eq!(toks.len(), 1);
    let tok = toks[0];
    assert_eq!(name_of(&t, tok), "Dack's Duplicate");
    assert_eq!(t.pt(tok), (1, 1));
    assert!(!t.obj_now(tok).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(tok).has_keyword(KeywordKind::Dethrone));
    // Positive control: copying the Bears, it has both.
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    let before = crate::r_s01_common::tokens(&t, P0);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, cc).target(obj(c)).go();
    t.resolve_all();
    let tok = crate::r_s01_common::tokens(&t, P0)
        .into_iter()
        .find(|x| !before.contains(x))
        .expect("the second token");
    assert_eq!(name_of(&t, tok), "Grizzly Bears");
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Haste));
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Dethrone));
}

#[test]
fn creatures_with_their_own_size_that_copy_nothing() {
    cr!("707.9", "614.12", "208.1");
    ruling!("Quicksilver Gargantuan", "You can choose not to copy anything. In that case, Quicksilver Gargantuan simply enters as a 7/7 creature.");
    ruling!("Visage Bandit", "You can choose not to copy anything. In that case, Visage Bandit simply enters the battlefield as a 2/2 Shapeshifter Rogue creature.");
    let (t, c) = enter_copying_nothing("Quicksilver Gargantuan", Offer::Bears, false);
    assert_eq!(name_of(&t, c), "Quicksilver Gargantuan");
    assert_eq!(t.pt(c), (7, 7));
    assert!(has_subtype(&t, c, "Shapeshifter"));
    let (t, c) = enter_copying_nothing("Visage Bandit", Offer::Bears, false);
    assert_eq!(name_of(&t, c), "Visage Bandit");
    assert_eq!(t.pt(c), (2, 2));
    assert!(has_subtype(&t, c, "Shapeshifter") && has_subtype(&t, c, "Rogue"));
}

/// P0 casts Hulking Metamorph (prototyped when `proto`), choosing to copy nothing though
/// P0 controls a Grizzly Bears.
fn metamorph_copying_nothing(proto: bool) -> (TestGame, ObjectId) {
    supported("Hulking Metamorph");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    if proto {
        t.lands(P0, "Island", 4);
    } else {
        t.lands(P0, "Wastes", 9);
    }
    let card = t.hand(P0, "Hulking Metamorph");
    let method = if proto {
        CastMethod::Keyword(KeywordKind::Prototype)
    } else {
        CastMethod::Normal
    };
    t.cast(P0, card).method(method).go();
    let from = t.asked().len();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(choices_since(&t, from)
        .iter()
        .any(|(cs, min)| *min == 0 && cs.contains(&obj(bears))));
    let m = t.named_on_battlefield("Hulking Metamorph");
    assert_eq!(m.len(), 1);
    (t, m[0])
}

#[test]
fn hulking_metamorph_copying_nothing_is_a_7_7_or_3_3_artifact_creature() {
    cr!("707.9", "718.3", "718.3b");
    ruling!("Hulking Metamorph", "You can choose not to copy anything. In that case, Hulking Metamorph simply enters the battlefield as a 7/7 (or 3/3) artifact creature.");
    for (proto, pt) in [(false, (7, 7)), (true, (3, 3))] {
        let (t, m) = metamorph_copying_nothing(proto);
        assert_eq!(t.pt(m), pt);
        let o = t.obj_now(m);
        assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
        assert!(!has_subtype(&t, m, "Bear"));
    }
}

/// P0 casts Altered Ego with X = `x`, copying `what` (a Grizzly Bears P1 controls) or
/// nothing. Returns the game and the Altered Ego (followed onto the battlefield or into
/// the graveyard).
fn altered_ego(x: i64, copy: bool) -> (TestGame, ObjectId) {
    supported("Altered Ego");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Altered Ego");
    t.lands(P0, "Wastes", x as usize);
    let card = t.hand(P0, "Altered Ego");
    t.cast(P0, card).x(x).go();
    t.answer_choose(P0, if copy { vec![obj(bears)] } else { vec![] }.as_slice());
    t.resolve_all();
    (t, card)
}

#[test]
fn altered_ego_with_x_0_is_just_a_copy() {
    cr!("707.9", "614.1c", "107.3");
    ruling!("Altered Ego", "X can be 0. Altered Ego won't enter with any additional +1/+1 counters, and it will just be a copy of the chosen creature.");
    let (t, _) = altered_ego(0, true);
    let ego = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(ego.len(), 2);
    let mine = *ego
        .iter()
        .find(|id| t.obj_now(**id).controller == P0)
        .unwrap();
    assert_eq!(t.counters(mine, counters::PLUS1), 0);
    assert_eq!(t.pt(mine), (2, 2));
    // Positive control: with X = 2 it gets two.
    let (t, _) = altered_ego(2, true);
    let mine = *t
        .named_on_battlefield("Grizzly Bears")
        .iter()
        .find(|id| t.obj_now(**id).controller == P0)
        .unwrap();
    assert_eq!(t.counters(mine, counters::PLUS1), 2);
}

#[test]
fn altered_ego_copying_nothing_gets_no_counters_and_dies() {
    cr!("707.9", "704.5f");
    ruling!("Altered Ego", "You can choose not to copy anything. In that case, Altered Ego enters as a 0/0 creature and is probably put into the graveyard immediately. It won't have +1/+1 counters placed on it by its ability.");
    let (t, _) = altered_ego(3, false);
    assert!(t.named_on_battlefield("Altered Ego").is_empty());
    assert!(t.in_graveyard(P0, "Altered Ego"));
    // With Glorious Anthem it survives, a 1/1 with no counters.
    supported("Altered Ego");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Altered Ego");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Altered Ego");
    t.cast(P0, card).x(3).go();
    t.answer_choose(P0, &[]);
    t.resolve_all();
    let ego = t.named_on_battlefield("Altered Ego");
    assert_eq!(ego.len(), 1);
    assert_eq!(t.counters(ego[0], counters::PLUS1), 0);
    assert_eq!(t.pt(ego[0]), (1, 1));
}

/// P0 casts Altered Ego with X = 3 while P0 controls Essence of the Wild ("Creatures you
/// control enter as a copy of this creature."), answering the choice of which copy
/// effect to apply first with `first`, and the choice of what Altered Ego copies (if its
/// own copy effect still applies) with `copy` (P1's Grizzly Bears) or nothing. Returns
/// the game, the permanent, and whether Essence's copy effect was applied first.
fn altered_ego_with_essence(first: usize, copy: bool) -> (TestGame, ObjectId, bool) {
    supported("Altered Ego");
    supported("Essence of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Altered Ego");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Altered Ego");
    t.cast(P0, card).x(3).go();
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
    t.answer_choose(P0, if copy { vec![obj(bears)] } else { vec![] }.as_slice());
    t.resolve_all();
    let options = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseReplacement { options } => Some(options.clone()),
            _ => None,
        })
        .expect("P0 chooses which copy effect to apply first");
    assert_eq!(options.len(), 2, "{options:?}");
    let essence_first = options[first].contains("Creatures you control");
    let ego = t.g.current(card);
    assert!(t.on_battlefield(ego));
    (t, ego, essence_first)
}

#[test]
fn altered_ego_entering_as_an_essence_of_the_wild_gets_no_counters() {
    cr!("707.9", "707.9e", "616.1c");
    ruling!("Essence of the Wild", "If a creature such as Clone is entering under your control, there will be two copy effects to apply: the creature's own and Essence of the Wild's. No matter what order these effects are applied, the creature will be a copy of Essence of the Wild when it enters.");
    // Whichever copy effect is applied first, and whether Altered Ego's own copies
    // something or nothing, it's a plain Essence of the Wild: the counters are an
    // exception of Altered Ego's own copy effect only (CR 707.9), which copied nothing,
    // was followed by Essence's copy effect (CR 707.9e), or no longer applied.
    let mut orders = vec![];
    for first in [0, 1] {
        for copy in [false, true] {
            let (t, ego, essence_first) = altered_ego_with_essence(first, copy);
            let case = format!(
                "{} first, Altered Ego copying {}",
                if essence_first {
                    "Essence"
                } else {
                    "Altered Ego"
                },
                if copy { "the Bears" } else { "nothing" }
            );
            assert_eq!(name_of(&t, ego), "Essence of the Wild", "{case}");
            assert_eq!(t.pt(ego), (6, 6), "{case}");
            assert_eq!(t.counters(ego, counters::PLUS1), 0, "{case}");
            orders.push(essence_first);
        }
    }
    orders.sort();
    orders.dedup();
    assert_eq!(orders, vec![false, true], "both orders");
}
