//! Structure-coverage tests, stream 2 (item `structure-tests-2`): activated- and
//! mana-ability structures that alone kept their cards from being verified (each test
//! verifies the cards that share its structure; see `docs/CARD_COVERAGE.md`). Each test
//! checks that a real card's ability does what its text says.

use super::structure_activated_2::{act, next_turn, target_choices, untapped};
use super::structure_activated_6::entity_choices;
use super::structure_mana_2::{pool, pool_size};
use super::structure_seed_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Casts a Lightning Bolt from `p`'s hand at `target` (without resolving it).
fn bolt_at(t: &mut TestGame, p: PlayerId, target: Entity) -> ObjectId {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go()
}

#[test]
fn conservator_prevents_the_next_two_damage_to_you() {
    cr!("602.2", "615.1", "615.7");
    // "{3}, {T}: Prevent the next 2 damage that would be dealt to you this turn."
    supported("Conservator");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Conservator");
    t.lands(P0, "Wastes", 2);
    assert!(
        act(&mut t, P0, c, "prevent", &[]).is_err(),
        "{{3}} needs three mana"
    );
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, c, "prevent", &[]).unwrap();
    assert!(t.obj_now(c).tapped);
    assert_eq!(untapped(&t, P0, "Wastes"), 0);
    t.resolve_all();
    // Damage to the opponent isn't prevented and doesn't use up the shield.
    bolt_at(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    bolt_at(&mut t, P1, Entity::Player(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 19, "2 of the 3 damage prevented");
    bolt_at(&mut t, P1, Entity::Player(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 16, "the shield is used up");
}

#[test]
fn rishadan_port_taps_target_land() {
    cr!("602.2", "115.1", "701.26a");
    // "{1}, {T}: Tap target land."
    supported("Rishadan Port");
    let mut t = TestGame::new(2);
    let port = t.battlefield(P0, "Rishadan Port");
    let theirs = t.battlefield(P1, "Forest");
    let other = t.battlefield(P1, "Island");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(
        act(&mut t, P0, port, "tap target", &[obj(theirs)]).is_err(),
        "{{1}} unpaid"
    );
    t.clear_answers();
    t.lands(P0, "Wastes", 1);
    act(&mut t, P0, port, "tap target", &[obj(theirs)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(theirs)) && c.contains(&obj(other)));
    assert!(!c.contains(&obj(bears)), "lands only");
    assert!(t.obj_now(port).tapped);
    assert!(!t.obj_now(theirs).tapped, "not before it resolves");
    t.resolve_all();
    assert!(t.obj_now(theirs).tapped);
    assert!(!t.obj_now(other).tapped && !t.obj_now(bears).tapped);
}

#[test]
fn aurora_griffin_makes_target_permanent_white_until_end_of_turn() {
    cr!("602.2", "105.3", "611.2a", "514.2");
    // "{W}: Target permanent becomes white until end of turn."
    supported("Aurora Griffin");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Aurora Griffin");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Plains", 2);
    act(&mut t, P0, g, "white", &[obj(dragon)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(
        c.contains(&obj(land)) && c.contains(&obj(bears)),
        "any permanent"
    );
    t.resolve_all();
    assert_eq!(
        t.obj_now(dragon).chars.colors,
        ColorSet::single(Color::White),
        "only white"
    );
    assert_eq!(
        t.obj_now(bears).chars.colors,
        ColorSet::single(Color::Green)
    );
    act(&mut t, P0, g, "white", &[obj(land)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(land).chars.colors, ColorSet::single(Color::White));
    next_turn(&mut t);
    assert_eq!(t.obj_now(dragon).chars.colors, ColorSet::single(Color::Red));
    assert!(t.obj_now(land).chars.colors.is_colorless());
}

#[test]
fn pilgrim_of_justice_prevents_damage_from_a_chosen_red_source() {
    cr!("602.2", "609.7a", "615.1");
    // "{W}, Sacrifice this creature: The next time a red source of your choice would deal
    // damage this turn, prevent that damage."
    supported("Pilgrim of Justice");
    let mut t = TestGame::new(2);
    let p = t.battlefield(P0, "Pilgrim of Justice");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let b = bolt_at(&mut t, P1, Entity::Player(P0));
    t.answer_choose(P0, &[obj(b)]);
    act(&mut t, P0, p, "prevent", &[]).unwrap();
    assert!(
        t.in_graveyard(P0, "Pilgrim of Justice"),
        "sacrificed as a cost"
    );
    t.resolve(); // the Pilgrim's ability
    let c = entity_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(b)), "the red spell can be chosen");
    assert!(!c.contains(&obj(bears)), "a green source can't");
    t.resolve_all();
    assert_eq!(
        t.life(P0),
        20,
        "the chosen red source's damage is prevented"
    );
    bolt_at(&mut t, P1, Entity::Player(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "only the next time");
}

#[test]
fn bonecaller_cleric_reanimates_only_as_a_sorcery() {
    cr!("602.2", "602.5d", "115.1");
    // "{3}{B}, Sacrifice this creature: Return target creature card from your graveyard
    // to the battlefield. Activate only as a sorcery."
    supported("Bonecaller Cleric");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Bonecaller Cleric");
    let giant = t.graveyard(P0, "Hill Giant");
    let land = t.graveyard(P0, "Forest");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    // Not with a spell on the stack.
    bolt_at(&mut t, P1, Entity::Player(P1));
    assert!(
        act(&mut t, P0, c, "return", &[obj(giant)]).is_err(),
        "not at instant speed"
    );
    t.clear_answers();
    t.resolve_all();
    act(&mut t, P0, c, "return", &[obj(giant)]).unwrap();
    let ch = target_choices(&t, P0).pop().unwrap();
    assert!(ch.contains(&obj(giant)));
    assert!(!ch.contains(&obj(land)) && !ch.contains(&obj(theirs)));
    assert!(t.in_graveyard(P0, "Bonecaller Cleric"));
    t.resolve_all();
    assert_eq!(count_named(&t, P0, "Hill Giant"), 1);
    assert!(t.in_graveyard(P0, "Forest") && t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn sunscape_master_returns_target_creature_to_its_owners_hand() {
    cr!("602.2", "115.1");
    // "{U}{U}, {T}: Return target creature to its owner's hand."
    supported("Sunscape Master");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Sunscape Master");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 2);
    act(&mut t, P0, m, "return target", &[obj(giant)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(m)) && c.contains(&obj(bears)));
    assert!(!c.contains(&obj(land)), "creatures only");
    assert!(t.obj_now(m).tapped);
    t.resolve_all();
    assert!(t.in_hand(P1, "Hill Giant"), "its owner's hand");
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears));
}

#[test]
fn cunning_advisor_makes_an_opponent_discard_before_attackers_on_your_turn() {
    cr!("602.2", "602.5", "701.9a");
    // "{T}: Target opponent discards a card. Activate only during your turn, before
    // attackers are declared."
    supported("Cunning Advisor");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cunning Advisor");
    let card = t.hand(P1, "Hill Giant");
    t.hand(P0, "Grizzly Bears");
    act(&mut t, P0, a, "discard", &[Entity::Player(P1)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert_eq!(c, vec![Entity::Player(P1)], "opponents only");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Graveyard(P1));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // On the opponent's turn: not allowed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cunning Advisor");
    t.hand(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    assert!(act(&mut t, P0, a, "discard", &[Entity::Player(P1)]).is_err());
    t.clear_answers();
    // In the beginning of combat step of your turn: still before attackers.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cunning Advisor");
    t.hand(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(act(&mut t, P0, a, "discard", &[Entity::Player(P1)]).is_ok());
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // Once attackers have been declared: not allowed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Cunning Advisor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(act(&mut t, P0, a, "discard", &[Entity::Player(P1)]).is_err());
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn collectors_vault_loots_and_creates_a_treasure() {
    cr!("602.2", "121.1", "701.9a", "111.10a");
    // "{2}, {T}: Draw a card, then discard a card. Create a Treasure token."
    supported("Collector's Vault");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Collector's Vault");
    let giant = t.hand(P0, "Hill Giant");
    let keep = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(giant)]);
    act(&mut t, P0, v, "draw", &[]).unwrap();
    assert!(t.obj_now(v).tapped);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 29, "drew one");
    assert_eq!(t.hand_size(P0), 2, "drew one, discarded one");
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.zone(keep), Zone::Hand(P0));
    let tr = subtype_ids(&t, P0, "Treasure");
    assert_eq!(tr.len(), 1);
    assert!(t.obj_now(tr[0]).is_token());
    assert_eq!(count_subtype(&t, P1, "Treasure"), 0);
}

#[test]
fn air_servant_taps_target_creature_with_flying() {
    cr!("602.2", "115.1", "702.9a");
    // "{2}{U}: Tap target creature with flying."
    supported("Air Servant");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Air Servant");
    let flyer = t.battlefield(P1, "Wind Drake");
    let ground = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 3);
    act(&mut t, P0, s, "tap target", &[obj(flyer)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(flyer)) && c.contains(&obj(s)));
    assert!(!c.contains(&obj(ground)), "only creatures with flying");
    assert!(!t.obj_now(s).tapped, "no {{T}} in the cost");
    t.resolve_all();
    assert!(t.obj_now(flyer).tapped);
    assert!(!t.obj_now(ground).tapped);
}

#[test]
fn goblin_taskmaster_pumps_target_goblin_until_end_of_turn() {
    cr!("602.2", "115.1", "611.2a");
    // "{1}{R}: Target Goblin creature gets +1/+0 until end of turn."
    supported("Goblin Taskmaster");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Goblin Taskmaster");
    let gob = t.battlefield(P1, "Raging Goblin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    act(&mut t, P0, m, "+1/+0", &[obj(gob)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(gob)) && c.contains(&obj(m)), "any Goblin");
    assert!(!c.contains(&obj(bears)), "Goblins only");
    t.resolve_all();
    act(&mut t, P0, m, "+1/+0", &[obj(m)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(gob), (2, 1));
    assert_eq!(t.pt(m), (2, 1));
    assert_eq!(t.pt(bears), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(gob), (1, 1));
    assert_eq!(t.pt(m), (1, 1));
}

#[test]
fn frostburn_weird_gets_plus_one_minus_one_per_activation() {
    cr!("602.2", "611.2a", "107.4e");
    // "{U/R}: This creature gets +1/-1 until end of turn."
    supported("Frostburn Weird");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Frostburn Weird");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, w, "+1/-1", &[]).unwrap();
    act(&mut t, P0, w, "+1/-1", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Island") + untapped(&t, P0, "Mountain"), 0);
    assert!(act(&mut t, P0, w, "+1/-1", &[]).is_err(), "no mana left");
    t.clear_answers();
    t.resolve_all();
    assert_eq!(t.pt(w), (3, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(w), (1, 4));
}

#[test]
fn welding_jar_regenerates_target_artifact() {
    cr!("602.2", "701.19a");
    // "Sacrifice this artifact: Regenerate target artifact."
    supported("Welding Jar");
    let mut t = TestGame::new(2);
    let jar = t.battlefield(P0, "Welding Jar");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, jar, "regenerate", &[obj(thopter)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(thopter)));
    assert!(!c.contains(&obj(bears)), "artifacts only");
    assert!(t.in_graveyard(P0, "Welding Jar"), "sacrificed as the cost");
    t.resolve_all();
    t.g.destroy(thopter, None);
    t.resolve_all();
    assert!(t.on_battlefield(thopter), "regenerated");
    assert!(t.obj_now(thopter).tapped, "regeneration taps it");
    t.g.destroy(thopter, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"), "the shield was used up");
}

#[test]
fn wildheart_invoker_gives_plus_five_and_trample_until_end_of_turn() {
    cr!("602.2", "611.2a", "702.19a");
    // "{8}: Target creature gets +5/+5 and gains trample until end of turn."
    supported("Wildheart Invoker");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wildheart Invoker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 7);
    assert!(
        act(&mut t, P0, w, "+5/+5", &[obj(bears)]).is_err(),
        "{{8}} needs eight mana"
    );
    t.clear_answers();
    t.lands(P0, "Forest", 1);
    act(&mut t, P0, w, "+5/+5", &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(w), (4, 3));
    assert!(!t.obj_now(w).has_keyword(KeywordKind::Trample));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Trample));
}

#[test]
fn lithatog_sacrifices_a_land_for_plus_one_plus_one() {
    cr!("602.2", "118.3", "611.2a");
    // "Sacrifice a land: This creature gets +1/+1 until end of turn."
    supported("Lithatog");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lithatog");
    let art = t.battlefield(P0, "Ornithopter");
    assert!(
        act(&mut t, P0, l, "sacrifice a land", &[]).is_err(),
        "no land"
    );
    t.clear_answers();
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    act(&mut t, P0, l, "sacrifice a land", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(
        t.on_battlefield(theirs) && t.on_battlefield(art),
        "only your land"
    );
    t.resolve_all();
    assert_eq!(t.pt(l), (2, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(l), (1, 2));
}

#[test]
fn glittermonger_taps_for_a_treasure() {
    cr!("602.2", "111.10a");
    // "{T}: Create a Treasure token."
    supported("Glittermonger");
    let mut t = TestGame::new(2);
    let g = t.battlefield_sick(P0, "Glittermonger");
    assert!(
        act(&mut t, P0, g, "treasure", &[]).is_err(),
        "summoning sick"
    );
    t.clear_answers();
    let g2 = t.battlefield(P0, "Glittermonger");
    act(&mut t, P0, g2, "treasure", &[]).unwrap();
    assert!(t.obj_now(g2).tapped);
    t.resolve_all();
    let tr = subtype_ids(&t, P0, "Treasure");
    assert_eq!(tr.len(), 1);
    assert!(t.obj_now(tr[0]).is_token());
    assert_eq!(count_subtype(&t, P1, "Treasure"), 0);
    assert!(
        act(&mut t, P0, g2, "treasure", &[]).is_err(),
        "already tapped"
    );
}

#[test]
fn eternal_dragon_returns_from_the_graveyard_only_during_your_upkeep() {
    cr!("602.2", "602.5");
    // "{3}{W}{W}: Return this card from your graveyard to your hand. Activate only during
    // your upkeep."
    supported("Eternal Dragon");
    let mut t = TestGame::new(2);
    let d = t.graveyard(P0, "Eternal Dragon");
    t.lands(P0, "Plains", 5);
    assert!(
        act(&mut t, P0, d, "return", &[]).is_err(),
        "not in the main phase"
    );
    t.clear_answers();
    t.set_step(P1, Step::Upkeep);
    assert!(
        act(&mut t, P0, d, "return", &[]).is_err(),
        "not in an opponent's upkeep"
    );
    t.clear_answers();
    t.set_step(P0, Step::Upkeep);
    act(&mut t, P0, d, "return", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Plains"), 0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Eternal Dragon"));
    assert!(!t.in_graveyard(P0, "Eternal Dragon"));
}

#[test]
fn hotfoot_gnome_gives_another_target_creature_haste() {
    cr!("602.2", "702.10b");
    // "{T}: Another target creature gains haste until end of turn."
    supported("Hotfoot Gnome");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Hotfoot Gnome");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    act(&mut t, P0, g, "haste", &[obj(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(bears)) && c.contains(&obj(theirs)));
    assert!(!c.contains(&obj(g)), "another creature");
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Haste));
    // The summoning-sick Bears can attack now.
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.obj_now(bears).tapped, "it attacked");
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
}

#[test]
fn harbinger_of_the_hunt_pings_each_other_flyer() {
    cr!("602.2", "120.3e");
    // "{2}{G}: This creature deals 1 damage to each other creature with flying."
    supported("Harbinger of the Hunt");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Harbinger of the Hunt");
    let mine = t.battlefield(P0, "Wind Drake");
    let theirs = t.battlefield(P1, "Birds of Paradise");
    let ground = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    act(&mut t, P0, h, "each other creature with flying", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).damage, 1, "your flyers too");
    assert_eq!(t.obj_now(h).damage, 0, "not itself");
    assert_eq!(t.obj_now(ground).damage, 0, "not creatures without flying");
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
}

