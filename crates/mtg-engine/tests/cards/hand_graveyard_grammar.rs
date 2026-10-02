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

/// A sorcery costing {0} compiled from oracle text with the real compiler.
fn sorcery(name: &str, text: &str) -> mtg_engine::card::CardDef {
    use mtg_engine::oracle::{self, CompileContext};
    use mtg_engine::types::TypeLine;
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    mtg_engine::card::CardDef::custom(mtg_engine::object::Characteristics {
        name: name.into(),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        ..Default::default()
    })
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
    cr!("406.2");
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
    cr!("406.2");
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
    cr!("406.2");
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
    cr!("121.1");
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
    // On the bottom (index 0 is the bottom card): not drawn back.
    assert!(lib[..2].contains(&t.g.current(a)) && lib[..2].contains(&t.g.current(b)));
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
    cr!("608.2d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let bears = t.hand(P1, "Grizzly Bears");
    let shock = t.hand(P1, "Shock");
    t.answer_choose(P1, &objs(&[shock]));
    let pq = t.hand(P0, "Prying Questions");
    t.cast(P0, pq).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // The opponent chose which card.
    assert!(last_choice_of(&t, P1).contains(&Entity::Object(bears)));
    let top = *t.g.player(P1).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name.as_str(), "Shock");
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
    cr!("603.5");
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
    cr!("608.2d");
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
    cr!("400.3");
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

// ---------------------------------------------------------------------------
// Second batch: scaled instructions, targets in a target player's graveyard,
// distributing looked-at cards
// ---------------------------------------------------------------------------

#[test]
fn second_batch_compiles() {
    assert_supported(&[
        "Suffer the Past",
        "Drafna's Restoration",
        "Metalworker",
        "Scent of Brine",
        "Brine Seer",
        "Rofellos's Gift",
        "Borrowed Knowledge",
        "Forget",
        "Neheb, Dreadhorde Champion",
        "Apocalypse",
        "Mind Maggots",
        "Graveyard Trespasser // Graveyard Glutton",
        "The Binding of the Titans",
        "Expressive Iteration",
        "Moment of Truth",
        "Telling Time",
        "Waste Management",
        "Grub's Command",
        "Endurance",
        "Arjun, the Shifting Flame",
    ]);
    assert_compiles(&[(
        "Sanctifier en-Vec",
        "exile all cards that are black or red from all graveyards",
    )]);
}

#[test]
fn suffer_the_past_targets_cards_in_the_target_players_graveyard() {
    cr!("115.1", "601.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Shock");
    let mine = t.graveyard(P0, "Forest");
    let s = t.hand(P0, "Suffer the Past");
    t.cast(P0, s)
        .x(2)
        .target(P1)
        .targets(&objs(&[a, b]))
        .go();
    // Only cards in the chosen player's graveyard could be chosen.
    let asked = t.asked();
    let cands = asked
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. }
                if candidates.contains(&Entity::Object(a)) =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .unwrap();
    assert!(!cands.contains(&Entity::Object(mine)));
    t.resolve();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn drafnas_restoration_puts_artifact_cards_on_top() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let a = t.graveyard(P1, "Ornithopter");
    let a2 = t.graveyard(P1, "Sol Ring");
    let b = t.graveyard(P1, "Grizzly Bears");
    let s = t.hand(P0, "Drafna's Restoration");
    t.cast(P0, s).target(P1).targets(&objs(&[a, a2])).go();
    t.resolve();
    assert_eq!(t.zone(a), Zone::Library(P1));
    let lib = &t.g.player(P1).library;
    let top2 = &lib[lib.len() - 2..];
    assert!(top2.contains(&t.g.current(a)) && top2.contains(&t.g.current(a2)));
    assert_eq!(t.zone(b), Zone::Graveyard(P1));
    // The cards' owner arranged them (CR 401.4).
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Order { items, .. } if items.len() == 2)));
}

#[test]
fn metalworker_adds_two_colorless_per_revealed_artifact() {
    cr!("701.20a", "605.1a");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Metalworker");
    let a = t.hand(P0, "Ornithopter");
    let b = t.hand(P0, "Sol Ring");
    t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &objs(&[a, b]));
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::C),
        4
    );
}

