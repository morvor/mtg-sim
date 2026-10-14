//! Rulings batch P044 — more death and graveyard triggers: last known information
//! (CR 608.2h), objects that changed zones (CR 400.7), targets chosen as the ability is
//! put on the stack (CR 601.2c-d, 603.3d) or not at all (CR 603.3d), and exchanges of
//! control (CR 701.12b).

use crate::r_p044_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts Archfiend of the Dross onto the battlefield for P0 with `oil` oil counters and
/// advances to P0's next upkeep with its trigger on the stack.
fn archfiend_upkeep(oil: u32) -> (TestGame, ObjectId) {
    supported("Archfiend of the Dross");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Archfiend of the Dross");
    let have = t.counters(a, "oil");
    t.g.remove_counters(Entity::Object(a), "oil", have);
    t.g.add_counters(Entity::Object(a), "oil", oil, None);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    (t, a)
}

#[test]
fn archfiend_uses_its_last_known_oil_counters() {
    cr!("608.2h", "122.1");
    ruling!(
        "Archfiend of the Dross",
        "If Archfiend of the Dross is no longer on the battlefield as its upkeep triggered ability resolves, use the number of oil counters it had the last time it existed on the battlefield to determine whether you lose the game."
    );
    ruling!(
        "Archfiend of the Dross",
        "If Archfiend of the Dross somehow loses all of its oil counters due to an effect other than its upkeep triggered ability, this won't cause you to lose the game until the next time that ability resolves."
    );
    // One oil counter, and it leaves in response: no counter removed, no loss.
    let (mut t, a) = archfiend_upkeep(1);
    assert_eq!(triggers_on_stack(&t, "oil"), 1);
    kill(&mut t, a);
    t.resolve_all();
    assert!(!t.has_lost(P0));
    // No oil counters, and it leaves in response: it had none, so P0 loses.
    let (mut t, a) = archfiend_upkeep(0);
    kill(&mut t, a);
    t.resolve_all();
    assert!(t.has_lost(P0));
    // Losing the counters some other way: nothing until the upkeep ability resolves.
    let (mut t, a) = archfiend_upkeep(2);
    t.resolve_all();
    assert_eq!(t.counters(a, "oil"), 1);
    t.g.remove_counters(Entity::Object(a), "oil", 1);
    t.settle();
    assert!(!t.has_lost(P0));
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.has_lost(P0));
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P0));
}

