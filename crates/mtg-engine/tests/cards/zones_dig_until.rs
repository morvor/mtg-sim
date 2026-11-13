//! Card-flow grammar IV (`cards2-zones-dig`): reveal-until variants with another player
//! revealing ("Target opponent reveals cards from the top of their library until four
//! land cards are revealed"), a running total ("until that player has exiled cards with
//! total mana value 20 or greater"), a count cap ("until an artifact card or X cards are
//! revealed, whichever comes first"), the found card described again ("the nonland card",
//! "the creature card"), "If it is" and "If an artifact card is revealed this way", and
//! that player putting the rest somewhere.

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// Replaces `p`'s library with the named cards; the last one named ends up on top.
fn library(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.0 as usize].library.clear();
    names.iter().map(|n| t.library_top(p, n)).collect()
}

fn names_in(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    let mut v: Vec<String> = ids
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    names_in(t, &t.g.player(p).graveyard.clone())
}

fn exile_names(t: &TestGame) -> Vec<String> {
    names_in(t, &t.g.exile.clone())
}

/// `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

#[test]
fn mind_funeral_target_opponent_reveals_until_four_land_cards() {
    cr!("701.20a");
    ruling!(
        "Mind Funeral",
        "The four land cards, as well as all the other cards revealed during this process"
    );
    assert_supported("Mind Funeral");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    // Top first: Forest, Shock, Island, Grizzly Bears, Swamp, Mountain, Lightning Bolt.
    library(
        &mut t,
        P1,
        &[
            "Lightning Bolt",
            "Mountain",
            "Swamp",
            "Grizzly Bears",
            "Island",
            "Shock",
            "Forest",
        ],
    );
    let spell = t.hand(P0, "Mind Funeral");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(
        graveyard_names(&t, P1),
        ["Forest", "Grizzly Bears", "Island", "Mountain", "Shock", "Swamp"]
    );
    // The card below the fourth land wasn't revealed.
    assert_eq!(library_names(&t, P1), ["Lightning Bolt"]);
}

#[test]
fn mind_funeral_with_fewer_than_four_lands_mills_the_whole_library() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 2);
    library(&mut t, P1, &["Lightning Bolt", "Forest", "Shock", "Island"]);
    let spell = t.hand(P0, "Mind Funeral");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 4);
}

#[test]
fn tashas_hideous_laughter_exiles_until_a_total_mana_value_of_twenty() {
    cr!("701.20a", "202.3");
    assert_supported("Tasha's Hideous Laughter");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 3);
    // P1, top first: Forest (0), Serra Angel (5), Emrakul (15) — 20 — then Grizzly Bears.
    library(
        &mut t,
        P1,
        &[
            "Grizzly Bears",
            "Emrakul, the Aeons Torn",
            "Serra Angel",
            "Forest",
        ],
    );
    // P2, top first: Serra Angel (5), Shock (1): the library runs out at 6.
    library(&mut t, P2, &["Shock", "Serra Angel"]);
    let spell = t.hand(P0, "Tasha's Hideous Laughter");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(library_names(&t, P1), ["Grizzly Bears"]);
    assert_eq!(t.library_size(P2), 0);
    assert_eq!(
        exile_names(&t),
        [
            "Emrakul, the Aeons Torn",
            "Forest",
            "Serra Angel",
            "Serra Angel",
            "Shock"
        ]
    );
    // P0 (not an opponent) exiled nothing.
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn improvisation_capstone_exiles_until_total_mana_value_four() {
    cr!("701.20a");
    assert_supported("Improvisation Capstone");
    let mut t = TestGame::new(2);
    // Top first: Forest, Shock (1), Grizzly Bears (2), Llanowar Elves (1) — 4 — then
    // Lightning Bolt.
    library(
        &mut t,
        P0,
        &[
            "Lightning Bolt",
            "Llanowar Elves",
            "Grizzly Bears",
            "Shock",
            "Forest",
        ],
    );
    t.lands(P0, "Mountain", 7);
    let spell = t.hand(P0, "Improvisation Capstone");
    // Cast nothing.
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.cast(P0, spell).go();
    t.resolve();
    // Paradigm exiles the spell itself too.
    assert_eq!(
        exile_names(&t),
        [
            "Forest",
            "Grizzly Bears",
            "Improvisation Capstone",
            "Llanowar Elves",
            "Shock"
        ]
    );
    assert_eq!(library_names(&t, P0), ["Lightning Bolt"]);
}