#[test]
fn scent_of_brine_scales_the_tax() {
    cr!("701.20a", "118.12");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    let spell = t.g.stack[0];
    t.lands(P0, "Island", 2);
    let a = t.hand(P0, "Counterspell");
    let b = t.hand(P0, "Opt");
    let c = t.hand(P0, "Brainstorm");
    let s = t.hand(P0, "Scent of Brine");
    t.answer_choose(P0, &objs(&[a, b, c]));
    t.cast(P0, s).target(spell).go();
    // P1 has two untapped lands: can't pay {3}.
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn rofellos_gift_returns_one_enchantment_per_revealed_card() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let a = t.hand(P0, "Llanowar Elves");
    let b = t.hand(P0, "Giant Growth");
    let e1 = t.graveyard(P0, "Pacifism");
    let e2 = t.graveyard(P0, "Rancor");
    let e3 = t.graveyard(P0, "Wild Growth");
    let s = t.hand(P0, "Rofellos's Gift");
    t.answer_choose(P0, &objs(&[a, b]));
    t.answer_choose(P0, &objs(&[e1, e3]));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.zone(e1), Zone::Hand(P0));
    assert_eq!(t.zone(e3), Zone::Hand(P0));
    assert_eq!(t.zone(e2), Zone::Graveyard(P0));
}

#[test]
fn borrowed_knowledge_draws_as_many_as_discarded() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 2);
    for _ in 0..3 {
        t.hand(P0, "Shock");
    }
    for _ in 0..5 {
        t.hand(P1, "Shock");
    }
    let s = t.hand(P0, "Borrowed Knowledge");
    t.cast(P0, s).modes(&[1]).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    assert!(!t.in_hand(P0, "Shock"));
}

#[test]
fn forget_draws_as_many_as_they_discarded() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.hand(P1, "Shock");
    let f = t.hand(P0, "Forget");
    t.cast(P0, f).target(P1).go();
    t.resolve();
    // Only one card could be discarded: one is drawn.
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert!(!t.in_hand(P1, "Shock"));
}

#[test]
fn neheb_draws_and_adds_red_for_each_discarded_card() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Neheb, Dreadhorde Champion");
    let a = t.hand(P0, "Shock");
    let b = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[a, b]));
    t.attack(&[(n, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.hand_size(P0), 3);
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(
        t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::R),
        2
    );
}

#[test]
fn apocalypse_discards_your_hand() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.hand(P0, "Shock");
    t.hand(P1, "Shock");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let a = t.hand(P0, "Apocalypse");
    t.cast(P0, a).go();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn mind_maggots_counters_per_creature_card_discarded() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &objs(&[a, b]));
    let mm = t.enter(P0, "Mind Maggots");
    t.resolve_all();
    assert!(!last_choice_of(&t, P0).contains(&Entity::Object(shock)));
    assert_eq!(t.counters(mm, "+1/+1"), 4);
}

#[test]
fn expressive_iteration_distributes_three_cards() {
    cr!("608.2d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    let c = t.library_top(P0, "Shock");
    let b = t.library_top(P0, "Grizzly Bears");
    let a = t.library_top(P0, "Forest");
    let ei = t.hand(P0, "Expressive Iteration");
    t.answer_choose(P0, &objs(&[b]));
    t.answer_choose(P0, &objs(&[c]));
    t.answer_choose(P0, &objs(&[a]));
    t.cast(P0, ei).go();
    t.resolve();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    assert_eq!(t.zone(c), Zone::Library(P0));
    assert_eq!(t.g.player(P0).library[0], t.g.current(c));
    assert_eq!(t.zone(a), Zone::Exile);
    // The exiled land may be played this turn.
    let forest = t.g.current(a);
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(forest) || t.named_on_battlefield("Forest").len() == 1);
}

#[test]
fn telling_time_one_to_hand_one_on_top_one_on_bottom() {
    cr!("608.2d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let c = t.library_top(P0, "Shock");
    let b = t.library_top(P0, "Grizzly Bears");
    let a = t.library_top(P0, "Forest");
    let tt = t.hand(P0, "Telling Time");
    t.answer_choose(P0, &objs(&[a]));
    t.answer_choose(P0, &objs(&[c]));
    t.answer_choose(P0, &objs(&[b]));
    t.cast(P0, tt).go();
    t.resolve();
    assert_eq!(t.zone(a), Zone::Hand(P0));
    let lib = &t.g.player(P0).library;
    assert_eq!(*lib.last().unwrap(), t.g.current(c));
    assert_eq!(lib[0], t.g.current(b));
}

#[test]
fn waste_management_kicked_exiles_a_whole_graveyard_and_counts_creatures() {
    cr!("702.33d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 7);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Shock");
    let wm = t.hand(P0, "Waste Management");
    t.cast(P0, wm).kicked(true).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.named_on_battlefield("Rogue Token").len(), 2);
    // Unkicked: up to two cards.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Shock");
    t.graveyard(P1, "Hill Giant");
    let wm = t.hand(P0, "Waste Management");
    t.cast(P0, wm).kicked(false).targets(&objs(&[a, b])).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.named_on_battlefield("Rogue Token").len(), 1);
}

