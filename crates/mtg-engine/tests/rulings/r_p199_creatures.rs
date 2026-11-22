//! Rulings batch P199 — creatures with attack, death, cast and enters triggers: Mijae
//! Djinn's coin flip (CR 506.4), Pashalik Mons's death triggers (CR 603.10a), Erhnam
//! Djinn's upkeep target (CR 603.3d), Phelia's delayed return (CR 603.7), Tura
//! Kennerüd's cast trigger, divided counters (CR 601.2d, 608.2b), Glint Hawk's "unless",
//! and Arna Kennerüd's attached token copies (CR 303.4f).

use crate::r_p194_common::mana;
use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::attached_to;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn mijae_djinn_losing_the_flip_after_it_attacked() {
    cr!("506.4", "508.1m", "705.2");
    ruling!(
        "Mijae Djinn",
        "The coin flip is done as a triggered ability on declaring the Djinn as an attacker."
    );
    ruling!(
        "Mijae Djinn",
        "Any abilities which have already resolved before the coin flip are not undone. Any abilities that trigger when it attacks which have not already resolved will still resolve."
    );
    ruling!(
        "Mijae Djinn",
        "If the Djinn attacks, but you lose the coin toss, then it is still considered to have attacked, and any other abilities that trigger on it attacking will resolve normally."
    );
    supported("Gleam of Battle");
    supported("Campaign of Vengeance");
    // Both orders of the flip and Gleam of Battle's trigger.
    for order in [vec![0, 1, 2], vec![2, 1, 0]] {
        let mut t = TestGame::new(2);
        let djinn = t.battlefield(P0, "Mijae Djinn");
        t.battlefield(P0, "Gleam of Battle");
        t.battlefield(P0, "Campaign of Vengeance");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        attack_with(&mut t, &[(djinn, Entity::Player(P1))]);
        // Three attack triggers: the flip is one of them.
        assert_eq!(t.stack_len(), 3);
        assert_eq!(triggers_on_stack(&t, "flip a coin"), 1);
        // Call heads; it comes up tails: lost.
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        t.g.dice.loaded_coins.push_back(false);
        t.resolve_all();
        assert!(t.obj_now(djinn).tapped);
        assert_eq!(t.counters(djinn, counters::PLUS1), 1, "{order:?}");
        assert_eq!((t.life(P0), t.life(P1)), (21, 19), "{order:?}");
        block_and_finish(&mut t, P1, &[]);
        assert_eq!(t.life(P1), 19);
        // It still attacked this turn.
        assert!(t.g.turn_events.iter().any(|e| matches!(
            e,
            mtg_engine::events::Event::AttackersDeclared { attackers, .. } if attackers.iter().any(|(a, _)| *a == djinn)
        )));
    }
}

