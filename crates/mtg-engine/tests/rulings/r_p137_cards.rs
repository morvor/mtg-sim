//! Rulings batch P137 — the batch's other removal cards: what an Aura that stops activated
//! abilities leaves alone, what Vampire Hexmage can target, Bone Shards' additional cost,
//! protection from colored spells (Emrakul), hexproof from planeswalkers, alternative and
//! reduced costs (Hand of Emrakul, Awaken the Blood Avatar), triggers waiting for a modal
//! spell to finish, play permissions following the normal timing rules, a punisher trigger
//! that triggers twice, and "each player who can't" after a sacrifice.

use crate::r_p137_common::*;
use crate::r_p108_common::put_counters;
use crate::r_s02_common::can_play_land;
use crate::r_s07_common::cast_methods;
use crate::r_s01_common::{tapped_lands, with_subtype};
use crate::r_s08_common::mana_value;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Creates a 0/1 colorless Eldrazi Spawn creature token for `p`.
fn spawn(t: &mut TestGame, p: PlayerId) -> ObjectId {
    use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
    use mtg_engine::types::ColorSet;
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec: TokenSpec {
                name: "Eldrazi Spawn Token".into(),
                colors: ColorSet::NONE,
                supertypes: vec![],
                card_types: vec![CardType::Creature],
                subtypes: vec!["Eldrazi".into(), "Spawn".into()],
                power: Some(0),
                toughness: Some(1),
                abilities: vec![],
                scryfall_name: None,
                pt_values: None,
            },
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token")
}

#[test]
fn hold_for_questioning_leaves_static_and_triggered_abilities_alone() {
    cr!("602.5", "603.2", "613.4c");
    ruling!(
        "Hold for Questioning",
        "Static and triggered abilities are unaffected by Hold for Questioning."
    );
    supported("Hold for Questioning");
    let mut t = TestGame::new(2);
    let warden = t.battlefield(P1, "Soul Warden");
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    for target in [warden, marshal, sorcerer] {
        give_mana_for(&mut t, P0, "Hold for Questioning");
        let aura = t.hand(P0, "Hold for Questioning");
        t.cast(P0, aura).target(obj(target)).go();
        t.resolve_all();
    }
    // The activated ability can't be activated...
    t.g.untap(sorcerer);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P1, sorcerer));
    // ... but Benalish Marshal's static ability still applies and Soul Warden still
    // triggers.
    let bears = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn vampire_hexmage_cant_target_a_suspended_card() {
    cr!("115.1", "702.62a");
    ruling!(
        "Vampire Hexmage",
        "Vampire Hexmage's ability can't target a permanent card in a zone other than the battlefield, such as a suspended card."
    );
    supported("Vampire Hexmage");
    let mut t = TestGame::new(2);
    let hexmage = t.battlefield(P0, "Vampire Hexmage");
    let suspended = t.exile(P1, "Rift Bolt");
    put_counters(&mut t, suspended, "time", 3);
    let walker = t.battlefield(P1, "Liliana of the Veil");
    let from = t.asked().len();
    t.activate(P0, hexmage, 0, &[obj(walker)]).unwrap();
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(walker)));
    assert!(!offered[0].contains(&obj(suspended)));
    t.resolve_all();
    assert!(!t.on_battlefield(walker));
    assert_eq!(t.counters(suspended, "time"), 3);
}

#[test]
fn bone_shards_needs_exactly_one_creature_or_card() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Bone Shards",
        "You must sacrifice exactly one creature or discard exactly one card to cast this spell; you can't cast it without sacrificing a creature or discarding a card, and you can't sacrifice additional creatures or discard additional cards."
    );
    supported("Bone Shards");
    // Nothing to sacrifice or discard: it can't be cast.
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let shards = t.hand(P0, "Bone Shards");
    assert!(t.cast(P0, shards).target(obj(victim)).try_go().is_err());
    assert!(t.on_battlefield(victim));
    // Two creatures and two other cards: exactly one creature is sacrificed.
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Runeclaw Bear");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    assert!(can_cast(&mut t, P0, shards, CastMethod::Normal));
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.cast(P0, shards).target(obj(victim)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(victim));
    let gone = usize::from(!t.on_battlefield(a)) + usize::from(!t.on_battlefield(b));
    let discarded = t.graveyard_size(P0) - gone - 1;
    assert_eq!(gone + discarded, 1, "one creature or one card");
    assert_eq!(t.hand_size(P0) + discarded, 2);
}

