//! Rulings batch P141 — leaves-the-battlefield triggers of permanents that leave at the
//! same time as their source (CR 603.10a: they look back in time), and combat damage
//! triggers of creatures that die from the same combat damage.

use crate::r_p141_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn aetherworks_marvel_counts_itself_and_permanents_leaving_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Aetherworks Marvel",
        "Aetherworks Marvel's first ability triggers whenever any permanent you control is put into a graveyard from the battlefield, including Aetherworks Marvel itself"
    );
    supported("Aetherworks Marvel");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Aetherworks Marvel");
    let f = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy_together(&mut t, &[m, f, b]);
    assert_eq!(stacked_triggers(&t), 3);
    t.resolve_all();
    assert_eq!(player_counter(&t, P0, "energy"), 3);
}

/// Destroys `source` (a real card put onto the battlefield for P0) together with the given
/// other P0 permanents; returns the number of triggered abilities put on the stack.
fn dies_with(t: &mut TestGame, source: &str, others: &[&str]) -> usize {
    let s = t.battlefield(P0, source);
    let mut ids = vec![s];
    ids.extend(others.iter().map(|n| t.battlefield(P0, n)));
    destroy_together(t, &ids);
    stacked_triggers(t)
}

#[test]
fn leaves_triggers_see_creatures_dying_with_their_source() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Archghoul of Thraben",
        "If Archghoul of Thraben and another Zombie you control die at the same time, this ability will trigger once for each of them."
    );
    ruling!(
        "Colfenor, the Last Yew",
        "If Colfenor dies at the same time as another creature you control, its last ability triggers for each of them."
    );
    ruling!(
        "Meren of Clan Nel Toth",
        "If Meren of Clan Nel Toth leaves the battlefield at the same time as other creatures you control die, its first ability will trigger for each of those creatures."
    );
    ruling!(
        "Rienne, Angel of Rebirth",
        "If Rienne dies at the same time as one or more other multicolored creatures you control, its last ability triggers for each of those creatures."
    );
    ruling!(
        "Spinner of Souls",
        "If Spinner of Souls dies at the same time as one or more other nontoken creatures you control, its last ability will trigger for each of those other creatures."
    );
    ruling!(
        "Sigil of the New Dawn",
        "If a creature you own dies at the same time that Sigil of the New Dawn leaves the battlefield, Sigil of the New Dawn's ability triggers for that creature."
    );
    ruling!(
        "Pia's Revolution",
        "If a nontoken artifact is put into your graveyard at the same time as Pia’s Revolution, Pia’s Revolution’s ability triggers."
    );
    // (source, other permanents dying with it, triggers expected)
    let cases: &[(&str, &[&str], usize)] = &[
        // Archghoul itself and another Zombie.
        (
            "Archghoul of Thraben",
            &["Walking Corpse", "Grizzly Bears"],
            2,
        ),
        // Colfenor itself and another creature.
        ("Colfenor, the Last Yew", &["Grizzly Bears"], 2),
        // Other creatures only (Meren isn't "another creature").
        (
            "Meren of Clan Nel Toth",
            &["Grizzly Bears", "Hill Giant"],
            2,
        ),
        // Other multicolored creatures only.
        (
            "Rienne, Angel of Rebirth",
            &["Watchwolf", "Watchwolf", "Grizzly Bears"],
            2,
        ),
        // Other nontoken creatures.
        ("Spinner of Souls", &["Grizzly Bears", "Hill Giant"], 2),
        ("Sigil of the New Dawn", &["Grizzly Bears"], 1),
        ("Pia's Revolution", &["Ornithopter"], 1),
    ];
    for (source, others, n) in cases {
        supported(source);
        let mut t = TestGame::new(2);
        assert_eq!(dies_with(&mut t, source, others), *n, "{source}");
    }
    // Meren gets an experience counter for each.
    let mut t = TestGame::new(2);
    dies_with(
        &mut t,
        "Meren of Clan Nel Toth",
        &["Grizzly Bears", "Hill Giant"],
    );
    t.resolve_all();
    assert_eq!(player_counter(&t, P0, "experience"), 2);
    // Pia's Revolution: the opponent declines, the artifact returns.
    let mut t = TestGame::new(2);
    dies_with(&mut t, "Pia's Revolution", &["Ornithopter"]);
    t.answer_yes(P1, false);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));
}

