//! Rulings batch P186 — typal Vampire and Warrior cards: counts made as an ability
//! resolves (CR 608.2h), linked abilities granted to several tokens (CR 607.2), "dies"
//! triggers for permanents you control but don't own, simultaneous sacrifices (CR
//! 101.4), casting restrictions checked as the spell is cast (CR 601.3), targets chosen
//! as an ability is put on the stack and costs paid on resolution (CR 603.3d, 117.12),
//! intervening "if" clauses (CR 603.4) and marked damage after a lord leaves (CR 120.6).

use crate::r_p076_common::mana;
use crate::r_p185_common::*;
use crate::r_p186_common::*;
use crate::r_p189_common::stack_triggers_from;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::give_control;
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn bishop_of_the_bloodstained_counts_vampires_as_it_resolves_including_itself() {
    cr!("608.2h", "603.3");
    ruling!(
        "Bishop of the Bloodstained",
        "The number of Vampires you control is counted only as Bishop of the Bloodstained's ability resolves. If Bishop of the Bloodstained is still on the battlefield, it'll count itself."
    );
    supported("Bishop of the Bloodstained");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arrogant Vampire");
    let bishop = enter_with(&mut t, P0, "Bishop of the Bloodstained", &[Entity::Player(P1)]);
    assert_eq!(stack_triggers_from(&t, bishop).len(), 1);
    // A Vampire that arrives while the ability is on the stack is counted.
    t.battlefield(P0, "Highborn Vampire");
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "Bishop, Arrogant Vampire and Highborn Vampire");
    // Bishop gone before it resolves: it isn't counted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arrogant Vampire");
    let bishop = enter_with(&mut t, P0, "Bishop of the Bloodstained", &[Entity::Player(P1)]);
    destroy(&mut t, bishop);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// P0 controls Timothar and the Vampire `vampire` (owned by `owner`), pays {1} as it dies
/// and exiles it. Returns the game and the Bat tokens created.
fn timothar_exiles(vampire_owner: PlayerId, extra: &[&str]) -> (TestGame, Vec<ObjectId>) {
    supported("Timothar, Baron of Bats");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Timothar, Baron of Bats");
    for e in extra {
        t.battlefield(P0, e);
    }
    let vampire = t.battlefield(vampire_owner, "Arrogant Vampire");
    if vampire_owner != P0 {
        give_control(&mut t, vampire, P0);
    }
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    destroy(&mut t, vampire);
    t.resolve_all();
    let bats = with_subtype(&t, P0, "Bat");
    for b in &bats {
        t.g.objects[b.0 as usize].summoning_sick = false;
    }
    (t, bats)
}

#[test]
fn timothar_exiles_a_vampire_it_controls_but_doesnt_own() {
    cr!("603.10a", "700.4");
    ruling!(
        "Timothar, Baron of Bats",
        "Timothar's last ability triggers when any nontoken Vampire you control dies, not just nontoken Vampires you own. If you control Timothar and another player's nontoken Vampire and that Vampire dies, you may pay {1} to exile that card from their graveyard."
    );
    let (t, bats) = timothar_exiles(P1, &[]);
    assert_eq!(bats.len(), 1);
    assert!(t.in_exile("Arrogant Vampire"));
    assert!(!t.in_graveyard(P1, "Arrogant Vampire"));
}

