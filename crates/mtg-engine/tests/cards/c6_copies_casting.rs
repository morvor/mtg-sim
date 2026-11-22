//! Casting from among copies of cards with a cap (round 6, cards6-copies-casting,
//! `oracle/patterns/copies_7.rs`, `kw/cast_up_to_total_mana_value.rs`): Uldaros Theorix'
//! one target per card type and total mana value over the copies, Baron Helmut Zemo's
//! copies of the cards its boast cost exiled and its count of the copies.

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::object::{Characteristics, ObjKind, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::TypeLine;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The copies of cards made so far, by name (ids are deterministic: a game set up and
/// answered the same way makes the same objects).
fn copies(t: &TestGame) -> Vec<(String, ObjectId)> {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::CardCopy)
        .map(|o| (o.chars.name.to_string(), o.id))
        .collect()
}

fn copy_named(copies: &[(String, ObjectId)], name: &str) -> Entity {
    let id = copies
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("no copy of {name}: {copies:?}"))
        .1;
    Entity::Object(id)
}

fn names_on_stack(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// The cards of [`uldaros`]'s graveyard.
const GRAVEYARD: [&str; 6] = [
    "Grizzly Bears",
    "Ornithopter",
    "Lightning Bolt",
    "Divination",
    "Pacifism",
    "Forest",
];

/// P0 has Uldaros Theorix in hand and `cards` in their graveyard; they cast Uldaros and
/// its trigger is put on the stack targeting the cards at `targets`. Returns the cards.
fn uldaros_with(t: &mut TestGame, cards: &[&str], targets: &[usize]) -> Vec<ObjectId> {
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 3);
    let cards: Vec<ObjectId> = cards.iter().map(|c| t.graveyard(P0, c)).collect();
    let uldaros = t.hand(P0, "Uldaros Theorix");
    let ts: Vec<Entity> = targets.iter().map(|i| Entity::Object(cards[*i])).collect();
    t.answer_targets(P0, &ts);
    t.cast(P0, uldaros).go();
    t.resolve();
    t.settle();
    cards
}

/// [`uldaros_with`] with Grizzly Bears (creature, mana value 2), Ornithopter (artifact
/// creature, 0), Lightning Bolt (instant, 1), Divination (sorcery, 3), Pacifism
/// (enchantment, 2) and a Forest in the graveyard.
fn uldaros(t: &mut TestGame, targets: &[usize]) -> Vec<ObjectId> {
    uldaros_with(t, &GRAVEYARD, targets)
}

/// The targets Uldaros' trigger was put on the stack with.
fn trigger_targets(t: &TestGame) -> Vec<Entity> {
    assert_eq!(
        names_on_stack(t),
        vec!["Uldaros Theorix ability"],
        "{}",
        t.dump_log()
    );
    targets_of(t, t.g.stack[0])
}