#[test]
fn thought_dissector_stops_at_x_cards_without_an_artifact() {
    cr!("701.20a");
    assert_supported("Thought Dissector");
    let mut t = TestGame::new(2);
    let dissector = t.battlefield(P0, "Thought Dissector");
    t.lands(P0, "Island", 2);
    // Top first: Forest, Shock, Sol Ring.
    library(&mut t, P1, &["Sol Ring", "Shock", "Forest"]);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, dissector, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // Two cards were revealed (the cap came first) and put into P1's graveyard.
    assert_eq!(graveyard_names(&t, P1), ["Forest", "Shock"]);
    assert_eq!(library_names(&t, P1), ["Sol Ring"]);
    assert!(t.on_battlefield(dissector));
}

#[test]
fn thought_dissector_takes_the_artifact_and_is_sacrificed() {
    cr!("701.20a", "110.2a");
    ruling!(
        "Thought Dissector",
        "You put the artifact card onto the battlefield even if Thought Dissector has already left"
    );
    let mut t = TestGame::new(2);
    let dissector = t.battlefield(P0, "Thought Dissector");
    t.lands(P0, "Island", 3);
    // Top first: Forest, Sol Ring, Shock.
    library(&mut t, P1, &["Shock", "Sol Ring", "Forest"]);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, dissector, 0, &[Entity::Player(P1)]).unwrap();
    // Thought Dissector leaves before the ability resolves; the artifact is still put
    // onto the battlefield.
    t.g.destroy(dissector, None);
    t.resolve();
    let ring = t.named_on_battlefield("Sol Ring");
    assert_eq!(ring.len(), 1);
    assert_eq!(t.obj_now(ring[0]).controller, P0);
    assert_eq!(graveyard_names(&t, P1), ["Forest"]);
    assert_eq!(library_names(&t, P1), ["Shock"]);
}

#[test]
fn thought_dissector_sacrificed_after_finding_an_artifact() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let dissector = t.battlefield(P0, "Thought Dissector");
    t.lands(P0, "Island", 3);
    library(&mut t, P1, &["Shock", "Sol Ring", "Forest"]);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, dissector, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Sol Ring").len(), 1);
    assert!(t.in_graveyard(P0, "Thought Dissector"));
}

#[test]
fn telemin_performance_mills_the_noncreature_cards_and_steals_the_creature() {
    cr!("701.20a", "110.2a");
    ruling!(
        "Telemin Performance",
        "If there are no creature cards in the targeted opponent"
    );
    assert_supported("Telemin Performance");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    library(
        &mut t,
        P1,
        &["Llanowar Elves", "Grizzly Bears", "Shock", "Forest"],
    );
    let spell = t.hand(P0, "Telemin Performance");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert_eq!(graveyard_names(&t, P1), ["Forest", "Shock"]);
    assert_eq!(library_names(&t, P1), ["Llanowar Elves"]);

    // No creature card: the whole library goes to the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    library(&mut t, P1, &["Shock", "Forest"]);
    let spell = t.hand(P0, "Telemin Performance");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(graveyard_names(&t, P1), ["Forest", "Shock"]);
}

