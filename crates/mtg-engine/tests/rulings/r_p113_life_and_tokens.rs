//! Rulings batch P113 — removal that makes the creature's controller gain or lose life,
//! "target opponent" abilities, opponents discarding, spells that also create tokens,
//! tokens that enter attacking, and damage counted as the spell resolves.

use crate::r_p113_common::*;
use crate::r_s01_common::{attack_with, supported, tapped_lands};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s05_common::run_from;
use crate::r_s07_common::{damage_on, resolved};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn removal_with_life_change_for_its_controller_needs_a_legal_target() {
    cr!("608.2b");
    ruling!(
        "Last Breath",
        "If the creature is an illegal target when Last Breath tries to resolve, it won't resolve and none of its effects will happen. Its controller won't gain 4 life."
    );
    ruling!(
        "Hideous End",
        "If the targeted creature is an illegal target by the time Hideous End resolves, the entire spell doesn’t resolve. No one loses life."
    );
    ruling!(
        "Sip of Hemlock",
        "If the creature is an illegal target when Sip of Hemlock tries to resolve, it won't resolve and none of its effects will occur. The creature's controller won't lose 2 life."
    );
    // (spell, P1's life after it resolves with a legal target)
    for (spell, life) in [("Last Breath", 24), ("Hideous End", 18), ("Sip of Hemlock", 18)] {
        for illegal in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let s = cast_modes(&mut t, P0, spell, &[], &[ent(bears)]);
            if illegal {
                spoil(&mut t, bears, Spoil::Hexproof);
            }
            t.resolve_all();
            assert_eq!(resolved(&t, s), !illegal, "{spell}");
            assert_eq!(t.on_battlefield(bears), illegal, "{spell}");
            assert_eq!(t.life(P1), if illegal { 20 } else { life }, "{spell}");
        }
    }
}

#[test]
fn sip_of_hemlock_life_loss_without_the_creature_dying() {
    cr!("608.2c", "702.12b");
    ruling!(
        "Sip of Hemlock",
        "However, if Sip of Hemlock resolves and the creature isn't destroyed (perhaps because it regenerated or had indestructible), the creature's controller will still lose 2 life."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Indestructible,
        ))],
    );
    cast_modes(&mut t, P0, "Sip of Hemlock", &[], &[ent(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn swords_to_plowshares_uses_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Swords to Plowshares",
        "Use the power of the creature from when it was last on the battlefield to determine how much life is gained."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_modes(&mut t, P0, "Swords to Plowshares", &[], &[ent(giant)]);
    // In response, the Giant gets +3/+3 until end of turn (the card in exile is a 3/3).
    cast_new(&mut t, P1, "Giant Growth", &[ent(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.life(P1), 26);
}

#[test]
fn alms_beast_lifelink_is_redundant() {
    cr!("702.15f", "510.2");
    ruling!(
        "Alms Beast",
        "Multiple instances of lifelink on the same creature are redundant. If a creature that had lifelink already is blocking or blocked by Alms Beast, the controller of that creature won't gain extra life."
    );
    supported("Alms Beast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Alms Beast");
    // Vampire Nighthawk: 2/3 flying, deathtouch, lifelink.
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    attack_with(&mut t, &[(beast, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(hawk, beast)]);
    assert_eq!(t.life(P1), 22, "2 damage, 2 life");
    assert_eq!(t.life(P0), 20, "the Beast itself doesn't have lifelink");
}

#[test]
fn indulgent_tormentor_without_a_legal_target_does_nothing() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Indulgent Tormentor",
        "If there are no legal targets to choose, perhaps because all your opponents have hexproof, the ability is removed from the stack with no effect. You won’t draw a card. This is also true if the target opponent becomes an illegal target in response to the ability."
    );
    supported("Indulgent Tormentor");
    // P1 has hexproof (Leyline of Sanctity) at the beginning of P0's upkeep.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Indulgent Tormentor");
    t.battlefield(P1, "Leyline of Sanctity");
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), hand);
    // P1 gets hexproof in response.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Indulgent Tormentor");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let hand = t.hand_size(P0);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 20);
}

/// P0 activates Hired Torturer targeting P1 (with mana for it) and it resolves.
fn torture(t: &mut TestGame, torturer: ObjectId) {
    let now = t.g.current(torturer);
    t.g.objects[now.0 as usize].tapped = false;
    t.lands(P0, "Swamp", 4);
    t.activate(P0, torturer, 0, &[Entity::Player(P1)])
        .expect("Hired Torturer");
    t.resolve_all();
}

fn revealed_cards(t: &TestGame) -> Vec<ObjectId> {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
            _ => None,
        })
        .collect()
}

