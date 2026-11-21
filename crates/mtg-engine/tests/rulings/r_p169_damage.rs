//! Rulings batch P169 — damage-modifying replacement effects (CR 614.1a, 616.1, 615):
//! the affected player or controller orders them, prevention can make them no longer
//! apply, divided damage is divided first, and Soul-Scar Mage's counters (CR 120.3,
//! 701.14 fights).

use crate::r_p169_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts the back face of a transforming double-faced card onto the battlefield.
fn back_face(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    enter_with(t, p, name, 0, true)
}

/// P0's Artist's Talent at level 3.
fn artists_talent_level_3(t: &mut TestGame) -> ObjectId {
    supported("Artist's Talent");
    let talent = t.battlefield(P0, "Artist's Talent");
    for level in 0..2 {
        mana(t, P0, ManaType::R, 3);
        t.activate(P0, talent, level, &[]).unwrap();
        t.resolve_all();
    }
    talent
}

/// P1 gives itself Healing Salve's "prevent the next 3 damage" shield.
fn salve_shield(t: &mut TestGame) {
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    mana(t, P1, ManaType::W, 1);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).target(Entity::Player(P1)).go();
    t.resolve_all();
}

/// P1's life after 2 damage from `hit` with the shield, for each order P1 can pick.
fn orders(setup: fn(&mut TestGame), hit: fn(&mut TestGame)) -> Vec<i32> {
    let mut out = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        setup(&mut t);
        salve_shield(&mut t);
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        hit(&mut t);
        t.resolve_all();
        out.push(t.life(P1));
    }
    out.sort();
    out
}