#[test]
fn grubs_command_returns_milled_goblins() {
    cr!("701.17a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 3);
    t.library_top(P1, "Goblin Guide");
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Goblin Bushwhacker");
    let creature = t.battlefield(P1, "Hill Giant");
    let gc = t.hand(P0, "Grub's Command");
    t.cast(P0, gc).modes(&[2, 3]).target(creature).target(P1).go();
    t.resolve();
    assert!(t.in_hand(P1, "Goblin Guide"));
    assert!(t.in_hand(P1, "Goblin Bushwhacker"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.graveyard_size(P1), 4); // Bears, Hill Giant, two fillers
}

#[test]
fn endurance_may_target_no_one() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Endurance");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), 32);
    // No target: nothing happens.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Shock");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Endurance");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn arjun_cycles_the_whole_hand() {
    cr!("401.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arjun, the Shifting Flame");
    t.lands(P0, "Mountain", 1);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    // The two other cards went to the bottom; two were drawn.
    assert_eq!(t.hand_size(P0), 2);
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.library_size(P0), 30);
    // Their owner arranged them.
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { items, .. } if items.len() == 2)));
}

#[test]
fn sanctifier_en_vec_exiles_black_and_red_cards_from_graveyards() {
    cr!("406.2");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Doom Blade");
    t.graveyard(P1, "Grizzly Bears");
    t.enter(P0, "Sanctifier en-Vec");
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Doom Blade"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------
// Third batch: graveyard and hand counts, cards put into a graveyard this turn, looking
// ---------------------------------------------------------------------------

#[test]
fn third_batch_compiles() {
    assert_supported(&[
        "Dark Matter Manipulator",
        "Swimmer in Nightmares",
        "Adamaro, First to Desire",
        "Nighthawk Scavenger",
        "Abyssal Harvester",
        "Reenact the Crime",
        "Case the Joint",
        "Lay Bare",
        "Pore Over the Pages",
    ]);
}

#[test]
fn dark_matter_manipulator_counts_groups_of_seven() {
    cr!("613.4c");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dark Matter Manipulator");
    for _ in 0..6 {
        t.graveyard(P0, "Shock");
    }
    t.g.recompute();
    assert_eq!(t.pt(d), (1, 2));
    t.graveyard(P0, "Shock");
    t.g.recompute();
    assert_eq!(t.pt(d), (3, 2));
    for _ in 0..7 {
        t.graveyard(P0, "Shock");
    }
    t.g.recompute();
    assert_eq!(t.pt(d), (5, 2));
}

#[test]
fn swimmer_in_nightmares_any_single_graveyard() {
    cr!("613.4c");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Swimmer in Nightmares");
    for _ in 0..5 {
        t.graveyard(P0, "Shock");
        t.graveyard(P1, "Shock");
    }
    t.g.recompute();
    // Ten cards in all, but not in a single graveyard.
    assert_eq!(t.pt(s), (1, 4));
    for _ in 0..5 {
        t.graveyard(P1, "Shock");
    }
    t.g.recompute();
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn adamaro_counts_the_largest_opponent_hand() {
    cr!("604.3");
    let mut t = TestGame::with_config(3, Default::default());
    let a = t.battlefield(P0, "Adamaro, First to Desire");
    for _ in 0..5 {
        t.hand(P0, "Shock");
    }
    t.hand(P1, "Shock");
    for _ in 0..3 {
        t.hand(P2, "Shock");
    }
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 3));
}

#[test]
fn nighthawk_scavenger_counts_card_types_in_opponents_graveyards() {
    cr!("604.3");
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nighthawk Scavenger");
    t.graveyard(P0, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(n), (1, 3));
    t.graveyard(P1, "Forest");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Ornithopter"); // artifact creature: two types
    t.g.recompute();
    assert_eq!(t.pt(n), (5, 3));
}

