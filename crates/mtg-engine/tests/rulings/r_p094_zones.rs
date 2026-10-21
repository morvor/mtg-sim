//! Rulings batch P094 — zone changes and replacement effects around life payments:
//! Meathook Massacre II and Dawn of the Dead (new objects), Underrealm Lich (draws one at
//! a time), Mornsong Aria, Purgatory (control changes, simultaneous
//! destruction), Nameless Race, Spatial Binding, Blood Operative and Lorcan.

use crate::r_s01_common::{give_mana_for, stack_library, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, give_control};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Casts the real card `name` for `p` with lands for its mana cost.
fn cast(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    t.g.turn.priority = Some(p);
    t.cast(p, c).targets(targets).go()
}

#[test]
fn meathook_massacre_ii_card_left_the_graveyard_doesnt_return() {
    cr!("400.7", "608.2b", "603.6c");
    ruling!(
        "Meathook Massacre II",
        "If a creature that dies leaves the graveyard before Meathook Massacre II's second or third ability (as appropriate) resolves, that creature won't return to the battlefield regardless of whether or not life was paid."
    );
    supported("Meathook Massacre II");
    for (owner, leaves) in [(P0, false), (P0, true), (P1, false), (P1, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Meathook Massacre II");
        let bears = t.battlefield(owner, "Grizzly Bears");
        // P0 pays for its own creature; P1 declines to pay for theirs.
        t.answer_yes(owner, owner == P0);
        destroy(&mut t, bears);
        assert_eq!(t.stack_len(), 1);
        if leaves {
            move_to(&mut t, bears, Zone::Exile);
        }
        t.resolve_all();
        let back = t.named_on_battlefield("Grizzly Bears");
        let label = format!("{owner:?} leaves: {leaves}");
        assert_eq!(back.len(), !leaves as usize, "{label}");
        if let Some(b) = back.first() {
            assert_eq!(t.g.obj(*b).controller, P0, "{label}");
        }
    }
}

#[test]
fn dawn_of_the_dead_returned_creature_that_left_and_came_back_isnt_exiled() {
    cr!("400.7", "603.7c");
    ruling!(
        "Dawn of the Dead",
        "If the creature leaves the battlefield, but returns to the battlefield before the triggered ability resolves, you won’t exile it. This is because it is treated as a different object than the one to which the ability refers."
    );
    supported("Dawn of the Dead");
    for flicker in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dawn of the Dead");
        let bears = t.graveyard(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        let back = t.g.current(bears);
        assert!(t.on_battlefield(back));
        if flicker {
            let gone = move_to(&mut t, back, Zone::Exile).unwrap();
            move_to(&mut t, gone, Zone::Battlefield);
        }
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(
            t.named_on_battlefield("Grizzly Bears").len(),
            flicker as usize,
            "flicker: {flicker}"
        );
    }
}

#[test]
fn underrealm_lich_replaces_each_of_several_draws_in_turn() {
    cr!("121.2", "614.11", "614.6");
    ruling!(
        "Underrealm Lich",
        "If an effect instructs you to draw multiple cards, each draw is performed—and replaced by Underrealm Lich's ability—one at a time. This means that if you're drawing two cards, you'll look at the top three cards of your library, put one into your hand, put the other two into your graveyard, and then repeat the process with the next three cards in your library."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underrealm Lich");
    let ids = stack_library(
        &mut t,
        P0,
        &[
            "Memnite",
            "Ornithopter",
            "Grizzly Bears",
            "Hill Giant",
            "Llanowar Elves",
            "Savannah Lions",
        ],
    );
    let from = t.asked().len();
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    let looks: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    // The top three, then the next three.
    let set = |v: &[ObjectId]| {
        let mut v: Vec<Entity> = v.iter().map(|o| obj(*o)).collect();
        v.sort();
        v
    };
    assert_eq!(looks.len(), 2);
    let mut first = looks[0].clone();
    first.sort();
    let mut second = looks[1].clone();
    second.sort();
    assert_eq!(first, set(&ids[0..3]));
    assert_eq!(second, set(&ids[3..6]));
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 4);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn mornsong_aria_life_total_cant_be_set_higher() {
    cr!("119.5", "119.7");
    ruling!(
        "Mornsong Aria",
        "If an effect says to set a player's life total to a number that's higher than the player's current life total while Mornsong Aria is on the battlefield, the player's life total doesn't change."
    );
    supported("Mornsong Aria");
    supported("Blessed Wind");
    // At 10: stays at 10. At 25: set to 20 (a loss of life is allowed).
    for (life, after) in [(10, 10), (25, 20)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mornsong Aria");
        t.g.players[0].life = life;
        cast(&mut t, P0, "Blessed Wind", &[Entity::Player(P0)]);
        t.resolve_all();
        assert_eq!(t.life(P0), after, "from {life}");
    }
}

/// P0's Purgatory with Grizzly Bears exiled with it (P0's Bears died).
fn purgatory_with_bears() -> (TestGame, ObjectId) {
    supported("Purgatory");
    let mut t = TestGame::new(2);
    let purg = t.battlefield(P0, "Purgatory");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    (t, purg)
}

#[test]
fn purgatory_under_an_opponents_control_returns_the_creature_to_them() {
    cr!("108.4", "603.3a", "110.2");
    ruling!(
        "Purgatory",
        "If an opponent takes control of this card and pays for the upkeep ability, the creatures will enter under their control even if you exiled them."
    );
    let (mut t, purg) = purgatory_with_bears();
    give_control(&mut t, purg, P1);
    t.lands(P1, "Plains", 4);
    t.answer_yes(P1, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.g.obj(bears[0]).controller, P1);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn purgatory_control_change_on_the_stack_uses_the_abilitys_controller() {
    cr!("603.3a", "113.8");
    ruling!(
        "Purgatory",
        "If this card changes controllers between putting the upkeep triggered ability on the stack and resolving it, the creature is put onto the battlefield under the control of the player who controlled this card when the ability was put on the stack."
    );
    let (mut t, purg) = purgatory_with_bears();
    t.lands(P0, "Plains", 4);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "pay {4}"), 1);
    give_control(&mut t, purg, P1);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.g.obj(bears[0]).controller, P0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn purgatory_destroyed_with_creatures_still_exiles_them() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Purgatory",
        "If destroyed at the same time some creatures are destroyed, the destroyed creatures are exiled and stay exiled."
    );
    supported("Akroma's Vengeance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Purgatory");
    t.battlefield(P0, "Grizzly Bears");
    cast(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Purgatory"));
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn nameless_race_pays_life_as_the_reanimation_spell_resolves() {
    cr!("614.1c", "614.12", "608.2");
    ruling!(
        "Nameless Race",
        "If put onto the battlefield by a spell or ability, you pay life during the resolution of that spell or ability."
    );
    supported("Nameless Race");
    supported("Zombify");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Savannah Lions");
    t.graveyard(P1, "Savannah Lions");
    let race = t.graveyard(P0, "Nameless Race");
    t.answer(P0, DecisionKind::Number, Answer::Number(2));
    let spell = cast(&mut t, P0, "Zombify", &[obj(race)]);
    assert_eq!(t.life(P0), 20);
    t.settle();
    t.g.resolve_top();
    // As Zombify finishes resolving the life is paid; nothing else used the stack.
    assert!(!t.g.stack.contains(&spell));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 18);
    let race = t.named_on_battlefield("Nameless Race")[0];
    assert_eq!(t.pt(race), (2, 2));
}

#[test]
fn spatial_binding_on_an_aura_it_stays_and_falls_off() {
    cr!("702.26g", "704.5m", "702.26d");
    ruling!(
        "Spatial Binding",
        "If used on an Aura and the permanent enchanted by that Aura phases out, the Aura will not phase out. It will stay on the battlefield and then immediately be put into the graveyard since the permanent it was enchanting is phased out."
    );
    supported("Spatial Binding");
    supported("Slip Out the Back");
    let mut t = TestGame::new(2);
    let binding = t.battlefield(P0, "Spatial Binding");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = attach_new(&mut t, P0, "Holy Strength", bears);
    t.activate(P0, binding, 0, &[obj(strength)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    cast(&mut t, P0, "Slip Out the Back", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out);
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn blood_operative_surveiled_into_the_graveyard_triggers_after_surveil() {
    cr!("701.25a", "603.2", "603.3");
    ruling!(
        "Blood Operative",
        "If you surveil and put Blood Operative into your graveyard as you do so, its ability triggers once you've finished surveilling."
    );
    ruling!(
        "Blood Operative",
        "No player may take actions between the time you choose to pay 3 life and the time you return Blood Operative to your hand."
    );
    supported("Blood Operative");
    let mut t = TestGame::new(2);
    let ids = stack_library(&mut t, P0, &["Blood Operative", "Grizzly Bears"]);
    t.battlefield(P0, "Blood Operative");
    // Surveil 2: both cards into the graveyard.
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], ids.clone()));
    mtg_engine::library::surveil(&mut t.g, P0, 2);
    t.g.flush_events();
    assert!(t.in_graveyard(P0, "Blood Operative"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "pay 3 life"), 1);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.g.resolve_top();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.life(P0), 17);
    assert!(t.in_hand(P0, "Blood Operative"));
}

