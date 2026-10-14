//! Rulings batch P129: reflexive triggered abilities choose their targets only when they
//! trigger ("You don't choose a target for ~'s ability at the time it triggers. Rather, a
//! second 'reflexive' ability triggers ..."). The first ability (or spell) has no target;
//! the reflexive one goes on the stack with its targets after the "you may" action is done
//! as the first resolves, and players may respond to it (CR 603.12).

use crate::r_p129_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn undercity_eliminator_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Undercity Eliminator",
        "You don't choose a target at the time Undercity Eliminator's ability triggers."
    );
    supported("Undercity Eliminator");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Undercity Eliminator");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert!(!t.on_battlefield(thopter));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn novel_nunchaku_chooses_the_creature_to_fight_when_it_is_attached() {
    cr!("603.12", "701.14a");
    ruling!(
        "Novel Nunchaku",
        "You don't choose a target creature an opponent controls for Novel Nunchaku's first ability"
    );
    supported("Novel Nunchaku");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    t.answer_targets(P0, &[obj(theirs)]);
    let nun = t.enter(P0, "Novel Nunchaku");
    t.settle();
    // The first ability targets only the creature you control.
    assert_eq!(targets_of(&t, top(&t)), vec![obj(mine)]);
    resolve_into_reflexive(&mut t, &[obj(theirs)]);
    assert_eq!(
        t.obj_now(nun).attached_to,
        Some(Entity::Object(t.g.current(mine)))
    );
    assert!(t.on_battlefield(theirs));
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn hypothesizzle_targets_when_you_discard_and_only_that_is_a_crime() {
    cr!("603.12", "601.2c", "700.13");
    ruling!(
        "Hypothesizzle",
        "You don't choose a target creature at the time you cast Hypothesizzle."
    );
    supported("Hypothesizzle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P0, "Hill Giant");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Hypothesizzle", &[]);
    no_target_yet(&t, P0, from);
    assert_eq!(crimes(&t, P0), 0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    // The reflexive ability targets a creature an opponent controls: that's a crime.
    assert_eq!(crimes(&t, P0), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn faebloom_trick_targets_when_the_faeries_are_created() {
    cr!("603.12", "601.2c");
    ruling!(
        "Faebloom Trick",
        "You don't choose a target creature at the time you cast this spell."
    );
    supported("Faebloom Trick");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Faebloom Trick", &[]);
    no_target_yet(&t, P0, from);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    assert_eq!(creature_tokens(&t, P0), 2);
    assert!(!t.obj_now(bears).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn back_for_more_targets_the_creature_to_fight_when_the_card_returns() {
    cr!("603.12", "700.13", "701.14a");
    ruling!(
        "Back for More",
        "You don't choose a target creature you don't control at the time you cast Back for More."
    );
    supported("Back for More");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Back for More", &[obj(giant)]);
    // Only the card in your graveyard was chosen: casting it isn't a crime.
    assert_eq!(crimes(&t, P0), 0);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    assert_eq!(crimes(&t, P0), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(t.g.current(giant)));
}

#[test]
fn atraxas_skitterfang_targets_when_you_remove_an_oil_counter() {
    cr!("603.12", "608.2d");
    ruling!(
        "Atraxa's Skitterfang",
        "You don't choose a target for Atraxa's Skitterfang's last ability at the time it triggers."
    );
    ruling!(
        "Atraxa's Skitterfang",
        "You choose which ability the target creature gets as the reflexive ability resolves."
    );
    supported("Atraxa's Skitterfang");
    let mut t = TestGame::new(2);
    let fang = t.battlefield(P0, "Atraxa's Skitterfang");
    put_counters(&mut t, fang, counters::OIL, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(t.counters(fang, counters::OIL), 2);
    // No keyword choice has been made yet: it's made as the reflexive ability resolves.
    let before = t.asked().len();
    let choices = |t: &TestGame| {
        asked_since(t, before)
            .iter()
            .filter(|(_, d)| !matches!(d, mtg_engine::decision::Decision::Priority { .. }))
            .count()
    };
    assert_eq!(choices(&t), 0);
    t.resolve_all();
    assert!(choices(&t) >= 1);
    let kws = [
        KeywordKind::Flying,
        KeywordKind::Vigilance,
        KeywordKind::Deathtouch,
        KeywordKind::Lifelink,
    ];
    let got: Vec<_> = kws
        .iter()
        .filter(|k| t.obj_now(bears).has_keyword(**k))
        .collect();
    assert_eq!(got.len(), 1);
}

#[test]
fn augusta_targets_when_nonland_cards_are_exiled() {
    cr!("603.12");
    ruling!(
        "Augusta, Order Returned",
        "You don't choose a target for Augusta's last ability at the time it triggers."
    );
    supported("Augusta, Order Returned");
    let mut t = TestGame::new(2);
    let aug = t.battlefield(P0, "Augusta, Order Returned");
    let mine = t.graveyard(P0, "Lightning Bolt");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(theirs)]);
    t.answer_targets(P0, &[obj(aug)]);
    attack_with(&mut t, &[(aug, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(aug)]);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.counters(aug, counters::PLUS1), 2);
}

#[test]
fn auron_targets_when_the_counter_is_put_on_it() {
    cr!("603.12", "122.1");
    ruling!(
        "Auron, Venerated Guardian",
        "You don't choose a target for Auron's last ability at the time it triggers."
    );
    supported("Auron, Venerated Guardian");
    let mut t = TestGame::new(2);
    let auron = t.battlefield(P0, "Auron, Venerated Guardian");
    // Power 2: only after the counter (power 3) can the 2/2 be targeted.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(auron, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(t.counters(auron, counters::PLUS1), 1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn sacrifice_on_entering_then_damage_any_target() {
    cr!("603.12", "701.21a", "120.3");
    ruling!(
        "Boilerbilges Ripper",
        "You don't choose a target for Boilerbilges Ripper's ability at the time it triggers."
    );
    ruling!(
        "Cornered Crook",
        "You don't choose a target for Cornered Crook's ability at the time it triggers."
    );
    // (card, what to sacrifice, damage dealt)
    for (name, fodder, dmg) in [
        ("Boilerbilges Ripper", "Glorious Anthem", 2),
        ("Cornered Crook", "Ornithopter", 3),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let f = t.battlefield(P0, fodder);
        let from = t.asked().len();
        t.enter(P0, name);
        t.settle();
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(f)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
        assert!(!t.on_battlefield(f), "{name}");
        assert_eq!(t.life(P1), 20);
        t.resolve_all();
        assert_eq!(t.life(P1), 20 - dmg, "{name}");
    }
}

#[test]
fn forgehammer_centurion_targets_when_you_remove_oil_counters() {
    cr!("603.12", "509.1b");
    ruling!(
        "Forgehammer Centurion",
        "You don't choose a target for Forgehammer Centurion's last ability at the time it triggers."
    );
    supported("Forgehammer Centurion");
    let mut t = TestGame::new(2);
    let cent = t.battlefield(P0, "Forgehammer Centurion");
    put_counters(&mut t, cent, counters::OIL, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(cent, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(t.counters(cent, counters::OIL), 0);
    assert!(t.g.can_block_at_all(bears));
    t.resolve_all();
    assert!(!t.g.can_block_at_all(bears));
}

#[test]
fn forum_filibuster_targets_when_the_token_is_created() {
    cr!("603.12", "303.4f");
    ruling!(
        "Forum Filibuster",
        "You don't choose a target for Forum Filibuster's ability at the time it triggers."
    );
    supported("Forum Filibuster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forum Filibuster");
    let sword = t.graveyard(P0, "Bonesplitter");
    t.answer_targets(P0, &[obj(sword)]);
    let from = t.asked().len();
    next_upkeep(&mut t, P0);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(sword)]);
    let inkling = with_subtype(&t, P0, "Inkling");
    assert_eq!(inkling.len(), 1);
    t.resolve_all();
    let s = t.named_on_battlefield("Bonesplitter");
    assert_eq!(s.len(), 1);
    assert_eq!(t.obj_now(s[0]).attached_to, Some(Entity::Object(inkling[0])));
}

#[test]
fn general_traag_targets_when_you_sacrifice_an_artifact() {
    cr!("603.12", "701.21a");
    ruling!(
        "General Traag, Heart of Stone",
        "You don't choose a target for General Traag's last ability at the time it triggers."
    );
    supported("General Traag, Heart of Stone");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.enter(P0, "General Traag, Heart of Stone");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(giant)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn glissas_retriever_targets_when_you_exile_it() {
    cr!("603.12", "603.10a");
    ruling!(
        "Glissa's Retriever",
        "You don't choose a target for Glissa's Retriever's last ability at the time it triggers."
    );
    supported("Glissa's Retriever");
    let mut t = TestGame::new(2);
    let ret = t.battlefield(P0, "Glissa's Retriever");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.g.add_counters(Entity::Player(P1), counters::POISON, 3, None);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bolt)]);
    destroy(&mut t, ret);
    no_target_yet(&t, P0, from);
    let r = resolve_into_reflexive(&mut t, &[obj(bolt)]);
    assert!(t.in_exile("Glissa's Retriever"));
    assert!(t.g.stack.contains(&r));
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn glorifier_of_suffering_chooses_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Glorifier of Suffering",
        "You don't choose a target for Glorifier of Suffering's ability at the time it triggers."
    );
    supported("Glorifier of Suffering");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    let glor = t.enter(P0, "Glorifier of Suffering");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(bears), obj(glor)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears), obj(glor)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.counters(glor, counters::PLUS1), 1);
}