#[test]
fn meathook_massacre_not_cast_has_x_of_zero() {
    cr!("107.3m", "603.2");
    ruling!(
        "The Meathook Massacre",
        "If The Meathook Massacre enters the battlefield and it wasn't cast (or it was cast without paying its mana cost), X will be 0. The first ability will trigger, but it won't be much of a massacre."
    );
    supported("The Meathook Massacre");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.enter(P0, "The Meathook Massacre");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn prince_of_thralls_returns_unless_that_opponent_pays() {
    cr!("608.2c", "118.12", "603.10a");
    ruling!(
        "Prince of Thralls",
        "If the ability triggers and your opponent chooses not to pay 3 life, you must return the card to the battlefield, even if you don’t want to."
    );
    ruling!(
        "Prince of Thralls",
        "It doesn’t matter whose graveyard the permanent is put into as long as an opponent controlled it when it last existed on the battlefield."
    );
    ruling!(
        "Prince of Thralls",
        "If the permanent that caused the ability to trigger leaves the graveyard before the ability resolves"
    );
    supported("Prince of Thralls");
    // P1 doesn't pay: the Bears return under P0's control; P0 isn't asked.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prince of Thralls");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P1, false);
    let from = t.asked().len();
    kill(&mut t, bears);
    t.resolve_all();
    let back = named(&t, P0, "Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).owner, P1);
    assert!(asked_since(&t, from)
        .iter()
        .all(|(p, d)| *p != P0 || !matches!(d, Decision::YesNo { .. })));
    // P1 controlled a creature P0 owns: it goes to P0's graveyard, and it still returns.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prince of Thralls");
    let mine = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    cast_spell(&mut t, P1, "Act of Treason", &[Entity::Object(mine)]);
    assert_eq!(t.obj_now(mine).controller, P1);
    t.answer_yes(P1, false);
    kill(&mut t, mine);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert_eq!(named(&t, P0, "Hill Giant").len(), 1);
    // The card leaves the graveyard first: P1 may still pay; nothing returns either way.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Prince of Thralls");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    kill(&mut t, bears);
    let card = t.g.current(bears);
    t.g.move_object(card, Zone::Exile, MoveCause::Effect, None);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn balrog_goes_to_the_bottom_only_from_the_battlefield() {
    cr!("400.7", "608.2b");
    ruling!(
        "The Balrog, Flame of Udûn",
        "The Balrog, Flame of Udûn will be put on the bottom of its owner's library only if it's still on the battlefield when the ability resolves."
    );
    supported("The Balrog, Flame of Udûn");
    supported("Isamaru, Hound of Konda");
    // Still there: bottom of the library.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Balrog, Flame of Udûn");
    let legend = t.battlefield(P1, "Isamaru, Hound of Konda");
    kill(&mut t, legend);
    t.resolve_all();
    let balrog = t.g.player(P0).library.iter().copied().find(|o| {
        t.g.obj(*o).chars.name.as_str() == "The Balrog, Flame of Udûn"
    });
    let balrog = balrog.expect("in the library");
    let n = t.library_size(P0) as u32;
    assert_eq!(mtg_engine::library::top_cards(&t.g, P0, n).last(), Some(&balrog));
    // Bounced in response: it stays in the hand.
    let mut t = TestGame::new(2);
    let balrog = t.battlefield(P0, "The Balrog, Flame of Udûn");
    let legend = t.battlefield(P1, "Isamaru, Hound of Konda");
    kill(&mut t, legend);
    t.g.move_object(balrog, Zone::Hand(P0), MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "The Balrog, Flame of Udûn"));
    // Flickered in response: the new Balrog stays on the battlefield.
    let mut t = TestGame::new(2);
    let balrog = t.battlefield(P0, "The Balrog, Flame of Udûn");
    let legend = t.battlefield(P1, "Isamaru, Hound of Konda");
    kill(&mut t, legend);
    let ex =
        t.g.move_object(balrog, Zone::Exile, MoveCause::Effect, None)
            .unwrap();
    t.g.move_object(ex, Zone::Battlefield, MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("The Balrog, Flame of Udûn").len(), 1);
}

#[test]
fn noxious_dragon_target_turned_face_up_becomes_illegal() {
    cr!("708.2", "608.2b", "702.37e");
    ruling!(
        "Noxious Dragon",
        "A face-down creature has a mana value of 0. However, its mana value changes when it's turned face up."
    );
    supported("Noxious Dragon");
    supported("Scornful Egotist");
    // P1's face-down creature is Scornful Egotist (morph {U}), cast face down.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Noxious Dragon");
    t.lands(P1, "Island", 1);
    let egotist = t.hand(P1, "Scornful Egotist");
    t.lands(P1, "Wastes", 3);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, egotist)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let fd = t.g.current(egotist);
    assert!(t.g.obj(fd).face_down);
    assert_eq!(t.g.obj(fd).chars.mana_value(), 0);
    // The Dragon dies; P0 targets the face-down creature (mana value 0).
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(fd)]);
    kill(&mut t, dragon);
    assert_eq!(t.stack_len(), 1);
    // P1 turns it face up in response ({U}): Scornful Egotist has mana value 8.
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    t.g.perform_action(
        P1,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: fd,
        }),
    )
    .unwrap();
    assert_eq!(t.obj_now(fd).chars.mana_value(), 8);
    t.resolve_all();
    assert!(t.on_battlefield(fd));
}

