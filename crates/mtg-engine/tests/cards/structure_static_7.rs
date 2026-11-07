//! Structure-coverage tests, round 4 (item 7): static and replacement ability structures
//! shared by several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`).
//! Each test checks a real card's ability does what its text says, for the objects and
//! players it covers and not others, while its condition holds and not otherwise.

use super::structure_activated_2::{act, target_choices};
use super::structure_seed_common::*;
use super::structure_triggered_5::{next_turn, obj};
use super::structure_triggered_7::{bolt, kill};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Declares `attackers` attacking P1 and advances to the declare attackers step.
fn attack_with(t: &mut TestGame, attackers: &[ObjectId]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let decl: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    declare(t, &decl);
    go_to(t, Step::DeclareAttackers);
}

#[test]
fn necrodominance_exiles_your_cards_and_tokens_instead_of_the_graveyard() {
    cr!("614.1a");
    // "If a card or token would be put into your graveyard from anywhere, exile it
    // instead."
    supported("Necrodominance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necrodominance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Forest");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    assert_eq!(t.zone(bears), Zone::Exile, "from the battlefield");
    t.g.discard(P0, in_hand, None);
    t.resolve_all();
    assert_eq!(t.zone(in_hand), Zone::Exile, "from your hand");
    t.g.mill(P0, 2);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 28);
    bolt(&mut t, P0, obj(theirs));
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1), "an opponent's card");
    assert!(t.in_exile("Lightning Bolt"), "your resolved instant");
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.g.exile.len(), 5);
    // A token never reaches your graveyard either (nor does it trigger "dies").
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    let soldiers = subtype_ids(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    let token = t.g.current(soldiers[0]);
    kill(&mut t, token);
    let went = t.g.turn_events.iter().find_map(|e| match e {
        Event::ZoneChange { old, from, to, .. } if *old == token => Some((*from, *to)),
        _ => None,
    });
    assert_eq!(went, Some((Zone::Battlefield, Zone::Exile)));
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn foundry_screecher_gets_plus_1_while_you_control_an_artifact() {
    cr!("611.3a", "613.4c");
    // "This creature gets +1/+0 as long as you control an artifact."
    supported("Foundry Screecher");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Foundry Screecher");
    t.battlefield(P1, "Mind Stone");
    assert_eq!(t.pt(s), (2, 1), "an opponent's artifact");
    let stone = t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 1));
    t.battlefield(P0, "Ornithopter");
    t.g.recompute();
    assert_eq!(t.pt(s), (3, 1), "not per artifact");
    kill(&mut t, stone);
    assert_eq!(t.pt(s), (3, 1));
    let o = t.named_on_battlefield("Ornithopter")
        .into_iter()
        .find(|o| t.obj_now(*o).controller == P0)
        .unwrap();
    kill(&mut t, o);
    assert_eq!(t.pt(s), (2, 1));
}

#[test]
fn frodo_prevents_damage_to_itself_during_your_turn() {
    cr!("615.1a");
    // "During your turn, prevent all damage that would be dealt to Frodo."
    supported("Frodo, Determined Hero");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Frodo, Determined Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    bolt(&mut t, P0, obj(f));
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).damage, 0);
    t.set_step(P1, Step::PrecombatMain);
    bolt(&mut t, P1, obj(bears));
    assert!(!t.on_battlefield(bears), "only Frodo");
    // On an opponent's turn it isn't prevented.
    bolt(&mut t, P1, obj(f));
    assert!(!t.on_battlefield(f));
}

#[test]
fn reflex_sliver_gives_all_slivers_haste() {
    cr!("611.3a", "613.1f", "702.10b");
    // "All Sliver creatures have haste."
    supported("Reflex Sliver");
    let mut t = TestGame::new(2);
    let r = t.battlefield_sick(P0, "Reflex Sliver");
    let mine = t.battlefield_sick(P0, "Metallic Sliver");
    let theirs = t.battlefield(P1, "Metallic Sliver");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.recompute();
    for s in [r, mine, theirs] {
        assert!(t.obj_now(s).has_keyword(KeywordKind::Haste));
    }
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert!(t.g.can_attack(mine) && t.g.can_attack(r));
    assert!(!t.g.can_attack(bears), "summoning sick");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mine, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 19, "the summoning-sick Sliver attacked");
}