#[test]
fn timothar_every_bat_a_replacement_effect_creates_has_the_linked_ability() {
    cr!("607.2a", "614.1a", "510.2");
    ruling!(
        "Timothar, Baron of Bats",
        "Timothar's last ability is linked to the ability it grants the Bat token. If a replacement effect causes you to create more than one Bat token with this effect, they will all have that linked ability, and the exiled card will be returned to the battlefield if any of them deals combat damage to a player."
    );
    let (mut t, bats) = timothar_exiles(P0, &["Parallel Lives"]);
    assert_eq!(bats.len(), 2, "Parallel Lives doubles the Bat");
    assert!(t.in_exile("Arrogant Vampire"));
    // The second Bat (not the first) connects.
    attack_with(&mut t, &[(bats[1], Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(!t.on_battlefield(bats[1]), "the Bat that connected is sacrificed");
    assert!(t.on_battlefield(bats[0]));
    let back = t.named_on_battlefield("Arrogant Vampire");
    assert_eq!(back.len(), 1, "the exiled card returns");
    assert!(t.g.obj(back[0]).tapped);
}

#[test]
fn anowon_choices_are_made_in_turn_order_then_sacrificed_at_once() {
    cr!("101.4", "701.21a");
    ruling!(
        "Anowon, the Ruin Sage",
        "When the triggered ability resolves, first you choose which creature you'll sacrifice (if you control any non-Vampire creatures), then each other player in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    supported("Anowon, the Ruin Sage");
    let mut t = TestGame::new(2);
    let anowon = t.battlefield(P0, "Anowon, the Ruin Sage");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let keep = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let theirs2 = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.answer_choose(P1, &[Entity::Object(theirs2)]);
    // When P1 chooses, P0's chosen creature hasn't been sacrificed yet.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| creature_count(g, P0),
    );
    let from = t.asked().len();
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
    assert_eq!(*seen.lock().unwrap(), vec![3]);
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs2));
    assert!(t.on_battlefield(keep) && t.on_battlefield(theirs) && t.on_battlefield(anowon));
}

#[test]
fn feast_of_blood_checks_its_vampires_only_as_it_is_cast() {
    cr!("601.3", "608.2b");
    ruling!(
        "Feast of Blood",
        "Whether you control two or more Vampires is checked only as you try to move Feast of Blood to the stack as the first step of casting it."
    );
    supported("Feast of Blood");
    let mut t = TestGame::new(2);
    let v1 = t.battlefield(P0, "Arrogant Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // One Vampire: it can't be cast.
    let card = t.hand(P0, "Feast of Blood");
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Feast of Blood");
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        card,
        mtg_engine::object::CastMethod::Normal
    ));
    t.battlefield(P0, "Highborn Vampire");
    t.cast(P0, card).target(bears).go();
    // A Vampire leaves before it resolves: it still resolves.
    destroy(&mut t, v1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn kalastria_highborn_targets_on_trigger_and_pays_once_on_resolution() {
    cr!("603.3d", "118.12");
    ruling!(
        "Kalastria Highborn",
        "You target a player when the ability triggers. You choose whether to pay {B} as the ability resolves. You may pay {B} only once per resolution."
    );
    supported("Kalastria Highborn");
    let mut t = TestGame::new(2);
    let kalastria = t.battlefield(P0, "Kalastria Highborn");
    let v = t.battlefield(P0, "Arrogant Vampire");
    mana(&mut t, P0, ManaType::B, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    destroy(&mut t, v);
    let trig = stack_triggers_from(&t, kalastria);
    assert_eq!(trig.len(), 1);
    assert_eq!(targets_of(&t, trig[0]), vec![Entity::Player(P1)]);
    // Nothing about paying was asked yet.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::YesNo { .. })));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
    assert_eq!(
        crate::r_s24_common::pool(&t, P0, ManaType::B),
        1,
        "{{B}} is paid only once"
    );
}

#[test]
fn kargan_warleader_marked_damage_becomes_lethal_when_it_leaves() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Kargan Warleader",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to another Warrior you control may become lethal if Kargan Warleader leaves the battlefield during that turn."
    );
    supported("Kargan Warleader");
    let mut t = TestGame::new(2);
    let kargan = t.battlefield(P0, "Kargan Warleader");
    let warrior = vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    assert_eq!(t.pt(warrior), (3, 3));
    lethal_when_it_leaves(&mut t, warrior, kargan, "Kargan Warleader");
}

#[test]
fn warrior_conditions_dont_scale_with_more_warriors() {
    cr!("613.4c", "603.2");
    ruling!(
        "Expedition Champion",
        "Expedition Champion gets just +2/+0, no matter how many other Warriors you control beyond the first."
    );
    ruling!(
        "Paired Tactician",
        "Paired Tactician gets just one +1/+1 counter from its ability, no matter how many Warriors attack with it beyond the first."
    );
    supported("Expedition Champion");
    supported("Paired Tactician");
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Expedition Champion");
    assert_eq!(t.pt(champ), (2, 3));
    vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    assert_eq!(t.pt(champ), (4, 3));
    vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    assert_eq!(t.pt(champ), (4, 3));
    // Paired Tactician attacking with three other Warriors.
    let mut t = TestGame::new(2);
    let tac = t.battlefield(P0, "Paired Tactician");
    let mut attackers = vec![(tac, Entity::Player(P1))];
    for _ in 0..3 {
        let w = vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
        attackers.push((w, Entity::Player(P1)));
    }
    attack_with(&mut t, &attackers);
    t.resolve_all();
    assert_eq!(t.counters(tac, counters::PLUS1), 1);
}

#[test]
fn paired_tactician_other_warrior_may_attack_someone_else() {
    cr!("508.1", "603.2");
    ruling!(
        "Paired Tactician",
        "The other attacking Warrior doesn’t have to be attacking the same player or planeswalker as Paired Tactician."
    );
    let mut t = TestGame::new(3);
    let tac = t.battlefield(P0, "Paired Tactician");
    let w = vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    attack_with(&mut t, &[(tac, Entity::Player(P1)), (w, Entity::Player(P2))]);
    t.resolve_all();
    assert_eq!(t.counters(tac, counters::PLUS1), 1);
    // Alone, it gets nothing.
    let mut t = TestGame::new(3);
    let tac = t.battlefield(P0, "Paired Tactician");
    attack_with(&mut t, &[(tac, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(tac, counters::PLUS1), 0);
}

#[test]
fn sickle_dancer_checks_on_trigger_and_resolution_and_keeps_the_bonus() {
    cr!("603.4", "611.2c");
    ruling!(
        "Sickle Dancer",
        "Whether your team controls another Warrior is checked as Sickle Dancer’s triggered ability triggers and again as it resolves. Once it gets +1/+1 this way, it won’t lose it, even if your team loses its other Warriors."
    );
    supported("Sickle Dancer");
    // No other Warrior: no trigger.
    let mut t = TestGame::new(2);
    let dancer = t.battlefield(P0, "Sickle Dancer");
    attack_with(&mut t, &[(dancer, Entity::Player(P1))]);
    assert_eq!(stack_triggers_from(&t, dancer).len(), 0);
    // It resolves with another Warrior, which then leaves: the bonus stays.
    let mut t = TestGame::new(2);
    let dancer = t.battlefield(P0, "Sickle Dancer");
    let w = vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    attack_with(&mut t, &[(dancer, Entity::Player(P1))]);
    assert_eq!(stack_triggers_from(&t, dancer).len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(dancer), (4, 3));
    destroy(&mut t, w);
    assert_eq!(t.pt(dancer), (4, 3));
    // The Warrior leaves while the trigger is on the stack: no bonus.
    let mut t = TestGame::new(2);
    let dancer = t.battlefield(P0, "Sickle Dancer");
    let w = vanilla(&mut t, P0, "Creature — Human Warrior", (2, 2));
    attack_with(&mut t, &[(dancer, Entity::Player(P1))]);
    assert_eq!(stack_triggers_from(&t, dancer).len(), 1);
    destroy(&mut t, w);
    t.resolve_all();
    assert_eq!(t.pt(dancer), (3, 2));
}