fn shock_p1(t: &mut TestGame) {
    mana(t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
}

#[test]
fn the_player_dealt_damage_orders_damage_increases_and_prevention() {
    cr!("616.1", "616.1f", "615.1a");
    ruling!(
        "Invasion of Regatha // Disciples of the Inferno",
        "If another effect modifies how much damage a source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, Disciples of the Inferno’s effect no longer applies."
    );
    ruling!(
        "Artist's Talent",
        "If another effect modifies how much damage a source would deal, including preventing some of it, the player being dealt damage or the controller of the permanent being dealt damage chooses an order in which to apply those effects. If all of the damage is prevented, the effect of Artist's Talent's last ability no longer applies."
    );
    ruling!(
        "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied",
        "If another effect (or effects) modifies how much damage a Wizard you control would deal—by preventing some of it, for example—the player being dealt damage or the controller of the permanent being dealt damage chooses the order in which any such effects (including Trance Kuja's) apply. If all of the damage is prevented, Trance Kuja's effect no longer applies."
    );
    // Prevention first: all 2 prevented, nothing to add to. The increase first: 4, 3 of
    // it prevented.
    assert_eq!(
        orders(
            |t| {
                back_face(t, P0, "Invasion of Regatha // Disciples of the Inferno");
            },
            shock_p1
        ),
        vec![19, 20]
    );
    assert_eq!(
        orders(
            |t| {
                artists_talent_level_3(t);
            },
            shock_p1
        ),
        vec![19, 20]
    );
    // Trance Kuja doubles damage a Wizard (Hurkyl) deals.
    assert_eq!(
        orders(
            |t| {
                back_face(t, P0, "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied");
                t.battlefield(P0, "Hurkyl, Master Wizard");
            },
            |t| {
                let hurkyl = t.named_on_battlefield("Hurkyl, Master Wizard")[0];
                t.g.deal_damage(hurkyl, Entity::Player(P1), 2, false);
                t.g.flush_events();
            }
        ),
        vec![19, 20]
    );
}

#[test]
fn disciples_of_the_inferno_adds_two_after_dividing() {
    cr!("601.2d", "614.1a");
    ruling!(
        "Invasion of Regatha // Disciples of the Inferno",
        "If damage dealt by a source is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control, divide the original amount before adding 2. For example, if you cast a sorcery spell that deals 4 damage divided as you choose among any number of targets, you could have it deal 3 damage to one creature and 1 damage to a battle. Those amounts would then be modified to 5 damage and 3 damage, respectively."
    );
    let mut t = TestGame::new(2);
    back_face(
        &mut t,
        P0,
        "Invasion of Regatha // Disciples of the Inferno",
    );
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 3);
    let arc = t.hand(P0, "Arc Lightning");
    t.answer_targets(P0, &[obj(maw), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.cast(P0, arc).go();
    t.resolve_all();
    assert_eq!(marked(&t, maw), 4);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn disciples_of_the_inferno_hits_any_creature_and_battle_but_not_teammates() {
    cr!("614.1a", "102.3", "120.3h");
    ruling!(
        "Invasion of Regatha // Disciples of the Inferno",
        "Disciples of the Inferno modifies damage that would be dealt to any creature, no matter who controls it, as well as any battle, no matter who protects it. Your teammates are safe. For now."
    );
    let shock = |t: &mut TestGame, to: Entity| {
        mana(t, P0, ManaType::R, 1);
        let s = t.hand(P0, "Shock");
        t.cast(P0, s).target(to).go();
        t.resolve_all();
    };
    // Two-Headed Giant: P0 and P1 against P2 and P3.
    let mut t = two_headed_giant();
    back_face(
        &mut t,
        P0,
        "Invasion of Regatha // Disciples of the Inferno",
    );
    let mine = t.battlefield(P0, "Colossal Dreadmaw");
    let theirs = t.battlefield(P2, "Colossal Dreadmaw");
    shock(&mut t, obj(mine));
    shock(&mut t, obj(theirs));
    assert_eq!((marked(&t, mine), marked(&t, theirs)), (4, 4));
    // A battle P2 controls, protected by P0's teammate.
    let battle = t.battlefield(P2, "Invasion of Regatha // Disciples of the Inferno");
    t.g.objects[battle.0 as usize].choices.player = Some(P1);
    let defense = t.counters(battle, counters::DEFENSE);
    assert_eq!(defense, 5);
    shock(&mut t, obj(battle));
    assert_eq!(t.counters(battle, counters::DEFENSE), 1);
    // A teammate: 2. An opponent: 4.
    let (us, them) = (t.life(P1), t.life(P2));
    shock(&mut t, Entity::Player(P1));
    assert_eq!(t.life(P1), us - 2);
    shock(&mut t, Entity::Player(P2));
    assert_eq!(t.life(P2), them - 4);
}

#[test]
fn soul_scar_mage_converts_damage_its_controllers_source_deals_for_an_opponent() {
    cr!("614.1a", "701.14a", "701.14d");
    ruling!(
        "Soul-Scar Mage",
        "If a spell or ability an opponent controls causes a source you control to deal noncombat damage to a creature an opponent controls, that damage will be replaced with -1/-1 counters."
    );
    supported("Soul-Scar Mage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul-Scar Mage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // P1's Prey Upon: P1's Hill Giant fights P0's Grizzly Bears.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    mana(&mut t, P1, ManaType::G, 1);
    let prey = t.hand(P1, "Prey Upon");
    t.answer_targets(P1, &[obj(giant)]);
    t.answer_targets(P1, &[obj(bears)]);
    t.cast(P1, prey).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(marked(&t, giant), 0);
    assert_eq!(t.pt(giant), (1, 1));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn soul_scar_mage_fight_damage_is_simultaneous() {
    cr!("701.14a", "701.14d");
    ruling!(
        "Soul-Scar Mage",
        "If an effect causes a creature you control to fight a creature an opponent controls, the damage is dealt simultaneously. For instance, if your 2/2 creature fights an opponent’s 4/4 creature, your creature will be dealt 4 damage and your opponent’s creature will have two -1/-1 counters put on it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul-Scar Mage");
    let mastodon = t.battlefield(P0, "Siege Mastodon");
    let air = t.battlefield(P1, "Air Elemental");
    t.answer_targets(P0, &[obj(mastodon)]);
    t.answer_targets(P0, &[obj(air)]);
    cast_new(&mut t, P0, "Prey Upon", &[]);
    t.resolve_all();
    // The Elemental dealt its full 4, though its counters make it 1/1 afterwards.
    assert_eq!(marked(&t, mastodon), 4);
    assert_eq!(t.counters(air, counters::MINUS1), 3);
    assert_eq!(t.pt(air), (1, 1));
}

#[test]
fn soul_scar_mage_the_damaged_creatures_controller_orders_the_effects() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Soul-Scar Mage",
        "If multiple prevention and/or replacement effects are trying to apply to the same damage, the controller of the creature that would be dealt damage chooses the order in which to apply them. As examples, an opponent could choose to apply the effect of Djeru’s Resolve and prevent the damage rather than put -1/-1 counters on it; or an opponent could choose to convert the damage to -1/-1 counters before Insult could double it, and then Insult’s effect won’t apply anymore."
    );
    supported("Djeru's Resolve");
    supported("Insult // Injury");
    let shock = |t: &mut TestGame, maw: ObjectId| {
        mana(t, P0, ManaType::R, 1);
        let s = t.hand(P0, "Shock");
        t.cast(P0, s).target(maw).go();
        t.resolve_all();
    };
    // Djeru's Resolve on P1's creature: prevent, or take the counters.
    let mut out = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul-Scar Mage");
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        mana(&mut t, P1, ManaType::W, 1);
        let dr = t.hand(P1, "Djeru's Resolve");
        t.cast(P1, dr).target(maw).go();
        t.resolve_all();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        shock(&mut t, maw);
        out.push((t.counters(maw, counters::MINUS1), marked(&t, maw)));
    }
    out.sort();
    assert_eq!(out, vec![(0, 0), (2, 0)]);
    // Insult: counters before doubling (2), or doubling first (4).
    let mut out = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul-Scar Mage");
        let maw = t.battlefield(P1, "Colossal Dreadmaw");
        mana(&mut t, P0, ManaType::R, 3);
        let insult = t.hand(P0, "Insult // Injury");
        t.cast(P0, insult)
            .method(mtg_engine::object::CastMethod::Half(0))
            .go();
        t.resolve_all();
        t.answer(P1, DecisionKind::Replacement, Answer::Index(pick));
        shock(&mut t, maw);
        out.push(t.counters(maw, counters::MINUS1));
    }
    out.sort();
    assert_eq!(out, vec![2, 4]);
}