#[test]
fn reenact_the_crime_only_cards_put_there_this_turn() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    // A card that went to the graveyard on an earlier turn.
    let old = t.graveyard(P1, "Grizzly Bears");
    t.g.objects[old.0 as usize].entered_turn = 0;
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    let bolt_gy = t.g.current(bolt);
    t.lands(P0, "Island", 4);
    let r = t.hand(P0, "Reenact the Crime");
    t.cast(P0, r).target(bolt_gy).go();
    let asked = t.asked();
    let cands = asked
        .iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap();
    assert!(cands.contains(&Entity::Object(bolt_gy)));
    assert!(!cands.contains(&Entity::Object(old)));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(bolt_gy), Zone::Exile);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn exile_creature_cards_put_into_graveyards_from_the_battlefield_this_turn() {
    cr!("400.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let discarded = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let s = t.custom(
        P0,
        sorcery(
            "Cry Probe",
            "Exile all creature cards in all graveyards that were put there from the battlefield this turn.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    // A card not put there from the battlefield stays.
    assert_eq!(t.zone(discarded), Zone::Graveyard(P1));
}

#[test]
fn case_the_joint_looks_at_each_top_card() {
    cr!("401.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let a = t.library_top(P0, "Shock");
    let b = t.library_top(P1, "Grizzly Bears");
    let c = t.hand(P0, "Case the Joint");
    t.cast(P0, c).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.zone(b), Zone::Library(P1));
    assert_eq!(*t.g.player(P1).library.last().unwrap(), t.g.current(b));
    let _ = a;
}

#[test]
fn lay_bare_counters_then_looks() {
    cr!("701.6a");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    let spell = t.g.stack[0];
    t.lands(P0, "Island", 4);
    let lb = t.hand(P0, "Lay Bare");
    t.cast(P0, lb).target(spell).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn pore_over_the_pages_draws_untaps_discards() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Island", 5);
    let p = t.hand(P0, "Pore Over the Pages");
    t.answer_choose(P0, &objs(&lands[..2]));
    t.cast(P0, p).go();
    t.resolve();
    let untapped = lands.iter().filter(|l| !t.obj_now(**l).tapped).count();
    assert_eq!(untapped, 2);
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 2);
}

#[test]
fn has_activated_abilities_of_graveyard_cards_compiles() {
    assert_supported(&[
        "Necrotic Ooze",
        "Mirran Safehouse",
        "Trazyn the Infinite",
        "Thranduil, the Elvenking",
    ]);
}

#[test]
fn necrotic_ooze_gains_activated_abilities_of_creature_cards_in_graveyards() {
    cr!("613.1f");
    ruling!(
        "Necrotic Ooze",
        "Necrotic Ooze gains only activated abilities"
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    t.g.recompute();
    let activated = |t: &TestGame| {
        t.obj_now(ooze)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
            .count()
    };
    assert_eq!(activated(&t), 0);
    // A creature card in an opponent's graveyard: its {T} ability.
    t.graveyard(P1, "Prodigal Sorcerer");
    // A noncreature card's abilities aren't gained; nor are triggered/static abilities.
    t.graveyard(P1, "Sol Ring");
    t.graveyard(P0, "Wall of Omens");
    t.g.recompute();
    assert_eq!(activated(&t), 1);
    t.activate(P0, ooze, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // When the card leaves the graveyard, the ability is gone.
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    let ps = t.graveyard(P1, "Prodigal Sorcerer");
    t.g.recompute();
    assert_eq!(activated(&t), 1);
    t.g.move_object(ps, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.recompute();
    let _ = ooze;
    assert_eq!(activated(&t), 0);
}

#[test]
fn thranduil_only_elf_cards_in_your_graveyard() {
    cr!("613.1f");
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Thranduil, the Elvenking");
    t.graveyard(P0, "Llanowar Elves");
    t.graveyard(P1, "Elvish Mystic");
    t.graveyard(P0, "Prodigal Sorcerer");
    t.g.recompute();
    let n = t
        .obj_now(th)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(n, 1);
}

// ---------------------------------------------------------------------------
// "twice X" and "half X"
// ---------------------------------------------------------------------------

#[test]
fn twice_and_half_x_compile() {
    assert_supported(&[
        "Erebos's Intervention",
        "Drown in Dreams",
        "Heliod's Intervention",
        "Procrastinate",
        "Sanguine Sacrament",
        "Wan Shi Tong, Librarian",
        "Hydroid Krasis",
    ]);
}

#[test]
fn drown_in_dreams_mills_twice_x() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let d = t.hand(P0, "Drown in Dreams");
    t.cast(P0, d).x(3).modes(&[1]).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 6);
}

#[test]
fn erebos_intervention_exiles_up_to_twice_x_cards() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let cards: Vec<ObjectId> = (0..5).map(|_| t.graveyard(P1, "Shock")).collect();
    let d = t.hand(P0, "Erebos's Intervention");
    t.cast(P0, d).x(2).modes(&[1]).targets(&objs(&cards[..4])).go();
    // At most four could be chosen.
    let max = t
        .asked()
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .unwrap();
    assert_eq!(max, 4);
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn hydroid_krasis_halves_x_rounding_down() {
    cr!("107.1a", "107.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 3);
    t.g.players[P0.idx()].life = 10;
    let k = t.hand(P0, "Hydroid Krasis");
    t.cast(P0, k).x(5).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 12, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.counters(t.g.current(k), "+1/+1"), 5);
}

#[test]
fn wan_shi_tong_draws_half_x_rounded_down() {
    cr!("107.1a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let w = t.hand(P0, "Wan Shi Tong, Librarian");
    t.cast(P0, w).x(3).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(w), "+1/+1"), 3);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn mana_seism_adds_colorless_per_land_sacrificed() {
    cr!("701.21a");
    assert_supported(&["Mana Seism"]);
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 4);
    let ms = t.hand(P0, "Mana Seism");
    // Two Mountains pay for it; the other two are sacrificed.
    t.cast(P0, ms).go();
    let untapped: Vec<ObjectId> = lands
        .iter()
        .copied()
        .filter(|l| !t.obj_now(*l).tapped)
        .collect();
    assert_eq!(untapped.len(), 2);
    t.answer_choose(P0, &objs(&untapped));
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(
        t.g.player(P0).mana_pool.count(mtg_engine::mana::ManaType::C),
        2
    );
}

// ---------------------------------------------------------------------------
// Fourth batch: "~'s owner/controller" subjects, values of objects, hand differences
// ---------------------------------------------------------------------------

#[test]
fn fourth_batch_compiles() {
    assert_supported(&[
        "Gandalf, Wandering Wizard",
        "Morbid Curiosity",
        "Lifeblood Hydra",
        "Balance of Power",
    ]);
}

#[test]
fn gandalf_owner_shuffles_him_away_and_draws() {
    cr!("701.24a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    let g = t.battlefield(P0, "Gandalf, Wandering Wizard");
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(g), Zone::Library(P0));
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn morbid_curiosity_draws_the_sacrificed_permanents_mana_value() {
    cr!("601.2h");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let mc = t.hand(P0, "Morbid Curiosity");
    t.answer_choose(P0, &objs(&[giant]));
    t.cast(P0, mc).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.hand_size(P0), 4);
}

#[test]
fn lifeblood_hydra_gains_and_draws_its_power() {
    cr!("603.10a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let h = t.hand(P0, "Lifeblood Hydra");
    t.cast(P0, h).x(3).go();
    t.resolve_all();
    let h = t.g.current(h);
    assert_eq!(t.pt(h), (3, 3));
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(h).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn balance_of_power_draws_the_difference() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    for _ in 0..5 {
        t.hand(P1, "Shock");
    }
    t.hand(P0, "Shock");
    let b = t.hand(P0, "Balance of Power");
    t.cast(P0, b).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 5);
    // Fewer cards: nothing.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.hand(P0, "Shock");
    let b = t.hand(P0, "Balance of Power");
    t.cast(P0, b).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1);
}