#[test]
fn uldaros_theorix_targets_one_card_of_each_card_type() {
    cr!("115.1d", "601.2c", "603.3d", "205.2a");
    interp!("one-target-per-card-type");
    assert_supported(&["Uldaros Theorix"]);
    // "When Uldaros Theorix enters, if you cast him, exile up to one target nonland card
    // of each card type from your graveyard." Bears for creature and Ornithopter for
    // artifact (an artifact creature stands for one of its card types), Bolt, Divination
    // and Pacifism: five targets.
    let mut t = TestGame::new(2);
    let cards = uldaros(&mut t, &[0, 1, 2, 3, 4]);
    let all: Vec<Entity> = cards[..5].iter().map(|c| Entity::Object(*c)).collect();
    assert_eq!(trigger_targets(&t), all);
    // Two creature cards and an artifact creature card: one of them stands for creature,
    // one for artifact, and the third has no card type left.
    let three = ["Grizzly Bears", "Hill Giant", "Ornithopter"];
    let mut t = TestGame::new(2);
    let cards = uldaros_with(&mut t, &three, &[0, 1, 2]);
    // The choice is fitted to the largest group of it, in choice order, that has the
    // relationship: Bears (creature) and Ornithopter (artifact).
    let chosen = trigger_targets(&t);
    assert_eq!(
        chosen,
        vec![Entity::Object(cards[0]), Entity::Object(cards[2])]
    );
    // Two creature cards alone can't both be targets either.
    let mut t = TestGame::new(2);
    let cards = uldaros_with(&mut t, &three, &[0, 1]);
    let chosen = trigger_targets(&t);
    assert!(
        !(chosen.contains(&Entity::Object(cards[0])) && chosen.contains(&Entity::Object(cards[1]))),
        "two cards for creature: {chosen:?}"
    );
    // An artifact creature card chosen before an artifact card: it stands for creature, so
    // both are targets.
    let mut t = TestGame::new(2);
    let cards = uldaros_with(&mut t, &["Ornithopter", "Sol Ring"], &[0, 1]);
    assert_eq!(
        trigger_targets(&t),
        vec![Entity::Object(cards[0]), Entity::Object(cards[1])]
    );
    // A land card isn't a nonland card.
    let mut t = TestGame::new(2);
    let cards = uldaros(&mut t, &[5]);
    assert!(!trigger_targets(&t).contains(&Entity::Object(cards[5])));
}

#[test]
fn uldaros_theorix_casts_copies_with_total_mana_value_six_or_less() {
    cr!("707.12", "707.12a", "608.2g", "704.5e", "111.13");
    interp!("capped-casts-one-at-a-time");
    assert_supported(&["Uldaros Theorix"]);
    // "Copy those cards. You may cast any number of spells with total mana value 6 or less
    // from among the copies without paying their mana costs."
    let targets = [0, 1, 2, 3, 4];
    // A first run, casting none of the copies, names them.
    let mut t = TestGame::new(2);
    uldaros(&mut t, &targets);
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 5, "{made:?}");
    // The copies not cast ceased to exist; the cards stay in exile.
    assert!(made.iter().all(|(_, c)| !t.g.is_live(*c)));
    for c in [
        "Grizzly Bears",
        "Ornithopter",
        "Lightning Bolt",
        "Divination",
        "Pacifism",
    ] {
        assert!(t.in_exile(c), "{c}");
    }
    assert!(t.in_graveyard(P0, "Forest"));
    // Divination (3), Lightning Bolt (1) at P1 and Grizzly Bears (2): a total of 6, and
    // Ornithopter (0) still fits; Pacifism (2) doesn't.
    let mut t = TestGame::new(2);
    uldaros(&mut t, &targets);
    for name in ["Divination", "Lightning Bolt"] {
        t.answer_choose(P0, &[copy_named(&made, name)]);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    for name in ["Grizzly Bears", "Ornithopter"] {
        t.answer_choose(P0, &[copy_named(&made, name)]);
    }
    let from = t.asked().len();
    t.resolve();
    assert_eq!(
        names_on_stack(&t),
        vec![
            "Divination",
            "Lightning Bolt",
            "Grizzly Bears",
            "Ornithopter"
        ],
        "{}",
        t.dump_log()
    );
    // The copies offered for each spell: with 2 left, Bears, Pacifism and Ornithopter;
    // with 0 left, only Ornithopter (Pacifism's mana value 2 is more than what's left).
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 4, "{offered:?}");
    assert_eq!(offered[0].len(), 5);
    let mut third = offered[2].clone();
    third.sort();
    let mut want: Vec<Entity> = ["Grizzly Bears", "Ornithopter", "Pacifism"]
        .iter()
        .map(|n| copy_named(&made, n))
        .collect();
    want.sort();
    assert_eq!(third, want);
    assert_eq!(offered[3], vec![copy_named(&made, "Ornithopter")]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 2);
    // The permanent spells became tokens (CR 111.13).
    for name in ["Grizzly Bears", "Ornithopter"] {
        let ps = t.named_on_battlefield(name);
        assert_eq!(ps.len(), 1, "{name}");
        assert_eq!(t.g.obj(ps[0]).kind, ObjKind::Token, "{name}");
    }
    assert!(t.named_on_battlefield("Pacifism").is_empty());
}