#[test]
fn ghor_clan_rampager_bloodrushes_an_attacking_creature() {
    cr!("602.2", "702.19a", "611.2a");
    // "Bloodrush — {R}{G}, Discard this card: Target attacking creature gets +4/+4 and
    // gains trample until end of turn."
    supported("Ghor-Clan Rampager");
    let mut t = TestGame::new(2);
    let r = t.hand(P0, "Ghor-Clan Rampager");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let home = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    act(&mut t, P0, r, "attacking creature", &[obj(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(bears)));
    assert!(
        !c.contains(&obj(home)) && !c.contains(&obj(wall)),
        "attacking creatures only"
    );
    assert!(
        t.in_graveyard(P0, "Ghor-Clan Rampager"),
        "discarded as the cost"
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (6, 6));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Trample));
    block(&mut t, P1, &[(wall, bears)]);
    t.advance_to_step(Step::EndOfCombat);
    assert_eq!(
        t.life(P1),
        17,
        "6 trample damage past a 0/3 wall: 3 trample over"
    );
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn myr_scrapling_sacrifices_for_a_counter_on_target_creature() {
    cr!("602.2", "122.1");
    // "Sacrifice this creature: Put a +1/+1 counter on target creature."
    supported("Myr Scrapling");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Myr Scrapling");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    act(&mut t, P0, m, "counter", &[obj(bears)]).unwrap();
    let c = target_choices(&t, P0).pop().unwrap();
    assert!(c.contains(&obj(bears)) && !c.contains(&obj(land)));
    assert!(t.in_graveyard(P0, "Myr Scrapling"));
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn faerie_dreamthief_exiles_itself_from_the_graveyard_to_draw_and_lose_life() {
    cr!("602.2", "121.1", "119.3");
    // "{2}{B}, Exile this card from your graveyard: You draw a card and you lose 1 life."
    supported("Faerie Dreamthief");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Faerie Dreamthief");
    t.lands(P0, "Swamp", 3);
    assert!(
        act(&mut t, P0, f, "draw", &[]).is_err(),
        "only from the graveyard"
    );
    t.clear_answers();
    let f = t.graveyard(P0, "Faerie Dreamthief");
    act(&mut t, P0, f, "draw", &[]).unwrap();
    assert!(t.in_exile("Faerie Dreamthief"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 19);
    assert_eq!((t.hand_size(P1), t.life(P1)), (0, 20));
}

#[test]
fn initiates_of_the_ebon_hand_is_sacrificed_after_four_activations() {
    cr!("605.1a", "605.3b", "603.7");
    // "{1}: Add {B}. If this ability has been activated four or more times this turn,
    // sacrifice this creature at the beginning of the next end step."
    supported("Initiates of the Ebon Hand");
    for n in [3usize, 4] {
        let mut t = TestGame::new(2);
        let i = t.battlefield(P0, "Initiates of the Ebon Hand");
        t.lands(P0, "Wastes", n);
        for _ in 0..n {
            act(&mut t, P0, i, "add {b}", &[]).unwrap();
        }
        assert_eq!(t.stack_len(), 0, "a mana ability");
        // Each {B} it adds pays the next activation's {1}: one Wastes paid for them all.
        assert_eq!(pool(&t, P0, ManaType::B), 1);
        assert_eq!(pool_size(&t, P0), 1);
        assert_eq!(untapped(&t, P0, "Wastes"), n - 1);
        t.g.player_mut(P0).mana_pool.mana.clear();
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(
            t.on_battlefield(i),
            n < 4,
            "sacrificed at the end step only after four activations ({n})"
        );
    }
}