#[test]
fn guide_of_souls_targets_when_you_pay_energy_and_the_angel_type_lasts() {
    cr!("603.12", "611.2a", "514.2");
    ruling!(
        "Guide of Souls",
        "You don't choose a target for Guide of Souls's last ability at the time it triggers."
    );
    ruling!(
        "Guide of Souls",
        "The type-changing effect of the reflexive triggered ability lasts indefinitely."
    );
    supported("Guide of Souls");
    let mut t = TestGame::new(2);
    let guide = t.battlefield(P0, "Guide of Souls");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_energy(&mut t, P0, 3);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    assert_eq!(energy(&t, P0), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(has_subtype(&t, bears, "Angel"));
    // It doesn't wear off in the cleanup step or when Guide of Souls leaves.
    destroy(&mut t, guide);
    t.advance_to(P1, Step::Upkeep);
    assert!(has_subtype(&t, bears, "Angel"));
    assert!(has_subtype(&t, bears, "Bear"));
}

#[test]
fn hurska_targets_when_you_pay_and_uses_the_life_gained() {
    cr!("603.12", "119.10");
    ruling!(
        "Hurska Sweet-Tooth",
        "You don't choose a target for Hurska's last ability at the time it triggers."
    );
    supported("Hurska Sweet-Tooth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hurska Sweet-Tooth");
    t.lands(P0, "Forest", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn inti_targets_when_you_discard() {
    cr!("603.12", "701.9a");
    ruling!(
        "Inti, Seneschal of the Sun",
        "You don't choose a target for Inti, Seneschal of the Sun's first ability at the time it triggers."
    );
    let mut t = TestGame::new(2);
    let inti = t.battlefield(P0, "Inti, Seneschal of the Sun");
    let card = t.hand(P0, "Hill Giant");
    t.library_top(P0, "Mountain");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(inti)]);
    attack_with(&mut t, &[(inti, Entity::Player(P1))]);
    no_target_yet(&t, P0, from);
    let trig = top(&t);
    t.resolve();
    // The reflexive ability and Inti's discard trigger both triggered.
    assert!(!t.g.stack.contains(&trig));
    assert!(t.g.stack.iter().any(|s| targets_of(&t, *s) == vec![obj(inti)]));
    t.resolve_all();
    assert_eq!(t.counters(inti, counters::PLUS1), 1);
}

#[test]
fn leatherhead_targets_when_you_remove_a_counter() {
    cr!("603.12", "510.3a");
    ruling!(
        "Leatherhead, Swamp Stalker",
        "You don't choose a target for Leatherhead's last ability at the time it triggers."
    );
    supported("Leatherhead, Swamp Stalker");
    let mut t = TestGame::new(2);
    let lh = t.enter(P0, "Leatherhead, Swamp Stalker");
    t.settle();
    assert_eq!(t.counters(lh, "hexproof"), 1);
    let sword = t.battlefield(P1, "Bonesplitter");
    t.g.objects[lh.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(lh, Entity::Player(P1))]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(sword)]);
    to_combat_damage_triggers(&mut t);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(sword)]);
    assert_eq!(t.counters(lh, "hexproof"), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(sword));
}