/// Baron Helmut Zemo on the battlefield, having attacked this turn, with Phyrexian
/// Obliterator ({B}{B}{B}{B}), Necropotence ({B}{B}{B}), Dread Shade ({B}{B}{B}), Sign in
/// Blood ({B}{B}), Hypnotic Specter ({1}{B}{B}) and Dark Ritual ({B}) in P0's graveyard:
/// fifteen black mana symbols. Returns Zemo and the cards.
fn zemo(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    let zemo = t.battlefield(P0, "Baron Helmut Zemo");
    let cards: Vec<ObjectId> = [
        "Phyrexian Obliterator",
        "Necropotence",
        "Dread Shade",
        "Sign in Blood",
        "Hypnotic Specter",
        "Dark Ritual",
    ]
    .iter()
    .map(|c| t.graveyard(P0, c))
    .collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(zemo, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    (zemo, cards)
}

/// Activates Zemo's boast exiling `exiled` (indices into the cards).
fn boast(t: &mut TestGame, zemo: ObjectId, cards: &[ObjectId], exiled: &[usize]) -> bool {
    let es: Vec<Entity> = exiled.iter().map(|i| Entity::Object(cards[*i])).collect();
    t.answer(P0, DecisionKind::X, Answer::Number(es.len() as i64));
    t.answer_choose(P0, &es);
    t.activate(P0, zemo, 0, &[]).is_ok()
}

#[test]
fn baron_helmut_zemo_casts_up_to_three_copies_of_the_cards_its_boast_exiled() {
    cr!("400.7j", "602.2b", "707.12", "707.12a", "608.2g", "704.5e", "111.13");
    interp!("capped-casts-one-at-a-time");
    assert_supported(&["Baron Helmut Zemo"]);
    // "Boast — Exile any number of black cards from your graveyard with fifteen or more
    // black mana symbols among their mana costs: Copy those exiled cards. You may cast up
    // to three of the copies without paying their mana costs."
    // Without Dark Ritual: fourteen symbols, not enough.
    let mut t = TestGame::new(2);
    let (zemo, cards) = self::zemo(&mut t);
    assert!(!boast(&mut t, zemo, &cards, &[0, 1, 2, 3, 4]));
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Graveyard(P0)));
    // All six: the copies are of the exiled cards. A first run, casting none, names them.
    let all = [0, 1, 2, 3, 4, 5];
    let mut t = TestGame::new(2);
    let (zemo, cards) = self::zemo(&mut t);
    assert!(boast(&mut t, zemo, &cards, &all));
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 6, "{made:?}");
    assert!(made.iter().all(|(_, c)| !t.g.is_live(*c)));
    assert!(cards.iter().all(|c| t.zone(t.g.current(*c)) == Zone::Exile));
    // Sign in Blood at P1, Dread Shade and Phyrexian Obliterator: three, and no fourth
    // is offered.
    let mut t = TestGame::new(2);
    let (zemo, cards) = self::zemo(&mut t);
    assert!(boast(&mut t, zemo, &cards, &all));
    t.answer_choose(P0, &[copy_named(&made, "Sign in Blood")]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[copy_named(&made, "Dread Shade")]);
    t.answer_choose(P0, &[copy_named(&made, "Phyrexian Obliterator")]);
    t.answer_choose(P0, &[copy_named(&made, "Hypnotic Specter")]);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(
        names_on_stack(&t),
        vec!["Sign in Blood", "Dread Shade", "Phyrexian Obliterator"],
        "{}",
        t.dump_log()
    );
    let choices = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .count();
    assert_eq!(choices, 3);
    let hand = t.hand_size(P1);
    t.resolve_all();
    // Zemo's 3 combat damage, then Sign in Blood.
    assert_eq!(t.hand_size(P1), hand + 2);
    assert_eq!(t.life(P1), 20 - 3 - 2);
    for name in ["Dread Shade", "Phyrexian Obliterator"] {
        let ps = t.named_on_battlefield(name);
        assert_eq!(ps.len(), 1, "{name}");
        assert_eq!(t.g.obj(ps[0]).kind, ObjKind::Token, "{name}");
    }
    assert!(t.named_on_battlefield("Hypnotic Specter").is_empty());
}

