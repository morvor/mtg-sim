//! Card-flow grammar IV (`cards2-zones-dig`): hands, libraries and graveyards of other
//! players and odd positions — the bottom card of a library, reordering a graveyard,
//! "just beneath the top X cards", cards revealed at random from a hand, a card put from
//! a hand just revealed, a card chosen from a hand looked at, "you may then have that
//! player shuffle", and an Aura card that could enchant the attacking creature.

use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
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

/// `p`'s library, top first.
fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    v.sort();
    v
}

#[test]
fn grenzo_mills_the_bottom_card_and_puts_a_small_creature_onto_the_battlefield() {
    cr!("401.2");
    assert_supported("Grenzo, Dungeon Warden");
    let mut t = TestGame::new(2);
    let grenzo = t.battlefield(P0, "Grenzo, Dungeon Warden");
    t.lands(P0, "Mountain", 4);
    // Bottom first: Grizzly Bears, then Serra Angel above it, then Forest on top.
    library(&mut t, P0, &["Grizzly Bears", "Serra Angel", "Forest"]);
    t.activate(P0, grenzo, 0, &[]).unwrap();
    t.resolve();
    // Grizzly Bears (power 2 <= Grenzo's 2) enters from the graveyard.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(library_names(&t, P0), ["Forest", "Serra Angel"]);
    // Serra Angel (power 4) stays in the graveyard.
    t.activate(P0, grenzo, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Serra Angel"));
    assert_eq!(library_names(&t, P0), ["Forest"]);
}

#[test]
fn cellar_door_target_player_mills_their_bottom_card() {
    cr!("401.2");
    assert_supported("Cellar Door");
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, "Cellar Door");
    t.lands(P0, "Swamp", 3);
    library(&mut t, P1, &["Grizzly Bears", "Shock"]);
    t.activate(P0, door, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(library_names(&t, P1), ["Shock"]);
    // A creature card: P0 creates a Zombie.
    let zombies = t.named_on_battlefield("Zombie Token");
    assert_eq!(zombies.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(zombies[0]).controller, P0);
}

#[test]
fn unexpectedly_absent_puts_the_permanent_beneath_the_top_x_cards() {
    cr!("401.4");
    assert_supported("Unexpectedly Absent");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let target = t.battlefield(P1, "Serra Angel");
    library(&mut t, P1, &["Forest", "Island", "Swamp", "Mountain"]);
    let spell = t.hand(P0, "Unexpectedly Absent");
    t.cast(P0, spell).target(Entity::Object(target)).x(2).go();
    t.resolve();
    assert_eq!(
        library_names(&t, P1),
        ["Mountain", "Swamp", "Serra Angel", "Island", "Forest"]
    );
}

#[test]
fn unexpectedly_absent_with_x_zero_puts_it_on_top() {
    cr!("401.4");
    ruling!(
        "Unexpectedly Absent",
        "If you choose 0 as the value for X, put that permanent on top"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let target = t.battlefield(P1, "Serra Angel");
    library(&mut t, P1, &["Forest", "Island"]);
    let spell = t.hand(P0, "Unexpectedly Absent");
    t.cast(P0, spell).target(Entity::Object(target)).x(0).go();
    t.resolve();
    assert_eq!(library_names(&t, P1), ["Serra Angel", "Island", "Forest"]);
}

#[test]
fn unexpectedly_absent_with_too_few_cards_puts_it_on_the_bottom() {
    cr!("401.4");
    ruling!(
        "Unexpectedly Absent",
        "If there are fewer than X cards in that player’s library"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 7);
    let target = t.battlefield(P1, "Serra Angel");
    library(&mut t, P1, &["Forest", "Island"]);
    let spell = t.hand(P0, "Unexpectedly Absent");
    t.cast(P0, spell).target(Entity::Object(target)).x(5).go();
    t.resolve();
    assert_eq!(library_names(&t, P1), ["Island", "Forest", "Serra Angel"]);
}

#[test]
fn quarry_colossus_x_is_the_number_of_plains() {
    cr!("401.4");
    ruling!(
        "Quarry Colossus",
        "The value of X is determined as the ability resolves"
    );
    assert_supported("Quarry Colossus");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let target = t.battlefield(P1, "Grizzly Bears");
    library(&mut t, P1, &["Forest", "Island", "Swamp"]);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.enter(P0, "Quarry Colossus");
    t.settle();
    // Another Plains arrives before the ability resolves: X is 2.
    t.battlefield(P0, "Plains");
    t.resolve_all();
    assert_eq!(
        library_names(&t, P1),
        ["Swamp", "Island", "Grizzly Bears", "Forest"]
    );
}

#[test]
fn fossil_find_returns_a_random_card_then_reorders_the_graveyard() {
    cr!("404.2");
    assert_supported("Fossil Find");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let a = t.graveyard(P0, "Shock");
    let b = t.graveyard(P0, "Grizzly Bears");
    let c = t.graveyard(P0, "Forest");
    let spell = t.hand(P0, "Fossil Find");
    t.cast(P0, spell).go();
    t.resolve();
    // One of the three is in hand; the other two stay in the graveyard.
    assert_eq!(t.hand_size(P0), 1);
    let left: Vec<ObjectId> = [a, b, c]
        .into_iter()
        .filter(|o| t.zone(*o) == Zone::Graveyard(P0))
        .collect();
    assert_eq!(left.len(), 2);
    // The reorder was asked for (the graveyard's order may change, CR 404.2).
    assert!(t.asked().iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Order { prompt, .. } if prompt.contains("graveyard"))));
}

