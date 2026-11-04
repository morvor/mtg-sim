//! Rulings batch P198 — effects from resolved spells and abilities with no duration (CR
//! 611.2): they outlast the turn and their source, apply only to the objects affected as
//! they began (611.2c), end when the object changes zones (400.7), and aren't copiable
//! values (707.2).

use crate::r_p198_common::*;
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn fated_returns_indestructible_lasts_until_the_creature_leaves_and_isnt_copied() {
    cr!("611.2a", "400.7", "707.2");
    ruling!(
        "Fated Return",
        "The effect that causes the creature to gain indestructible doesn’t have a duration. It lasts until the creature leaves the battlefield."
    );
    ruling!(
        "Fated Return",
        "The indestructible granted by Fated Return isn’t part of the creature’s copiable values."
    );
    supported("Fated Return");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P1, "Hill Giant");
    cast_at(&mut t, P0, "Fated Return", &[obj(card)]);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(ctl(&t, giant), P0);
    // Turns later it's still indestructible: destroying it does nothing.
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, giant);
    assert!(t.on_battlefield(giant));
    // A Clone copying it isn't indestructible.
    t.answer_choose(P1, &[obj(giant)]);
    t.answer_yes(P1, true);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    assert!(!t.obj_now(clone).has_keyword(KeywordKind::Indestructible));
    // Once it leaves and returns it's a new object without indestructible.
    bounce(&mut t, giant);
    let hand = t.g.find_in_zone(Zone::Hand(P1), "Hill Giant")[0];
    t.g.move_object(hand, Zone::Battlefield, MoveCause::Effect, Some(P1));
    t.g.recompute();
    let again = t.named_on_battlefield("Hill Giant");
    assert!(again
        .iter()
        .all(|g| !t.obj_now(*g).has_keyword(KeywordKind::Indestructible)));
}

#[test]
fn archangel_elspeths_minus_two_outlasts_elspeth() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Archangel Elspeth",
        "The second ability doesn’t have a duration. It lasts until the creature leaves the battlefield."
    );
    supported("Archangel Elspeth");
    let mut t = TestGame::new(2);
    let el = t.battlefield(P0, "Archangel Elspeth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, el, 1, &[obj(bears)]);
    assert_eq!(t.pt(bears), (4, 4));
    kill(&mut t, el);
    pass_turn(&mut t);
    pass_turn(&mut t);
    let o = t.obj_now(bears);
    assert!(o.has_keyword(KeywordKind::Flying));
    assert!(o.chars.has_subtype("Angel") && o.chars.has_subtype("Bear"));
    // Only the creature affected then: a Grizzly Bears entering later isn't an Angel.
    let later = t.battlefield(P0, "Grizzly Bears");
    assert!(!t.obj_now(later).chars.has_subtype("Angel"));
}

#[test]
fn riding_the_dilu_horse_lasts_until_the_creature_leaves() {
    cr!("611.2a", "400.7");
    ruling!(
        "Riding the Dilu Horse",
        "The spell’s effect has no duration. The targeted creature gets +2/+2 until the game ends or it leaves the battlefield. It has horsemanship until the game ends, it leaves the battlefield, or some other effect causes it to lose horsemanship."
    );
    supported("Riding the Dilu Horse");
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Riding the Dilu Horse", &[obj(bears)]);
    t.resolve_all();
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Horsemanship));
    // A later "loses all abilities" effect removes horsemanship (and sets it 1/1, with
    // the +2/+2 still applied in layer 7c).
    cast_at(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Horsemanship));
    assert_eq!(t.pt(bears), (3, 3));
    // It ends when the creature leaves the battlefield.
    bounce(&mut t, bears);
    let hand = t.g.find_in_zone(Zone::Hand(P0), "Grizzly Bears")[0];
    t.g.move_object(hand, Zone::Battlefield, MoveCause::Effect, Some(P0));
    t.g.recompute();
    let again = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.pt(again), (2, 2));
    assert!(!t.obj_now(again).has_keyword(KeywordKind::Horsemanship));
}

#[test]
fn verdant_touch_land_stays_a_land_with_its_abilities() {
    cr!("611.2a", "613.1d", "205.1b");
    ruling!(
        "Verdant Touch",
        "The land is still a land and retains all its abilities, plus any other characteristics."
    );
    supported("Verdant Touch");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    cast_at(&mut t, P0, "Verdant Touch", &[obj(land)]);
    t.resolve_all();
    pass_turn(&mut t);
    let o = t.obj_now(land);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Forest"));
    assert_eq!(t.pt(land), (2, 2));
    assert!(has_ability(&t, land, "Add {G}"));
}