/// The candidates of each choice of an object asked since `from`.
fn offered_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn baron_helmut_zemo_a_copy_that_cant_be_cast_isnt_one_of_the_three() {
    cr!("733.1", "733.2", "707.12a", "608.2g");
    interp!("capped-casts-one-at-a-time");
    // With a Doom Blade exiled too and no nonblack creature on the battlefield, its copy
    // can't be cast (no legal target): trying to is reversed (CR 733.1), and three other
    // copies may still be cast.
    let setup = |t: &mut TestGame| {
        let (zemo, mut cards) = self::zemo(t);
        cards.push(t.graveyard(P0, "Doom Blade"));
        assert!(boast(t, zemo, &cards, &[0, 1, 2, 3, 4, 5, 6]));
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 7, "{made:?}");
    let mut t = TestGame::new(2);
    setup(&mut t);
    for name in [
        "Doom Blade",
        "Dread Shade",
        "Phyrexian Obliterator",
        "Hypnotic Specter",
        "Necropotence",
    ] {
        t.answer_choose(P0, &[copy_named(&made, name)]);
    }
    let from = t.asked().len();
    t.resolve();
    assert_eq!(
        names_on_stack(&t),
        vec!["Dread Shade", "Phyrexian Obliterator", "Hypnotic Specter"],
        "{}",
        t.dump_log()
    );
    // Doom Blade's copy was offered first; right after it was tried it isn't offered, and
    // once another spell has been cast it is again (CR 733.2), still with no target.
    let offered = offered_since(&t, from);
    assert_eq!(offered.len(), 4, "{offered:?}");
    let blade = copy_named(&made, "Doom Blade");
    assert!(offered[0].contains(&blade));
    assert!(!offered[1].contains(&blade));
    assert!(offered[2].contains(&blade) && offered[3].contains(&blade));
    assert!(!t.g.is_live(blade.object().unwrap()));
    assert!(t.named_on_battlefield("Necropotence").is_empty());
}