#[test]
fn fossil_find_graveyard_order_as_chosen() {
    cr!("404.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let cards: Vec<ObjectId> = ["Shock", "Island", "Swamp", "Plains"]
        .iter()
        .map(|n| t.graveyard(P0, n))
        .collect();
    let spell = t.hand(P0, "Fossil Find");
    // The three cards left are offered top first: reverse them.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![2, 1, 0]));
    t.cast(P0, spell).go();
    t.resolve();
    let left: Vec<ObjectId> = cards
        .iter()
        .copied()
        .filter(|o| t.zone(*o) == Zone::Graveyard(P0))
        .collect();
    assert_eq!(left.len(), 3);
    let mut gy: Vec<ObjectId> = t.g.player(P0).graveyard.clone();
    // Fossil Find itself went on top after the reorder, as it finished resolving.
    assert_eq!(gy.pop().map(|o| t.g.obj(o).chars.name.to_string()).as_deref(), Some("Fossil Find"));
    let mut expected = left.clone();
    expected.reverse();
    assert_eq!(gy, expected);
}

#[test]
fn search_for_survivors_puts_a_random_creature_onto_the_battlefield() {
    cr!("404.2");
    assert_supported("Search for Survivors");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Search for Survivors");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);

    // A noncreature card chosen is exiled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.graveyard(P0, "Shock");
    let spell = t.hand(P0, "Search for Survivors");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_exile("Shock"));
}

#[test]
fn fall_reveals_two_cards_at_random_and_discards_the_nonland_ones() {
    cr!("701.20a", "701.9a");
    assert_supported("Rise // Fall");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.hand(P1, "Forest");
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "Rise // Fall");
    t.cast(P0, spell)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.resolve();
    // Both cards were revealed: the Shock is discarded, the Forest isn't.
    assert_eq!(graveyard_names(&t, P1), ["Shock"]);
    assert!(t.in_hand(P1, "Forest"));
}

#[test]
fn friendly_fire_deals_the_revealed_cards_mana_value() {
    cr!("701.20a");
    assert_supported("Friendly Fire");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let angel = t.battlefield(P1, "Colossal Dreadmaw");
    t.hand(P1, "Hill Giant");
    let spell = t.hand(P0, "Friendly Fire");
    t.cast(P0, spell).target(Entity::Object(angel)).go();
    t.resolve();
    // Hill Giant has mana value 4.
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert_eq!(t.obj_now(angel).damage, 4);
    assert!(t.in_hand(P1, "Hill Giant"));
}