#[test]
fn beorns_hospitality_stays_a_bear_after_the_turn() {
    cr!("611.2a", "613.4a");
    ruling!(
        "Beorn's Hospitality",
        "The type-changing and ability-granting effects last indefinitely. They don't wear off during the cleanup step."
    );
    supported("Beorn's Hospitality");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Beorn's Hospitality");
    t.lands(P0, "Forest", 3);
    act(&mut t, P0, b, 0, &[]);
    assert_eq!(t.pt(b), (3, 3));
    pass_turn(&mut t);
    pass_turn(&mut t);
    let o = t.obj_now(b);
    assert!(o.is(CardType::Creature) && o.is(CardType::Enchantment));
    assert!(o.chars.has_subtype("Bear"));
    t.battlefield(P0, "Forest");
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn dranas_returned_creature_stays_a_vampire_after_drana_leaves() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Drana, the Last Bloodchief",
        "The returned creature remains a Vampire in addition to its other types even after Drana leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, the Last Bloodchief");
    let card = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P1, &[obj(card)]);
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(drana, pl(P1))]);
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    kill(&mut t, drana);
    pass_turn(&mut t);
    let o = t.obj_now(giant);
    assert!(o.chars.has_subtype("Vampire") && o.chars.has_subtype("Giant"));
}