#[test]
fn lorcan_exiles_every_warlock_you_control() {
    cr!("614.6", "205.3m");
    ruling!(
        "Lorcan, Warlock Collector",
        "Lorcan's last ability affects all Warlocks you control, not only ones he collected from an opponent's graveyard."
    );
    supported("Lorcan, Warlock Collector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorcan, Warlock Collector");
    // Grim Servant is a Zombie Warlock of P0's own.
    let gs = t.battlefield(P0, "Grim Servant");
    destroy(&mut t, gs);
    assert!(t.in_exile("Grim Servant"));
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn lorcan_looks_at_the_card_type_in_the_graveyard() {
    cr!("400.7", "603.6a", "715.4");
    ruling!(
        "Lorcan, Warlock Collector",
        "Lorcan, Warlock Collector's second ability looks only at the type the card has in the graveyard, not the type it had in any other zone. For example, animated lands dying won't cause the ability to trigger. The opposite is also true. For example, if a creature that was cast as an Adventure is countered, it's a creature card in the graveyard and the triggered ability will trigger."
    );
    supported("Mutavault");
    supported("Bonecrusher Giant // Stomp");
    // An animated Mutavault of P1's dies: a land card in the graveyard, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorcan, Warlock Collector");
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    crate::r_s06_common::activate_containing(&mut t, P1, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).is_creature());
    destroy(&mut t, vault);
    assert!(t.in_graveyard(P1, "Mutavault"));
    assert_eq!(triggers_on_stack(&t, "pay life equal"), 0);
    // P1's Stomp (the Adventure of Bonecrusher Giant) is countered: a creature card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lorcan, Warlock Collector");
    t.lands(P1, "Mountain", 2);
    let giant = t.hand(P1, "Bonecrusher Giant // Stomp");
    t.g.turn.priority = Some(P1);
    let stomp = t
        .cast(P1, giant)
        .method(mtg_engine::object::CastMethod::Half(1))
        .target(Entity::Player(P0))
        .go();
    cast(&mut t, P0, "Cancel", &[obj(stomp)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    // P0 paid 3 life and got the Giant, a Warlock.
    let g = t.named_on_battlefield("Bonecrusher Giant");
    assert_eq!(g.len(), 1, "{}", t.dump_log());
    assert_eq!(t.g.obj(g[0]).controller, P0);
    assert!(t.g.obj(g[0]).chars.has_subtype("Warlock"));
    assert_eq!(t.life(P0), 17);
}
