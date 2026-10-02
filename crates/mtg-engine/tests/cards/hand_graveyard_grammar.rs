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

// ---------------------------------------------------------------------------
// Revealing cards from a hand
// ---------------------------------------------------------------------------

#[test]
fn reveal_family_compiles() {
    assert_supported(&[
        "Sacellum Godspeaker",
        "Nightshade Assassin",
        "Priest of the Wakening Sun",
        "Magus of the Scroll",
        "Hired Torturer",
        "Assembly Hall",
        "Domri Rade",
        "Herald's Horn",
        "Scent of Nightshade",
        "Infernal Tutor",
    ]);
    assert_compiles(&[(
        "Urza Assembles the Titans",
        "If a planeswalker card is revealed this way, put it into your hand",
    )]);
}

#[test]
fn sacellum_godspeaker_adds_mana_per_card_revealed() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let sg = t.battlefield(P0, "Sacellum Godspeaker");
    let a = t.hand(P0, "Craw Wurm");
    let b = t.hand(P0, "Shivan Dragon");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[a, b]));
    t.activate(P0, sg, 0, &[]).unwrap();
    t.resolve_all();
    let offered = last_choice_of(&t, P0);
    assert!(!offered.contains(&Entity::Object(bears)));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    // The revealed cards stay in hand.
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn scent_of_nightshade_counts_revealed_black_cards() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Doom Blade");
    let c = t.hand(P0, "Duress");
    let target = t.battlefield(P1, "Hill Giant");
    let s = t.hand(P0, "Scent of Nightshade");
    t.answer_choose(P0, &objs(&[b, c]));
    t.cast(P0, s).target(target).go();
    t.resolve();
    let offered = last_choice_of(&t, P0);
    assert!(!offered.contains(&Entity::Object(a)));
    assert_eq!(t.pt(target), (1, 1));
}

#[test]
fn priest_of_the_wakening_sun_needs_a_dinosaur_to_gain_life() {
    cr!("603.5", "701.20a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Priest of the Wakening Sun");
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Priest of the Wakening Sun");
    let d = t.hand(P0, "Carnage Tyrant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[d]));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_hand(P0, "Carnage Tyrant"));
}

#[test]
fn domri_rade_reveals_a_creature_from_the_top() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri Rade");
    t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.activate(P0, domri, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // A noncreature card stays on top.
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri Rade");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.activate(P0, domri, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bolt), Zone::Library(P0));
    assert_eq!(*t.g.player(P0).library.last().unwrap(), t.g.current(bolt));
}

#[test]
fn hired_torturer_reveals_a_random_card() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let ht = t.battlefield(P0, "Hired Torturer");
    t.hand(P1, "Grizzly Bears");
    t.activate(P0, ht, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P1), 1);
    assert!(t.dump_log().to_lowercase().contains("reveal"));
}

// ---------------------------------------------------------------------------
// Moving cards between hands, libraries and graveyards
// ---------------------------------------------------------------------------

#[test]
fn put_family_compiles() {
    assert_supported(&[
        "Draugr Thought-Thief",
        "Eye Spy",
        "Wu Spy",
        "Jace, the Living Guildpact",
        "Agonizing Memories",
        "Landscaper Colos",
        "Rishadan Pawnshop",
        "Dramatic Accusation",
        "Nulltread Gargantuan",
        "Repopulate",
        "Kellan, Daring Traveler // Journey On",
        "Skirk Drill Sergeant",
    ]);
    assert_compiles(&[(
        "Jace, the Mind Sculptor",
        "You may put that card on the bottom of that player's library",
    )]);
}

#[test]
fn draugr_thought_thief_may_mill_the_looked_at_card() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    let top = t.library_top(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Draugr Thought-Thief");
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Graveyard(P1));
}

#[test]
fn wu_spy_puts_one_of_the_two_into_the_graveyard() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    let a = t.library_top(P1, "Grizzly Bears");
    let b = t.library_top(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &objs(&[a]));
    t.enter(P0, "Wu Spy");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Graveyard(P1));
    assert_eq!(t.zone(b), Zone::Library(P1));
    assert_eq!(*t.g.player(P1).library.last().unwrap(), t.g.current(b));
}