#[test]
fn emrakul_can_be_blocked_by_colored_creatures_and_targeted_by_triggers() {
    cr!("702.16b", "702.16e", "702.16f");
    ruling!(
        "Emrakul, the Aeons Torn",
        "\"Colored spells\" is not synonymous with \"colored cards.\" For example, even though creatures are spells when they're cast, they're not spells when they're on the battlefield and can block Emrakul; and triggered abilities of permanents entering the battlefield (such as that of Banishing Light) can target it."
    );
    supported("Emrakul, the Aeons Torn");
    supported("Banishing Light");
    // A green creature (with reach, as Emrakul flies) blocks it.
    let mut t = TestGame::new(2);
    let emrakul = t.battlefield(P0, "Emrakul, the Aeons Torn");
    let bears = t.battlefield(P1, "Giant Spider");
    let lands = t.lands(P1, "Forest", 6);
    let fodder: Vec<Entity> = lands.iter().map(|l| obj(*l)).collect();
    t.answer_choose(P1, &fodder);
    attack_with(&mut t, &[(emrakul, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[(bears, emrakul)]);
    assert_eq!(t.life(P1), 20, "blocked");
    assert!(!t.on_battlefield(bears));
    // Banishing Light's enters trigger targets it.
    let mut t = TestGame::new(2);
    let emrakul = t.battlefield(P1, "Emrakul, the Aeons Torn");
    give_mana_for(&mut t, P0, "Banishing Light");
    let light = t.hand(P0, "Banishing Light");
    t.answer_targets(P0, &[obj(emrakul)]);
    t.cast(P0, light).go();
    t.resolve_all();
    assert_eq!(t.zone(emrakul), Zone::Exile);
}

#[test]
fn emrakul_targeted_by_a_counterspell_that_does_more() {
    cr!("101.2", "608.2b");
    ruling!(
        "Emrakul, the Aeons Torn",
        "Emrakul can be targeted by spells that try to counter it (such as Ionize). Those spells will resolve, but the part of their effect that would counter Emrakul won't do anything. Any other effects those spells have will work as normal."
    );
    supported("Ionize");
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Emrakul, the Aeons Torn");
    let card = t.hand(P0, "Emrakul, the Aeons Torn");
    let emrakul = t.cast(P0, card).go();
    t.settle();
    give_mana_for(&mut t, P1, "Ionize");
    let ionize = t.hand(P1, "Ionize");
    let ionize = t.cast(P1, ionize).target(obj(emrakul)).go();
    t.resolve();
    assert!(resolved(&t, ionize));
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Emrakul, the Aeons Torn").len(), 1);
}

#[test]
fn emrakul_can_be_enchanted_by_a_colored_aura_put_onto_the_battlefield() {
    cr!("702.16b", "702.16c", "303.4f");
    ruling!(
        "Emrakul, the Aeons Torn",
        "Emrakul can't be the target of colored Aura spells, but colored Auras can be put onto the battlefield enchanting it."
    );
    supported("Pacifism");
    supported("Show and Tell");
    let mut t = TestGame::new(2);
    let emrakul = t.battlefield(P1, "Emrakul, the Aeons Torn");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Cast, Pacifism can't target it.
    give_mana_for(&mut t, P0, "Pacifism");
    let pacifism = t.hand(P0, "Pacifism");
    let from = t.asked().len();
    t.cast(P0, pacifism).target(obj(bears)).go();
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&obj(bears)));
    assert!(!offered[0].contains(&obj(emrakul)));
    t.resolve_all();
    // Put onto the battlefield by Show and Tell, it enchants Emrakul and stays.
    let other = t.hand(P0, "Pacifism");
    give_mana_for(&mut t, P0, "Show and Tell");
    let show = t.hand(P0, "Show and Tell");
    t.answer_yes(P0, true).answer_yes(P1, false);
    t.answer_choose(P0, &[obj(other)]);
    t.cast(P0, show).go();
    t.answer_choose(P0, &[obj(emrakul)]);
    t.resolve_all();
    let aura = t.g.current(other);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.obj(aura).attached_to, Some(obj(emrakul)));
}

#[test]
fn eradicator_valkyrie_has_hexproof_from_opponents_planeswalkers() {
    cr!("702.11d", "115.4");
    ruling!(
        "Eradicator Valkyrie",
        "Eradicator Valkyrie can't be the target of abilities whose source is a planeswalker an opponent controls."
    );
    supported("Eradicator Valkyrie");
    supported("Chandra Nalaar");
    // (Chandra Nalaar −X: "deals X damage to target creature.")
    let offered = |owner: PlayerId| {
        let mut t = TestGame::new(2);
        t.set_step(owner, Step::PrecombatMain);
        let valkyrie = t.battlefield(P0, "Eradicator Valkyrie");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let chandra = t.battlefield(owner, "Chandra Nalaar");
        t.answer(owner, DecisionKind::X, Answer::Number(1));
        let from = t.asked().len();
        let _ = t.activate(owner, chandra, 1, &[obj(bears)]);
        let c = target_candidates(&t, owner, from);
        (c[0].contains(&obj(valkyrie)), c[0].contains(&obj(bears)))
    };
    assert_eq!(offered(P1), (false, true), "an opponent's planeswalker");
    assert_eq!(offered(P0), (true, true), "your own planeswalker");
}