#[test]
fn pashalik_mons_triggers_for_each_goblin_dying_with_it() {
    cr!("603.10a", "603.2");
    ruling!(
        "Pashalik Mons",
        "If Pashalik Mons and one or more other Goblins you control die at the same time, its ability will trigger for each of those creatures."
    );
    supported("Day of Judgment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pashalik Mons");
    t.battlefield(P0, "Goblin Piker");
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Grizzly Bears");
    let wrath = t.hand(P0, "Day of Judgment");
    mana(&mut t, P0, ManaType::W, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, wrath).go();
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    t.resolve();
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn pashalik_mons_can_be_sacrificed_to_its_own_ability() {
    cr!("602.2b", "603.10a");
    ruling!(
        "Pashalik Mons",
        "You can sacrifice Pashalik Mons to pay the cost of its last ability."
    );
    let mut t = TestGame::new(2);
    let mons = t.battlefield(P0, "Pashalik Mons");
    t.lands(P0, "Mountain", 4);
    t.answer_choose(P0, &[obj(mons)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, mons, 0, &[]).unwrap();
    assert_eq!(t.zone(mons), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(tokens_with("Goblin", &t), 2);
    // It saw itself die.
    assert_eq!(t.life(P1), 19);
}

fn tokens_with(sub: &str, t: &TestGame) -> usize {
    tokens(t, P0)
        .into_iter()
        .filter(|id| t.g.obj(*id).chars.has_subtype(sub))
        .count()
}

#[test]
fn erhnam_djinn_with_no_target_does_nothing() {
    cr!("603.3d");
    ruling!(
        "Erhnam Djinn",
        "If there are no creatures your opponent controls to target at the beginning of your upkeep, ignore this ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erhnam Djinn");
    t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn erhnam_djinns_may_target_the_same_creature() {
    cr!("115.3", "603.3d");
    ruling!(
        "Erhnam Djinn",
        "If you have more than one Djinn, you can have all of them target the same creature with their ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erhnam Djinn");
    t.battlefield(P0, "Erhnam Djinn");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    // Both abilities target the Bears (neither answer was refused for the Hill Giant).
    for id in t.g.stack.clone() {
        let chosen = &t.g.obj(id).stack.as_ref().unwrap().chosen;
        let targets: Vec<&Entity> = chosen
            .iter()
            .flat_map(|c| c.targets.iter().flatten())
            .collect();
        assert_eq!(targets, vec![&obj(bears)]);
    }
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Landwalk));
    assert!(!t.obj_now(giant).has_keyword(KeywordKind::Landwalk));
}

#[test]
fn erhnam_djinn_targets_any_opponents_creature_in_multiplayer() {
    cr!("603.3d", "115.1");
    ruling!(
        "Erhnam Djinn",
        "In multiplayer games you can choose a different player's creature each upkeep. You are forced to pick a creature that some opponent controls if there is at least one creature on the battlefield that is a legal target."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Erhnam Djinn");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    let wall = t.battlefield(P2, "Wall of Wood");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(b)]);
    next_upkeep(&mut t, P0);
    let cands = target_candidates(&t, P0, from);
    let c = cands.last().expect("target asked");
    assert!(c.contains(&obj(a)) && c.contains(&obj(b)));
    assert!(!c.contains(&obj(mine)) && !c.contains(&obj(wall)));
    t.resolve_all();
    assert!(t.obj_now(b).has_keyword(KeywordKind::Landwalk));
    assert!(!t.obj_now(a).has_keyword(KeywordKind::Landwalk));
}

#[test]
fn phelia_returns_the_card_even_if_she_is_gone() {
    cr!("603.7", "603.7c");
    ruling!(
        "Phelia, Exuberant Shepherd",
        "The exiled card will return to the battlefield at the beginning of the end step even if Phelia is no longer on the battlefield."
    );
    let mut t = TestGame::new(2);
    let phelia = t.battlefield(P0, "Phelia, Exuberant Shepherd");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(phelia, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    move_to(&mut t, phelia, Zone::Graveyard(P0));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn tura_kennerud_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Tura Kennerüd, Skyknight",
        "Tura’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tura Kennerüd, Skyknight");
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // The trigger is on top.
    assert_ne!(top_of_stack(&t), spell);
    t.resolve();
    assert_eq!(tokens_with("Soldier", &t), 1);
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A second spell, countered in response: its trigger still makes a Soldier.
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    let cancel = t.hand(P1, "Cancel");
    mana(&mut t, P1, ManaType::U, 2);
    mana(&mut t, P1, ManaType::C, 1);
    t.cast(P1, cancel).target(spell).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with("Soldier", &t), 2);
    // Only the first Shock dealt damage.
    assert_eq!(t.life(P1), 18);
}

fn divide(t: &mut TestGame, amounts: &[i64]) {
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

#[test]
fn distributed_counters_are_divided_as_the_trigger_is_put_on_the_stack() {
    cr!("601.2d", "603.3d", "115.1");
    ruling!(
        "Armament Dragon",
        "You choose how many targets Armament Dragon’s triggered ability has and how the counters are distributed as you put the ability onto the stack. Each target must receive at least one counter."
    );
    ruling!(
        "Verdurous Gearhulk",
        "You choose how the counters will be distributed as you put Verdurous Gearhulk's triggered ability onto the stack. Each target creature must be assigned at least one counter."
    );
    ruling!(
        "Verdurous Gearhulk",
        "Verdurous Gearhulk can be the target of its own triggered ability."
    );
    for (name, total) in [("Armament Dragon", 3u32), ("Verdurous Gearhulk", 4)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        // The new creature itself and the Bears.
        let new_id = ObjectId(t.g.objects.len() as u32 + 1);
        t.answer_targets(P0, &[obj(new_id), obj(bears)]);
        divide(&mut t, &[total as i64 - 1, 1]);
        let id = t.enter(P0, name);
        assert_eq!(id, new_id, "{name}");
        t.settle();
        let divided: Vec<_> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::Divide {
                    total, min_each, ..
                } => Some((*total, *min_each)),
                _ => None,
            })
            .collect();
        assert_eq!(divided, vec![(total, 1)], "{name}");
        assert_eq!(t.counters(id, counters::PLUS1), 0);
        t.resolve_all();
        assert_eq!(t.counters(id, counters::PLUS1), total - 1, "{name}");
        assert_eq!(t.counters(bears, counters::PLUS1), 1, "{name}");
    }
}

#[test]
fn armament_dragon_counters_for_illegal_targets_are_lost() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Armament Dragon",
        "If some of the creatures are illegal targets as Armament Dragon’s triggered ability resolves, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    divide(&mut t, &[2, 1]);
    t.enter(P0, "Armament Dragon");
    t.settle();
    move_to(&mut t, a, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn iron_bully_can_target_itself() {
    cr!("603.3d", "115.1");
    ruling!(
        "Iron Bully",
        "Iron Bully can be the target of its own ability."
    );
    let mut t = TestGame::new(2);
    let new_id = ObjectId(t.g.objects.len() as u32 + 1);
    t.answer_targets(P0, &[obj(new_id)]);
    let id = t.enter(P0, "Iron Bully");
    assert_eq!(id, new_id);
    t.resolve_all();
    assert_eq!(t.counters(id, counters::PLUS1), 1);
}

#[test]
fn glint_hawk_chooses_an_artifact_or_sacrifice_on_resolution() {
    cr!("118.12", "608.2c");
    ruling!(
        "Glint Hawk",
        "You choose which artifact to return to its owner’s hand as the triggered ability resolves. If you control no artifacts at that time, you must sacrifice Glint Hawk."
    );
    ruling!(
        "Glint Hawk",
        "You may choose to sacrifice Glint Hawk as its triggered ability resolves even if you control an artifact."
    );
    // An artifact arriving after the trigger can be returned.
    let mut t = TestGame::new(2);
    let hawk = t.enter(P0, "Glint Hawk");
    t.settle();
    let stone = t.battlefield(P0, "Mind Stone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stone)]);
    t.resolve_all();
    assert!(t.on_battlefield(hawk));
    assert_eq!(t.zone(stone), Zone::Hand(P0));
    // An artifact gone before it resolves: it's sacrificed.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let hawk = t.enter(P0, "Glint Hawk");
    t.settle();
    move_to(&mut t, stone, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(!t.on_battlefield(hawk));
    // Declining with an artifact: sacrificed, the artifact stays.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let hawk = t.enter(P0, "Glint Hawk");
    t.settle();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.on_battlefield(hawk));
    assert!(t.on_battlefield(stone));
}

#[test]
fn arna_kennerud_aura_token_that_cant_attach_isnt_created() {
    cr!("303.4i", "702.5a", "301.5e");
    ruling!(
        "Arna Kennerüd, Skycaptain",
        "If, for any reason, an Aura token that would be created with Arna Kennerüd's last ability can't be attached to the appropriate creature, that token isn't created."
    );
    supported("Anti-Magic Aura");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arna Kennerüd, Skycaptain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Anti-Magic Aura: the Bears are modified, and can't be enchanted by other Auras.
    let aura = t.battlefield(P0, "Anti-Magic Aura");
    assert!(t.g.attach(aura, obj(bears)));
    t.g.add_counters(obj(bears), counters::PLUS1, 1, None);
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    // Without the restriction, the Aura (Rancor) is copied attached to it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arna Kennerüd, Skycaptain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rancor = t.battlefield(P0, "Rancor");
    assert!(t.g.attach(rancor, obj(bears)));
    t.g.recompute();
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(attached_to(&t, toks[0]), Some(obj(bears)));
    // An Equipment token that can't be attached is created unattached: Gate Smasher ("can
    // be attached only to a creature with toughness 4 or greater") on an Earth Elemental
    // with a -1/-1 counter (3/4). Doubling its counters first leaves it 3 toughness.
    supported("Gate Smasher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arna Kennerüd, Skycaptain");
    let elemental = t.battlefield(P0, "Earth Elemental");
    t.g.add_counters(obj(elemental), counters::MINUS1, 1, None);
    let smasher = t.battlefield(P0, "Gate Smasher");
    assert!(t.g.attach(smasher, obj(elemental)));
    t.g.recompute();
    assert_eq!(t.pt(elemental), (6, 4));
    attack_with(&mut t, &[(elemental, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(elemental, counters::MINUS1), 2);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.g.obj(toks[0]).chars.name, "Gate Smasher");
    assert_eq!(attached_to(&t, toks[0]), None);
}
