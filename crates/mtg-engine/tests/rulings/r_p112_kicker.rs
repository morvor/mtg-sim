//! Rulings batch P112 — kicker cards (CR 702.33): optional kicker costs chosen as the
//! spell is cast (CR 601.2b), targets required only when kicked (CR 601.2c), "if it was
//! kicked" (CR 702.33d), sacrifices chosen in APNAP order (CR 101.4) and a static haste
//! grant that ends when its source leaves (CR 611.3a, 506.4).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::has_kw;
use crate::r_s10_common::attacking;
use crate::r_s25_common::targets_of;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn cast_kicked(t: &mut TestGame, name: &str, kicked: bool) -> ObjectId {
    supported(name);
    let card = t.hand(P0, name);
    t.cast(P0, card).kicked(kicked).go()
}

#[test]
fn benalish_sleeper_sacrifices_are_chosen_in_turn_order_then_happen_together() {
    cr!("101.4", "701.21a");
    ruling!(
        "Benalish Sleeper",
        "As Benalish Sleeper's ability resolves, first the player whose turn it is chooses a creature to sacrifice, then each other player in turn order does the same knowing the choices made before them. Then all those creatures are sacrificed simultaneously."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Ornithopter");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    cast_kicked(&mut t, "Benalish Sleeper", true);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    // When P1 chooses, P0's choice (the Bears) hasn't been sacrificed yet.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.battlefield.len(),
    );
    let from = t.asked().len();
    t.answer_choose(P0, &[bears.into()]);
    t.answer_choose(P1, &[giant.into()]);
    t.resolve_all();
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P0, P1]);
    assert_eq!(seen.lock().unwrap().len(), 1);
    // Bears, Sleeper, Hill Giant, Ornithopter all still there as P1 chose.
    assert_eq!(seen.lock().unwrap()[0], 4);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.named_on_battlefield("Benalish Sleeper").len(), 1);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn benalish_sleeper_may_or_must_sacrifice_itself() {
    cr!("701.21a", "702.33d");
    ruling!(
        "Benalish Sleeper",
        "If you kicked it, you may sacrifice Benalish Sleeper itself as its ability resolves. If you control no other creatures, you'll have to sacrifice Benalish Sleeper."
    );
    // No other creatures: it must go.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    cast_kicked(&mut t, "Benalish Sleeper", true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Benalish Sleeper"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // With another creature, it may still choose itself.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    cast_kicked(&mut t, "Benalish Sleeper", true);
    t.resolve();
    let sleeper = t.named_on_battlefield("Benalish Sleeper")[0];
    t.answer_choose(P0, &[sleeper.into()]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Benalish Sleeper"));
    assert!(t.on_battlefield(bears));
}

/// P0 casts Keldon Strike Team kicked and resolves it (and its trigger).
fn kicked_strike_team(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    cast_kicked(t, "Keldon Strike Team", true);
    t.resolve_all();
    t.named_on_battlefield("Keldon Strike Team")[0]
}

#[test]
fn keldon_strike_team_hastes_itself_its_tokens_and_later_creatures() {
    cr!("702.10b", "611.3a", "302.6");
    ruling!(
        "Keldon Strike Team",
        "Creatures that enter the battlefield after Keldon Strike Team on the turn it entered the battlefield will have haste as long as it's still on the battlefield. Notably, this includes the Soldier creature tokens it creates if it was kicked."
    );
    ruling!(
        "Keldon Strike Team",
        "Keldon Strike Team's last ability grants itself haste in addition to any other creatures you control."
    );
    let mut t = TestGame::new(2);
    let team = kicked_strike_team(&mut t);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    let later = t.battlefield_sick(P0, "Hill Giant");
    for id in toks.iter().copied().chain([team, later]) {
        assert!(has_kw(&t, id, KeywordKind::Haste));
    }
    // Once Keldon Strike Team leaves, the later creature loses haste.
    destroy(&mut t, team);
    assert!(!has_kw(&t, later, KeywordKind::Haste));
}

#[test]
fn keldon_strike_team_leaving_during_combat_doesnt_remove_attackers() {
    cr!("506.4", "302.6");
    ruling!(
        "Keldon Strike Team",
        "If Keldon Strike Team leaves the battlefield during combat during the turn it entered the battlefield, any attacking creatures that came under your control this turn will continue to attack, even if they no longer have haste."
    );
    let mut t = TestGame::new(2);
    let team = kicked_strike_team(&mut t);
    let toks = tokens(&t, P0);
    let mut attackers: Vec<(ObjectId, Entity)> =
        toks.iter().map(|id| (*id, Entity::Player(P1))).collect();
    attackers.push((team, Entity::Player(P1)));
    attack_with(&mut t, &attackers);
    destroy(&mut t, team);
    for id in &toks {
        assert!(!has_kw(&t, *id, KeywordKind::Haste));
        assert!(attacking(&t, *id));
    }
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn thornscape_battlemage_put_onto_the_battlefield_is_not_kicked() {
    cr!("702.33d", "603.4");
    ruling!(
        "Thornscape Battlemage",
        "If Thornscape Battlemage is put onto the battlefield as the result of a spell or ability, there's no opportunity to kick it."
    );
    supported("Thornscape Battlemage");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    t.enter(P0, "Thornscape Battlemage");
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn thornscape_battlemage_kicking_is_chosen_as_cast_and_optional() {
    cr!("601.2b", "702.33a");
    ruling!(
        "Thornscape Battlemage",
        "You choose whether to kick a spell as you cast it, and you pay that much along with the spell's mana cost at the same time. Kicking a spell is always optional."
    );
    // Not kicked: no triggers, and only its mana cost is paid.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Thornscape Battlemage");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, card).go();
    let offered = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::OptionalCost { .. }))
        .count();
    assert_eq!(offered, 2);
    assert_eq!(tapped_lands(&t, P0), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.named_on_battlefield("Thornscape Battlemage").len(), 1);
    // {R} kicker: paid with the mana cost as it's cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Thornscape Battlemage");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, card).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn urborg_repossession_targets() {
    cr!("601.2c", "702.33d");
    ruling!(
        "Urborg Repossession",
        "If it isn't kicked, Urborg Repossession has only one target."
    );
    ruling!(
        "Urborg Repossession",
        "You can't cast Urborg Repossession kicked if there aren't both a creature card and another permanent card in your graveyard."
    );
    // Unkicked: one target.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Ornithopter");
    t.lands(P0, "Swamp", 1);
    supported("Urborg Repossession");
    let card = t.hand(P0, "Urborg Repossession");
    let spell = t.cast(P0, card).kicked(false).target(bears).go();
    assert_eq!(targets_of(&t, spell), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.life(P0), 22);
    // Kicked needs both a creature card and another permanent card.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Urborg Repossession");
    let r = t.cast(P0, card).kicked(true).target(bears).try_go();
    assert!(r.is_err());
    assert_eq!(t.zone(card), Zone::Hand(P0));
}

#[test]
fn dismantling_blow_draws_only_if_its_target_is_legal() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Dismantling Blow",
        "If the target artifact or enchantment is an illegal target by the time Dismantling Blow tries to resolve, the spell doesn't resolve. You won't draw two cards if it was kicked. If the target is legal but not destroyed (most likely because it has indestructible), you do draw two cards."
    );
    for (name, spoil, draws) in [("Ornithopter", true, 0), ("Darksteel Myr", false, 2)] {
        let mut t = TestGame::new(2);
        let target = t.battlefield(P1, name);
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 4);
        let hand = t.hand_size(P0);
        supported("Dismantling Blow");
        let card = t.hand(P0, "Dismantling Blow");
        t.cast(P0, card).kicked(true).target(target).go();
        if spoil {
            destroy(&mut t, target);
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{name}");
        if !spoil {
            assert!(t.on_battlefield(target));
        }
    }
}