#[test]
fn colfenor_cant_target_the_trigger_card_but_each_can_target_the_other() {
    cr!("603.10a", "115.1");
    ruling!(
        "Colfenor, the Last Yew",
        "Colfenor's last ability can't target the creature card whose death caused the ability to trigger, but if two or more creatures die at the same time, each can be the target of the ability caused by one of the others."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Colfenor, the Last Yew");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = n_asked(&t);
    destroy_together(&mut t, &[giant, bears]);
    assert_eq!(stacked_triggers(&t), 2);
    let gb = now(&t, bears);
    let gg = now(&t, giant);
    // The Hill Giant's trigger (toughness 3) may target the Bears (toughness 2), never the
    // Giant itself; the Bears' trigger has neither (the Giant's toughness isn't lesser).
    let offers = target_candidates(&t, P0, from);
    assert!(offers.iter().any(|c| c.contains(&Entity::Object(gb))));
    assert!(offers.iter().all(|c| !c.contains(&Entity::Object(gg))));
}

/// An Aura ("Holy Strength") controlled by `aura_owner` attached to a fresh `name`
/// controlled by `p`; returns the creature.
fn enchanted(t: &mut TestGame, p: PlayerId, aura_owner: PlayerId, name: &str) -> ObjectId {
    let c = t.battlefield(p, name);
    attach_new(t, aura_owner, "Holy Strength", c);
    c
}

#[test]
fn dawn_evangel_triggers_for_creatures_enchanted_by_your_auras() {
    cr!("603.10a", "603.4");
    ruling!(
        "Dawn Evangel",
        "Dawn Evangel's ability triggers when any player's creature dies, so long as that creature had an Aura you controlled attached to it."
    );
    ruling!(
        "Dawn Evangel",
        "If Dawn Evangel dies while enchanted by an Aura you control, its ability triggers."
    );
    ruling!(
        "Dawn Evangel",
        "If Dawn Evangel dies at the same time as another creature enchanted by an Aura you control, its ability triggers."
    );
    ruling!(
        "Dawn Evangel",
        "Dawn Evangel's ability can target the card whose death caused the ability to trigger if that card's mana value is 2 or less."
    );
    supported("Dawn Evangel");
    // An opponent's creature with P0's Aura: triggers; without one: doesn't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawn Evangel");
    t.graveyard(P0, "Grizzly Bears");
    let theirs = enchanted(&mut t, P1, P0, "Hill Giant");
    let plain = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, plain);
    assert_eq!(stacked_triggers(&t), 0);
    destroy(&mut t, theirs);
    assert_eq!(stacked_triggers(&t), 1);
    // An Aura the opponent controls on P0's creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawn Evangel");
    t.graveyard(P0, "Grizzly Bears");
    let mine = enchanted(&mut t, P0, P1, "Hill Giant");
    destroy(&mut t, mine);
    assert_eq!(stacked_triggers(&t), 0);
    // Dawn Evangel itself enchanted.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    let e = enchanted(&mut t, P0, P0, "Dawn Evangel");
    destroy(&mut t, e);
    assert_eq!(stacked_triggers(&t), 1);
    // Dawn Evangel dying with an enchanted Grizzly Bears: the Bears (mana value 2) can
    // be the target (Dawn Evangel, mana value 3, can't), and returns.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Dawn Evangel");
    let b = enchanted(&mut t, P0, P0, "Grizzly Bears");
    let from = n_asked(&t);
    destroy_together(&mut t, &[e, b]);
    assert_eq!(stacked_triggers(&t), 1);
    assert!(offered_target(&t, P0, from, now(&t, b)));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn tiana_returns_auras_dying_with_her_but_not_auras_falling_off_afterward() {
    cr!("603.10a", "704.5m", "603.7");
    ruling!(
        "Tiana, Ship's Caretaker",
        "If an Aura or Equipment you control is put into a graveyard at the same time as Tiana is, most likely because an effect destroyed all nonland permanents, you'll be able to return it to its owner's hand at the beginning of the next end step."
    );
    ruling!(
        "Tiana, Ship's Caretaker",
        "If an Aura you control is put into a graveyard immediately after Tiana leaves the battlefield, most likely because Tiana left the battlefield but the Aura was put into the graveyard as a state-based action after it found itself not attached to anything, Tiana's last ability won't trigger."
    );
    supported("Tiana, Ship's Caretaker");
    // Destroyed together: triggers, returns at the end step.
    let mut t = TestGame::new(2);
    let tiana = t.battlefield(P0, "Tiana, Ship's Caretaker");
    let eq = t.battlefield(P0, "Bonesplitter");
    destroy_together(&mut t, &[tiana, eq]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bonesplitter"));
    yes(&mut t, P0);
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Bonesplitter"));
    // Tiana enchanted by P0's Aura: the Aura is put into the graveyard by an SBA after
    // Tiana is gone, so nothing triggers.
    let mut t = TestGame::new(2);
    let tiana = t.battlefield(P0, "Tiana, Ship's Caretaker");
    attach_new(&mut t, P0, "Holy Strength", tiana);
    destroy(&mut t, tiana);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(stacked_triggers(&t), 0);
}

#[test]
fn glissa_can_target_your_artifact_creature_an_opponent_controlled() {
    cr!("603.10a", "115.1");
    ruling!(
        "Glissa, the Traitor",
        "If an artifact you own and a creature an opponent controls go to the graveyard at the same time, you may choose that artifact card as the target"
    );
    supported("Glissa, the Traitor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glissa, the Traitor");
    let thopter = t.battlefield(P0, "Ornithopter");
    gain_control(&mut t, P1, thopter);
    assert_eq!(t.obj_now(thopter).controller, P1);
    let from = n_asked(&t);
    destroy(&mut t, thopter);
    assert_eq!(stacked_triggers(&t), 1);
    let card = now(&t, thopter);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert!(offered_target(&t, P0, from, card));
    yes(&mut t, P0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ornithopter"));
}

#[test]
fn turntimber_sower_triggers_once_for_lands_at_once() {
    cr!("603.2c");
    ruling!(
        "Turntimber Sower",
        "If multiple land cards are put into your graveyard at once, Turntimber Sower's first ability triggers only once."
    );
    supported("Turntimber Sower");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Turntimber Sower");
    stack_library(&mut t, P0, &["Forest", "Island", "Swamp"]);
    mill(&mut t, P0, 3);
    assert_eq!(stacked_triggers(&t), 1);
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Island");
    t.resolve_all();
    destroy_together(&mut t, &[a, b]);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Plant").len(), 2);
}

#[test]
fn ravos_leaving_makes_marked_damage_lethal() {
    cr!("613.4c", "704.5g", "120.6");
    ruling!(
        "Ravos, Soultender",
        "nonlethal damage dealt to creatures you control may become lethal if Ravos leaves the battlefield during that turn."
    );
    supported("Ravos, Soultender");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Ravos, Soultender");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(b), (3, 3));
    let shock = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(shock, Entity::Object(b), 2, false);
    t.settle();
    assert!(t.on_battlefield(b));
    destroy(&mut t, r);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn fell_shepherd_returns_creatures_dying_with_its_combat_damage() {
    cr!("510.2", "603.10a", "702.19b");
    ruling!(
        "Fell Shepherd",
        "Creatures dealt lethal damage at the same time Fell Shepherd deals combat damage to a player will also be returned to your hand from your graveyard."
    );
    supported("Fell Shepherd");
    // A blocked Bears dies in the same combat damage step.
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fell Shepherd");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Hill Giant");
    yes(&mut t, P0);
    t.attack(
        &[(fs, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[(wall, bears)],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // Fell Shepherd with trample, blocked, dies itself: it's returned too.
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fell Shepherd");
    grant(&mut t, fs, KeywordKind::Trample);
    let wurm = t.battlefield(P1, "Craw Wurm");
    yes(&mut t, P0);
    t.attack(&[(fs, Entity::Player(P1))], &[(wurm, fs)]);
    t.resolve_all();
    assert!(t.life(P1) < 20);
    assert!(t.in_hand(P0, "Fell Shepherd"));
}

#[test]
fn charnelhoard_wurm_can_return_itself_after_dying_in_combat() {
    cr!("510.2", "603.10a", "702.19b");
    ruling!(
        "Charnelhoard Wurm",
        "If Charnelhoard Wurm is dealt lethal damage at the same time it deals damage to an opponent, Charnelhoard Wurm will be in your graveyard by the time you put its ability on the stack. In that case, you can target it with its own ability."
    );
    supported("Charnelhoard Wurm");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Charnelhoard Wurm");
    let a = t.battlefield(P1, "Craw Wurm");
    let from = n_asked(&t);
    attack_with(&mut t, &[(w, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        decision::Answer::Blockers(vec![(a, w)]),
    );
    t.advance_to(P0, turn::Step::CombatDamage);
    t.settle();
    assert!(t.life(P1) < 20, "the Wurm tramples over");
    assert!(t.in_graveyard(P0, "Charnelhoard Wurm"));
    let card = now(&t, w);
    assert!(offered_target(&t, P0, from, card));
    t.resolve_all();
    assert!(t.in_hand(P0, "Charnelhoard Wurm"));
}

#[test]
fn storrev_dying_from_combat_damage_still_triggers() {
    cr!("510.2", "603.10a");
    ruling!(
        "Storrev, Devkarin Lich",
        "If Storrev dies due to combat damage dealt at the same time that it deals combat damage to a player or planeswalker, its ability still triggers."
    );
    supported("Storrev, Devkarin Lich");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Storrev, Devkarin Lich");
    t.graveyard(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.attack(&[(s, Entity::Player(P1))], &[(wurm, s)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P0, "Storrev, Devkarin Lich"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn sword_of_the_realms_triggers_for_a_token_that_cant_return() {
    cr!("603.10a", "111.7", "704.5d");
    ruling!(
        "Halvar, God of Battle // Sword of the Realms",
        "If Sword of the Realms is attached to a token creature that dies, the Sword's triggered ability will trigger, but the token will cease to exist after going to the graveyard. It won't be returned to its owner's hand."
    );
    supported("Halvar, God of Battle // Sword of the Realms");
    let mut t = TestGame::new(2);
    let sword = back_face(&mut t, P0, "Halvar, God of Battle // Sword of the Realms");
    let token = create_token(&mut t, P0, "Soldier");
    assert!(t.g.attach(sword, Entity::Object(token)));
    t.g.recompute();
    let hand = t.hand_size(P0);
    destroy(&mut t, token);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Positive control: a nontoken creature returns.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(now(&t, sword), Entity::Object(bears)));
    t.g.recompute();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn atla_palani_triggers_for_eggs_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Atla Palani, Nest Tender",
        "If Atla Palani dies at the same time as one or more Eggs you control, its last ability triggers for each of those Eggs."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Atla Palani, Nest Tender");
    let e1 = create_token(&mut t, P0, "Egg");
    let e2 = create_token(&mut t, P0, "Egg");
    stack_library(&mut t, P0, &["Grizzly Bears", "Hill Giant"]);
    destroy_together(&mut t, &[a, e1, e2]);
    assert_eq!(stacked_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(creatures(&t, P0).len(), 2);
}