#[test]
fn hired_torturer_reveals_at_random_each_time() {
    cr!("701.20a", "701.20c", "608.2c");
    ruling!(
        "Hired Torturer",
        "Each card in the opponent’s hand has an equal chance of being revealed, even if it was previously revealed because of Hired Torturer’s ability."
    );
    ruling!(
        "Hired Torturer",
        "The card stops being revealed after Hired Torturer’s ability resolves."
    );
    ruling!(
        "Hired Torturer",
        "You can choose an opponent with no cards in hand as the target of Hired Torturer’s ability. If the target opponent has no cards in hand when the ability resolves, that player will just lose 2 life."
    );
    supported("Hired Torturer");
    // No cards in hand: just 2 life.
    let mut t = TestGame::new(2);
    let torturer = t.battlefield(P0, "Hired Torturer");
    torture(&mut t, torturer);
    assert_eq!(t.life(P1), 18);
    assert!(revealed_cards(&t).is_empty());
    // Two cards: over many activations, each is revealed, also again after it was.
    let mut t = TestGame::new(2);
    let torturer = t.battlefield(P0, "Hired Torturer");
    let a = t.hand(P1, "Grizzly Bears");
    let b = t.hand(P1, "Hill Giant");
    let mut seen = vec![];
    for _ in 0..24 {
        t.g.players[1].life = 20;
        let before = revealed_cards(&t).len();
        torture(&mut t, torturer);
        let now = revealed_cards(&t);
        assert_eq!(now.len(), before + 1);
        let card = *now.last().unwrap();
        assert!(card == a || card == b);
        // It's no longer revealed once the ability has resolved.
        assert!(!mtg_engine::reveal::is_revealed(&t.g, card));
        seen.push(card);
    }
    let na = seen.iter().filter(|c| **c == a).count();
    assert!((4..=20).contains(&na), "{na} of 24");
    assert!(seen.windows(2).any(|w| w[0] == w[1]), "the same card again");
}

/// Puts `n` cards into P1's hand and advances to P1's cleanup step, where P1 discards down
/// to seven; stops when players get priority in the cleanup step (because abilities
/// triggered), with the triggers on the stack.
fn discard_in_cleanup(t: &mut TestGame, n: usize) {
    for _ in 0..n {
        t.hand(P1, "Grizzly Bears");
    }
    t.set_step(P1, Step::End);
    t.advance_to_step(Step::Cleanup);
    assert_eq!(t.hand_size(P1), 7);
}

#[test]
fn discarding_in_cleanup_triggers_once_per_card() {
    cr!("514.1", "514.3a", "603.2c");
    ruling!(
        "Liliana's Caress",
        "If an opponent discards multiple cards (due to a spell, an ability, or the action of discarding down to the maximum hand size during their cleanup step), Liliana’s Caress’s ability triggers that many times."
    );
    ruling!(
        "Abyssal Nocturnus",
        "If your opponent discards any cards during their cleanup step to get down to the maximum hand size, the ability will trigger and players will receive priority. After that cleanup step ends, a new one will begin and Abyssal Nocturnus's effect will end. It won't carry over into the next turn."
    );
    supported("Liliana's Caress");
    supported("Abyssal Nocturnus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana's Caress");
    let noc = t.battlefield(P0, "Abyssal Nocturnus");
    discard_in_cleanup(&mut t, 9);
    assert_eq!(t.stack_len(), 4, "two triggers of each");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.pt(noc), (6, 6));
    // The next cleanup step ends the effect; it doesn't last into P0's turn.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(noc), (2, 2));
}