#[test]
fn blessed_reincarnation_that_player_puts_the_creature_found_onto_the_battlefield() {
    cr!("701.20a", "110.2a");
    assert_supported("Blessed Reincarnation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let target = t.battlefield(P1, "Serra Angel");
    library(
        &mut t,
        P1,
        &["Lightning Bolt", "Grizzly Bears", "Shock", "Forest"],
    );
    let spell = t.hand(P0, "Blessed Reincarnation");
    t.cast(P0, spell).target(Entity::Object(target)).go();
    t.resolve();
    assert!(t.in_exile("Serra Angel"));
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    // That player put it onto the battlefield: they control it (CR 110.2a).
    assert_eq!(t.obj_now(bears[0]).controller, P1);
    // The rest were shuffled into the library.
    let mut lib = library_names(&t, P1);
    lib.sort();
    assert_eq!(lib, ["Forest", "Lightning Bolt", "Shock"]);
}

#[test]
fn smeagol_steals_the_land_found_and_mills_the_rest() {
    cr!("701.20a");
    assert_supported("Sméagol, Helpful Guide");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sméagol, Helpful Guide");
    library(&mut t, P1, &["Shock", "Island", "Grizzly Bears", "Lightning Bolt"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    mtg_engine::kwa::ring::tempt(&mut t.g, P0, None);
    t.g.flush_events();
    t.resolve_all();
    let island = t.named_on_battlefield("Island");
    assert_eq!(island.len(), 1);
    assert_eq!(t.obj_now(island[0]).controller, P0);
    assert!(t.obj_now(island[0]).tapped);
    assert_eq!(graveyard_names(&t, P1), ["Grizzly Bears", "Lightning Bolt"]);
    assert_eq!(library_names(&t, P1), ["Shock"]);
}

#[test]
fn nascent_metamorph_that_player_puts_the_revealed_cards_on_the_bottom() {
    cr!("701.20a");
    assert_supported("Nascent Metamorph");
    let mut t = TestGame::new(2);
    let meta = t.battlefield(P0, "Nascent Metamorph");
    library(
        &mut t,
        P1,
        &["Lightning Bolt", "Serra Angel", "Shock", "Forest"],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(meta, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(meta).chars.name, "Serra Angel");
    // All three revealed cards went to the bottom, below the Lightning Bolt.
    let lib = library_names(&t, P1);
    assert_eq!(lib[0], "Lightning Bolt");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Serra Angel", "Shock"]);
}

#[test]
fn dazzling_sphinx_uncast_card_goes_to_the_bottom_with_the_rest() {
    cr!("701.20a");
    ruling!(
        "Dazzling Sphinx",
        "You may choose not to cast the instant or sorcery card that you exiled"
    );
    assert_supported("Dazzling Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Dazzling Sphinx");
    library(&mut t, P1, &["Grizzly Bears", "Divination", "Forest"]);
    t.answer_yes(P0, false);
    t.attack(&[(sphinx, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    let lib = library_names(&t, P1);
    assert_eq!(lib[0], "Grizzly Bears");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Divination", "Forest"]);
}

#[test]
fn dazzling_sphinx_cast_card_isnt_put_on_the_bottom() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Dazzling Sphinx");
    library(&mut t, P1, &["Grizzly Bears", "Divination", "Forest"]);
    t.answer_yes(P0, true);
    t.attack(&[(sphinx, Entity::Player(P1))], &[]);
    t.resolve_all();
    t.advance_to_step(mtg_engine::turn::Step::EndOfCombat);
    t.resolve_all();
    // Divination resolved: P0 drew two cards; it's in P1's graveyard (its owner's).
    assert_eq!(t.hand_size(P0), 2);
    assert!(t.in_graveyard(P1, "Divination"));
    let lib = library_names(&t, P1);
    assert_eq!(lib, ["Grizzly Bears", "Forest"]);
}

#[test]
fn explosive_revelation_puts_the_nonland_card_into_hand() {
    cr!("701.20a");
    assert_supported("Explosive Revelation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    library(&mut t, P0, &["Shock", "Serra Angel", "Island", "Forest"]);
    let spell = t.hand(P0, "Explosive Revelation");
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_hand(P0, "Serra Angel"));
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Shock");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Island"]);
}

#[test]
fn desperate_research_puts_all_cards_with_the_name_into_hand_and_exiles_the_rest() {
    cr!("201.3", "701.20a");
    assert_supported("Desperate Research");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    library(
        &mut t,
        P0,
        &[
            "Grizzly Bears",
            "Shock",
            "Forest",
            "Shock",
            "Island",
            "Shock",
            "Grizzly Bears",
            "Swamp",
        ],
    );
    let spell = t.hand(P0, "Desperate Research");
    t.answer(P0, DecisionKind::Name, Answer::Text("Shock".into()));
    t.cast(P0, spell).go();
    t.resolve();
    // The top seven: Swamp, Bears, Shock, Island, Shock, Forest, Shock.
    assert_eq!(t.hand_size(P0), 3);
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(
        exile_names(&t),
        ["Forest", "Grizzly Bears", "Island", "Swamp"]
    );
    assert_eq!(library_names(&t, P0), ["Grizzly Bears"]);
}

#[test]
fn tunnel_vision_mills_down_to_the_named_card_and_leaves_it_on_top() {
    cr!("701.20a", "201.3");
    ruling!(
        "Tunnel Vision",
        "If the named card is found, everything from the top of the library down to the named card"
    );
    assert_supported("Tunnel Vision");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    library(
        &mut t,
        P1,
        &["Lightning Bolt", "Serra Angel", "Shock", "Forest"],
    );
    let spell = t.hand(P0, "Tunnel Vision");
    t.answer(P0, DecisionKind::Name, Answer::Text("Serra Angel".into()));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(graveyard_names(&t, P1), ["Forest", "Shock"]);
    assert_eq!(library_names(&t, P1), ["Serra Angel", "Lightning Bolt"]);
}

#[test]
fn tunnel_vision_without_the_named_card_shuffles() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    // Twenty different cards, in a known order.
    let names = [
        "Lightning Bolt", "Shock", "Forest", "Island", "Swamp", "Mountain", "Plains",
        "Grizzly Bears", "Llanowar Elves", "Divination", "Sol Ring", "Ornithopter",
        "Hill Giant", "Raging Goblin", "Runeclaw Bear", "Giant Growth", "Counterspell",
        "Duress", "Opt", "Disenchant",
    ];
    library(&mut t, P1, &names);
    let before = library_names(&t, P1);
    let spell = t.hand(P0, "Tunnel Vision");
    t.answer(P0, DecisionKind::Name, Answer::Text("Serra Angel".into()));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    // Nothing is put into the graveyard; the library is shuffled.
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.library_size(P1), 20);
    assert_ne!(library_names(&t, P1), before);
}

#[test]
fn curse_of_unbinding_steals_the_creature_and_mills_the_rest() {
    cr!("701.20a");
    assert_supported("Curse of Unbinding");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Unbinding");
    t.g.attach(curse, Entity::Player(P1));
    library(&mut t, P1, &["Shock", "Grizzly Bears", "Forest", "Island"]);
    t.set_step(P1, mtg_engine::turn::Step::Untap);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj_now(bears[0]).controller, P0);
    assert_eq!(graveyard_names(&t, P1), ["Forest", "Island"]);
    assert_eq!(library_names(&t, P1), ["Shock"]);
}

#[test]
fn michelangelos_technique_two_creatures_with_total_mana_value_six_or_less() {
    cr!("701.20e");
    assert_supported("Michelangelo's Technique");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Shock",
            "Island",
            "Swamp",
            "Llanowar Elves",
            "Mountain",
            "Grizzly Bears",
            "Serra Angel",
        ],
    );
    let (angel, bears, elves) = (ids[7], ids[6], ids[4]);
    let spell = t.hand(P0, "Michelangelo's Technique");
    // Serra Angel and Grizzly Bears total 7: only a group of total 6 or less is kept.
    let _ = elves;
    t.answer_choose(P0, &[Entity::Object(angel), Entity::Object(bears)]);
    t.cast(P0, spell).go();
    t.resolve();
    let on: Vec<String> = ["Serra Angel", "Grizzly Bears", "Llanowar Elves"]
        .iter()
        .filter(|n| !t.named_on_battlefield(n).is_empty())
        .map(|n| n.to_string())
        .collect();
    assert_eq!(on, ["Serra Angel"], "{}", t.dump_log());
    let lib = library_names(&t, P0);
    assert_eq!(lib.len(), 7);
}

