//! Card-flow grammar IV (`cards2-zones-dig`): digs and moves whose description or
//! destination refers to another object or player — "a card that shares a creature type
//! with it/that creature" after a dig, "put that card onto the battlefield attached to
//! that creature", "then attach it to that creature/token", "return it to the battlefield
//! attached to you", "under that player's/an opponent's control" — and other players'
//! libraries and graveyards ("the top of its owner's library", "that player's library",
//! "up to one card of each card type from defending player's graveyard").

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
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

/// `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

fn exile_names(t: &TestGame) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .exile
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn tajuru_paragon_card_sharing_a_creature_type_with_it() {
    cr!("205.3m", "701.20e");
    assert_supported("Tajuru Paragon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let ids = library(
        &mut t,
        P0,
        &[
            "Forest",
            "Shock",
            "Grizzly Bears",
            "Llanowar Elves",
            "Lightning Bolt",
            "Island",
        ],
    );
    let elves = ids[3];
    let paragon = t.hand(P0, "Tajuru Paragon");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    t.cast(P0, paragon).kicked(true).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    // Grizzly Bears (a Bear) wasn't offered.
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice");
    assert_eq!(offered, vec![Entity::Object(elves)]);
    assert_eq!(t.library_size(P0), 5);
}

#[test]
fn conjurers_mantle_card_sharing_a_creature_type_with_the_attacker() {
    cr!("701.20e");
    assert_supported("Conjurer's Mantle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mantle = t.battlefield(P0, "Conjurer's Mantle");
    t.g.attach(mantle, Entity::Object(bears));
    let ids = library(
        &mut t,
        P0,
        &["Forest", "Runeclaw Bear", "Llanowar Elves", "Shock"],
    );
    let runeclaw = ids[1];
    t.answer_choose(P0, &[Entity::Object(runeclaw)]);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert!(t.in_hand(P0, "Runeclaw Bear"), "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 3);
}

#[test]
fn rohirrim_chargers_attaches_the_equipment_found_to_the_exerted_creature() {
    cr!("701.20a", "301.5c");
    assert_supported("Rohirrim Chargers");
    let mut t = TestGame::new(2);
    let chargers = t.battlefield(P0, "Rohirrim Chargers");
    library(&mut t, P0, &["Lightning Bolt", "Bonesplitter", "Forest", "Shock"]);
    t.answer_yes(P0, true);
    t.attack(&[(chargers, Entity::Player(P1))], &[]);
    let split = t.named_on_battlefield("Bonesplitter");
    assert_eq!(split.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(split[0]).attached_to, Some(Entity::Object(chargers)));
    let lib = library_names(&t, P0);
    assert_eq!(lib[0], "Lightning Bolt");
    let mut bottom = lib[1..].to_vec();
    bottom.sort();
    assert_eq!(bottom, ["Forest", "Shock"]);
}

#[test]
fn sword_of_the_meek_returns_and_attaches_to_the_one_one() {
    cr!("701.3a", "301.5c");
    assert_supported("Sword of the Meek");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Sword of the Meek");
    t.answer_yes(P0, true);
    let elves = t.enter(P0, "Llanowar Elves");
    t.resolve_all();
    let sword = t.named_on_battlefield("Sword of the Meek");
    assert_eq!(sword.len(), 1);
    assert_eq!(t.obj_now(sword[0]).attached_to, Some(Entity::Object(elves)));
    assert_eq!(t.pt(elves), (2, 3));
}

#[test]
fn ratonhnhaketon_attaches_the_returned_equipment_to_the_new_token() {
    cr!("603.12", "701.3a");
    assert_supported("Ratonhnhaké꞉ton");
    let mut t = TestGame::new(2);
    let rat = t.battlefield(P0, "Ratonhnhaké꞉ton");
    let split = t.graveyard(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(split)]);
    t.attack(&[(rat, Entity::Player(P1))], &[]);
    t.resolve_all();
    let token = t.named_on_battlefield("Assassin Token");
    assert_eq!(token.len(), 1, "{}", t.dump_log());
    let on = t.named_on_battlefield("Bonesplitter");
    assert_eq!(on.len(), 1);
    assert_eq!(t.obj_now(on[0]).attached_to, Some(Entity::Object(token[0])));
}

#[test]
fn lynde_returns_a_curse_attached_to_you_at_the_next_end_step() {
    cr!("303.4f", "603.7");
    assert_supported("Lynde, Cheerful Tormentor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lynde, Cheerful Tormentor");
    let curse = t.battlefield(P0, "Curse of the Pierced Heart");
    t.g.attach(curse, Entity::Player(P1));
    t.g.destroy(curse, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Curse of the Pierced Heart"));
    t.advance_to_step(Step::End);
    t.resolve_all();
    let on = t.named_on_battlefield("Curse of the Pierced Heart");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).attached_to, Some(Entity::Player(P0)));
}