#[test]
fn discarding_several_cards_to_one_effect_triggers_several_times() {
    cr!("603.2c", "701.9a");
    ruling!(
        "Abyssal Nocturnus",
        "If your opponent discards multiple cards to a single effect, this will trigger multiple times."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liliana's Caress");
    let noc = t.battlefield(P0, "Abyssal Nocturnus");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    cast_modes(&mut t, P0, "Mind Rot", &[], &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.pt(noc), (6, 6));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn seize_the_soul_can_target_a_colorless_creature() {
    cr!("105.2c", "115.1");
    ruling!(
        "Seize the Soul",
        "A \"nonwhite nonblack\" creature is a creature that's neither white nor black. It can be any other color or combination of colors, or it can be colorless."
    );
    supported("Seize the Soul");
    // (creature, legal target)
    for (name, ok) in [
        ("Ornithopter", true),
        ("Grizzly Bears", true),
        ("Serra Angel", false),
        ("Vampire Nighthawk", false),
    ] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        crate::r_s25_common::lands_for_cost(&mut t, P0, "Seize the Soul");
        let card = t.hand(P0, "Seize the Soul");
        t.answer_targets(P0, &[ent(c)]);
        let r = t.cast(P0, card).try_go();
        assert_eq!(r.is_ok(), ok, "{name}");
        if ok {
            t.resolve_all();
            assert!(!t.on_battlefield(c), "{name}");
        }
    }
}

#[test]
fn fungal_infection_needs_a_legal_target_for_the_token() {
    cr!("608.2b");
    ruling!(
        "Fungal Infection",
        "If the target creature is an illegal target by the time Fungal Infection tries to resolve, the spell doesn’t resolve. You won’t create a Saproling token."
    );
    for illegal in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_modes(&mut t, P0, "Fungal Infection", &[], &[ent(bears)]);
        if illegal {
            spoil(&mut t, bears, Spoil::Hexproof);
        }
        t.resolve_all();
        let saprolings = crate::r_s05_common::tokens_with_subtype(&t, P0, "Saproling");
        assert_eq!(saprolings.len(), usize::from(!illegal));
    }
}

#[test]
fn fungal_infection_target_sees_the_token_enter_before_dying() {
    cr!("704.3", "603.2", "603.10");
    ruling!(
        "Fungal Infection",
        "The target creature will be on the battlefield when you create the Saproling token, even if it’s about to die for having 0 toughness or lethal damage. Its abilities may affect the token’s creation or trigger when the token enters the battlefield. Abilities that trigger this way will resolve after the target creature has died."
    );
    // Soul Warden (1/1): "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P0, "Soul Warden");
    cast_modes(&mut t, P0, "Fungal Infection", &[], &[ent(warden)]);
    t.resolve();
    // The Warden died; its trigger is on the stack.
    assert!(!t.on_battlefield(warden));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn dunes_of_the_dead_token_comes_before_the_ability_it_paid_for() {
    cr!("602.2", "603.3", "117.3b");
    ruling!(
        "Dunes of the Dead",
        "If you sacrifice Dunes of the Dead to pay the activation cost of an ability, you'll create the Zombie token before that activated ability resolves."
    );
    ruling!(
        "Dunes of the Dead",
        "Dunes of the Dead's second ability doesn't allow you to sacrifice it whenever you'd like. You must find another way to get Dunes of the Dead into the graveyard."
    );
    supported("Dunes of the Dead");
    let mut t = TestGame::new(2);
    let dunes = t.battlefield(P0, "Dunes of the Dead");
    // Dunes of the Dead has no ability that sacrifices it.
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, dunes)
        || t.obj(dunes).chars.abilities.iter().all(|a| !a.text.contains("Sacrifice")));
    // Zuran Orb: "Sacrifice a land: You gain 2 life."
    let orb = t.battlefield(P0, "Zuran Orb");
    t.answer_choose(P0, &[ent(dunes)]);
    t.activate(P0, orb, 0, &[]).expect("Zuran Orb");
    assert!(t.in_graveyard(P0, "Dunes of the Dead"));
    t.settle();
    assert_eq!(t.stack_len(), 2, "the Dunes trigger is above the Orb's ability");
    t.resolve();
    assert_eq!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Zombie").len(), 1);
    assert_eq!(t.life(P0), 20, "the Orb's ability hasn't resolved yet");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn tokens_put_onto_the_battlefield_attacking_werent_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Captain's Claws",
        "The token is never declared as an attacking creature. It simply enters the battlefield attacking. This won’t cause any abilities that trigger “whenever a creature attacks” to trigger."
    );
    ruling!(
        "Gut, True Soul Zealot",
        "Although the Skeleton is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    supported("Captain's Claws");
    supported("Gut, True Soul Zealot");
    // Hellrider: "Whenever a creature you control attacks, Hellrider deals 1 damage to the
    // player or planeswalker it's attacking."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Captain's Claws", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let kor = crate::r_s05_common::tokens_with_subtype(&t, P0, "Kor");
    assert_eq!(kor.len(), 1);
    assert!(t.g.is_attacking(kor[0]));
    assert_eq!(t.life(P1), 19, "Hellrider triggered for the Bears only");
    // Gut's Skeleton.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hellrider");
    let gut = t.battlefield(P0, "Gut, True Soul Zealot");
    let fodder = t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[ent(fodder)]);
    attack_with(&mut t, &[(gut, Entity::Player(P1))]);
    t.resolve_all();
    let skel = crate::r_s05_common::tokens_with_subtype(&t, P0, "Skeleton");
    assert_eq!(skel.len(), 1);
    assert!(t.g.is_attacking(skel[0]));
    assert_eq!(t.life(P1), 19, "Hellrider triggered for Gut only");
}

