//! Card actions on hands, graveyards and libraries (grammar in
//! `src/oracle/patterns/hand_graveyard_grammar.rs`): exiling cards from graveyards with
//! counts and sources, discard/draw variants, revealing from a hand, moving cards between
//! a hand and a library, "A or B" instructions, and statics that grant abilities to cards
//! in a graveyard or hand.

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The ability of `name` containing `piece` compiles (other abilities may not).
fn assert_compiles(cards: &[(&str, &str)]) {
    for (n, piece) in cards {
        let c = card(n);
        let all: String = c.faces.iter().map(|f| f.chars.rules_text.to_string()).collect();
        assert!(
            all.to_lowercase().contains(&piece.to_lowercase()),
            "{n} has no text {piece:?}"
        );
        for u in c.unsupported_text() {
            assert!(
                !u.to_lowercase().contains(&piece.to_lowercase()),
                "{n}: {piece:?} is unsupported: {u}"
            );
        }
    }
}

/// The candidates of the most recent "choose entities" decision asked of `p`.
fn last_choice_of(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

// ---------------------------------------------------------------------------
// Exiling cards from graveyards
// ---------------------------------------------------------------------------

#[test]
fn exile_family_texts_compile() {
    assert_compiles(&[
        ("Kaya, Orzhov Usurper", "Exile up to two target cards from a single graveyard"),
        ("Ashiok, Nightmare Weaver", "Exile all cards from all opponents' hands and graveyards"),
        ("Summon: Esper Valigarmanda", "Exile an instant or sorcery card from each graveyard"),
    ]);
}

#[test]
fn exile_family_compiles() {
    assert_supported(&[
        "Decompose",
        "Griffnaut Tracker",
        "Faerie Macabre",
        "Worldfire",
        "Identity Crisis",
        "Thraben Charm",
        "Titania's Command",
        "Shred Memory",
        "Rats' Feast",
        "Pestilent Cauldron // Restorative Burst",
        "Decree of Annihilation",
        "Ultimate Nullification",
        "Thought Distortion",
        "Aegis Sculptor",
        "Bloodcurdler",
    ]);
}

#[test]
fn decompose_targets_up_to_three_cards_in_a_single_graveyard() {
    cr!("115.1", "115.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Lightning Bolt");
    let c = t.graveyard(P1, "Shock");
    let mine = t.graveyard(P0, "Forest");
    // Cards from two graveyards can't be chosen together (the choice is fitted to a
    // single graveyard).
    let mut t2 = TestGame::new(2);
    t2.lands(P0, "Swamp", 2);
    let x = t2.graveyard(P1, "Grizzly Bears");
    let y = t2.graveyard(P0, "Forest");
    let d = t2.hand(P0, "Decompose");
    t2.cast(P0, d).targets(&objs(&[x, y])).go();
    t2.resolve();
    assert!(t2.zone(x) != Zone::Exile || t2.zone(y) != Zone::Exile);
    let d = t.hand(P0, "Decompose");
    t.cast(P0, d).targets(&objs(&[a, b, c])).go();
    t.resolve();
    for x in [a, b, c] {
        assert_eq!(t.zone(x), Zone::Exile);
    }
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
}

#[test]
fn faerie_macabre_exiles_from_different_graveyards() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P0, "Forest");
    let fm = t.hand(P0, "Faerie Macabre");
    t.answer_targets(P0, &objs(&[a, b]));
    t.activate(P0, fm, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(t.in_graveyard(P0, "Faerie Macabre"));
}

#[test]
fn worldfire_exiles_every_hand_and_graveyard() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 9);
    t.hand(P0, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    let wf = t.hand(P0, "Worldfire");
    t.cast(P0, wf).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 0);
    // Worldfire itself goes to the graveyard after it resolves.
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Forest") && t.in_exile("Shock"));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.life(P1), 1);
}

#[test]
fn identity_crisis_exiles_a_hand_and_graveyard() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    let ic = t.hand(P0, "Identity Crisis");
    t.cast(P0, ic).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn titanias_command_life_for_each_card_exiled_this_way() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    for _ in 0..3 {
        t.graveyard(P1, "Grizzly Bears");
    }
    let tc = t.hand(P0, "Titania's Command");
    t.cast(P0, tc).modes(&[0, 2]).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.named_on_battlefield("Bear Token").len(), 2, "{}", t.dump_log());
}