#[test]
fn uldaros_theorix_a_copy_tried_too_early_may_be_cast_after_another() {
    cr!("733.1", "733.2", "608.2g");
    interp!("capped-casts-one-at-a-time");
    // Twincast ("Copy target instant or sorcery spell.") and Divination among the copies.
    // Twincast's copy, tried first, has no spell to target, so trying is reversed (CR
    // 733.1); once Divination's copy is cast, Twincast's may be cast targeting it: a total
    // of 2 + 3.
    let cards = ["Twincast", "Divination"];
    let mut t = TestGame::new(2);
    uldaros_with(&mut t, &cards, &[0, 1]);
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 2, "{made:?}");
    let mut t = TestGame::new(2);
    uldaros_with(&mut t, &cards, &[0, 1]);
    for name in ["Twincast", "Divination", "Twincast"] {
        t.answer_choose(P0, &[copy_named(&made, name)]);
    }
    let from = t.asked().len();
    t.resolve();
    assert_eq!(
        names_on_stack(&t),
        vec!["Divination", "Twincast"],
        "{}",
        t.dump_log()
    );
    let (divination, twincast) = (t.g.stack[0], t.g.stack[1]);
    assert_eq!(targets_of(&t, twincast), vec![Entity::Object(divination)]);
    // Not offered again until another spell was cast.
    let offered = offered_since(&t, from);
    assert_eq!(offered.len(), 3, "{offered:?}");
    assert_eq!(offered[1], vec![copy_named(&made, "Divination")]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    // Divination and its copy.
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn uldaros_theorix_counts_the_mana_value_of_the_half_of_a_split_card_cast() {
    cr!("709.3c", "709.3a", "202.3d", "702.102a");
    interp!("capped-casts-one-at-a-time");
    // Alive // Well ({3}{G} // {W}, mana value 5 in exile) among the copies: with 4 left
    // after Grizzly Bears, its copy may be cast as Alive (4) or Well (1), but not fused
    // (only from a hand). Cast as Alive, it leaves 0: only Ornithopter still fits.
    let cards = [
        "Grizzly Bears",
        "Alive // Well",
        "Ornithopter",
        "Lightning Bolt",
        "Pacifism",
    ];
    let targets = [0, 1, 2, 3, 4];
    let mut t = TestGame::new(2);
    uldaros_with(&mut t, &cards, &targets);
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 5, "{made:?}");
    let mut t = TestGame::new(2);
    uldaros_with(&mut t, &cards, &targets);
    for name in ["Grizzly Bears", "Alive // Well", "Ornithopter"] {
        t.answer_choose(P0, &[copy_named(&made, name)]);
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let from = t.asked().len();
    t.resolve();
    assert_eq!(
        names_on_stack(&t),
        vec!["Grizzly Bears", "Alive", "Ornithopter"],
        "{}",
        t.dump_log()
    );
    let offered = offered_since(&t, from);
    assert_eq!(offered.len(), 3, "{offered:?}");
    assert!(offered[1].contains(&copy_named(&made, "Alive // Well")));
    assert_eq!(offered[2], vec![copy_named(&made, "Ornithopter")]);
    let halves: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        halves,
        vec![vec!["Cast Alive".to_string(), "Cast Well".to_string()]]
    );
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Centaur Token").len(), 1);
    assert_eq!(t.life(P1), 20);
}

/// Compiles `text` as the abilities of a custom card with type line `types`; the parts
/// left unsupported.
fn unsupported(text: &str, types: &str) -> Vec<String> {
    let tl = TypeLine::parse(types);
    let ctx = CompileContext {
        card_name: "Probe",
        full_name: "Probe",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("2".into()),
        toughness: Some("2".into()),
    };
    oracle::compile(text, &ctx).unsupported
}

fn compiled(text: &str, types: &str) -> Characteristics {
    let tl = TypeLine::parse(types);
    let ctx = CompileContext {
        card_name: "Probe",
        full_name: "Probe",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("2".into()),
        toughness: Some("2".into()),
    };
    let c = oracle::compile(text, &ctx);
    assert!(c.unsupported.is_empty(), "{text}: {:?}", c.unsupported);
    Characteristics {
        name: "Probe".into(),
        card_types: tl.card_types,
        abilities: c.abilities,
        ..Default::default()
    }
}

#[test]
fn the_grammar_of_capped_casts_from_among_copies() {
    // The caps compose with any copy of cards, in other wordings too.
    for text in [
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast up to two of the copies without paying their mana costs.",
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast any number of spells with total mana value 4 or less from among the copies without paying their mana costs.",
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast up to two instant and/or sorcery spells with total mana value 5 or less from among the copies without paying their mana costs.",
        "Exile any number of target cards from your graveyard. Copy them. You may cast up to two of the copies without paying their mana costs.",
    ] {
        assert!(unsupported(text, "Sorcery").is_empty(), "{text}");
    }
    // The cost's cards as the referent: "Copy those exiled cards" and "Copy those cards".
    for text in [
        "Exile any number of black cards from your graveyard: Copy those exiled cards. You may cast up to three of the copies without paying their mana costs.",
        "Exile any number of black cards from your graveyard: Copy those cards. You may cast the copies without paying their mana costs.",
    ] {
        assert!(unsupported(text, "Artifact").is_empty(), "{text}");
    }
    // One target per card type, also without "nonland", and only "up to one" of each.
    assert!(unsupported(
        "When ~ enters, exile up to one target card of each card type from your graveyard.",
        "Creature — Sphinx"
    )
    .is_empty());
    for text in [
        "When ~ enters, exile target card of each card type from your graveyard.",
        "When ~ enters, exile up to two target cards of each card type from your graveyard.",
        // A cap with a cost to pay isn't this construct.
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast up to two of the copies.",
    ] {
        assert!(!unsupported(text, "Creature — Sphinx").is_empty(), "{text}");
    }
}