/// P0's Hand of Emrakul and four Eldrazi Spawn; the alternative cost's method, if it can be
/// cast that way now.
fn hand_of_emrakul(t: &mut TestGame) -> (ObjectId, Option<CastMethod>) {
    for _ in 0..4 {
        spawn(t, P0);
    }
    let hand = t.hand(P0, "Hand of Emrakul");
    let alt = cast_methods(t, P0, hand)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)));
    (hand, alt)
}

#[test]
fn hand_of_emrakul_alternative_cost_keeps_mana_value_and_timing_and_cost_increases() {
    cr!("118.9c", "202.3", "307.1", "601.2f");
    ruling!(
        "Hand of Emrakul",
        "Casting Hand of Emrakul by paying its alternative cost doesn't change its mana cost or mana value."
    );
    ruling!(
        "Hand of Emrakul",
        "Casting Hand of Emrakul by paying its alternative cost doesn't change when you can cast it. You can cast it only at the normal time you could cast a creature spell."
    );
    ruling!(
        "Hand of Emrakul",
        "Effects that increase or reduce the cost to cast Hand of Emrakul will apply to it even if you choose to pay its alternative cost."
    );
    supported("Hand of Emrakul");
    supported("Sphere of Resistance");
    // Cast for the Spawn: still mana value 9.
    let mut t = TestGame::new(2);
    let (hand, alt) = hand_of_emrakul(&mut t);
    let spell = t.cast(P0, hand).method(alt.unwrap()).go();
    assert_eq!(mana_value(&t, spell), 9);
    assert!(with_subtype(&t, P0, "Spawn").is_empty());
    // Not on an opponent's turn, nor while something is on the stack.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(hand_of_emrakul(&mut t).1, None);
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Divination");
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    assert_eq!(hand_of_emrakul(&mut t).1, None);
    // Sphere of Resistance: {1} more, alternative cost or not.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    let (hand, alt) = hand_of_emrakul(&mut t);
    assert!(alt.is_none() || t.cast(P0, hand).method(alt.unwrap()).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    let alt = cast_methods(&mut t, P0, hand)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)));
    t.cast(P0, hand).method(alt.unwrap()).go();
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn awaken_the_blood_avatar_sacrifices_more_creatures_than_reduce_its_cost() {
    cr!("601.2b", "601.2f", "118.7d");
    ruling!(
        "Extus, Oriq Overlord // Awaken the Blood Avatar",
        "Awaken the Blood Avatar’s first ability can’t reduce its cost below {B}{R}. However, you can sacrifice any number of creatures, even if they won’t reduce its cost any further."
    );
    let mut t = TestGame::new(2);
    let mine: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let card = t.hand(P0, "Extus, Oriq Overlord // Awaken the Blood Avatar");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(5));
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    assert!(mine.iter().all(|c| !t.on_battlefield(*c)), "all five sacrificed");
    assert_eq!(tapped_lands(&t, P0), 2, "{{B}}{{R}} still paid");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Avatar").len(), 1);
}

#[test]
fn kayas_guile_modes_in_order_and_triggers_wait() {
    cr!("700.2", "608.2c", "603.3");
    ruling!(
        "Kaya's Guile",
        "Each chosen mode is performed in the order specified. Abilities that trigger while one mode is being performed won’t be put onto the stack until Kaya’s Guile has finished resolving."
    );
    supported("Kaya's Guile");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1 chooses its creature before the Spirit exists.
    let seen = watch(&mut t, P1, is_choose, |g| {
        g.permanents().filter(|o| o.chars.has_subtype("Spirit")).count()
    });
    give_mana_for(&mut t, P0, "Kaya's Guile");
    let spell = t.hand(P0, "Kaya's Guile");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    let spell = t.cast(P0, spell).modes(&[0, 2]).go();
    t.g.resolve_top();
    assert!(resolved(&t, spell));
    assert!(!t.on_battlefield(bears));
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
    // Neither the dies trigger nor the enters trigger is on the stack yet.
    assert_eq!(t.stack_len(), 0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn dark_intimations_opponents_choose_a_creature_or_planeswalker() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Dark Intimations",
        "Each opponent chooses a permanent to sacrifice from among the creatures and planeswalkers that player controls. You don't choose which type of permanent the player has to sacrifice."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let walker = t.battlefield(P1, "Liliana of the Veil");
    give_mana_for(&mut t, P0, "Dark Intimations");
    let spell = t.hand(P0, "Dark Intimations");
    t.answer_choose(P1, &[obj(walker)]);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(offered_to(&t, P1, from)
        .iter()
        .any(|c| c.contains(&bears) && c.contains(&walker)));
    // P0 chose nothing about it.
    assert!(offered_to(&t, P0, from)
        .iter()
        .all(|c| !c.contains(&bears) && !c.contains(&walker)));
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(walker));
}

