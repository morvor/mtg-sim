//! Rulings batch P163 — damage and life: creatures dealing noncombat damage to each other
//! (CR 120.2b, 120.3, 702.2b, 702.15b, 702.80, 702.90, 704.3); divided damage (CR
//! 601.2d, 608.2b); life-gain events and triggers (CR 119.9, 119.10, 603.2); Two-Headed
//! Giant shared life (CR 810.9); "if you have at least N life more than your starting
//! life total" (CR 119.1a, 608.2c); cast triggers and power on the stack (CR 601.2i,
//! 603.3, 604.3, 208.2a, 122.6).

use crate::r_p163_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Alpha Brawl for P0 targeting `target` and resolves it.
fn alpha_brawl(t: &mut TestGame, target: ObjectId) {
    supported("Alpha Brawl");
    cast_resolve(t, P0, "Alpha Brawl", &[obj(target)]);
}

#[test]
fn alpha_brawl_damage_is_undivided_sequential_and_noncombat() {
    cr!("120.2b", "608.2h", "704.3");
    ruling!(
        "Alpha Brawl",
        "The damage dealt by the targeted creature is not divided in any way. For example, if a 3/3 creature is targeted, it will deal 3 damage to each other creature controlled by its controller."
    );
    ruling!(
        "Alpha Brawl",
        "The two parts of Alpha Brawl’s effect happen sequentially: first the targeted creature deals damage, then the other creatures deal damage. State-based actions aren’t checked in the middle, so creatures that are dealt lethal damage by the targeted creature will still deal damage."
    );
    ruling!("Alpha Brawl", "None of the damage dealt due to Alpha Brawl is combat damage.");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    alpha_brawl(&mut t, giant);
    let by_giant = damage_by(&t, giant);
    assert_eq!(by_giant.len(), 2);
    assert!(by_giant.contains(&(obj(wurm), 3, false)));
    assert!(by_giant.contains(&(obj(bears), 3, false)));
    // The Bears were dealt lethal damage but still dealt 2 back.
    assert_eq!(damage_by(&t, bears), vec![(obj(giant), 2, false)]);
    assert_eq!(damage_by(&t, wurm), vec![(obj(giant), 6, false)]);
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(bears));
    assert_eq!(marked(&t, wurm), 3);
}

#[test]
fn alpha_brawl_ignores_combat_abilities() {
    cr!("702.4b", "702.19a", "120.2b");
    ruling!(
        "Alpha Brawl",
        "Combat abilities like first strike, double strike, and trample don’t apply and will have no impact on Alpha Brawl’s effect."
    );
    supported("Boros Swiftblade");
    // A 1/2 double striker deals its damage once, at the same time as the others.
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P1, "Boros Swiftblade");
    let bears = t.battlefield(P1, "Grizzly Bears");
    alpha_brawl(&mut t, blade);
    assert_eq!(damage_by(&t, blade), vec![(obj(bears), 1, false)]);
    assert_eq!(damage_by(&t, bears), vec![(obj(blade), 2, false)]);
    assert_eq!(marked(&t, bears), 1);
}

#[test]
fn alpha_brawl_lifelink_deathtouch_wither() {
    cr!("702.2b", "702.15b", "702.80a", "120.3d");
    ruling!(
        "Alpha Brawl",
        "Abilities like lifelink and deathtouch that care about any kind of damage are taken into account."
    );
    ruling!(
        "Alpha Brawl",
        "If the targeted creature has infect or wither, the damage it deals will result in -1/-1 counters being put on each creature it deals damage to. The power of those creatures will be reduced when they deal damage back to the targeted creature."
    );
    // Vampire Nighthawk (lifelink, deathtouch) against a Craw Wurm.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    alpha_brawl(&mut t, hawk);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 22);
    // Boggart Ram-Gang (wither, 3/3) against a Craw Wurm (6/4): the Wurm deals 3 back.
    let mut t = TestGame::new(2);
    let gang = t.battlefield(P1, "Boggart Ram-Gang");
    let wurm = t.battlefield(P1, "Craw Wurm");
    alpha_brawl(&mut t, gang);
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert_eq!(damage_by(&t, wurm), vec![(obj(gang), 3, false)]);
    assert!(!t.on_battlefield(gang));
    // Blight Mamba (infect, 1/1) against a Hill Giant: the Giant deals 2 back.
    let mut t = TestGame::new(2);
    let mamba = t.battlefield(P1, "Blight Mamba");
    let giant = t.battlefield(P1, "Hill Giant");
    alpha_brawl(&mut t, mamba);
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(damage_by(&t, giant), vec![(obj(mamba), 2, false)]);
}

