//! Rulings batch P130: reflexive triggered abilities ("When you do, ..."). The first
//! ability has no target; the reflexive one triggers when its "you may" action is
//! performed, and its targets are chosen as it goes on the stack (CR 603.12), where players
//! may respond to it.

use crate::r_p130_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn shrapnel_slinger_targets_when_you_sacrifice() {
    cr!("603.12", "115.1");
    ruling!(
        "Shrapnel Slinger",
        "You don't choose a target for Shrapnel Slinger last ability at the time it triggers."
    );
    supported("Shrapnel Slinger");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let from = t.asked().len();
    let slinger = t.enter(P0, "Shrapnel Slinger");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(thopter)]);
    resolve_into_reflexive(&mut t, &[obj(thopter)]);
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(thopter));
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
    assert!(t.on_battlefield(slinger));
}

#[test]
fn slinza_targets_when_you_pay() {
    cr!("603.12", "701.14a");
    ruling!(
        "Slinza, the Spiked Stampede",
        "You don't choose a target for Slinza's last ability at the time it triggers."
    );
    supported("Slinza, the Spiked Stampede");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let slinza = t.enter(P0, "Slinza, the Spiked Stampede");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(slinza).damage, 2);
}

#[test]
fn snaremaster_sprite_targets_when_you_pay() {
    cr!("603.12", "122.1g");
    ruling!(
        "Snaremaster Sprite",
        "You don't choose a target for Snaremaster Sprite's ability at the time it triggers."
    );
    supported("Snaremaster Sprite");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Snaremaster Sprite");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(!t.obj_now(bears).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, "stun"), 1);
}

#[test]
fn snaremaster_sprite_can_target_a_tapped_creature() {
    cr!("603.12", "701.26a", "122.1g");
    ruling!(
        "Snaremaster Sprite",
        "You may target a creature that is already tapped with the reflexive triggered ability."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Snaremaster Sprite");
    t.settle();
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, "stun"), 1);
}

#[test]
fn sparktongue_dragon_targets_when_you_pay() {
    cr!("603.12", "120.3");
    ruling!(
        "Sparktongue Dragon",
        "You don't choose a target for Sparktongue Dragon's last ability at the time it triggers."
    );
    supported("Sparktongue Dragon");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let from = t.asked().len();
    t.enter(P0, "Sparktongue Dragon");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    resolve_into_reflexive(&mut t, &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sparktongue_dragon_without_paying_nothing_triggers() {
    cr!("603.12");
    ruling!(
        "Sparktongue Dragon",
        "Each player may respond to this triggered ability as normal."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let from = t.asked().len();
    t.enter(P0, "Sparktongue Dragon");
    t.settle();
    t.answer_yes(P0, false);
    t.resolve();
    // Nothing was paid: no reflexive trigger, and no target was ever chosen.
    assert_eq!(t.stack_len(), 0);
    assert!(target_candidates(&t, P0, from).is_empty());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn spellbook_vendor_targets_when_you_pay() {
    cr!("603.12", "303.7");
    ruling!(
        "Spellbook Vendor",
        "You don't choose a target for Spellbook Vendor's ability at the time it triggers."
    );
    supported("Spellbook Vendor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spellbook Vendor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn talions_messenger_targets_when_you_discard() {
    cr!("603.12", "701.9a");
    ruling!(
        "Talion's Messenger",
        "You don't choose a target for Talion's Messenger's ability at the time it triggers."
    );
    supported("Talion's Messenger");
    let mut t = TestGame::new(2);
    let msg = t.battlefield(P0, "Talion's Messenger");
    let card = t.hand(P0, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(msg, Entity::Player(P1))]);
    no_target_yet(&t, P0, from);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(msg)]);
    resolve_into_reflexive(&mut t, &[obj(msg)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.counters(msg, "+1/+1"), 0);
    t.resolve_all();
    assert_eq!(t.counters(msg, "+1/+1"), 1);
}

#[test]
fn terror_ballista_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Terror Ballista",
        "You don't choose a target for Terror Ballista's triggered ability when it attacks."
    );
    supported("Terror Ballista");
    let mut t = TestGame::new(2);
    let ballista = t.battlefield(P0, "Terror Ballista");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    attack_with(&mut t, &[(ballista, Entity::Player(P1))]);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(giant)]);
    resolve_into_reflexive(&mut t, &[obj(giant)]);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn thousand_moons_crackshot_targets_when_you_pay() {
    cr!("603.12", "701.26a");
    ruling!(
        "Thousand Moons Crackshot",
        "You don't choose a target for Thousand Moons Crackshot's triggered ability at the time it triggers."
    );
    supported("Thousand Moons Crackshot");
    let mut t = TestGame::new(2);
    let shot = t.battlefield(P0, "Thousand Moons Crackshot");
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(shot, Entity::Player(P1))]);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(!t.obj_now(bears).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn itzquinth_chooses_both_targets_when_you_pay() {
    cr!("603.12", "115.1");
    ruling!(
        "Itzquinth, Firstborn of Gishath",
        "You don't choose targets for Itzquinth, Firstborn of Gishath's triggered ability at the time it triggers."
    );
    supported("Itzquinth, Firstborn of Gishath");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let itz = t.enter(P0, "Itzquinth, Firstborn of Gishath");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(itz)]);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(itz), obj(bears)]);
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn dokuchi_silencer_targets_after_you_discard() {
    cr!("603.12", "510.3a");
    ruling!(
        "Dokuchi Silencer",
        "You don't choose the target for Dokuchi Silencer's last ability until after you have chosen whether or not to discard a card."
    );
    supported("Dokuchi Silencer");
    let mut t = TestGame::new(2);
    let sil = t.battlefield(P0, "Dokuchi Silencer");
    let card = t.hand(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(sil, Entity::Player(P1))]);
    let from = t.asked().len();
    to_combat_damage_triggers(&mut t);
    assert_eq!(t.life(P1), 18);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn evie_frye_targets_when_you_discard_a_creature_card() {
    cr!("603.12", "602.2");
    ruling!(
        "Evie Frye",
        "You don’t choose a target for Evie Frye’s last ability at the time you activate it."
    );
    supported("Evie Frye");
    let mut t = TestGame::new(2);
    let evie = t.battlefield(P0, "Evie Frye");
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.activate(P0, evie, 0, &[]).unwrap();
    no_target_yet(&t, P0, from);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // The Bears can't be blocked this turn.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!t.g.can_block(blocker, bears));
}