#[test]
fn gut_skeleton_enters_attacking_despite_attack_restrictions() {
    cr!("508.4", "508.1d");
    ruling!(
        "Gut, True Soul Zealot",
        "Any effects that say the Skeleton can't attack (such as that of Propaganda) affect only the declaration of attackers. They won't stop the Skeleton from entering the battlefield attacking."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let gut = t.battlefield(P0, "Gut, True Soul Zealot");
    let fodder = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[ent(fodder)]);
    attack_with(&mut t, &[(gut, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 2, "only Gut's attack was paid for");
    let skel = crate::r_s05_common::tokens_with_subtype(&t, P0, "Skeleton");
    assert_eq!(skel.len(), 1);
    assert!(t.g.is_attacking(skel[0]));
}

#[test]
fn creature_count_damage_is_counted_as_the_spell_resolves() {
    cr!("608.2h");
    ruling!(
        "Outnumber",
        "Count the number of creatures you control as Outnumber resolves to determine how much damage Outnumber deals."
    );
    ruling!(
        "Massive Raid",
        "Count the number of creatures you control when Massive Raid resolves to determine how much damage is dealt."
    );
    for spell in ["Outnumber", "Massive Raid"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        let wurm = t.battlefield(P1, "Craw Wurm");
        cast_modes(&mut t, P0, spell, &[], &[ent(wurm)]);
        // Two more creatures arrive before it resolves.
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        t.resolve_all();
        assert_eq!(damage_on(&t, wurm), 3, "{spell}");
    }
}

#[test]
fn superior_numbers_opponent_and_creature_are_separate_targets() {
    cr!("115.1", "608.2b");
    ruling!(
        "Superior Numbers",
        "As you cast this, you target a creature and an opponent. The opponent does not have to be that creature’s controller."
    );
    ruling!(
        "Superior Numbers",
        "If the creature becomes an illegal target before the spell resolves (but the opponent doesn’t), the spell will be unable to affect that creature. Although the spell will still resolve, it won’t deal any damage."
    );
    supported("Superior Numbers");
    for illegal in [false, true] {
        let mut t = TestGame::new(3);
        for _ in 0..4 {
            t.battlefield(P0, "Grizzly Bears");
        }
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(PlayerId(2), "Grizzly Bears");
        // P1's creature, P2 as the opponent: 4 - 1 = 3 damage.
        let s = cast_modes(
            &mut t,
            P0,
            "Superior Numbers",
            &[],
            &[ent(wurm), Entity::Player(PlayerId(2))],
        );
        if illegal {
            spoil(&mut t, wurm, Spoil::Hexproof);
        }
        t.resolve_all();
        assert!(resolved(&t, s));
        assert_eq!(damage_on(&t, wurm), if illegal { 0 } else { 3 });
        assert_eq!(t.life(P1), 20);
        assert_eq!(t.life(PlayerId(2)), 20);
    }
}