/// The divisions asked since decision `from`: (total, recipients, min each).
fn divisions(t: &TestGame, from: usize) -> Vec<(u32, usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

#[test]
fn monstrous_onslaught_divides_x_as_cast() {
    cr!("601.2d", "107.3a", "608.2b");
    ruling!(
        "Monstrous Onslaught",
        "You divide the damage among the target creatures as you cast Monstrous Onslaught. Each target must be assigned at least 1 damage."
    );
    ruling!(
        "Monstrous Onslaught",
        "The value of X is determined at the time you divide damage. It won't change later, even if the greatest power among creatures you control changes."
    );
    ruling!(
        "Monstrous Onslaught",
        "If some of the targets become illegal, the original division of damage still applies. The damage that was divided among the illegal targets simply isn't dealt; it isn't re-assigned to remaining legal targets."
    );
    supported("Monstrous Onslaught");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Craw Wurm");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    let c = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2, 1]));
    let from = n_asked(&t);
    cast_new(&mut t, P0, "Monstrous Onslaught", &[]);
    assert_eq!(divisions(&t, from), vec![(6, 3, 1)]);
    // The Wurm shrinks and the first target leaves: the others are dealt what was
    // assigned to them.
    crate::r_p057_common::pump(&mut t, mine, -5, 0);
    move_to(&mut t, a, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!((marked(&t, b), marked(&t, c)), (2, 1));
}

#[test]
fn monstrous_onslaught_deals_the_damage_itself() {
    cr!("120.3", "702.15b", "608.2h");
    ruling!(
        "Monstrous Onslaught",
        "The damage is dealt by Monstrous Onslaught, not by the creature with the greatest power among creatures you control."
    );
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    cast_resolve(&mut t, P0, "Monstrous Onslaught", &[obj(wurm)]);
    // The Wurm was dealt 2, by the spell (no deathtouch, no lifelink).
    assert_eq!(marked(&t, wurm), 2);
    assert!(damage_by(&t, hawk).is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ral_divided_damage_with_an_illegal_target() {
    cr!("601.2d", "608.2b");
    ruling!(
        "Ral, Caller of Storms",
        "If some of the targets become illegal for Ral's second ability, the original division of damage still applies, and the damage that would have been dealt to illegal targets isn't dealt at all."
    );
    supported("Ral, Caller of Storms");
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral, Caller of Storms");
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.activate(P0, ral, 1, &[]).unwrap();
    move_to(&mut t, a, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(marked(&t, b), 1);
}

/// Ajani, Strength of the Pride for P0, and an Ajani's Pridemate token from its −2.
fn ajani_and_pridemate(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Ajani, Strength of the Pride");
    let a = t.battlefield(P0, "Ajani, Strength of the Pride");
    activate_resolve(t, P0, a, 1, &[]);
    let token = tokens(t, P0)[0];
    assert_eq!(t.obj(token).chars.name, "Ajani's Pridemate");
    t.g.objects[a.0 as usize].activations_this_turn.clear();
    (a, token)
}

#[test]
fn ajanis_pridemate_token_is_a_mana_value_0_token() {
    cr!("111.4", "202.3b", "111.10");
    ruling!(
        "Ajani, Strength of the Pride",
        "The tokens created by Ajani's second ability are similar to the card Ajani's Pridemate, but they have no mana cost and their mana value is 0."
    );
    let mut t = TestGame::new(2);
    let (_, token) = ajani_and_pridemate(&mut t);
    let o = t.obj(token);
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.mana_value(), 0);
    assert_eq!(t.pt(token), (2, 2));
    assert!(o.chars.subtypes.iter().any(|s| s == "Cat"));
}

#[test]
fn pridemate_triggers_once_per_life_gain_event() {
    cr!("119.9", "119.10", "702.15b", "603.2c");
    ruling!(
        "Ajani, Strength of the Pride",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Ajani's Pridemate's ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Ajani, Strength of the Pride",
        "The ability of Ajani's Pridemate triggers just once for each life-gaining event, whether it's 1 life from Impassioned Orator or 4 life from Dawning Angel. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Ajani's Pridemate triggers only once."
    );
    // Two lifelinkers unblocked: two triggers.
    let mut t = TestGame::new(2);
    let (_, token) = ajani_and_pridemate(&mut t);
    let h1 = t.battlefield(P0, "Vampire Nighthawk");
    let h2 = t.battlefield(P0, "Vampire Nighthawk");
    attack_with(&mut t, &[(h1, Entity::Player(P1)), (h2, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(life_gains(&t, P0), vec![2, 2]);
    assert_eq!(t.counters(token, counters::PLUS1), 2);
    // One lifelinker blocked by two creatures: one trigger.
    let mut t = TestGame::new(2);
    let (_, token) = ajani_and_pridemate(&mut t);
    let h = t.battlefield(P0, "Child of Night");
    let l1 = t.battlefield(P1, "Savannah Lions");
    let l2 = t.battlefield(P1, "Savannah Lions");
    attack_with(&mut t, &[(h, Entity::Player(P1))]);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 1]));
    block_and_finish(&mut t, P1, &[(l1, h), (l2, h)]);
    t.resolve_all();
    assert!(!t.on_battlefield(l1) && !t.on_battlefield(l2));
    assert_eq!(life_gains(&t, P0), vec![2]);
    assert_eq!(t.counters(token, counters::PLUS1), 1);
    // Impassioned Orator (1 life) and Dawning Angel (4 life): one trigger each.
    supported("Impassioned Orator");
    supported("Dawning Angel");
    let mut t = TestGame::new(2);
    let (_, token) = ajani_and_pridemate(&mut t);
    t.battlefield(P0, "Impassioned Orator");
    enter(&mut t, P0, "Dawning Angel");
    t.resolve_all();
    let mut gains = life_gains(&t, P0);
    gains.sort();
    assert_eq!(gains, vec![1, 4]);
    assert_eq!(t.counters(token, counters::PLUS1), 2);
    // Ajani's +1 with three creatures and Ajani: one event of 4.
    let mut t = TestGame::new(2);
    let (a, token) = ajani_and_pridemate(&mut t);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, a, 0, &[]);
    assert_eq!(life_gains(&t, P0), vec![4]);
    assert_eq!(t.counters(token, counters::PLUS1), 1);
}

#[test]
fn ajani_counts_a_creature_planeswalker_twice() {
    cr!("306.1", "608.2h");
    ruling!(
        "Ajani, Strength of the Pride",
        "If a planeswalker you control is also a creature (most likely because it's Gideon), it will be counted twice as Ajani's first ability resolves."
    );
    supported("Gideon Blackblade");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ajani, Strength of the Pride");
    let g = t.battlefield(P0, "Gideon Blackblade");
    assert!(t.obj(g).is(CardType::Creature) && t.obj(g).is(CardType::Planeswalker));
    activate_resolve(&mut t, P0, a, 0, &[]);
    // Gideon (creature and planeswalker) and Ajani: 3.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn ajani_ultimate_checks_life_on_resolution() {
    cr!("119.1", "608.2c");
    ruling!(
        "Ajani, Strength of the Pride",
        "Whether you have at least 15 life more than your starting life total is determined as Ajani's last ability resolves."
    );
    for (before, after, exiles) in [(20, 35, true), (35, 34, false)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Ajani, Strength of the Pride");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let relic = t.battlefield(P1, "Darksteel Relic");
        let land = t.battlefield(P1, "Forest");
        t.g.players[0].life = before;
        t.activate(P0, a, 2, &[]).unwrap();
        t.g.players[0].life = after;
        t.resolve_all();
        assert_eq!(!t.on_battlefield(wurm), exiles, "{before} -> {after}");
        assert_eq!(!t.on_battlefield(relic), exiles);
        assert_eq!(!t.on_battlefield(a), exiles);
        assert!(t.on_battlefield(land));
    }
}

#[test]
fn ajani_in_two_headed_giant() {
    cr!("810.9", "810.9a", "119.1a", "603.2");
    ruling!(
        "Ajani, Strength of the Pride",
        "In a Two-Headed Giant game, Ajani's last ability exiles permanents as long as your team's life total is greater than your team's starting life total by 15 or more life."
    );
    ruling!(
        "Ajani, Strength of the Pride",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Ajani's Pridemate's ability to trigger, even though it will cause your team's life total to increase."
    );
    for (life, exiles) in [(45, true), (44, false)] {
        let mut t = two_headed_giant();
        assert_eq!(t.life(P0), 30);
        let a = t.battlefield(P0, "Ajani, Strength of the Pride");
        let wurm = t.battlefield(P2, "Craw Wurm");
        t.g.players[0].life = life;
        t.g.players[1].life = life;
        activate_resolve(&mut t, P0, a, 2, &[]);
        assert_eq!(!t.on_battlefield(wurm), exiles, "{life}");
    }
    // The teammate gains life: the team's life total rises, P0's Pridemate doesn't
    // trigger.
    let mut t = two_headed_giant();
    let (_, token) = ajani_and_pridemate(&mut t);
    crate::r_p120_common::gain_life(&mut t, P1, 3);
    assert_eq!(t.life(P0), 33);
    assert_eq!(stacked_triggers(&t), 0);
    assert_eq!(t.counters(token, counters::PLUS1), 0);
    crate::r_p120_common::gain_life(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(token, counters::PLUS1), 1);
}

#[test]
fn sarkhans_unsealing_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("601.2i", "603.3", "405.5");
    ruling!(
        "Sarkhan's Unsealing",
        "Each of the triggered abilities of Sarkhan’s Unsealing resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Sarkhan's Unsealing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarkhan's Unsealing");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = cast_new(&mut t, P0, "Craw Wurm", &[]);
    t.settle();
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(stacked_triggers(&t), 1);
    counter(&mut t, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P0, "Craw Wurm"));
}

#[test]
fn sarkhans_unsealing_uses_the_power_on_the_stack() {
    cr!("604.3", "208.2a", "122.6", "601.2i");
    ruling!(
        "Sarkhan's Unsealing",
        "If a creature card’s power is written as * and an ability defines its power, that ability applies while the creature spell is on the stack. Sarkhan’s unsealing checks its power only as you finish paying costs; it doesn’t matter what the spell’s power is as you begin to cast it or when the triggered ability resolves."
    );
    ruling!(
        "Sarkhan's Unsealing",
        "If you cast a creature spell that will enter the battlefield with a number of +1/+1 counters, such as Hungering Hydra, those counters aren’t considered when determining whether either ability of Sarkhan’s Unsealing triggers. Similarly, effects that will raise the creature’s power once it has entered the battlefield won’t apply."
    );
    // Serra Avatar (power = your life total, 20): the 7-or-greater trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarkhan's Unsealing");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Serra Avatar", &[]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    // Losing life before the trigger resolves doesn't matter.
    t.g.players[0].life = 5;
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(bears));
    // Hungering Hydra with X = 7 (a 0/0 on the stack): no trigger.
    supported("Hungering Hydra");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarkhan's Unsealing");
    lands_for_cost(&mut t, P0, "Hungering Hydra");
    t.lands(P0, "Wastes", 7);
    let hh = t.hand(P0, "Hungering Hydra");
    t.cast(P0, hh).x(7).go();
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hungering Hydra").len(), 1);
    // A 3/3 with Glorious Anthem out (it'll be a 4/4 on the battlefield): no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarkhan's Unsealing");
    t.battlefield(P0, "Glorious Anthem");
    cast_new(&mut t, P0, "Hill Giant", &[]);
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
}

#[test]
fn sarkhans_unsealing_in_two_headed_giant() {
    cr!("810.9", "810.9a", "120.3a");
    ruling!(
        "Sarkhan's Unsealing",
        "In a Two-Headed Giant game, the last ability of Sarkhan’s Unsealing causes the opposing team to lose 8 life."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Sarkhan's Unsealing");
    let life = t.life(P2);
    cast_new(&mut t, P0, "Serra Avatar", &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), life - 8);
    assert_eq!(t.life(P3), life - 8);
}