#[test]
fn landscaper_colos_puts_an_opponents_card_on_the_bottom() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    let g = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &objs(&[g]));
    t.enter(P0, "Landscaper Colos");
    t.resolve_all();
    assert_eq!(t.zone(g), Zone::Library(P1));
    assert_eq!(t.g.player(P1).library[0], t.g.current(g));
}

#[test]
fn rishadan_pawnshop_shuffles_into_owners_library() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let pawn = t.battlefield(P0, "Rishadan Pawnshop");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, pawn, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P0));
    assert_eq!(t.library_size(P0), 31);
}

#[test]
fn repopulate_shuffles_creature_cards_back() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Shock");
    let r = t.hand(P0, "Repopulate");
    t.cast(P0, r).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.library_size(P1), 32);
}

#[test]
fn kellan_otherwise_may_mill_the_revealed_card() {
    cr!("701.20a");
    // A creature with mana value 3 or less goes to hand.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Daring Traveler // Journey On");
    t.library_top(P0, "Grizzly Bears");
    t.attack(&[(k, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Otherwise it may go to the graveyard.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Daring Traveler // Journey On");
    t.library_top(P0, "Shivan Dragon");
    t.answer_yes(P0, true);
    t.attack(&[(k, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
    // ... or stay on top.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kellan, Daring Traveler // Journey On");
    let sd = t.library_top(P0, "Shivan Dragon");
    t.answer_yes(P0, false);
    t.attack(&[(k, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(*t.g.player(P0).library.last().unwrap(), t.g.current(sd));
}

// ---------------------------------------------------------------------------
// "A or B" instructions
// ---------------------------------------------------------------------------

#[test]
fn either_or_family_compiles() {
    assert_supported(&[
        "K'un-Lun Warrior",
        "Crypt Lurker",
        "Highway Robbery",
        "Reckless Detective",
        "Contract Hero",
    ]);
    assert_compiles(&[(
        "Chandra, Spark Hunter",
        "You may sacrifice an artifact or discard a card",
    )]);
}

#[test]
fn kun_lun_warrior_sacrifices_or_discards_then_draws() {
    cr!("608.2c");
    // Discarding.
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.answer_choose(P0, &objs(&[c]));
    t.enter(P0, "K'un-Lun Warrior");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.hand_size(P0), 1);
    // Sacrificing an artifact.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(0),
    );
    t.enter(P0, "K'un-Lun Warrior");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.hand_size(P0), 1);
    // Declining: nothing happens.
    let mut t = TestGame::new(2);
    t.hand(P0, "Shock");
    t.answer_yes(P0, false);
    t.enter(P0, "K'un-Lun Warrior");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Shock"));
}

// ---------------------------------------------------------------------------
// Statics for cards in graveyards and hands
// ---------------------------------------------------------------------------

#[test]
fn zone_static_family_compiles() {
    assert_supported(&[
        "Lier, Disciple of the Drowned",
        "Yixlid Jailer",
        "Solemn Doomguide",
        "Iroh, Grand Lotus",
        "Return the Past",
    ]);
    assert_compiles(&[(
        "Norman Osborn // Green Goblin",
        "Each nonland card in your graveyard has mayhem",
    )]);
}

#[test]
fn lier_grants_flashback_to_instants_and_sorceries_in_your_graveyard() {
    cr!("613.1f", "702.34a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lier, Disciple of the Drowned");
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.cast(P0, bolt)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .target(P1)
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(bolt), Zone::Exile);
    // An opponent's graveyard isn't affected.
    let theirs = t.graveyard(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    let r = t
        .cast(P1, theirs)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .target(P0)
        .try_go();
    assert!(r.is_err());
}

#[test]
fn iroh_flashback_only_during_your_turn() {
    cr!("613.1f", "702.34a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iroh, Grand Lotus");
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let r = t
        .cast(P0, bolt)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .target(P1)
        .try_go();
    assert!(r.is_err());
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P0, bolt)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .target(P1)
        .go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn yixlid_jailer_removes_abilities_of_graveyard_cards() {
    cr!("613.1f");
    let mut t = TestGame::new(2);
    let g = t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert!(!t.obj_now(g).chars.abilities.is_empty());
    t.battlefield(P0, "Yixlid Jailer");
    t.g.recompute();
    assert!(t.obj_now(g).chars.abilities.is_empty());
}
