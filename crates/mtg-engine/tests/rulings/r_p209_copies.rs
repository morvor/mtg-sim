//! Rulings batch P209 — enchant (CR 303.4, 702.5): Splinter Twin's token copies of the
//! enchanted creature, Auras that make permanents legendary, and Precipitous Drop's
//! dungeon condition.

use crate::r_s01_common::*;
use crate::r_s06_common::*;
use mtg_engine::dungeons::venture;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's `creature` enchanted by Splinter Twin activates its granted ability; `respond`
/// runs while it's on the stack. Returns the token created.
fn twin(
    t: &mut TestGame,
    creature: ObjectId,
    respond: impl FnOnce(&mut TestGame),
) -> ObjectId {
    supported("Splinter Twin");
    attach_new(t, P0, "Splinter Twin", creature);
    t.set_step(P0, Step::PrecombatMain);
    // It's been under P0's control since the turn began.
    let c = t.g.current(creature);
    t.g.objects[c.0 as usize].summoning_sick = false;
    activate_containing(t, P0, creature, "Create a token").unwrap();
    t.settle();
    respond(t);
    let before = tokens(t, P0);
    t.resolve_all();
    let new: Vec<_> = tokens(t, P0)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect();
    assert_eq!(new.len(), 1, "one token");
    new[0]
}

#[test]
fn splinter_twin_copy_of_an_x_creature_has_x_zero() {
    cr!("107.3m", "707.2", "202.3");
    ruling!(
        "Splinter Twin",
        "If the enchanted creature has {X} in its mana cost (such as Protean Hydra), X is considered to be zero."
    );
    supported("Jadelight Spelunker");
    let mut t = TestGame::new(2);
    let j = t.battlefield(P0, "Jadelight Spelunker");
    let token = twin(&mut t, j, |_| {});
    assert_eq!(t.obj_now(token).chars.name, "Jadelight Spelunker");
    assert_eq!(t.obj_now(token).chars.mana_value(), 1);
    // Its enters ability explored X = 0 times: no counter.
    assert_eq!(t.counters(token, "+1/+1"), 0);
}

#[test]
fn splinter_twin_copy_of_a_token_uses_its_original_characteristics() {
    cr!("707.2", "111.3", "707.9a");
    ruling!(
        "Splinter Twin",
        "If the enchanted creature is a token, the new token copies the characteristics of the original token as stated by the effect that put it onto the battlefield, plus it has haste."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let before = tokens(&t, P0);
    crate::r_p209_common::cast_spell(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let soldier = tokens(&t, P0)
        .into_iter()
        .find(|x| !before.contains(x))
        .unwrap();
    let token = twin(&mut t, soldier, |_| {});
    let c = &t.obj_now(token).chars;
    assert!(c.has_subtype("Soldier"));
    assert_eq!(t.pt(token), (1, 1));
    assert!(has_kw(&t, token, KeywordKind::Haste));
}

#[test]
fn splinter_twin_copies_what_the_creature_is_copying() {
    cr!("707.3", "707.9a");
    ruling!(
        "Splinter Twin",
        "If the enchanted creature is copying something else when the ability resolves (for example, if it's a Renegade Doppelganger), then the token enters as a copy of whatever the enchanted creature is copying, plus it has haste."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let c = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Clone");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.cast(P0, c).go();
    t.resolve_all();
    let clone = t.g.current(c);
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    assert_eq!(t.obj_now(clone).controller, P0);
    let token = twin(&mut t, clone, |_| {});
    assert_eq!(t.obj_now(token).chars.name, "Hill Giant");
    assert_eq!(t.pt(token), (3, 3));
    assert!(has_kw(&t, token, KeywordKind::Haste));
}

#[test]
fn splinter_twin_creature_gone_still_makes_a_token_from_last_known_information() {
    cr!("608.2h", "707.2", "113.7a");
    ruling!(
        "Splinter Twin",
        "If you activate the ability and the enchanted creature leaves the battlefield before the ability resolves, you still get a token."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let token = twin(&mut t, bears, |t| {
        crate::r_s05_common::move_to(t, bears, Zone::Hand(P0));
    });
    assert_eq!(t.obj_now(token).chars.name, "Grizzly Bears");
    assert!(has_kw(&t, token, KeywordKind::Haste));
}

#[test]
fn splinter_twin_legendary_copy_triggers_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Splinter Twin",
        "If the enchanted creature is legendary, you will have to put either the original creature or the token into the graveyard as a state-based action."
    );
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    supported("Splinter Twin");
    attach_new(&mut t, P0, "Splinter Twin", isamaru);
    t.set_step(P0, Step::PrecombatMain);
    activate_containing(&mut t, P0, isamaru, "Create a token").unwrap();
    t.resolve_all();
    let all = t.named_on_battlefield("Isamaru, Hound of Konda");
    assert_eq!(all.len(), 1);
}

/// P0 controls two `aura`s attached to two Grizzly Bears (P1's if `steal`, else P0's), and
/// chooses to keep the second Bears and the second Aura.
fn two_legendary_auras(aura: &str, steal: bool) {
    supported(aura);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let owner = if steal { P1 } else { P0 };
    let b1 = t.battlefield(owner, "Grizzly Bears");
    let b2 = t.battlefield(owner, "Grizzly Bears");
    let a1 = attach_new(&mut t, P0, aura, b1);
    // Only one Aura so far: no legend rule.
    t.settle();
    assert!(t.on_battlefield(a1));
    let a2 = t.battlefield(P0, aura);
    t.answer_choose(P0, &[Entity::Object(b2)]);
    t.answer_choose(P0, &[Entity::Object(a2)]);
    assert!(t.g.attach(a2, Entity::Object(b2)));
    t.g.recompute();
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(b2), "{aura}");
    assert!(!t.on_battlefield(b1), "{aura}");
    assert!(t.on_battlefield(a2), "{aura}");
    assert!(!t.on_battlefield(a1), "{aura}");
    assert_eq!(attached_to(&t, a2), Some(Entity::Object(b2)));
    assert_eq!(t.obj_now(b2).controller, P0);
}

#[test]
fn in_bolas_clutches_two_of_them_legend_rule_applies_to_all_at_once() {
    cr!("704.5j", "704.3");
    ruling!(
        "In Bolas's Clutches",
        "If you control two In Bolas’s Clutches attached to two permanents with the same name, the “legend rule” applies to the enchanted permanents and to In Bolas’s Clutches at once."
    );
    two_legendary_auras("In Bolas's Clutches", true);
}

#[test]
fn on_serras_wings_two_of_them_legend_rule_applies_to_all_at_once() {
    cr!("704.5j", "704.3");
    ruling!(
        "On Serra's Wings",
        "If you control two On Serra's Wings attached to two creatures you control with the same name, the \"legend rule\" applies to the enchanted creatures and to On Serra's Wings at once."
    );
    two_legendary_auras("On Serra's Wings", false);
}

#[test]
fn precipitous_drop_counts_a_dungeon_completed_without_it() {
    cr!("309.7", "611.3a");
    ruling!(
        "Precipitous Drop",
        "Precipitous Drop does not need to be on the battlefield when you complete the dungeon in order for the enchanted creature to get -5/-5."
    );
    supported("Precipitous Drop");
    let mut t = TestGame::new(2);
    let mut n = 0;
    while t.g.player(P0).dungeons_completed == 0 {
        n += 1;
        assert!(n < 20, "no dungeon completed");
        venture(&mut t.g, P0, None);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
    }
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    attach_new(&mut t, P0, "Precipitous Drop", maw);
    assert_eq!(t.pt(maw), (1, 1));
}