// ---------------------------------------------------------------------------
// Fifth batch: opponents of another player, whichever is greater, hand/life comparisons
// ---------------------------------------------------------------------------

#[test]
fn fifth_batch_compiles() {
    assert_supported(&[
        "Heartwood Storyteller",
        "Standstill",
        "Prophet of the Scarab",
        "Wojek Investigator",
        "Survival Cache",
    ]);
}

#[test]
fn standstill_the_casters_opponents_draw() {
    cr!("806.1");
    let mut t = TestGame::with_config(3, Default::default());
    t.battlefield(P0, "Standstill");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P2).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Standstill"));
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P2), 3);
}

#[test]
fn heartwood_storyteller_each_opponent_of_the_caster_may_draw() {
    cr!("102.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartwood Storyteller");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P1, true);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn prophet_of_the_scarab_draws_the_greater_number() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.graveyard(P0, "Gravecrawler");
    }
    t.battlefield(P0, "Gravecrawler");
    t.enter(P0, "Prophet of the Scarab");
    t.resolve_all();
    // Zombies you control: Gravecrawler and the Prophet (2); Zombie cards: 3.
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn wojek_investigator_counts_opponents_with_bigger_hands() {
    cr!("701.16a");
    let mut t = TestGame::with_config(3, Default::default());
    t.battlefield(P0, "Wojek Investigator");
    t.hand(P0, "Shock");
    t.hand(P1, "Shock");
    t.hand(P1, "Shock");
    t.hand(P2, "Shock");
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    // P1 drew for their turn (3 cards), P2 too (2 cards); P0 has 1: two Clues.
    assert_eq!(t.named_on_battlefield("Clue Token").len(), 2);
}

