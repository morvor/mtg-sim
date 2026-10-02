//! Rulings batch P092 — "leaves the battlefield" abilities that return what an "enters"
//! ability exiled (Oblivion Ring, Fiend Hunter, Journey to Nowhere, Faceless Butcher,
//! Worldgorger Dragon, Wormfang Drake, Ashiok's Erasure, Fear of Abduction), and other
//! leaves-the-battlefield triggers (Sengir Autocrat, Thragtusk, Reveillark).

use crate::r_p058_common::decorate;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_cast, create_token, destroy, target_candidates};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `name` enters under P0's control (its "enters" ability aimed at `target`, answering
/// "yes" to a "may" when `may`); before that ability resolves, the permanent is destroyed.
/// Everything then resolves. Returns the permanent.
fn leaves_before_its_etb_resolves(
    t: &mut TestGame,
    name: &str,
    target: Option<Entity>,
    may: bool,
) -> ObjectId {
    if let Some(e) = target {
        t.answer_targets(P0, &[e]);
    }
    if may {
        t.answer_yes(P0, true);
    }
    let before = t.stack_len();
    let perm = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), before + 1, "{name}: its enters ability is on the stack");
    destroy(t, perm);
    assert_eq!(t.stack_len(), before + 2, "{name}: its leaves ability triggered");
    t.resolve_all();
    perm
}

#[test]
fn exilers_that_leave_before_their_etb_resolves_exile_forever() {
    cr!("603.6c", "603.10a", "608.2b");
    ruling!(
        "Oblivion Ring",
        "If Oblivion Ring leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing. Then its first ability will resolve and exile the targeted nonland permanent forever."
    );
    ruling!(
        "Fiend Hunter",
        "If Fiend Hunter leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing. Then its first ability will resolve and exile the target creature indefinitely."
    );
    ruling!(
        "Journey to Nowhere",
        "If Journey to Nowhere leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing. Then its first ability will resolve and exile the targeted creature forever."
    );
    for (name, may) in [
        ("Oblivion Ring", false),
        ("Fiend Hunter", true),
        ("Journey to Nowhere", false),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        leaves_before_its_etb_resolves(&mut t, name, Some(obj(bears)), may);
        assert_eq!(t.zone(bears), Zone::Exile, "{name}");
        // Nothing will ever return it.
        t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
        assert_eq!(t.zone(bears), Zone::Exile, "{name}");
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{name}");
    }
}

#[test]
fn worldgorger_dragon_that_leaves_first_exiles_everything_indefinitely() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Worldgorger Dragon",
        "If Worldgorger Dragon leaves the battlefield before its enters-the-battlefield ability resolves, the \"leaves-the-battlefield\" ability will trigger and resolve first. It won't return anything. Then the enters-the-battlefield ability will exile all other permanents you control indefinitely."
    );
    supported("Worldgorger Dragon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Hill Giant");
    leaves_before_its_etb_resolves(&mut t, "Worldgorger Dragon", None, false);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(t.on_battlefield(theirs));
    assert!(t.in_graveyard(P0, "Worldgorger Dragon"));
}