#[test]
fn burning_eye_zubera_counts_damage_from_all_sources() {
    cr!("603.4", "603.10a", "120.3");
    ruling!(
        "Burning-Eye Zubera",
        "Burning-Eye Zubera’s ability counts all damage dealt to it this turn, from all sources."
    );
    supported("Burning-Eye Zubera");
    // 2 + 2 from two sources (it survives as a 3/3? no: 4 damage kills it).
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Burning-Eye Zubera");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(a, Entity::Object(z), 2, false);
    t.settle();
    t.g.deal_damage(b, Entity::Object(z), 2, false);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Burning-Eye Zubera"));
    assert_eq!(t.life(P1), 17);
    // Only 3 damage, then destroyed: no trigger.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Burning-Eye Zubera");
    let a = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(a, Entity::Object(z), 2, false);
    t.settle();
    t.g.deal_damage(a, Entity::Object(z), 1, false);
    t.settle();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kill(&mut t, z);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn workshop_assistant_can_return_an_artifact_that_died_with_it() {
    cr!("603.10a", "603.3d");
    ruling!(
        "Workshop Assistant",
        "If another artifact card is put into your graveyard at the same time as Workshop Assistant, you can target it with Workshop Assistant's triggered ability."
    );
    supported("Workshop Assistant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Workshop Assistant");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    let both: Vec<ObjectId> = [t.g.current(thopter)]
        .into_iter()
        .chain(named(&t, P0, "Workshop Assistant"))
        .collect();
    t.g.destroy_all(both, None, false);
    t.settle();
    let card = t.g.current(thopter);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    t.answer_targets(P0, &[Entity::Object(card)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));
}

#[test]
fn jugan_divides_its_counters_as_the_ability_is_put_on_the_stack() {
    cr!("601.2d", "603.3d", "608.2b");
    ruling!(
        "Jugan, the Rising Star",
        "You choose how many targets Jugan's triggered ability has and how the counters are distributed as you put the ability onto the stack."
    );
    ruling!(
        "Jugan, the Rising Star",
        "If some of the creatures are illegal targets as Jugan's triggered ability tries to resolve, the original distribution of counters still applies and the counters that would have been put on the illegal targets are lost."
    );
    supported("Jugan, the Rising Star");
    let mut t = TestGame::new(2);
    let jugan = t.battlefield(P0, "Jugan, the Rising Star");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 3]));
    kill(&mut t, jugan);
    assert_eq!(t.stack_len(), 1);
    // A division of zero to a target isn't allowed: the decision asked for each target to
    // get at least one.
    let divide = t
        .asked()
        .into_iter()
        .rev()
        .find(|(_, d)| matches!(d, Decision::Divide { .. }))
        .map(|(_, d)| d);
    if let Some(Decision::Divide { min_each, .. }) = divide {
        assert_eq!(min_each, 1);
    }
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
}

#[test]
fn retributive_wand_has_no_way_to_sacrifice_itself() {
    cr!("602.1", "603.6c");
    ruling!(
        "Retributive Wand",
        "Retributive Wand's last ability doesn't allow you to sacrifice it whenever you wish."
    );
    supported("Retributive Wand");
    let mut t = TestGame::new(2);
    let wand = t.battlefield(P0, "Retributive Wand");
    t.lands(P0, "Wastes", 3);
    // Its only activated ability is the {3}, {T} one.
    let activated = t
        .obj_now(wand)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(activated, 1);
    // Broken by another effect: 5 damage.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    kill(&mut t, wand);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn cinderheart_giant_chooses_without_targeting() {
    cr!("115.10", "702.16e", "702.11b");
    ruling!(
        "Cinderheart Giant",
        "The creature an opponent controls isn’t targeted, so it’s chosen as the triggered ability resolves."
    );
    supported("Cinderheart Giant");
    supported("Gladecover Scout");
    supported("Kor Firewalker");
    for (name, dies) in [("Gladecover Scout", true), ("Kor Firewalker", false)] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Cinderheart Giant");
        let c = t.battlefield(P1, name);
        kill(&mut t, giant);
        assert_eq!(t.stack_len(), 1, "{name}: it doesn't need a legal target");
        t.resolve_all();
        assert_eq!(!t.on_battlefield(c), dies, "{name}");
        if !dies {
            assert_eq!(t.obj_now(c).damage, 0);
        }
    }
}