#[test]
fn plague_reaver_returns_under_that_players_control_at_their_upkeep() {
    cr!("110.2a", "603.7");
    assert_supported("Plague Reaver");
    let mut t = TestGame::new(2);
    let reaver = t.battlefield(P0, "Plague Reaver");
    let a = t.hand(P0, "Shock");
    let b = t.hand(P0, "Forest");
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.activate(P0, reaver, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plague Reaver"));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let on = t.named_on_battlefield("Plague Reaver");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(on[0]).controller, P1);
}

#[test]
fn solkanar_returns_under_an_opponents_control() {
    cr!("110.2a", "700.2");
    assert_supported("Sol'Kanar the Tainted");
    let mut t = TestGame::new(2);
    let sol = t.battlefield(P0, "Sol'Kanar the Tainted");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![3]));
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to_step(Step::End);
    t.resolve_all();
    let on = t.named_on_battlefield("Sol'Kanar the Tainted");
    assert_eq!(on.len(), 1, "{}", t.dump_log());
    assert_ne!(on[0], sol);
    assert_eq!(t.obj_now(on[0]).controller, P1);
}

#[test]
fn dead_mans_chest_exiles_cards_equal_to_its_power_from_its_owners_library() {
    cr!("701.13a");
    assert_supported("Dead Man's Chest");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let chest = t.battlefield(P0, "Dead Man's Chest");
    t.g.attach(chest, Entity::Object(giant));
    library(&mut t, P1, &["Lightning Bolt", "Forest", "Shock", "Island"]);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(exile_names(&t), ["Forest", "Island", "Shock"]);
    assert_eq!(library_names(&t, P1), ["Lightning Bolt"]);
}

#[test]
fn cunning_rhetoric_exiles_the_top_card_of_the_attacking_players_library() {
    cr!("701.13a");
    assert_supported("Cunning Rhetoric");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Rhetoric");
    let bears = t.battlefield(P1, "Grizzly Bears");
    library(&mut t, P1, &["Lightning Bolt", "Shock"]);
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(bears, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(exile_names(&t), ["Shock"], "{}", t.dump_log());
    assert_eq!(library_names(&t, P1), ["Lightning Bolt"]);
}

#[test]
fn grime_gorger_exiles_up_to_one_card_of_each_card_type() {
    cr!("205.2a");
    assert_supported("Grime Gorger");
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Grime Gorger");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let elves = t.graveyard(P1, "Llanowar Elves");
    let forest = t.graveyard(P1, "Forest");
    let shock = t.graveyard(P1, "Shock");
    // Two creature cards can't both be exiled.
    t.answer_choose(
        P0,
        &[
            Entity::Object(bears),
            Entity::Object(elves),
            Entity::Object(forest),
            Entity::Object(shock),
        ],
    );
    t.attack(&[(gorger, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.zone(elves), Zone::Graveyard(P1));
    assert_eq!(exile_names(&t), ["Forest", "Grizzly Bears", "Shock"]);
    assert_eq!(t.counters(gorger, "+1/+1"), 3);
}

#[test]
fn strategic_betrayal_opponent_exiles_a_creature_and_their_graveyard() {
    cr!("701.13a");
    assert_supported("Strategic Betrayal");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    t.battlefield(P1, "Serra Angel");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.graveyard(P1, "Shock");
    t.graveyard(P1, "Forest");
    let spell = t.hand(P0, "Strategic Betrayal");
    // The opponent chooses which creature.
    t.answer_choose(P1, &[Entity::Object(elves)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(exile_names(&t), ["Forest", "Llanowar Elves", "Shock"]);
    assert_eq!(t.named_on_battlefield("Serra Angel").len(), 1);
}

#[test]
fn vendilion_clique_the_chosen_card_goes_to_the_bottom_and_they_draw() {
    cr!("701.20a");
    assert_supported("Vendilion Clique");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    library(&mut t, P1, &["Island", "Shock"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.enter(P0, "Vendilion Clique");
    t.resolve_all();
    assert_eq!(library_names(&t, P1), ["Island", "Lightning Bolt"]);
    assert!(t.in_hand(P1, "Shock"));
    assert!(t.in_hand(P1, "Forest"));
    assert!(t.dump_log().contains("reveals Lightning Bolt"), "{}", t.dump_log());
}