#[test]
fn glacial_dragonhunt_targets_when_you_discard_a_nonland_card() {
    cr!("603.12", "601.2c");
    ruling!(
        "Glacial Dragonhunt",
        "You don’t choose a target for Glacial Dragonhunt at the time you cast it."
    );
    supported("Glacial Dragonhunt");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Hill Giant");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Glacial Dragonhunt", &[]);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    let r = top(&t);
    assert!(is_trigger(&t, r));
    assert_eq!(targets_of(&t, r), vec![obj(bears)]);
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn kishla_trawlers_targets_when_you_exile_a_creature_card() {
    cr!("603.12", "115.1");
    ruling!(
        "Kishla Trawlers",
        "You don’t choose a target for Kishla Trawlers’s ability at the time it triggers."
    );
    supported("Kishla Trawlers");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let from = t.asked().len();
    t.enter(P0, "Kishla Trawlers");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bolt)]);
    resolve_into_reflexive(&mut t, &[obj(bolt)]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_hand(P0, "Lightning Bolt"));
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn labyrinth_adversary_targets_when_you_pay() {
    cr!("603.12", "509.1b");
    ruling!(
        "Labyrinth Adversary",
        "You don’t choose a target for Labyrinth Adversary’s last ability at the time it triggers."
    );
    supported("Labyrinth Adversary");
    let mut t = TestGame::new(2);
    let adv = t.battlefield(P0, "Labyrinth Adversary");
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    attack_with(&mut t, &[(adv, Entity::Player(P1))]);
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    resolve_into_reflexive(&mut t, &[obj(bears)]);
    assert!(t.g.can_block_at_all(bears));
    t.resolve_all();
    assert!(!t.g.can_block_at_all(bears));
}

#[test]
fn ruthless_lawbringer_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Ruthless Lawbringer",
        "You don’t choose a target for Ruthless Lawbringer’s ability at the time it triggers."
    );
    supported("Ruthless Lawbringer");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let paci = t.battlefield(P1, "Ornithopter");
    let from = t.asked().len();
    t.enter(P0, "Ruthless Lawbringer");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(paci)]);
    resolve_into_reflexive(&mut t, &[obj(paci)]);
    t.resolve_all();
    assert!(!t.on_battlefield(paci));
}