#[test]
fn rip_spawn_hunter_creature_and_vehicle_cards_with_different_powers() {
    cr!("701.20e");
    assert_supported("Rip, Spawn Hunter");
    let mut t = TestGame::new(2);
    let rip = t.battlefield(P0, "Rip, Spawn Hunter");
    t.g.objects[rip.0 as usize].tapped = true;
    let ids = library(
        &mut t,
        P0,
        &["Llanowar Elves", "Grizzly Bears", "Runeclaw Bear", "Forest"],
    );
    let (elves, bears, runeclaw) = (ids[0], ids[1], ids[2]);
    // Rip's power is 4: the top four cards. The two 2-power Bears can't both be taken.
    t.answer_choose(
        P0,
        &[
            Entity::Object(bears),
            Entity::Object(runeclaw),
            Entity::Object(elves),
        ],
    );
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.advance_to_step(mtg_engine::turn::Step::PostcombatMain);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert!(!t.in_hand(P0, "Runeclaw Bear"));
    assert_eq!(t.hand_size(P0), 2);
    let mut lib = library_names(&t, P0);
    lib.sort();
    assert_eq!(lib, ["Forest", "Runeclaw Bear"]);
}

#[test]
fn muse_vortex_uncast_instants_and_sorceries_go_to_hand() {
    cr!("701.20a");
    ruling!(
        "Muse Vortex",
        "If you don’t, all exiled instant or sorcery cards will be put into your hand"
    );
    assert_supported("Muse Vortex");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    library(&mut t, P0, &["Forest", "Shock", "Grizzly Bears", "Divination"]);
    let spell = t.hand(P0, "Muse Vortex");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Divination"));
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(t.hand_size(P0), 2);
    // The creature card went to the bottom, below the Forest.
    assert_eq!(library_names(&t, P0), ["Forest", "Grizzly Bears"]);
}

#[test]
fn lonis_puts_a_permanent_card_onto_the_battlefield_under_your_control() {
    cr!("701.20e", "110.2a");
    assert_supported("Lonis, Cryptozoologist");
    let mut t = TestGame::new(2);
    let lonis = t.battlefield(P0, "Lonis, Cryptozoologist");
    let clues: Vec<Entity> = (0..3)
        .map(|_| Entity::Object(t.battlefield(P0, "Clue")))
        .collect();
    let ids = library(
        &mut t,
        P1,
        &["Serra Angel", "Grizzly Bears", "Shock", "Lightning Bolt"],
    );
    let bears = ids[1];
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_choose(P0, &clues);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let r = t.activate(P0, lonis, 0, &[Entity::Player(P1)]);
    assert!(r.is_ok(), "{r:?}");
    t.resolve_all();
    assert!(t.named_on_battlefield("Clue").is_empty());
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).controller, P0);
    let lib = library_names(&t, P1);
    assert_eq!(lib[0], "Serra Angel");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Lightning Bolt", "Shock"]);
}