/// P0 controls Conjured Currency; P0's upkeep trigger targets `target` and resolves.
fn currency_upkeep(t: &mut TestGame, cc: ObjectId, target: ObjectId) {
    let _ = cc;
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
}

#[test]
fn conjured_currency_exchanges_with_a_permanent_you_neither_own_nor_control() {
    cr!("701.12a", "701.12b", "115.1");
    ruling!(
        "Conjured Currency",
        "The only legal target for Conjured Currency’s ability is a permanent both owned and controlled by another player, although it doesn’t have to be the same player."
    );
    ruling!(
        "Conjured Currency",
        "The exchange occurs only if Conjured Currency and the target permanent are both on the battlefield and controlled by different players when the ability resolves."
    );
    supported("Conjured Currency");
    // Plain exchange with P1's Bears.
    let mut t = TestGame::new(3);
    let cc = t.battlefield(P0, "Conjured Currency");
    let bears = t.battlefield(P1, "Grizzly Bears");
    currency_upkeep(&mut t, cc, bears);
    assert_eq!(t.obj_now(cc).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P0);
    // Owned by P1, controlled by P2: a legal target; P0's own Giant (controlled by P2)
    // isn't.
    let mut t = TestGame::new(3);
    let cc = t.battlefield(P0, "Conjured Currency");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.set_step(P2, Step::PrecombatMain);
    cast_spell(&mut t, P2, "Act of Treason", &[Entity::Object(theirs)]);
    give_mana_for(&mut t, P2, "Act of Treason");
    let a = t.hand(P2, "Act of Treason");
    t.cast(P2, a).target(mine).go();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).controller, P2);
    // (Act of Treason ends at P2's cleanup; check legality now.)
    let spec = mtg_engine::card::card("Conjured Currency");
    let ab = &spec.front().chars.abilities[0];
    let mtg_engine::ability::AbilityKind::Triggered(tr) = &ab.kind else {
        panic!()
    };
    let ts = &tr.body.targets[0];
    let ctx = mtg_engine::eval::Ctx::new(Some(cc), P0);
    assert!(t.g.is_legal_target(ts, Entity::Object(theirs), &ctx, cc));
    assert!(!t.g.is_legal_target(ts, Entity::Object(mine), &ctx, cc));
    assert!(!t.g.is_legal_target(ts, Entity::Object(cc), &ctx, cc));
    // The target comes under P0's control before resolution: no exchange.
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Conjured Currency");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_spell(&mut t, P0, "Ray of Command", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(cc).controller, P0);
}

#[test]
fn conjured_currency_exchange_between_two_other_players() {
    cr!("701.12a", "608.2b");
    ruling!(
        "Conjured Currency",
        "You don’t necessarily have to control Conjured Currency when its ability resolves in order for the exchange to happen."
    );
    // P0's trigger targets P2's Bears; P1 gains control of Conjured Currency in response.
    let mut t = TestGame::new(3);
    let cc = t.battlefield(P0, "Conjured Currency");
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A permanent control change effect for P1 (as a response would create).
    let e = mtg_engine::ability::Effect::GainControl {
        what: mtg_engine::ability::Sel::This,
        who: mtg_engine::ability::PlayerRef::You,
        duration: mtg_engine::ability::Duration::Permanent,
    };
    let mut ctx = mtg_engine::eval::Ctx::new(Some(cc), P1);
    t.g.exec(&e, &mut ctx);
    t.g.recompute();
    assert_eq!(t.obj_now(cc).controller, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(cc).controller, P2);
    assert_eq!(t.obj_now(bears).controller, P1);
}