#[test]
fn quick_draw_katana_pumps_and_grants_first_strike_during_your_turn() {
    cr!("611.3a", "613.4c", "613.1f", "301.5");
    // "During your turn, equipped creature gets +2/+0 and has first strike. / Equip {2}"
    supported("Quick-Draw Katana");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Quick-Draw Katana");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, k, "equip", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert_eq!(t.pt(other), (2, 2));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::FirstStrike));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2), "not during an opponent's turn");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn mercenary_informer_cant_be_targeted_by_black_spells_or_abilities() {
    cr!("115.1", "601.2c", "603.3d");
    // "This creature can't be the target of black spells or abilities from black sources."
    supported("Mercenary Informer");
    // A black spell.
    let mut t = TestGame::new(2);
    let i = t.battlefield(P1, "Mercenary Informer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let blade = t.hand(P0, "Doom Blade");
    t.cast(P0, blade).target(bears).go();
    t.resolve_all();
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(i)));
    // An ability from a black source (Festering Goblin's death trigger).
    let mut t = TestGame::new(2);
    let i = t.battlefield(P1, "Mercenary Informer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let g = t.battlefield(P0, "Festering Goblin");
    t.answer_targets(P0, &[obj(bears)]);
    kill(&mut t, g);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(i)));
    // A red spell and a red source's ability can target it.
    let mut t = TestGame::new(2);
    let i = t.battlefield(P1, "Mercenary Informer");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    act(&mut t, P0, pyro, "damage", &[obj(i)]).unwrap();
    t.resolve_all();
    assert!(target_choices(&t, P0)[0].contains(&obj(i)));
    assert!(!t.on_battlefield(i), "a 2/1 dealt 1 damage");
    let mut t = TestGame::new(2);
    let i = t.battlefield(P1, "Mercenary Informer");
    bolt(&mut t, P0, obj(i));
    assert!(target_choices(&t, P0)[0].contains(&obj(i)));
    assert!(!t.on_battlefield(i));
}

#[test]
fn day_of_destiny_pumps_your_legendary_creatures() {
    cr!("611.3a", "613.4c");
    // "Legendary creatures you control get +2/+2."
    supported("Day of Destiny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Day of Destiny");
    let leg = t.battlefield(P0, "Isamaru, Hound of Konda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.g.recompute();
    assert_eq!(t.pt(leg), (4, 4));
    assert_eq!(t.pt(bears), (2, 2), "not legendary");
    assert_eq!(t.pt(theirs), (2, 2), "an opponent's");
}

#[test]
fn mobilization_gives_soldiers_vigilance() {
    cr!("611.3a", "613.1f", "702.20b");
    // "Soldier creatures have vigilance."
    supported("Mobilization");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mobilization");
    let s = t.battlefield(P0, "Glory Seeker");
    let theirs = t.battlefield(P1, "Glory Seeker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(s).has_keyword(KeywordKind::Vigilance));
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Vigilance), "all Soldiers");
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(s, Entity::Player(P1)), (bears, Entity::Player(P1))], &[]);
    assert!(!t.obj_now(s).tapped);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn moderation_allows_you_one_spell_each_turn() {
    cr!("601.3");
    // "You can't cast more than one spell each turn. / Whenever you cast a spell, draw a
    // card."
    supported("Moderation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Moderation");
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Lightning Bolt");
    let b = t.hand(P0, "Lightning Bolt");
    t.cast(P0, a).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), 2, "it drew a card for the spell");
    assert!(t.cast(P0, b).target(Entity::Player(P1)).try_go().is_err());
    t.clear_answers();
    assert_eq!(t.zone(b), Zone::Hand(P0));
    // It doesn't stop the opponent.
    t.lands(P1, "Mountain", 2);
    let c = t.hand(P1, "Lightning Bolt");
    let d = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, c).target(Entity::Player(P0)).go();
    t.resolve_all();
    t.cast(P1, d).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    // Next turn you may cast one again.
    next_turn(&mut t);
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn moldgraf_scavenger_gets_plus_3_with_delirium() {
    cr!("611.3a", "613.4c", "205.2a");
    // "Delirium — This creature gets +3/+0 as long as there are four or more card types
    // among cards in your graveyard."
    supported("Moldgraf Scavenger");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Moldgraf Scavenger");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Memnite");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Mass Hysteria");
    t.g.recompute();
    assert_eq!(t.pt(s), (0, 4), "land, artifact, creature: three types");
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(s), (0, 4), "still three");
    let b = t.hand(P0, "Lightning Bolt");
    t.g.discard(P0, b, None);
    t.resolve_all();
    assert_eq!(t.pt(s), (3, 4), "an instant makes four");
}

#[test]
fn prowlers_helm_lets_only_walls_block() {
    cr!("509.1b", "611.3a", "301.5");
    // "Equipped creature can't be blocked except by Walls. / Equip {2}"
    supported("Prowler's Helm");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Prowler's Helm");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, h, "equip", &[obj(bears)]).unwrap();
    t.resolve_all();
    attack_with(&mut t, &[bears, other]);
    assert!(!t.g.can_block(giant, bears));
    assert!(t.g.can_block(wall, bears));
    assert!(t.g.can_block(giant, other), "only the equipped creature");
    block(&mut t, P1, &[(wall, bears), (giant, other)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "a Wall blocked it");
    assert_eq!(t.obj_now(wall).damage, 2);
}