#[test]
fn dark_intimations_does_each_part_it_can() {
    cr!("608.2c", "609.3");
    ruling!(
        "Dark Intimations",
        "If an opponent can't sacrifice a creature or planeswalker, that player still discards a card if able. You still return a creature or planeswalker card to your hand if able, even if no opponent sacrifices a permanent and/or discards a card. You still draw a card even if you can't return a card to your hand."
    );
    // P1 has nothing to sacrifice; P0 has nothing to return.
    let mut t = TestGame::new(2);
    t.hand(P1, "Island");
    t.library_top(P0, "Forest");
    give_mana_for(&mut t, P0, "Dark Intimations");
    let spell = t.hand(P0, "Dark Intimations");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_hand(P0, "Forest"));
    // P1 has nothing at all; P0 returns a creature card and draws.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.library_top(P0, "Forest");
    give_mana_for(&mut t, P0, "Dark Intimations");
    let spell = t.hand(P0, "Dark Intimations");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Forest"));
}

#[test]
fn riveteers_charm_cards_are_played_following_the_normal_timing_rules() {
    cr!("305.2", "305.3", "601.2g");
    ruling!(
        "Riveteers Charm",
        "For the second mode, you must pay all costs and follow all normal timing rules for cards played this way. For example, you may play lands this way only during your main phase while no spells or abilities are on the stack and only if you haven't played a land yet this turn."
    );
    supported("Riveteers Charm");
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId) {
        let forest = t.library_top(P0, "Forest");
        let bolt = t.library_top(P0, "Lightning Bolt");
        t.library_top(P0, "Island");
        give_mana_for(t, P0, "Riveteers Charm");
        let charm = t.hand(P0, "Riveteers Charm");
        t.cast(P0, charm).modes(&[1]).go();
        t.resolve_all();
        (forest, bolt)
    };
    let mut t = TestGame::new(2);
    let (forest, bolt) = setup(&mut t);
    assert!(can_play_land(&mut t, P0, forest));
    // The Bolt's mana cost must be paid.
    assert!(!can_cast(&mut t, P0, bolt, CastMethod::Normal));
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));
    // Not with a spell on the stack, nor after the land drop is used.
    let div = t.hand(P0, "Divination");
    add_mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, div).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    let land = t.hand(P0, "Mountain");
    t.play_land(P0, land).unwrap();
    assert!(!can_play_land(&mut t, P0, forest));
    // Not on an opponent's turn.
    let mut t = TestGame::new(2);
    let (forest, _) = setup(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_play_land(&mut t, P0, forest));
}

#[test]
fn ulamogs_crusher_doesnt_attack_when_it_cant_or_it_costs_something() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Ulamog's Crusher",
        "If Ulamog's Crusher can't attack for any reason (such as becoming tapped or having just entered the battlefield), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    crate::r_p065_combat::attacks_if_able_but_not_when_tapped_or_costly("Ulamog's Crusher");
}