#[test]
fn selfcraft_mechan_targets_when_you_sacrifice_and_an_illegal_target_means_no_draw() {
    cr!("603.12", "608.2b");
    ruling!(
        "Selfcraft Mechan",
        "You don’t choose a target for Selfcraft Mechan’s ability at the time it triggers."
    );
    supported("Selfcraft Mechan");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let thopter = t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        t.enter(P0, "Selfcraft Mechan");
        t.settle();
        no_target_yet(&t, P0, from);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(thopter)]);
        t.answer_targets(P0, &[obj(bears)]);
        resolve_into_reflexive(&mut t, &[obj(bears)]);
        assert!(!t.on_battlefield(thopter));
        let hand = t.hand_size(P0);
        if respond {
            // The target becomes illegal: the ability doesn't resolve, no card is drawn.
            destroy(&mut t, bears);
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand);
        } else {
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(t.counters(bears, "+1/+1"), 1);
        }
    }
}

#[test]
fn gastal_blockbuster_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Gastal Blockbuster",
        "You don’t choose a target for the enters ability. If you choose to sacrifice a creature or Vehicle"
    );
    supported("Gastal Blockbuster");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let from = t.asked().len();
    t.enter(P0, "Gastal Blockbuster");
    t.settle();
    no_target_yet(&t, P0, from);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(thopter)]);
    resolve_into_reflexive(&mut t, &[obj(thopter)]);
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
}

#[test]
fn tip_the_scales_reflexive_ability_knows_x_before_it_resolves() {
    cr!("603.12", "701.21a", "608.2h");
    ruling!(
        "Tip the Scales",
        "You don’t choose which creature to sacrifice until Tip the Scales resolves."
    );
    supported("Tip the Scales");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let elf = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Tip the Scales", &[]);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    // The giant was sacrificed; the reflexive ability waits on the stack.
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert_eq!(t.pt(elf), (6, 4));
    t.resolve_all();
    assert_eq!(t.pt(elf), (3, 1));
}

#[test]
fn iroh_chooses_targets_as_it_triggers_and_decides_on_resolution() {
    cr!("603.3d", "608.2c");
    ruling!(
        "Iroh, Tea Master",
        "You will choose targets for Iroh's second ability at the beginning of every combat"
    );
    supported("Iroh, Tea Master");
    let mut t = TestGame::new(2);
    let iroh = t.battlefield(P0, "Iroh, Tea Master");
    let thopter = t.battlefield(P0, "Ornithopter");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(thopter)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(target_candidates(&t, P0, from).len(), 2);
    let trig = top(&t);
    assert_eq!(targets_of(&t, trig), vec![Entity::Player(P1), obj(thopter)]);
    // Not feeling generous: decline as it resolves.
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
    assert_eq!(t.obj_now(iroh).controller, P0);
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 0);
}

#[test]
fn royal_scions_draws_and_discards_during_one_resolution() {
    cr!("608.2c", "117.3");
    ruling!(
        "The Royal Scions",
        "You draw a card and discard a card all while The Royal Scions's first ability is resolving."
    );
    supported("The Royal Scions");
    let mut t = TestGame::new(2);
    let scions = t.battlefield(P0, "The Royal Scions");
    let card = t.hand(P0, "Hill Giant");
    let lib = t.library_size(P0);
    // When asked to discard, the card has already been drawn.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }),
        |g| g.player(P0).library.len(),
    );
    t.answer_choose(P0, &[obj(card)]);
    t.activate(P0, scions, 0, &[]).unwrap();
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(seen.lock().unwrap().clone(), vec![lib - 1]);
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn intis_exiled_land_follows_normal_timing_rules() {
    cr!("305.2", "305.1", "603.12");
    ruling!(
        "Inti, Seneschal of the Sun",
        "You pay all costs and follow all normal timing rules for cards played due to the last ability."
    );
    supported("Inti, Seneschal of the Sun");
    let mut t = TestGame::new(2);
    let inti = t.battlefield(P0, "Inti, Seneschal of the Sun");
    let card = t.hand(P0, "Hill Giant");
    let land = t.library_top(P0, "Mountain");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(inti)]);
    attack_with(&mut t, &[(inti, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Mountain"));
    assert_eq!(t.counters(inti, "+1/+1"), 1);
    // During combat the land can't be played.
    assert!(!can_play_land(&mut t, P0, land));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, land));
}