#[test]
fn olivias_discard_is_decided_on_resolution_and_the_rest_follows_at_once() {
    cr!("608.2c", "611.2a");
    ruling!(
        "Olivia, Mobilized for War",
        "You decide whether to discard a card as the triggered ability of Olivia, Mobilized for War resolves. If you do, the rest of the ability’s effects happen before any player may take actions."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Olivia, Mobilized for War");
    t.hand(P0, "Island");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let asked = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    // The discard choice was made during resolution (not as it triggered).
    assert!(t.asked()[asked..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    assert!(t.in_graveyard(P0, "Island"));
    let o = t.obj_now(bears);
    assert_eq!(o.counter("+1/+1"), 1);
    assert!(o.has_keyword(KeywordKind::Haste));
    assert!(o.chars.has_subtype("Vampire"));
    // Haste ends with the turn; being a Vampire doesn't.
    pass_turn(&mut t);
    let o = t.obj_now(bears);
    assert!(!o.has_keyword(KeywordKind::Haste));
    assert!(o.chars.has_subtype("Vampire"));
    // Declining: nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Olivia, Mobilized for War");
    t.hand(P0, "Island");
    let bears = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Island"));
    assert!(!t.obj_now(bears).chars.has_subtype("Vampire"));
}

#[test]
fn brilliance_unleashed_robot_keeps_the_artifacts_abilities() {
    cr!("611.2a", "613.1d");
    ruling!(
        "Brilliance Unleashed",
        "The last mode of Brilliance Unleashed doesn't remove any abilities the artifact has."
    );
    let mut t = TestGame::new(2);
    let ring = t.graveyard(P0, "Sol Ring");
    lots(&mut t, P0);
    let c = t.hand(P0, "Brilliance Unleashed");
    t.cast(P0, c).modes(&[1]).target(ring).go();
    t.resolve_all();
    let ring = t.named_on_battlefield("Sol Ring")[0];
    pass_turn(&mut t);
    let o = t.obj_now(ring);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Robot"));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(ring), (3, 3));
    assert!(has_ability(&t, ring, "Add {C}{C}"));
}

#[test]
fn grave_betrayal_returns_it_with_exactly_one_counter_as_a_black_zombie() {
    cr!("400.7", "122.6", "611.2a");
    ruling!(
        "Grave Betrayal",
        "The phrase \"with an additional +1/+1 counter on it\" means that the creature enters with one more +1/+1 counter than it would normally enter with. It doesn't keep any counters it had on it when it died."
    );
    supported("Grave Betrayal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grave Betrayal");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(obj(giant), "+1/+1", 3, None);
    kill(&mut t, giant);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(ctl(&t, back), P0);
    assert_eq!(t.counters(back, "+1/+1"), 1);
    assert_eq!(t.pt(back), (4, 4));
    pass_turn(&mut t);
    let o = t.obj_now(back);
    assert!(o.chars.has_subtype("Zombie") && o.chars.has_subtype("Giant"));
    assert!(o.chars.colors.contains(mtg_engine::types::Color::Black));
    assert!(o.chars.colors.contains(mtg_engine::types::Color::Red));
}

#[test]
fn grimoire_of_the_dead_returns_artifact_creature_cards_too() {
    cr!("205.2a", "611.2a");
    ruling!(
        "Grimoire of the Dead",
        "“Creature cards” includes each card with the type creature, even if it has additional types, such as artifact."
    );
    supported("Grimoire of the Dead");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grimoire of the Dead");
    t.g.add_counters(obj(g), "study", 3, None);
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P0, "Sol Ring");
    act(&mut t, P0, g, 1, &[]);
    let o = t.named_on_battlefield("Ornithopter");
    assert_eq!(o.len(), 1);
    assert_eq!(ctl(&t, o[0]), P0);
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
    pass_turn(&mut t);
    assert!(t.obj_now(o[0]).chars.has_subtype("Zombie"));
}

#[test]
fn whip_of_erebos_creature_is_exiled_and_never_dies() {
    cr!("614.1a", "700.4", "611.2a");
    ruling!(
        "Whip of Erebos",
        "The exiled creature is never put into the graveyard. Any abilities the creature has that trigger when it dies won't trigger."
    );
    ruling!(
        "Whip of Erebos",
        "Whip of Erebos grants haste to the creature that's returned to the battlefield. However, neither of the \"exile\" abilities is granted to that creature. If that creature loses all its abilities, it will still be exiled at the beginning of the end step, and if it would leave the battlefield, it is still exiled instead."
    );
    supported("Whip of Erebos");
    // Destroyed: exiled instead, and its dies trigger doesn't trigger.
    let mut t = TestGame::new(2);
    let whip = t.battlefield(P0, "Whip of Erebos");
    let card = t.graveyard(P0, "Doomed Traveler");
    act(&mut t, P0, whip, 0, &[obj(card)]);
    let dt = t.named_on_battlefield("Doomed Traveler")[0];
    assert!(t.obj_now(dt).has_keyword(KeywordKind::Haste));
    assert!(!has_ability(&t, dt, "exile"));
    assert!(!has_ability(&t, dt, "Exile"));
    kill(&mut t, dt);
    t.resolve_all();
    assert!(t.in_exile("Doomed Traveler"));
    assert!(!t.in_graveyard(P0, "Doomed Traveler"));
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
    // Losing all its abilities (and the Whip leaving): still exiled at the end step, and
    // still exiled instead of dying.
    let mut t = TestGame::new(2);
    let whip = t.battlefield(P0, "Whip of Erebos");
    let card = t.graveyard(P0, "Doomed Traveler");
    act(&mut t, P0, whip, 0, &[obj(card)]);
    let dt = t.named_on_battlefield("Doomed Traveler")[0];
    cast_at(&mut t, P0, "Turn to Frog", &[obj(dt)]);
    t.resolve_all();
    assert!(!t.obj_now(dt).has_keyword(KeywordKind::Haste));
    kill(&mut t, whip);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Doomed Traveler"));
    let mut t = TestGame::new(2);
    let whip = t.battlefield(P0, "Whip of Erebos");
    let card = t.graveyard(P0, "Doomed Traveler");
    act(&mut t, P0, whip, 0, &[obj(card)]);
    let dt = t.named_on_battlefield("Doomed Traveler")[0];
    cast_at(&mut t, P0, "Turn to Frog", &[obj(dt)]);
    t.resolve_all();
    bounce(&mut t, dt);
    assert!(t.in_exile("Doomed Traveler"));
}

#[test]
fn kheru_lich_lord_pays_on_resolution_and_its_creature_never_dies() {
    cr!("614.1a", "603.5");
    ruling!(
        "Kheru Lich Lord",
        "You decide whether to pay {2}{B} as the ability resolves."
    );
    ruling!(
        "Kheru Lich Lord",
        "The exiled creature is never put into the graveyard. Any abilities the creature has that trigger when it dies won’t trigger."
    );
    supported("Kheru Lich Lord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kheru Lich Lord");
    t.graveyard(P0, "Doomed Traveler");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    // The payment is asked as the ability resolves, with mana made then.
    let asked = t.asked().len();
    lots(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.asked()[asked..].iter().any(|(p, _)| *p == P0));
    let dt = t.named_on_battlefield("Doomed Traveler")[0];
    let o = t.obj_now(dt);
    assert!(o.has_keyword(KeywordKind::Flying) && o.has_keyword(KeywordKind::Trample));
    kill(&mut t, dt);
    t.resolve_all();
    assert!(t.in_exile("Doomed Traveler"));
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
}

#[test]
fn kioras_chapter_two_keeps_them_tapped_after_the_saga_leaves_and_ignores_hexproof() {
    cr!("611.2a", "702.11b", "502.3");
    ruling!(
        "Kiora Bests the Sea God",
        "The second chapter ability keeps the permanents tapped even if Kiora Bests the Sea God leaves the battlefield before that player’s next untap step."
    );
    ruling!(
        "Kiora Bests the Sea God",
        "The second chapter ability targets the player but not their permanents. Permanents with hexproof may be tapped this way."
    );
    supported("Kiora Bests the Sea God");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Kiora Bests the Sea God");
    t.g.add_counters(obj(s), "lore", 1, None);
    let hexproof = t.battlefield(P1, "Silhana Ledgewalker");
    let land = t.battlefield(P1, "Island");
    t.answer_targets(P0, &[pl(P1)]);
    t.g.add_counters(obj(s), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.obj_now(hexproof).tapped);
    assert!(!t.obj_now(land).tapped);
    bounce(&mut t, s);
    pass_turn(&mut t);
    assert!(t.obj_now(hexproof).tapped);
    // It untaps normally the turn after.
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(!t.obj_now(hexproof).tapped);
}

#[test]
fn okos_elk_effect_lasts_and_its_exchange_survives_a_power_boost() {
    cr!("611.2a", "613.1b", "613.4b");
    ruling!(
        "Oko, Thief of Crowns",
        "The effects of Oko's third ability lasts indefinitely. It doesn't expire during the cleanup step or if the creature you take has its power raised above 3 later."
    );
    supported("Oko, Thief of Crowns");
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    t.g.objects[oko.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, oko, 2, &[obj(ring), obj(bears)]);
    assert_eq!(ctl(&t, ring), P1);
    assert_eq!(ctl(&t, bears), P0);
    t.g.add_counters(obj(bears), "+1/+1", 3, None);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(ctl(&t, bears), P0);
    assert_eq!(ctl(&t, ring), P1);
}

#[test]
fn shifting_grift_exchange_outlasts_the_turn_and_a_type_change() {
    cr!("611.2a", "701.12b", "613.1b");
    ruling!(
        "Shifting Grift",
        "The effects of Shifting Grift’s modes last indefinitely. They don’t wear off during the cleanup step, and they don’t expire if one of the target permanents stops having that permanent type after Shifting Grift has resolved."
    );
    ruling!(
        "Shifting Grift",
        "You don’t have to control any of the target permanents."
    );
    // Mishra's Factory animated until end of turn is exchanged for a creature; at
    // cleanup it stops being a creature but stays with its new controller.
    let mut t = TestGame::new(2);
    let factory = t.battlefield(P0, "Mishra's Factory");
    act(&mut t, P0, factory, 1, &[]);
    assert!(t.obj_now(factory).is(CardType::Creature));
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let c = t.hand(P0, "Shifting Grift");
    t.cast(P0, c)
        .modes(&[0])
        .targets(&[obj(factory), obj(bears)])
        .go();
    t.resolve_all();
    assert_eq!(ctl(&t, factory), P1);
    assert_eq!(ctl(&t, bears), P0);
    pass_turn(&mut t);
    assert!(!t.obj_now(factory).is(CardType::Creature));
    assert_eq!(ctl(&t, factory), P1);
    assert_eq!(ctl(&t, bears), P0);
    // Two permanents controlled by two opponents.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    lots(&mut t, P0);
    let c = t.hand(P0, "Shifting Grift");
    t.cast(P0, c).modes(&[0]).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    assert_eq!(ctl(&t, a), P2);
    assert_eq!(ctl(&t, b), P1);
}

#[test]
fn gilt_leaf_archdruid_keeps_the_lands_after_it_leaves() {
    cr!("611.2a", "613.1b");
    ruling!(
        "Gilt-Leaf Archdruid",
        "The second ability’s effect has no duration. You retain control of those lands until the game ends or until some other effect causes them to change control."
    );
    supported("Gilt-Leaf Archdruid");
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Gilt-Leaf Archdruid");
    for _ in 0..6 {
        t.battlefield(P0, "Llanowar Elves");
    }
    let lands = t.lands(P1, "Island", 2);
    act(&mut t, P0, arch, 0, &[pl(P1)]);
    assert!(lands.iter().all(|l| ctl(&t, *l) == P0));
    kill(&mut t, arch);
    pass_turn(&mut t);
    assert!(lands.iter().all(|l| ctl(&t, *l) == P0));
    // A later control change overrides it (layer 2, timestamp order).
    lots(&mut t, P1);
    let annex = t.hand(P1, "Annex");
    t.cast(P1, annex).target(lands[0]).go();
    t.resolve_all();
    assert_eq!(ctl(&t, lands[0]), P1);
    assert_eq!(ctl(&t, lands[1]), P0);
}