#[test]
fn a_count_cap_over_copies_on_a_neutral_card() {
    cr!("707.12", "707.12a", "608.2g", "704.5e");
    // "Exile up to three target cards from your graveyard. Copy those cards. You may cast
    // up to two of the copies without paying their mana costs." with three Lightning Bolts:
    // two are cast, and the third copy ceases to exist.
    let mut chars = compiled(
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast up to two of the copies without paying their mana costs.",
        "Sorcery",
    );
    chars.mana_cost = mtg_engine::mana::ManaCost::parse("{0}");
    let def = CardDef::custom(chars);
    let setup = |t: &mut TestGame| {
        t.set_step(P0, Step::PrecombatMain);
        let bolts: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Lightning Bolt")).collect();
        let probe = t.custom(P0, def.clone(), Zone::Hand(P0));
        let ts: Vec<Entity> = bolts.iter().map(|b| Entity::Object(*b)).collect();
        t.cast(P0, probe).targets(&ts).go();
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.resolve();
    let made: Vec<Entity> = copies(&t)
        .into_iter()
        .map(|(_, o)| Entity::Object(o))
        .collect();
    assert_eq!(made.len(), 3);
    let mut t = TestGame::new(2);
    setup(&mut t);
    for c in &made {
        t.answer_choose(P0, &[*c]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.resolve();
    assert_eq!(names_on_stack(&t), vec!["Lightning Bolt", "Lightning Bolt"]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let Entity::Object(third) = made[2] else {
        unreachable!()
    };
    assert!(!t.g.is_live(third));
}

#[test]
fn a_land_cards_copy_isnt_offered_and_isnt_one_of_the_count() {
    cr!("305.9", "707.12a", "704.5e");
    // The same card with Dryad Arbor (a land creature card) and two Lightning Bolts: the
    // Arbor's copy can't be cast (CR 305.9), so it's never offered, and both Bolts are.
    let mut chars = compiled(
        "Exile up to three target cards from your graveyard. Copy those cards. You may cast up to two of the copies without paying their mana costs.",
        "Sorcery",
    );
    chars.mana_cost = mtg_engine::mana::ManaCost::parse("{0}");
    let def = CardDef::custom(chars);
    let setup = |t: &mut TestGame| {
        t.set_step(P0, Step::PrecombatMain);
        let mut cards = vec![t.graveyard(P0, "Dryad Arbor")];
        cards.extend((0..2).map(|_| t.graveyard(P0, "Lightning Bolt")));
        let probe = t.custom(P0, def.clone(), Zone::Hand(P0));
        let ts: Vec<Entity> = cards.iter().map(|b| Entity::Object(*b)).collect();
        t.cast(P0, probe).targets(&ts).go();
    };
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.resolve();
    let made = copies(&t);
    assert_eq!(made.len(), 3, "{made:?}");
    let arbor = copy_named(&made, "Dryad Arbor");
    let bolts: Vec<Entity> = made
        .iter()
        .filter(|(n, _)| n == "Lightning Bolt")
        .map(|(_, o)| Entity::Object(*o))
        .collect();
    let mut t = TestGame::new(2);
    setup(&mut t);
    for c in [bolts[0], bolts[1]] {
        t.answer_choose(P0, &[c]);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    t.resolve();
    assert_eq!(names_on_stack(&t), vec!["Lightning Bolt", "Lightning Bolt"]);
    let offered = offered_since(&t, from);
    assert_eq!(offered.len(), 2, "{offered:?}");
    assert!(offered.iter().all(|o| !o.contains(&arbor)), "{offered:?}");
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(!t.g.is_live(arbor.object().unwrap()));
    assert!(t.named_on_battlefield("Dryad Arbor").is_empty());
}