#[test]
fn merry_bards_targets_when_you_pay() {
    cr!("603.12", "303.7");
    ruling!(
        "Merry Bards",
        "You don't choose a target for Merry Bards's ability at the time its enters-the-battlefield ability triggers."
    );
    supported("Merry Bards");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Merry Bards");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    t.resolve_all();
    let roles = with_subtype(&t, P0, "Role");
    assert_eq!(roles.len(), 1);
    assert_eq!(t.obj_now(roles[0]).attached_to, Some(Entity::Object(bears)));
}

#[test]
fn old_man_willow_targets_when_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Old Man Willow",
        "You don't choose a target for Old Man Willow's last ability at the time it triggers."
    );
    supported("Old Man Willow");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let willow = t.battlefield(P0, "Old Man Willow");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(willow, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn riddle_gate_gargoyle_targets_when_you_pay_energy() {
    cr!("603.12", "107.14");
    ruling!(
        "Riddle Gate Gargoyle",
        "You don't choose a target for Riddle Gate Gargoyle's last ability at the time it triggers."
    );
    supported("Riddle Gate Gargoyle");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Riddle Gate Gargoyle");
    give_energy(&mut t, P0, 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(g)]);
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(g)]);
    t.resolve_all();
    assert!(t.obj_now(g).has_keyword(KeywordKind::Lifelink));
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn shire_shirriff_targets_when_you_sacrifice_a_token() {
    cr!("603.12", "701.21a", "610.3");
    ruling!(
        "Shire Shirriff",
        "You don't choose a target for Shire Shirriff's last ability at the time it triggers."
    );
    supported("Shire Shirriff");
    let mut t = TestGame::new(2);
    let tok = create_token(&mut t, P0, "Food");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    let sh = t.enter(P0, "Shire Shirriff");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(tok)]);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    destroy(&mut t, sh);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn sieges_target_with_their_reflexive_abilities() {
    cr!("603.12", "310.12a");
    ruling!(
        "Invasion of New Capenna // Holy Frazzle-Cannon",
        "Invasion of New Capenna’s ability triggers and goes on the stack without a target."
    );
    ruling!(
        "Invasion of Tarkir // Defiant Thundermaw",
        "Invasion of Tarkir’s enters-the-battlefield ability triggers and goes on the stack without a target."
    );
    ruling!(
        "Invasion of Tarkir // Defiant Thundermaw",
        "If you don’t reveal any Dragon cards from your hand, the reflexive triggered ability will cause"
    );
    supported("Invasion of New Capenna // Holy Frazzle-Cannon");
    supported("Invasion of Tarkir // Defiant Thundermaw");
    // New Capenna: sacrifice, then exile the target.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Invasion of New Capenna // Holy Frazzle-Cannon");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer_targets(P0, &[obj(bears)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Tarkir: reveal no Dragons; the reflexive ability still deals 0 + 2 damage.
    let mut t = TestGame::new(2);
    t.hand(P0, "Shivan Dragon");
    let from = t.asked().len();
    t.enter(P0, "Invasion of Tarkir // Defiant Thundermaw");
    t.settle();
    t.answer_choose(P0, &[]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn skyfisher_spider_targets_only_after_you_sacrifice() {
    cr!("603.12", "701.21a");
    ruling!(
        "Skyfisher Spider",
        "Skyfisher Spider's first triggered ability doesn't have any targets."
    );
    ruling!(
        "Skyfisher Spider",
        "players won't get to know what nonland permanent you plan to target before you choose whether to sacrifice"
    );
    supported("Skyfisher Spider");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let paci = t.battlefield(P1, "Glorious Anthem");
    let from = t.asked().len();
    t.enter(P0, "Skyfisher Spider");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(paci)]);
    // The sacrifice is chosen before the target.
    let order = watch(
        &mut t,
        P0,
        |d| {
            matches!(
                d,
                mtg_engine::decision::Decision::ChooseTargets { .. }
                    | mtg_engine::decision::Decision::ChooseEntities { .. }
            )
        },
        |g| g.stack.len(),
    );
    untargeted_then_reflexive(&mut t, P0, from, &[obj(paci)]);
    let seen = order.lock().unwrap().clone();
    assert_eq!(seen.len(), 2);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(paci));
}