#[test]
fn aegis_sculptor_may_exile_two_cards_only_with_two() {
    cr!("603.5");
    let text = card("Aegis Sculptor").faces[0].chars.rules_text.to_string();
    assert!(text.contains("you may exile two cards from your graveyard"));
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Aegis Sculptor");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[a, b]));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert_eq!(t.counters(s, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------
// Discard and draw variants
// ---------------------------------------------------------------------------

#[test]
fn discard_and_draw_family_compiles() {
    assert_supported(&[
        "Incendiary Command",
        "Nantuko Cultivator",
        "Rites of Refusal",
        "Forgotten Creation",
        "Valakut Awakening // Valakut Stoneforge",
        "Sawtooth Loon",
        "Prying Questions",
        "Manhole Missile",
        "Soul's Majesty",
        "Sandstone Oracle",
        "Shatter Assumptions",
        "Trapfinder's Trick",
        "Steal the Show",
        "Hypothesizzle",
        "Cragganwick Cremator",
        "Stormscale Anarch",
        "Recurring Insight",
        "Fateful Handoff",
        "See Beyond",
        "Credit Voucher",
        "Pitiless Carnage",
    ]);
    assert_compiles(&[
        ("Path of the Pyromancer", "Discard all the cards in your hand"),
        ("Fable of the Mirror-Breaker // Reflection of Kiki-Jiki", "You may discard up to two cards"),
        ("Felothar the Steadfast", "then discard cards equal to its power"),
    ]);
}

#[test]
fn incendiary_command_each_player_draws_their_own_number() {
    cr!("701.9a", "121.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.hand(P0, "Shock");
    t.hand(P0, "Shock");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    let ic = t.hand(P0, "Incendiary Command");
    t.cast(P0, ic).modes(&[1, 3]).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 3); // two Shocks and the Command
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.hand_size(P1), 3);
    assert!(!t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn nantuko_cultivator_counts_land_cards_discarded() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Island");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[f1, f2]));
    let nc = t.enter(P0, "Nantuko Cultivator");
    t.resolve_all();
    // Only land cards were offered.
    let offered = last_choice_of(&t, P0);
    assert!(!offered.contains(&Entity::Object(bolt)));
    assert_eq!(t.counters(nc, "+1/+1"), 2);
    assert_eq!(t.hand_size(P0), 3);
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P0, "Island"));
}

#[test]
fn rites_of_refusal_taxes_per_card_discarded() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Island", 5);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Shock");
    let rr = t.hand(P0, "Rites of Refusal");
    let bolt_spell = t.g.stack[0];
    t.answer_choose(P0, &objs(&[a, b]));
    t.cast(P0, rr).target(bolt_spell).go();
    // P1 has 5 untapped lands: can't pay {6}.
    t.answer_yes(P1, true);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn valakut_awakening_puts_cards_on_bottom_then_draws_one_more() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Shock");
    t.hand(P0, "Forest");
    let va = t.hand(P0, "Valakut Awakening // Valakut Stoneforge");
    t.answer_choose(P0, &objs(&[a, b]));
    t.cast(P0, va).go();
    t.resolve();
    // Two cards went to the bottom; three were drawn.
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.zone(a), Zone::Library(P0));
    let lib = &t.g.player(P0).library;
    assert!(lib[..2].contains(&t.g.current(a)) || lib[lib.len() - 2..].contains(&t.g.current(a)));
    assert_eq!(t.library_size(P0), 30 - 3 + 2);
}

#[test]
fn forgotten_creation_discards_whole_hand_then_draws_that_many() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forgotten Creation");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Shock");
    t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.hand_size(P0), 3);
    assert!(!t.in_hand(P0, "Shock"));
}

#[test]
fn prying_questions_opponent_puts_a_card_on_top() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Shock");
    t.answer_choose(P1, &objs(&[bears]));
    let pq = t.hand(P0, "Prying Questions");
    t.cast(P0, pq).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    let top = *t.g.player(P1).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name.as_str(), "Grizzly Bears");
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn manhole_missile_draws_only_if_a_card_was_put_on_the_bottom() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mm = t.hand(P0, "Manhole Missile");
    let shock = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[shock]));
    t.cast(P0, mm).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.zone(shock), Zone::Library(P0));
    assert_eq!(t.hand_size(P0), 1);
    // With an empty hand, nothing is put on the bottom and nothing is drawn.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let filler_drawn = t.hand_size(P0);
    let hand: Vec<ObjectId> = t.g.player(P0).hand.clone();
    for c in hand {
        t.g.discard(P0, c, None);
    }
    let _ = filler_drawn;
    let mm = t.hand(P0, "Manhole Missile");
    t.answer_yes(P0, true);
    t.cast(P0, mm).target(bears).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn draw_cards_equal_to_a_value() {
    cr!("121.1");
    // Soul's Majesty: the target's power.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sm = t.hand(P0, "Soul's Majesty");
    t.cast(P0, sm).target(bears).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    // Sandstone Oracle: the difference in hand sizes.
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.hand(P1, "Shock");
    }
    t.hand(P0, "Shock");
    t.enter(P0, "Sandstone Oracle");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn shatter_assumptions_discards_all_colorless_nonland_cards() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Ornithopter");
    t.hand(P1, "Wastes");
    t.hand(P1, "Grizzly Bears");
    let sa = t.hand(P0, "Shatter Assumptions");
    t.cast(P0, sa).modes(&[0]).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_hand(P1, "Wastes"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn steal_the_show_target_player_discards_any_number_then_draws_that_many() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Shock");
    t.answer_choose(P1, &objs(&[a]));
    let sts = t.hand(P0, "Steal the Show");
    t.cast(P0, sts).modes(&[0]).target(P1).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Shock"));
    assert_eq!(t.hand_size(P1), 2);
}