#[test]
fn ashioks_erasure_that_leaves_first_exiles_the_spell_for_the_rest_of_the_game() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Ashiok's Erasure",
        "If Ashiok's Erasure leaves the battlefield before its enters-the-battlefield ability has resolved, its last ability triggers and resolves with no effect, then its enters-the-battlefield ability exiles the target spell for the rest of the game."
    );
    supported("Ashiok's Erasure");
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    leaves_before_its_etb_resolves(&mut t, "Ashiok's Erasure", Some(obj(spell)), false);
    assert!(t.in_exile("Lightning Bolt"));
    assert!(!t.in_hand(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ashioks_erasure_exiles_a_spell_that_cant_be_countered() {
    cr!("101.1", "608.2b");
    ruling!(
        "Ashiok's Erasure",
        "The exiled spell isn't countered, but it won't resolve. This works against spells that can't be countered."
    );
    supported("Abrupt Decay");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Forest", 1);
    let decay = t.hand(P1, "Abrupt Decay");
    let decay = t.cast(P1, decay).target(bears).go();
    t.answer_targets(P0, &[obj(decay)]);
    t.enter(P0, "Ashiok's Erasure");
    t.resolve_all();
    assert!(t.in_exile("Abrupt Decay"));
    assert!(t.on_battlefield(bears));
}

/// P1 casts `name` (with enough mana); P0's Ashiok's Erasure exiles it.
fn erase(t: &mut TestGame, name: &str, method: CastMethod) -> ObjectId {
    give_mana_for(t, P1, name);
    let card = t.hand(P1, name);
    let spell = t.cast(P1, card).method(method).go();
    t.answer_targets(P0, &[obj(spell)]);
    let erasure = t.enter(P0, "Ashiok's Erasure");
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    let exiled = t.g.current(card);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile, "{name}");
    erasure
}

#[test]
fn ashioks_erasure_stops_either_half_of_an_exiled_split_card() {
    cr!("709.4", "201.2");
    ruling!(
        "Ashiok's Erasure",
        "If Ashiok's Erasure has exiled a split card, your opponents can't cast a spell with the same name as either half of that card."
    );
    supported("Fire // Ice");
    let mut t = TestGame::new(2);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    erase(&mut t, "Fire // Ice", CastMethod::Half(0));
    let other = t.hand(P1, "Fire // Ice");
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Island", 2);
    assert!(!can_cast(&mut t, P1, other, CastMethod::Half(0)), "Fire");
    assert!(!can_cast(&mut t, P1, other, CastMethod::Half(1)), "Ice");
}

#[test]
fn ashioks_erasure_lets_opponents_cast_the_adventure_of_an_exiled_adventurer() {
    cr!("715.3", "715.4");
    ruling!(
        "Ashiok's Erasure",
        "If Ashiok's Erasure has exiled an adventurer card, your opponents can't cast spells with the same name as the creature, but may cast their Adventures."
    );
    supported("Bonecrusher Giant");
    let mut t = TestGame::new(2);
    // P1 casts Stomp (the Adventure); the adventurer card Bonecrusher Giant is exiled.
    t.answer_targets(P1, &[Entity::Player(P0)]);
    erase(&mut t, "Bonecrusher Giant", CastMethod::Half(1));
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    let other = t.hand(P1, "Bonecrusher Giant");
    t.lands(P1, "Mountain", 3);
    assert!(!can_cast(&mut t, P1, other, CastMethod::Normal), "the creature");
    assert!(can_cast(&mut t, P1, other, CastMethod::Half(1)), "Stomp");
}

#[test]
fn ashioks_erasure_stops_nothing_once_the_exiled_card_leaves_exile() {
    cr!("400.7", "607.2a");
    ruling!(
        "Ashiok's Erasure",
        "If there is no exiled card (perhaps because the exiled spell was a copy of a spell), or if the exiled card leaves exile somehow, Ashiok's Erasure won't stop players from casting spells."
    );
    supported("Lightning Bolt");
    let mut t = TestGame::new(2);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    erase(&mut t, "Lightning Bolt", CastMethod::Normal);
    let other = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    assert!(!can_cast(&mut t, P1, other, CastMethod::Normal));
    let exiled = t.g.find_in_zone(Zone::Exile, "Lightning Bolt")[0];
    move_to(&mut t, exiled, Zone::Graveyard(P1));
    assert!(can_cast(&mut t, P1, other, CastMethod::Normal));
}

#[test]
fn fear_of_abduction_still_exiles_after_leaving_and_its_last_ability_does_nothing() {
    cr!("603.6c", "603.10a", "607.2a");
    ruling!(
        "Fear of Abduction",
        "If Fear of Abduction leaves the battlefield before its third ability resolves, the ability still exiles the target creature."
    );
    ruling!(
        "Fear of Abduction",
        "If there are no exiled cards when Fear of Abduction's last ability resolves (most likely because its third ability hasn't resolved yet), the ability won't do anything."
    );
    supported("Fear of Abduction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P1);
    leaves_before_its_etb_resolves(&mut t, "Fear of Abduction", Some(obj(bears)), false);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn sengir_autocrat_that_leaves_first_exiles_existing_serfs_then_makes_three() {
    cr!("603.6c", "603.10a", "111.1");
    ruling!(
        "Sengir Autocrat",
        "If Sengir Autocrat leaves the battlefield while its first ability is on the stack, its second ability triggers and exiles any Serf tokens that happen to be on the battlefield, and then its first ability resolves and gives you three Serf tokens."
    );
    ruling!(
        "Sengir Autocrat",
        "Sengir Autocrat’s last ability exiles all Serf tokens, not just the Serf tokens its first ability created."
    );
    supported("Sengir Autocrat");
    let mut t = TestGame::new(2);
    let mine = create_token(&mut t, P0, "Serf");
    let theirs = create_token(&mut t, P1, "Serf");
    leaves_before_its_etb_resolves(&mut t, "Sengir Autocrat", None, false);
    assert!(!t.g.is_live(mine) || !t.on_battlefield(mine));
    assert!(!t.g.is_live(theirs) || !t.on_battlefield(theirs));
    assert_eq!(with_subtype(&t, P0, "Serf").len(), 3);
    assert!(with_subtype(&t, P1, "Serf").is_empty());
}

#[test]
fn faceless_butcher_must_target_your_own_creature_if_it_is_the_only_one() {
    cr!("115.1", "603.3d");
    ruling!(
        "Faceless Butcher",
        "Faceless Butcher’s first ability isn’t optional. If no other players control any creatures and you do, you must choose a creature you control as the target."
    );
    supported("Faceless Butcher");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P0 tries to choose no target: the ability still targets the Grizzly Bears.
    t.answer_targets(P0, &[]);
    t.enter(P0, "Faceless Butcher");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn exiled_tokens_dont_return() {
    cr!("111.7", "704.5d", "610.3c");
    ruling!(
        "Faceless Butcher",
        "If Faceless Butcher exiles a token creature, the token will cease to exist after being exiled. It won’t return to the battlefield because of the second ability."
    );
    ruling!("Fiend Hunter", "If a token is exiled this way, it won't return to the battlefield.");
    for (name, may) in [("Faceless Butcher", false), ("Fiend Hunter", true)] {
        let mut t = TestGame::new(2);
        let token = create_token(&mut t, P1, "Goblin");
        t.answer_targets(P0, &[obj(token)]);
        if may {
            t.answer_yes(P0, true);
        }
        let perm = t.enter(P0, name);
        t.resolve_all();
        assert!(with_subtype(&t, P1, "Goblin").is_empty(), "{name}");
        destroy(&mut t, perm);
        t.resolve_all();
        assert!(with_subtype(&t, P1, "Goblin").is_empty(), "{name}");
        assert!(t.g.find_in_zone(Zone::Exile, "Goblin").is_empty(), "{name}");
    }
}

#[test]
fn a_loop_of_faceless_butchers_is_a_draw() {
    cr!("104.4b", "732.5");
    ruling!(
        "Faceless Butcher",
        "If no one can stop the loop, the game will be a draw."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 8);
    let first = t.battlefield(P0, "Faceless Butcher");
    let second = t.hand(P0, "Faceless Butcher");
    t.answer_targets(P0, &[obj(first)]);
    t.cast(P0, second).go();
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Exile);
    // A third one exiles the second, which returns the first, which exiles the third, ...
    let third = t.hand(P0, "Faceless Butcher");
    t.cast(P0, third).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn a_loop_of_oblivion_rings_is_a_draw() {
    cr!("104.4b", "732.5");
    ruling!(
        "Oblivion Ring",
        "casting a third Oblivion Ring will result in an involuntary infinite loop that will end the game in a draw"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let first = t.battlefield(P0, "Oblivion Ring");
    let second = t.hand(P0, "Oblivion Ring");
    t.answer_targets(P0, &[obj(first)]);
    t.cast(P0, second).go();
    t.resolve_all();
    assert_eq!(t.zone(first), Zone::Exile);
    let third = t.hand(P0, "Oblivion Ring");
    t.cast(P0, third).go();
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn oblivion_ring_exile_drops_auras_equipment_and_counters() {
    cr!("400.7", "704.5m", "704.5n", "122.2");
    ruling!(
        "Oblivion Ring",
        "Auras attached to the exiled permanent will be put into their owners’ graveyards. Equipment attached to the exiled permanent will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist."
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let d = decorate(&mut t, P1, ogre);
    t.answer_targets(P0, &[obj(ogre)]);
    t.enter(P0, "Oblivion Ring");
    t.resolve_all();
    assert_eq!(t.zone(ogre), Zone::Exile);
    assert_eq!(t.counters(ogre, "+1/+1"), 0);
    assert_eq!(t.zone(d.aura), Zone::Graveyard(P1));
    assert!(t.on_battlefield(d.equipment));
    assert_eq!(attached_to(&t, d.equipment), None);
}

#[test]
fn an_aura_returned_by_oblivion_ring_is_attached_by_its_owner_without_targeting() {
    cr!("303.4f", "303.4a", "610.3");
    ruling!(
        "Oblivion Ring",
        "If the exiled card is an Aura, that card’s owner chooses what it will enchant as it comes back onto the battlefield. An Aura put onto the battlefield this way doesn’t target anything, but the Aura’s enchant ability restricts what it can be attached to. If the Aura can’t legally be attached to anything, it remains exiled forever."
    );
    supported("Holy Strength");
    supported("Silhana Ledgewalker");
    // Holy Strength (P1's) can return enchanting P0's hexproof Silhana Ledgewalker.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P1, "Holy Strength", bears);
    let silhana = t.battlefield(P0, "Silhana Ledgewalker");
    t.answer_targets(P0, &[obj(aura)]);
    let ring = t.enter(P0, "Oblivion Ring");
    t.resolve_all();
    assert_eq!(t.zone(aura), Zone::Exile);
    let from = t.asked().len();
    t.answer_choose(P1, &[obj(silhana)]);
    destroy(&mut t, ring);
    t.resolve_all();
    assert!(t.on_battlefield(aura));
    assert_eq!(attached_to(&t, aura), Some(obj(silhana)));
    // The Aura's owner chose.
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. })));

    // With nothing it could enchant, it stays in exile.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = attach_new(&mut t, P1, "Holy Strength", bears);
    t.answer_targets(P0, &[obj(aura)]);
    let ring = t.enter(P0, "Oblivion Ring");
    t.resolve_all();
    let bears = t.g.current(bears);
    t.g.destroy(bears, None);
    t.settle();
    destroy(&mut t, ring);
    t.resolve_all();
    assert_eq!(t.zone(aura), Zone::Exile);
}

#[test]
fn fiend_hunters_creature_stays_exiled_when_its_controller_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Fiend Hunter",
        "In a multiplayer game, if you lose the game, the creature exiled with Fiend Hunter remains exiled indefinitely."
    );
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Fiend Hunter");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.players[0].life = 0;
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t.named_on_battlefield("Fiend Hunter").is_empty());
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn wormfang_drake_that_leaves_first_exiles_your_creature_indefinitely() {
    cr!("603.6c", "603.10a", "118.12");
    ruling!(
        "Wormfang Drake",
        "If Wormfang Drake leaves the battlefield before its enters-the-battlefield ability resolves, the “leaves-the-battlefield” ability will trigger and resolve first. It won’t return anything. Then the enters-the-battlefield ability will resolve. If you exile a creature you control, it will be exiled indefinitely. Wormfang Drake won’t return it."
    );
    supported("Wormfang Drake");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    leaves_before_its_etb_resolves(&mut t, "Wormfang Drake", None, false);
    assert_eq!(t.zone(bears), Zone::Exile, "{}", t.dump_log());
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn wormfang_drake_alone_must_be_sacrificed() {
    cr!("118.12");
    ruling!(
        "Wormfang Drake",
        "If you don’t control any other creatures as the enters-the-battlefield ability resolves, you must sacrifice Wormfang Drake."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let drake = t.enter(P0, "Wormfang Drake");
    t.resolve_all();
    assert!(!t.on_battlefield(drake));
    assert!(t.in_graveyard(P0, "Wormfang Drake"));
    assert!(!t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn wormfang_drakes_creature_is_chosen_as_the_ability_resolves() {
    cr!("118.12", "608.2c");
    ruling!(
        "Wormfang Drake",
        "You choose which creature you control to exile (if any) as the enters-the-battlefield ability resolves."
    );
    let mut t = TestGame::new(2);
    let drake = t.enter(P0, "Wormfang Drake");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A creature that arrives while the ability is on the stack can be chosen.
    let late = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(late)]);
    t.resolve_all();
    assert_eq!(t.zone(late), Zone::Exile);
    assert!(t.on_battlefield(drake));
}

#[test]
fn thragtusk_leaves_trigger_works_for_any_zone() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Thragtusk",
        "Thragtusk's second ability will trigger no matter what zone Thragtusk goes to."
    );
    supported("Thragtusk");
    for to in [Zone::Hand(P0), Zone::Exile, Zone::Library(P0), Zone::Graveyard(P0)] {
        let mut t = TestGame::new(2);
        let tusk = t.battlefield(P0, "Thragtusk");
        move_to(&mut t, tusk, to);
        t.settle();
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Beast").len(), 1, "{to:?}");
    }
}

#[test]
fn reveillark_targets_up_to_two_creature_cards_with_power_2_or_less() {
    cr!("115.1", "601.2c", "603.3d");
    ruling!(
        "Reveillark",
        "Reveillark's ability may target zero, one, or two creature cards in your graveyard. Each target must have power 2 or less."
    );
    supported("Reveillark");
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let lions = t.graveyard(P0, "Savannah Lions");
        let giant = t.graveyard(P0, "Hill Giant");
        let lark = t.battlefield(P0, "Reveillark");
        let chosen: Vec<Entity> = [obj(bears), obj(lions)][..n].to_vec();
        t.answer_targets(P0, &chosen);
        let from = t.asked().len();
        destroy(&mut t, lark);
        let cands = target_candidates(&t, P0, from);
        assert!(!cands.is_empty());
        assert!(cands.iter().all(|c| !c.contains(&obj(giant))), "power 3");
        assert!(cands[0].contains(&obj(bears)) && cands[0].contains(&obj(lions)));
        t.resolve_all();
        let back = [bears, lions].iter().filter(|&&c| t.on_battlefield(c)).count();
        assert_eq!(back, n);
        assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    }
}