#[test]
fn plaguecrafter_can_be_sacrificed_to_its_own_ability() {
    cr!("701.21a", "603.2");
    ruling!(
        "Plaguecrafter",
        "Plaguecrafter can be the creature its controller sacrifices for its own ability."
    );
    supported("Plaguecrafter");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.hand(P2, "Island");
    let from = t.asked().len();
    let crafter = t.enter(P0, "Plaguecrafter");
    t.answer_choose(P0, &[obj(crafter)]);
    t.settle();
    t.resolve_all();
    assert!(offered_to(&t, P0, from)
        .iter()
        .any(|c| c.contains(&crafter) && c.contains(&bears)));
    assert!(!t.on_battlefield(crafter) && t.on_battlefield(bears));
    assert!(!t.on_battlefield(theirs));
    // "Each player who can't discards a card": only P2, who had nothing to sacrifice.
    assert_eq!(t.hand_size(P2), 0);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn single_combat_players_choose_in_turn_order_then_sacrifice_the_rest() {
    cr!("101.4", "701.21a");
    ruling!(
        "Single Combat",
        "As Single Combat resolves, first the player whose turn it is chooses a creature or planeswalker they control, then each other player in turn order does the same, knowing the choices made before them. Then each unchosen creature and planeswalker is sacrificed at the same time."
    );
    supported("Single Combat");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Leyline of Anticipation");
    let mut kept = vec![];
    let mut lost = vec![];
    for p in [P0, P1, P2] {
        let bears = t.battlefield(p, "Grizzly Bears");
        let walker = t.battlefield(p, "Liliana of the Veil");
        t.answer_choose(p, &[obj(walker)]);
        kept.push(walker);
        lost.push(bears);
    }
    let seen = [P0, P1, P2].map(|p| {
        watch(&mut t, p, is_choose, |g| {
            g.permanents()
                .filter(|o| o.is(CardType::Creature))
                .count()
        })
    });
    give_mana_for(&mut t, P0, "Single Combat");
    let spell = t.hand(P0, "Single Combat");
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P1, P2, P0]);
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![3], "nothing sacrificed while choosing");
    }
    assert!(kept.iter().all(|k| t.on_battlefield(*k)));
    assert!(lost.iter().all(|l| !t.on_battlefield(*l)));
    // "Players can't cast creature or planeswalker spells until the end of your next
    // turn."
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    assert!(t.cast(P1, bears).try_go().is_err());
}

#[test]
fn limited_resources_a_player_with_fewer_than_five_lands_keeps_them_all() {
    cr!("101.4", "701.21a");
    ruling!(
        "Limited Resources",
        "If a player has less than 5 lands, they get to pick and keep all of their lands."
    );
    supported("Limited Resources");
    let mut t = TestGame::new(2);
    let mine = t.lands(P0, "Plains", 3);
    let theirs = t.lands(P1, "Island", 6);
    let keep: Vec<Entity> = theirs[..5].iter().map(|l| obj(*l)).collect();
    t.answer_choose(P1, &keep);
    t.enter(P0, "Limited Resources");
    t.settle();
    t.resolve_all();
    assert!(mine.iter().all(|l| t.on_battlefield(*l)));
    assert!(theirs[..5].iter().all(|l| t.on_battlefield(*l)));
    assert!(!t.on_battlefield(theirs[5]));
    // Eight lands on the battlefield: lands can still be played.
    let land = t.hand(P0, "Plains");
    t.play_land(P0, land).unwrap();
    // With ten, they can't.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Limited Resources");
    t.lands(P1, "Island", 10);
    let land = t.hand(P0, "Plains");
    assert!(!can_play_land(&mut t, P0, land));
}

#[test]
fn ogre_marauder_second_trigger_doesnt_undo_the_first() {
    cr!("603.2", "611.2a", "506.1");
    ruling!(
        "Ogre Marauder",
        "Attacking with Ogre Marauder a second time would cause a second trigger. If no creature was previously sacrificed, the Marauder already can’t be blocked. Sacrificing a creature for the second trigger will not nullify the first."
    );
    supported("Ogre Marauder");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Ogre Marauder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // First combat: P1 sacrifices nothing.
    t.answer_yes(P1, false);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[(giant, ogre)]);
    assert_eq!(t.life(P1), 17, "unblockable");
    // An additional combat phase: P1 sacrifices the Bears, but the Marauder still can't
    // be blocked.
    t.advance_to(P0, Step::PostcombatMain);
    give_mana_for(&mut t, P0, "Relentless Assault");
    let assault = t.hand(P0, "Relentless Assault");
    t.cast(P0, assault).go();
    t.resolve_all();
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(bears)]);
    attack_with(&mut t, &[(ogre, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    block_and_finish(&mut t, P1, &[(giant, ogre)]);
    assert_eq!(t.life(P1), 14);
    assert!(t.on_battlefield(ogre));
}

#[test]
fn fandaniel_surveils_before_the_spell_resolves_even_if_it_is_countered() {
    cr!("603.3", "405.5", "701.25a");
    ruling!(
        "Fandaniel, Telophoroi Ascian",
        "Fandaniel's first ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    // (Its first ability: "Whenever you cast an instant or sorcery spell, surveil 1.")
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fandaniel, Telophoroi Ascian");
    let top = t.library_top(P0, "Forest");
    give_mana_for(&mut t, P0, "Lightning Bolt");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is above the spell");
    give_mana_for(&mut t, P1, "Counterspell");
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(obj(bolt)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![top]));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"), "surveilled");
    assert_eq!(t.life(P1), 20);
}
