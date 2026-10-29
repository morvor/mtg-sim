//! Game-rules grammar, round 3 (`oracle/patterns/game_rules_4.rs`, `dice_coins_4.rs`):
//! commander color identity as protection, more loyalty activations this turn, a player
//! losing all counters of a kind, differently named dungeons completed, players on your
//! team, and Comet's results table with loyalty symbols as instructions.

use mtg_engine::game::Variant;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn make_commander(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[p.idx()].commander_names.push(name);
    t.g.recompute();
}

fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
    t.g.recompute();
}

#[test]
fn commanders_plate_protects_from_colors_outside_the_commanders_identity() {
    cr!("903.4", "702.16a");
    interp!("commander-identity-protection-player");
    ruling!(
        "Commander's Plate",
        "If you have no commander, Commander's Plate grants no protection abilities."
    );
    assert_supported(&["Commander's Plate"]);
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plate = t.battlefield(P0, "Commander's Plate");
    t.g.attach(plate, Entity::Object(bears));
    t.g.recompute();
    let bolt = t.hand(P1, "Lightning Bolt");
    let swords = t.hand(P1, "Swords to Plowshares");
    // No commander yet: only +3/+3.
    assert_eq!(t.pt(bears), (5, 5));
    assert!(!t.g.protected_from(bears, bolt));
    // A white commander: protection from blue, black, red and green, not white.
    let serra = t.command(P0, "Serra Angel");
    make_commander(&mut t, P0, serra);
    assert!(t.g.protected_from(bears, bolt));
    assert!(!t.g.protected_from(bears, swords));
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn commanders_plate_with_a_colorless_commander_protects_from_each_color() {
    cr!("903.4");
    ruling!(
        "Commander's Plate",
        "If your commander is a card that has no colors in its color identity"
    );
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plate = t.battlefield(P0, "Commander's Plate");
    t.g.attach(plate, Entity::Object(bears));
    let ornithopter = t.command(P0, "Ornithopter");
    make_commander(&mut t, P0, ornithopter);
    let swords = t.hand(P1, "Swords to Plowshares");
    assert!(t.g.protected_from(bears, swords));
}

#[test]
fn survivors_med_kit_radaway_removes_all_rad_counters() {
    cr!("122.1");
    ruling!(
        "Survivor's Med Kit",
        "They only go away when an effect instructs a player to remove rad counters from themselves."
    );
    assert_supported(&["Survivor's Med Kit"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let kit = t.battlefield(P0, "Survivor's Med Kit");
    t.g.add_counters(Entity::Player(P1), counters::RAD, 5, None);
    t.g.add_counters(Entity::Player(P1), counters::POISON, 2, None);
    t.lands(P0, "Plains", 1);
    // RadAway, the third mode.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    t.activate(P0, kit, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let p1 = t.g.player(P1);
    assert_eq!(p1.counters.get(counters::RAD).copied().unwrap_or(0), 0);
    assert_eq!(p1.counters.get(counters::POISON).copied(), Some(2));
    assert!(!t.on_battlefield(kit));
}

#[test]
fn ellywick_emblem_counts_differently_named_dungeons() {
    cr!("309.7", "114.1");
    ruling!(
        "Ellywick Tumblestrum",
        "counts all dungeons you've completed, whether they were completed before or after the emblem was created"
    );
    ruling!(
        "Ellywick Tumblestrum",
        "The emblem grants trample and haste even if you haven't completed a dungeon."
    );
    assert_supported(&["Ellywick Tumblestrum"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let elly = t.battlefield(P0, "Ellywick Tumblestrum");
    set_loyalty(&mut t, elly, 7);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, elly, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(t
        .obj_now(bears)
        .chars
        .has_keyword(keywords::KeywordKind::Trample));
    assert!(t
        .obj_now(bears)
        .chars
        .has_keyword(keywords::KeywordKind::Haste));
    t.g.players[0].completed_dungeons =
        vec!["Tomb of Annihilation".into(), "Tomb of Annihilation".into()];
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    t.g.players[0]
        .completed_dungeons
        .push("Lost Mine of Phandelver".into());
    t.g.recompute();
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn azra_bladeseeker_lets_only_players_on_your_team_rummage() {
    cr!("102.3");
    ruling!(
        "Azra Bladeseeker",
        "The choice of whether to discard a card is made by each player on your team individually."
    );
    assert_supported(&["Azra Bladeseeker"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Azra Bladeseeker");
    t.resolve_all();
    // You discarded and drew; your opponent isn't on your team.
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn comet_rolls_a_six_and_activates_two_more_times() {
    cr!("706.3", "606.3");
    interp!("more-loyalty-activations-add-up");
    ruling!(
        "Comet, Stellar Pup",
        "Each + loyalty symbol in Comet's abilities means to put the indicated number of loyalty counters on him."
    );
    assert_supported(&["Comet, Stellar Pup"]);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let comet = t.battlefield(P0, "Comet, Stellar Pup");
    set_loyalty(&mut t, comet, 3);
    t.g.dice.loaded.extend([6, 1, 3]);
    t.activate(P0, comet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(comet, counters::LOYALTY), 4);
    // 1 or 2: [+2], then two hasty Squirrels.
    t.activate(P0, comet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(comet, counters::LOYALTY), 6);
    let squirrels: Vec<ObjectId> =
        t.g.zone_objects(object::Zone::Battlefield)
            .into_iter()
            .filter(|o| t.obj_now(*o).chars.subtypes.iter().any(|s| s == "Squirrel"))
            .collect();
    assert_eq!(squirrels.len(), 2);
    assert!(t
        .obj_now(squirrels[0])
        .chars
        .has_keyword(keywords::KeywordKind::Haste));
    // 3: [−1], then return a card with mana value 2 or less.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, comet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(comet, counters::LOYALTY), 5);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // That was the third activation this turn: no fourth.
    assert!(t.activate(P0, comet, 0, &[]).is_err());
}

#[test]
fn comet_rolls_a_four_and_deals_damage_to_a_player_of_your_choice() {
    cr!("706.3", "115.1");
    ruling!(
        "Comet, Stellar Pup",
        "None of Comet's abilities require a target."
    );
    ruling!(
        "Comet, Stellar Pup",
        "If you roll a 4 or 5 while Comet has one loyalty counter on him"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let comet = t.battlefield(P0, "Comet, Stellar Pup");
    set_loyalty(&mut t, comet, 3);
    t.g.dice.loaded.push_back(4);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.activate(P0, comet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(comet, counters::LOYALTY), 1);
    // Only once this turn without a 6.
    assert!(t.activate(P0, comet, 0, &[]).is_err());
    // With one loyalty counter: 1 damage to a creature, then he's gone.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let comet = t.battlefield(P0, "Comet, Stellar Pup");
    set_loyalty(&mut t, comet, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.dice.loaded.push_back(5);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, comet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(!t.on_battlefield(comet));
}

#[test]
fn kaito_returns_a_creature_and_activates_twice() {
    cr!("606.3");
    ruling!(
        "Kaito, Dancing Shadow",
        "you may activate a single loyalty ability of Kaito twice"
    );
    assert_supported(&["Kaito, Dancing Shadow"]);
    let mut t = TestGame::new(2);
    let kaito = t.battlefield(P0, "Kaito, Dancing Shadow");
    set_loyalty(&mut t, kaito, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::PostcombatMain);
    let hand = t.hand_size(P0);
    t.activate(P0, kaito, 1, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, kaito, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.activate(P0, kaito, 1, &[]).is_err());
}