#[test]
fn terror_of_towashi_targets_when_you_pay() {
    cr!("603.12");
    ruling!(
        "Terror of Towashi",
        "Terror of Towashi’s triggered ability triggers and goes on the stack without a target."
    );
    supported("Terror of Towashi");
    let mut t = TestGame::new(2);
    let terror = t.battlefield(P0, "Terror of Towashi");
    t.lands(P0, "Swamp", 4);
    let giant = t.graveyard(P0, "Hill Giant");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(terror, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(giant)]);
    t.resolve_all();
    let g = t.g.current(giant);
    assert!(t.on_battlefield(g));
    assert!(has_subtype(&t, g, "Phyrexian"));
}

#[test]
fn royal_scions_ultimate_targets_after_you_draw() {
    cr!("603.12", "120.3");
    ruling!(
        "The Royal Scions",
        "The Royal Scions's last ability goes on the stack without a target."
    );
    let mut t = TestGame::new(2);
    let scions = t.battlefield(P0, "The Royal Scions");
    loyalty(&mut t, scions, 8);
    stack_library(&mut t, P0, &["Island"; 5]);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scions, 2, &[]).unwrap();
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    assert_eq!(t.hand_size(P0), hand + 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - (hand as i32 + 4));

    // Regression: "Draw four cards. When you do, ..." is done only if four cards were
    // drawn. With the first draw replaced by dredge, the reflexive ability doesn't trigger
    // (CR 614.6, 702.52a).
    let mut t = TestGame::new(2);
    let scions = t.battlefield(P0, "The Royal Scions");
    loyalty(&mut t, scions, 8);
    stack_library(&mut t, P0, &["Island"; 10]);
    t.graveyard(P0, "Life from the Loam");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, scions, 2, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_hand(P0, "Life from the Loam"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kardurs_vicious_return_chapter_one_targets_when_you_sacrifice() {
    cr!("603.12", "714.2b");
    ruling!(
        "Kardur's Vicious Return",
        "The chapter I ability goes on the stack without a target."
    );
    supported("Kardur's Vicious Return");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Kardur's Vicious Return");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn kroxa_and_kunoros_targets_when_you_exile_five_cards() {
    cr!("603.12");
    ruling!(
        "Kroxa and Kunoros",
        "The last ability of Kroxa and Kunoros triggers and goes on the stack without a target."
    );
    supported("Kroxa and Kunoros");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    let fodder: Vec<_> = (0..5).map(|_| t.graveyard(P0, "Lightning Bolt")).collect();
    let from = t.asked().len();
    t.enter(P0, "Kroxa and Kunoros");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &fodder.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.answer_targets(P0, &[obj(giant)]);
    untargeted_then_reflexive(&mut t, P0, from, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn slight_malfunction_rolls_then_targets() {
    cr!("603.12", "706.2", "700.2");
    ruling!(
        "Slight Malfunction",
        "If you choose the second mode, Slight Malfunction goes on the stack without any targets."
    );
    supported("Slight Malfunction");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Slight Malfunction");
    let card = t.hand(P0, "Slight Malfunction");
    let from = t.asked().len();
    t.cast(P0, card).modes(&[1]).go();
    no_target_yet(&t, P0, from);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn restoration_of_eiganjo_may_return_the_discarded_card() {
    cr!("603.12", "714.2b");
    ruling!(
        "The Restoration of Eiganjo // Architect of Restoration",
        "Notably, this means you may target the card you discarded"
    );
    supported("The Restoration of Eiganjo // Architect of Restoration");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The Restoration of Eiganjo // Architect of Restoration");
    put_counters(&mut t, saga, counters::LORE, 1);
    t.settle();
    t.resolve_all();
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(t.counters(saga, counters::LORE), 2);
    no_target_yet(&t, P0, from);
    // Discard the Bears; the reflexive ability may then target them in the graveyard
    // (the only permanent card with mana value 2 or less there: the default choice).
    t.resolve();
    let r = top(&t);
    let tg = targets_of(&t, r);
    assert_eq!(tg.len(), 1);
    let Entity::Object(card) = tg[0] else {
        panic!("no card target")
    };
    assert_eq!(t.g.obj(card).chars.name.as_str(), "Grizzly Bears");
    t.resolve_all();
    let on = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on.len(), 1);
    assert!(t.obj_now(on[0]).tapped);
}

#[test]
fn numa_distributes_counters_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "601.2d", "115.7");
    ruling!(
        "Numa, Joraga Chieftain",
        "Numa's triggered ability goes on the stack without a target."
    );
    supported("Numa, Joraga Chieftain");
    let mut t = TestGame::new(2);
    let numa = t.battlefield(P0, "Numa, Joraga Chieftain");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 4);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[obj(numa), obj(elf)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    untargeted_then_reflexive(&mut t, P0, from, &[obj(numa), obj(elf)]);
    t.resolve_all();
    // Each target gets at least one of the two counters.
    assert_eq!(t.counters(numa, counters::PLUS1), 1);
    assert_eq!(t.counters(elf, counters::PLUS1), 1);
}