#[test]
fn treacherous_urge_steals_a_creature_card_from_the_revealed_hand() {
    cr!("701.20a");
    assert_supported("Treacherous Urge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let bears = t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Shock");
    let spell = t.hand(P0, "Treacherous Urge");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 1);
    assert_eq!(t.obj_now(on[0]).controller, P0);
    assert!(t
        .obj_now(on[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert!(t.in_hand(P1, "Shock"));
    // Sacrificed at the beginning of the next end step.
    t.advance_to_step(mtg_engine::turn::Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn painful_memories_puts_the_chosen_card_on_top_of_the_library() {
    cr!("401.4");
    assert_supported("Painful Memories");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Forest");
    library(&mut t, P1, &["Island", "Swamp"]);
    let spell = t.hand(P0, "Painful Memories");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(library_names(&t, P1), ["Lightning Bolt", "Swamp", "Island"]);
    assert!(t.in_hand(P1, "Forest"));
}

#[test]
fn visions_may_have_that_player_shuffle() {
    cr!("701.24a");
    ruling!("Visions", "You put them back in the same order or have the whole library shuffled");
    assert_supported("Visions");
    // Declining: the order stays.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let names = [
        "Lightning Bolt", "Shock", "Forest", "Island", "Swamp", "Mountain", "Plains",
        "Grizzly Bears", "Llanowar Elves", "Divination", "Sol Ring", "Ornithopter",
        "Hill Giant", "Raging Goblin", "Runeclaw Bear", "Giant Growth", "Counterspell",
        "Duress", "Opt", "Disenchant",
    ];
    library(&mut t, P1, &names);
    let before = library_names(&t, P1);
    let spell = t.hand(P0, "Visions");
    t.answer_yes(P0, false);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(library_names(&t, P1), before);
    // Accepting: it's shuffled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    library(&mut t, P1, &names);
    let spell = t.hand(P0, "Visions");
    t.answer_yes(P0, true);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve();
    assert_ne!(library_names(&t, P1), before);
}

#[test]
fn sovereigns_of_lost_alara_finds_an_aura_that_could_enchant_the_attacker() {
    cr!("303.4f", "701.23a");
    ruling!(
        "Sovereigns of Lost Alara",
        "The attacking creature is not targeted by either this ability or the Aura, so a creature with shroud can be enchanted this way."
    );
    assert_supported("Sovereigns of Lost Alara");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sovereigns of Lost Alara");
    // A creature with shroud attacks alone.
    let attacker = t.battlefield(P0, "Gladecover Scout");
    library(&mut t, P0, &["Rancor", "Forest"]);
    let rancor = t.g.player(P0).library[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(rancor)]);
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    let r = t.named_on_battlefield("Rancor");
    assert_eq!(r.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(r[0]).attached_to, Some(Entity::Object(attacker)));
    // Exalted and Rancor: 1/1 + 1/1 + 2/0.
    assert_eq!(t.pt(attacker), (4, 2));
}

#[test]
fn sovereigns_of_lost_alara_cant_find_an_aura_that_couldnt_enchant_it() {
    cr!("303.4f");
    ruling!(
        "Sovereigns of Lost Alara",
        "must be able to enchant the attacking creature as it currently exists"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sovereigns of Lost Alara");
    // Animate Wall ("Enchant Wall") can't enchant a non-Wall creature.
    let attacker = t.battlefield(P0, "Gladecover Scout");
    library(&mut t, P0, &["Animate Wall", "Forest"]);
    let aura = t.g.player(P0).library[0];
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(aura)]);
    t.attack(&[(attacker, Entity::Player(P1))], &[]);
    assert!(t.named_on_battlefield("Animate Wall").is_empty());
    assert_eq!(t.library_size(P0), 2);
}