#[test]
fn survival_cache_draws_with_more_life_than_an_opponent() {
    cr!("119.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Survival Cache");
    t.g.players[P0.idx()].life = 19;
    t.cast(P0, s).go();
    t.resolve();
    // 21 > 20.
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), 1);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let s = t.hand(P0, "Survival Cache");
    t.g.players[P0.idx()].life = 17;
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn chaos_warp_owner_shuffles_then_reveals() {
    cr!("701.24a", "701.20a");
    assert_supported(&["Chaos Warp"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let target = t.battlefield(P1, "Hill Giant");
    let c = t.hand(P0, "Chaos Warp");
    t.cast(P0, c).target(target).go();
    t.resolve();
    // The shuffle puts the giant somewhere in a library of fillers and a giant; a filler
    // (not a permanent card) or the giant may be revealed.
    assert_eq!(t.zone(target), Zone::Library(P1));
    let on_bf = t.named_on_battlefield("Hill Giant").len();
    let lib = t.library_size(P1);
    assert_eq!(on_bf + lib, 31);
    // A permanent card on top goes onto the battlefield under its owner's control.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let target = t.battlefield(P1, "Hill Giant");
    // Only permanent cards in the library: whatever is revealed enters.
    t.g.players[P1.idx()].library.clear();
    t.library_top(P1, "Grizzly Bears");
    let c = t.hand(P0, "Chaos Warp");
    t.cast(P0, c).target(target).go();
    t.resolve();
    let entered: Vec<ObjectId> = ["Grizzly Bears", "Hill Giant"]
        .iter()
        .flat_map(|n| t.named_on_battlefield(n))
        .collect();
    assert_eq!(entered.len(), 1);
    assert_eq!(t.obj_now(entered[0]).controller, P1);
}

#[test]
fn body_snatcher_is_exiled_unless_you_discard_a_creature_card() {
    cr!("118.12a");
    assert_supported(&["Body Snatcher"]);
    let mut t = TestGame::new(2);
    t.hand(P0, "Shock");
    let bs = t.enter(P0, "Body Snatcher");
    t.resolve_all();
    assert_eq!(t.zone(bs), Zone::Exile);
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[bears]));
    let bs = t.enter(P0, "Body Snatcher");
    t.resolve_all();
    assert!(t.on_battlefield(bs));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn narset_transcendent_takes_a_noncreature_nonland_card() {
    cr!("701.20a");
    assert_supported(&["Narset Transcendent"]);
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset Transcendent");
    t.library_top(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.activate(P0, n, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Narset Transcendent");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.activate(P0, n, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P0));
}

#[test]
fn treasure_hunt_puts_every_revealed_card_into_hand() {
    cr!("701.20a");
    assert_supported(&["Treasure Hunt"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    t.library_top(P0, "Shock");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    let th = t.hand(P0, "Treasure Hunt");
    t.cast(P0, th).go();
    t.resolve();
    assert!(t.in_hand(P0, "Forest") && t.in_hand(P0, "Island") && t.in_hand(P0, "Shock"));
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn precognition_may_bottom_the_looked_at_card() {
    cr!("603.5");
    assert_supported(&["Precognition"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Precognition");
    let top = t.library_top(P1, "Grizzly Bears");
    // P1 draws the Shock on their turn; the Bears are on top at P0's upkeep.
    t.library_top(P1, "Shock");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert!(t.in_hand(P1, "Shock"));
    assert_eq!(t.g.player(P1).library[0], t.g.current(top));
}

#[test]
fn armored_kincaller_reveal_or_another_dinosaur() {
    cr!("608.2c");
    assert_supported(&["Armored Kincaller"]);
    // Neither: no life.
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Armored Kincaller");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Revealing a Dinosaur.
    let mut t = TestGame::new(2);
    let d = t.hand(P0, "Carnage Tyrant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[d]));
    t.enter(P0, "Armored Kincaller");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Controlling another Dinosaur, declining to reveal.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carnage Tyrant");
    t.answer_yes(P0, false);
    t.enter(P0, "Armored Kincaller");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

// ---------------------------------------------------------------------------
// Sixth batch
// ---------------------------------------------------------------------------

#[test]
fn sixth_batch_compiles() {
    assert_supported(&[
        "Witherbloom Command",
        "Rise of the Deathbringer",
        "Skull Raid",
        "Jace, the Perfected Mind",
        "Phyrexian Furnace",
        "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen",
    ]);
}

#[test]
fn witherbloom_command_mill_then_you_return_a_land() {
    cr!("701.17a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    let land = t.graveyard(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let w = t.hand(P0, "Witherbloom Command");
    t.answer_choose(P0, &objs(&[land]));
    t.cast(P0, w).modes(&[0, 2]).target(P1).target(bears).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 3);
    assert_eq!(t.pt(bears), (-1, 1));
    assert_eq!(t.zone(land), Zone::Hand(P0));
}

#[test]
fn rise_of_the_deathbringer_loses_life_per_card_drawn() {
    cr!("121.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.battlefield(P0, "Hill Giant");
    let r = t.hand(P0, "Rise of the Deathbringer");
    t.cast(P0, r).modes(&[0]).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn skull_raid_draws_the_shortfall() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Shock");
    let s = t.hand(P0, "Skull Raid");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.hand_size(P0), 1);
    // Two discarded: no draw.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Shock");
    t.hand(P1, "Shock");
    let s = t.hand(P0, "Skull Raid");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn chandra_ablaze_red_discard_condition() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    let s = t.custom(
        P0,
        sorcery(
            "Ablaze Probe",
            "Discard a card. If a red card is discarded this way, ~ deals 4 damage to any target.",
        ),
        Zone::Hand(P0),
    );
    t.hand(P0, "Shock");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    let mut t = TestGame::new(2);
    let s = t.custom(
        P0,
        sorcery(
            "Ablaze Probe",
            "Discard a card. If a red card is discarded this way, ~ deals 4 damage to any target.",
        ),
        Zone::Hand(P0),
    );
    t.hand(P0, "Grizzly Bears");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn jace_perfected_mind_twenty_cards_in_a_graveyard() {
    cr!("701.17a");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, the Perfected Mind");
    t.g.objects[j.0 as usize].counters.insert("loyalty".into(), 5);
    for _ in 0..17 {
        t.graveyard(P1, "Shock");
    }
    t.activate(P0, j, 1, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 20);
    assert_eq!(t.hand_size(P0), 3);
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, the Perfected Mind");
    t.g.objects[j.0 as usize].counters.insert("loyalty".into(), 5);
    t.activate(P0, j, 1, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn phyrexian_furnace_exiles_the_bottom_card() {
    cr!("404.2");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Phyrexian Furnace");
    let first = t.graveyard(P1, "Grizzly Bears");
    let second = t.graveyard(P1, "Shock");
    t.activate(P0, f, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Exile);
    assert_eq!(t.zone(second), Zone::Graveyard(P1));
}

#[test]
fn nicol_bolas_exiles_all_but_the_bottom_card() {
    cr!("406.2");
    let mut t = TestGame::new(2);
    let bottom = t.g.player(P1).library[0];
    let s = t.custom(
        P0,
        sorcery("Bolas Probe", "Exile all but the bottom card of target player's library."),
        Zone::Hand(P0),
    );
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.library_size(P1), 1);
    assert_eq!(t.g.player(P1).library[0], bottom);
}

// ---------------------------------------------------------------------------
// Seventh batch
// ---------------------------------------------------------------------------

#[test]
fn seventh_batch_compiles() {
    assert_supported(&[
        "Sea Gate Restoration // Sea Gate, Reborn",
        "Enter the Infinite",
        "Egon, God of Death // Throne of Death",
        "Skyfisher Spider",
    ]);
}

#[test]
fn sea_gate_restoration_removes_the_maximum_hand_size() {
    cr!("402.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 7);
    for _ in 0..7 {
        t.hand(P0, "Shock");
    }
    let s = t.hand(P0, "Sea Gate Restoration // Sea Gate, Reborn");
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 15);
    // At cleanup nothing is discarded.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.hand_size(P0), 15);
}

#[test]
fn egon_exiles_two_or_sacrifices_and_draws() {
    cr!("608.2c");
    ruling!(
        "Egon, God of Death // Throne of Death",
        "If there's only one card in your graveyard, you won't exile it."
    );
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Egon, God of Death // Throne of Death");
    let only = t.graveyard(P0, "Shock");
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(only), Zone::Graveyard(P0));
    assert!(!t.on_battlefield(e));
    // Hand: the card drawn by P0's... P0 hasn't drawn yet this turn (upkeep): one card.
    assert_eq!(t.hand_size(P0), 1);
    // With two cards: both are exiled; Egon stays.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Egon, God of Death // Throne of Death");
    let a = t.graveyard(P0, "Shock");
    let b = t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(t.on_battlefield(e));
}

#[test]
fn skyfisher_spider_may_exile_itself_from_the_graveyard() {
    cr!("603.10a");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skyfisher Spider");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(s).go();
    t.resolve_all();
    // Two creature cards (the Bears and the Spider itself): 2 life.
    assert_eq!(t.life(P0), 22);
    assert!(t.in_exile("Skyfisher Spider"));
}

#[test]
fn listed_kind_conditions_and_optional_target_players_compile() {
    assert_supported(&["Unagi's Spray", "Splash Portal", "Veteran Ice Climber"]);
}

#[test]
fn unagis_spray_draws_with_a_listed_creature_type() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Unagi's Spray");
    t.cast(P0, s).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (-2, 2));
    assert_eq!(t.hand_size(P0), 0);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let otter = t.battlefield(P0, "Elusive Otter // Grove's Bounty");
    t.g.recompute();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Unagi's Spray");
    t.cast(P0, s).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "{:?}", t.obj_now(otter).chars.subtypes);
}

#[test]
fn veteran_ice_climber_may_target_no_player() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Veteran Ice Climber");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(v, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Veteran Ice Climber");
    t.answer_targets(P0, &[]);
    t.attack(&[(v, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn guiding_spirit_moves_the_top_creature_card_of_a_graveyard() {
    cr!("404.2");
    assert_supported(&["Guiding Spirit"]);
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Guiding Spirit");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.activate(P0, gs, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1));
    assert_eq!(*t.g.player(P1).library.last().unwrap(), t.g.current(bears));
    // A creature card that isn't the top card isn't moved.
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Guiding Spirit");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Shock");
    t.activate(P0, gs, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn bison_whistle_a_bison_put_onto_the_battlefield_isnt_also_put_into_hand() {
    cr!("400.7");
    assert_supported(&["Bison Whistle"]);
    // A Bison creature card: onto the battlefield; the hand option no longer applies.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let w = t.battlefield(P0, "Bison Whistle");
    let bison = t.library_top(P0, "Appa, the Vigilant");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bison), Zone::Battlefield);
    assert_eq!(t.named_on_battlefield("Appa, the Vigilant").len(), 1);
    // A non-Bison creature card: into the hand (not the graveyard).
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let w = t.battlefield(P0, "Bison Whistle");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Hand(P0));
    // A land: may go to the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let w = t.battlefield(P0, "Bison Whistle");
    let land = t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.activate(P0, w, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(land), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------
// Review: random discards, delayed instructions in a list, conditions with commas, half X
// ---------------------------------------------------------------------------

#[test]
fn rag_man_discards_a_creature_card_at_random() {
    cr!("701.9b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let rag = t.battlefield(P0, "Rag Man");
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    let shock = t.hand(P1, "Shock");
    t.activate(P0, rag, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    // One of the creature cards, chosen by nobody.
    let gone = [bears, giant]
        .iter()
        .filter(|c| t.zone(**c) == Zone::Graveyard(P1))
        .count();
    assert_eq!(gone, 1);
    assert_eq!(t.zone(shock), Zone::Hand(P1));
    assert!(
        !t.asked()
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })),
        "the discarding player chose"
    );
}

#[test]
fn mangaras_blessing_gains_life_now_and_returns_at_end_step() {
    cr!("603.7a");
    ruling!(
        "Mangara's Blessing",
        "The 2 life from having it discarded is gained when the triggered ability resolves"
    );
    ruling!(
        "Mangara's Blessing",
        "only returned if it is still in the graveyard at end of turn"
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let mb = t.hand(P0, "Mangara's Blessing");
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        t.lands(P1, "Swamp", 3);
        let rot = t.hand(P1, "Mind Rot");
        t.g.turn.priority = Some(P1);
        t.cast(P1, rot).target(P0).go();
        t.resolve_all();
        // The life is gained as the trigger resolves; the card waits in the graveyard.
        assert_eq!(t.life(P0), 22);
        assert_eq!(t.zone(mb), Zone::Graveyard(P0));
        if leaves {
            // It leaves the graveyard and comes back: a new object, not returned.
            let now = t.g.current(mb);
            t.g.move_object(now, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
            let back = t.g.current(mb);
            t.g.move_object(back, Zone::Graveyard(P0), mtg_engine::events::MoveCause::Effect, None);
        }
        t.advance_to_step(mtg_engine::turn::Step::End);
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Mangara's Blessing"), !leaves);
        assert_eq!(t.life(P0), 22);
    }
}

#[test]
fn contaminated_drink_draws_x_and_gets_half_x_rad_counters() {
    cr!("107.1a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    t.lands(P0, "Swamp", 1);
    let cd = t.hand(P0, "Contaminated Drink");
    t.cast(P0, cd).x(5).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 5);
    assert_eq!(t.g.player(P0).counter(mtg_engine::types::counters::RAD), 3);
}

#[test]
fn necrotic_ooze_abilities_naming_their_card_refer_to_the_ooze() {
    cr!("613.1f");
    ruling!(
        "Necrotic Ooze",
        "treat Necrotic Ooze's version of that ability as though it referenced Necrotic Ooze"
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    let troll = t.graveyard(P1, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    t.activate(P0, ooze, 0, &[]).unwrap();
    t.resolve_all();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    t.cast(P1, bolt).target(ooze).go();
    t.resolve_all();
    // The Ooze (4/3) was regenerated: still on the battlefield, tapped.
    assert!(t.on_battlefield(ooze), "{}", t.dump_log());
    assert!(t.obj_now(ooze).tapped);
    assert_eq!(t.zone(troll), Zone::Graveyard(P1));
}

#[test]
fn reviewed_wordings_are_read_faithfully_or_not_at_all() {
    // "Discard ... at random" with a number nobody chose isn't read.
    assert!(!card("Rites of Initiation").unsupported_text().is_empty());
    // "From a single graveyard": all the cards from one graveyard.
    assert!(card("Jötun Grunt")
        .unsupported_text()
        .iter()
        .any(|u| u.contains("single graveyard")));
    // The fateful hour sentence has more instructions than the text up to the comma.
    assert!(card("Courageous Resolve")
        .unsupported_text()
        .iter()
        .any(|u| u.contains("can't lose life")));
}

#[test]
fn jace_perfected_mind_mills_three_times_x() {
    cr!("701.17a", "107.3a");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jace, the Perfected Mind");
    t.g.objects[j.0 as usize].counters.insert("loyalty".into(), 5);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.activate(P0, j, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 6);
    assert_eq!(t.obj_now(j).counters.get("loyalty").copied(), Some(3));
}
