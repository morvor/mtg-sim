//! Rulings batch P111 — costs, mana and casting permissions: "for each color of mana spent
//! to cast it" (CR 106.1, 105.1, 601.2h, 707.9); colors among permanents (CR 105.2);
//! exiling a card from a graveyard as a cost (CR 602.2b, 601.2h); alternative costs from a
//! graveyard keep the timing (CR 601.3, 118.9a); "can't cast more than one spell each
//! turn" (CR 601.3); playing exiled cards for free this turn (CR 118.9, 305.1); "the next
//! instant or sorcery spell you cast" (CR 608.2n); "activate only once each turn"
//! (CR 602.5b); a search judged by last known power (CR 608.2h) and {X} as 0 (CR 202.3e).

use crate::r_p009_common::kill;
use crate::r_p119_common::counter_spell;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s06_common::{activate_containing, give_control};
use crate::r_s07_common::cast_methods;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn mana(t: &mut TestGame, p: PlayerId, cost: &str) {
    for c in cost.chars() {
        let ty = match c {
            'W' => ManaType::W,
            'U' => ManaType::U,
            'B' => ManaType::B,
            'R' => ManaType::R,
            'G' => ManaType::G,
            'C' => ManaType::C,
            d => {
                let n = d.to_digit(10).expect("a mana symbol");
                t.g.players[p.idx()].mana_pool.add_type(ManaType::C, n);
                continue;
            }
        };
        t.g.players[p.idx()].mana_pool.add_type(ty, 1);
    }
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Casts Chamber Sentry from P0's hand for X = the number of mana in `paid` (all spent).
fn cast_sentry(t: &mut TestGame, paid: &str) -> ObjectId {
    let s = t.hand(P0, "Chamber Sentry");
    mana(t, P0, paid);
    t.cast(P0, s).x(paid.len() as i64).go();
    t.resolve_all();
    t.g.current(s)
}

#[test]
fn chamber_sentry_counts_the_colors_of_mana_actually_spent() {
    cr!("106.1", "105.1", "601.2h", "614.1c");
    ruling!(
        "Chamber Sentry",
        "Colorless mana won't give Chamber Sentry another +1/+1 counter. Colorless is not a color."
    );
    ruling!(
        "Chamber Sentry",
        "If Chamber Sentry enters the battlefield without being cast, it receives no +1/+1 counters."
    );
    ruling!(
        "Chamber Sentry",
        "If an effect allows you to spend mana as though it were mana of any color to cast Chamber Sentry, you consider only the actual mana spent to determine how many counters it receives."
    );
    supported("Chamber Sentry");
    supported("Mycosynth Lattice");
    // "This creature enters with a +1/+1 counter on it for each color of mana spent to
    // cast it."
    let mut t = TestGame::new(2);
    let s = cast_sentry(&mut t, "WUC");
    assert_eq!(t.counters(s, "+1/+1"), 2);
    let s = cast_sentry(&mut t, "CCC");
    assert_eq!(t.counters(s, "+1/+1"), 0);
    let s = t.enter(P0, "Chamber Sentry");
    t.settle();
    assert_eq!(t.counters(s, "+1/+1"), 0);
    assert!(t.in_graveyard(P0, "Chamber Sentry"), "a 0/0");
    // Mycosynth Lattice: "Players may spend mana as though it were mana of any color."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    let s = cast_sentry(&mut t, "CCR");
    assert_eq!(t.counters(s, "+1/+1"), 1);
}

#[test]
fn a_clone_of_chamber_sentry_counts_the_mana_spent_to_cast_the_clone() {
    cr!("707.9", "614.1c", "106.1");
    ruling!(
        "Chamber Sentry",
        "If another creature enters the battlefield as a copy of Chamber Sentry, consider what mana was spent to cast that creature to determine how many +1/+1 counters it enters with."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let s = cast_sentry(&mut t, "WU");
    assert_eq!(t.counters(s, "+1/+1"), 2);
    // Clone ({3}{U}) paid with {U}{B}{R}{G}: four colors.
    let clone = t.hand(P0, "Clone");
    mana(&mut t, P0, "UBRG");
    t.answer_choose(P0, &[obj(s)]);
    t.answer_yes(P0, true);
    t.cast(P0, clone).go();
    t.resolve_all();
    let c = t.g.current(clone);
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).chars.name, "Chamber Sentry");
    assert_eq!(t.counters(c, "+1/+1"), 4);
}

#[test]
fn earthen_ally_counts_at_most_five_colors() {
    cr!("105.1", "105.2", "105.2c");
    ruling!(
        "Earthen Ally",
        "Earthen Ally's first ability can give it at most +5/+0, that is, +1/+0 each for white, blue, black, red, and green. \"Gold,\" \"multicolor,\" and \"colorless\" aren't colors."
    );
    supported("Earthen Ally");
    // "This creature gets +1/+0 for each color among Allies you control."
    let mut t = TestGame::new(2);
    let ally = t.battlefield(P0, "Earthen Ally");
    assert_eq!(t.pt(ally), (1, 2));
    // Munda (red and white).
    t.battlefield(P0, "Munda, Ambush Leader");
    assert_eq!(t.pt(ally), (3, 2));
    t.battlefield(P0, "Umara Raptor");
    t.battlefield(P0, "Hagra Sharpshooter");
    assert_eq!(t.pt(ally), (5, 2));
    t.battlefield(P0, "Kazandu Blademaster");
    assert_eq!(t.pt(ally), (5, 2));
}

#[test]
fn exiling_the_card_is_part_of_the_cost() {
    cr!("602.2b", "601.2h", "118.3");
    ruling!(
        "Maestros Initiate",
        "Exiling Maestros Initiate from your graveyard is part of paying the cost of its ability. This means that no player can respond to the ability by removing it from your graveyard before you can pay the cost."
    );
    supported("Maestros Initiate");
    // "{4}{U/R}, Exile this card from your graveyard: Draw two cards, then discard a card."
    let mut t = TestGame::new(2);
    let m = t.graveyard(P0, "Maestros Initiate");
    mana(&mut t, P0, "U4");
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, m, "Draw two").unwrap();
    // Before anyone could respond, it's already in exile.
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_exile("Maestros Initiate"));
    assert!(!t.in_graveyard(P0, "Maestros Initiate"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

/// The ways P0 could cast `card` from its graveyard now, other than normally.
fn graveyard_methods(t: &mut TestGame, card: ObjectId) -> Vec<CastMethod> {
    cast_methods(t, P0, card)
}

#[test]
fn worldheart_phoenix_from_the_graveyard_is_a_creature_spell() {
    cr!("601.2a", "601.3", "118.9a", "701.6a");
    ruling!(
        "Worldheart Phoenix",
        "As soon as you start to cast Worldheart Phoenix from your graveyard, the card moves onto the stack. At that point, it's too late for an opponent to prevent you from casting it by removing it from your graveyard."
    );
    ruling!(
        "Worldheart Phoenix",
        "Casting Worldheart Phoenix from your graveyard by paying its alternative cost doesn't change when you can cast it. You can cast it only at the normal time you could cast a creature spell."
    );
    ruling!(
        "Worldheart Phoenix",
        "If you cast Worldheart Phoenix this way, it is a spell and can be countered."
    );
    supported("Worldheart Phoenix");
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Worldheart Phoenix");
    mana(&mut t, P0, "WUBRG");
    // Not in combat, nor in the opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(graveyard_methods(&mut t, phoenix).is_empty());
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(graveyard_methods(&mut t, phoenix).is_empty());
    t.set_step(P0, Step::PrecombatMain);
    let methods = graveyard_methods(&mut t, phoenix);
    assert_eq!(methods.len(), 1, "{methods:?}");
    let spell = t.cast(P0, phoenix).method(methods[0].clone()).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(!t.in_graveyard(P0, "Worldheart Phoenix"));
    counter_spell(&mut t, spell);
    assert!(t.in_graveyard(P0, "Worldheart Phoenix"));
    assert!(t.named_on_battlefield("Worldheart Phoenix").is_empty());
}

/// Whether `p` could cast `card` now.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    !cast_methods(t, p, card).is_empty()
}

#[test]
fn high_noon_counts_every_spell_cast_this_turn() {
    cr!("601.3", "601.2i");
    ruling!(
        "High Noon",
        "If you cast High Noon, you’ve already cast at least one spell that turn (High Noon itself) so you won’t be able to cast any more spells until the next turn. If High Noon enters the battlefield on a turn where you haven’t cast any spells, you’ll be able to cast one spell that turn."
    );
    ruling!(
        "High Noon",
        "If you cast a spell that was countered, you can’t cast another spell during the same turn while High Noon is on the battlefield."
    );
    supported("High Noon");
    // Cast: no more spells this turn.
    let mut t = TestGame::new(2);
    let hn = t.hand(P0, "High Noon");
    mana(&mut t, P0, "W1R");
    t.cast(P0, hn).go();
    t.resolve_all();
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(!castable(&mut t, P0, bolt));
    // Put onto the battlefield: one spell, then no more.
    let mut t = TestGame::new(2);
    t.enter(P0, "High Noon");
    let bolts = [t.hand(P0, "Lightning Bolt"), t.hand(P0, "Lightning Bolt")];
    mana(&mut t, P0, "RR");
    assert!(castable(&mut t, P0, bolts[0]));
    let spell = t.cast(P0, bolts[0]).target(Entity::Player(P1)).go();
    // Countered: it still counts.
    counter_spell(&mut t, spell);
    assert!(!castable(&mut t, P0, bolts[1]));
    // P1 may still cast one this turn.
    let theirs = t.hand(P1, "Lightning Bolt");
    mana(&mut t, P1, "R");
    assert!(castable(&mut t, P1, theirs));
}

/// P0's Golos exiles the three cards on top of P0's library (top first); returns them.
fn golos(t: &mut TestGame, top_first: &[&str]) -> Vec<ObjectId> {
    supported("Golos, Tireless Pilgrim");
    let g = t.battlefield(P0, "Golos, Tireless Pilgrim");
    let cards = stack_library(t, P0, top_first);
    mana(t, P0, "WUBRG2");
    activate_containing(t, P0, g, "Exile the top three").unwrap();
    t.resolve_all();
    cards
        .iter()
        .map(|c| {
            assert_eq!(t.zone(*c), Zone::Exile);
            t.g.current(*c)
        })
        .collect()
}

#[test]
fn golos_doesnt_change_when_you_can_play_the_cards() {
    cr!("305.1", "307.1", "118.9");
    ruling!(
        "Golos, Tireless Pilgrim",
        "Golos's last ability doesn't change when you can play the exiled cards. For example, if you exile a sorcery card, you can cast it only during your main phase when the stack is empty. If you exile a land card, you can play it only during your main phase and only if you have an available land play remaining."
    );
    ruling!(
        "Golos, Tireless Pilgrim",
        "If you don't play some or all of the cards exiled with Golos's last ability, those cards remain in exile."
    );
    let mut t = TestGame::new(2);
    let cards = golos(&mut t, &["Divination", "Forest", "Grizzly Bears"]);
    let (sorcery, land, bears) = (cards[0], cards[1], cards[2]);
    // In combat: neither.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, sorcery));
    assert!(t.play_land(P0, land).is_err());
    t.set_step(P0, Step::PostcombatMain);
    // Already played a land this turn: not the Forest.
    t.g.players[0].lands_played_this_turn = 1;
    assert!(t.play_land(P0, land).is_err());
    t.g.players[0].lands_played_this_turn = 0;
    // The sorcery and the land, for free.
    assert!(castable(&mut t, P0, sorcery));
    let library = t.library_size(P0);
    let m = cast_methods(&mut t, P0, sorcery);
    t.cast(P0, sorcery).method(m[0].clone()).go();
    t.resolve_all();
    assert_eq!(library - t.library_size(P0), 2);
    t.play_land(P0, land).expect("the exiled land");
    assert!(t.on_battlefield(land));
    // The Bears weren't played: after the turn, they stay in exile.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!castable(&mut t, P0, bears));
}

#[test]
fn free_casting_still_requires_mandatory_additional_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Golos, Tireless Pilgrim",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs (such as that of Bone Splinters), you must pay those to cast the card."
    );
    supported("Bone Splinters");
    // Bone Splinters: "As an additional cost to cast this spell, sacrifice a creature.
    // Destroy target creature."
    let mut t = TestGame::new(2);
    let cards = golos(&mut t, &["Bone Splinters", "Forest", "Forest"]);
    let giant = t.battlefield(P1, "Hill Giant");
    // Sacrificing Grizzly Bears for it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = cast_methods(&mut t, P0, cards[0]);
    assert!(!m.is_empty());
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, cards[0]).method(m[0].clone()).target(giant).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "the additional cost was paid");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // With no creature to sacrifice (Golos is gone too), it can't be cast.
    let mut t = TestGame::new(2);
    let cards = golos(&mut t, &["Bone Splinters", "Forest", "Forest"]);
    t.battlefield(P1, "Hill Giant");
    let golos_id = t.named_on_battlefield("Golos, Tireless Pilgrim")[0];
    kill(&mut t, golos_id);
    assert!(!castable(&mut t, P0, cards[0]));
}

#[test]
fn soulfire_grand_master_and_spells_that_dont_go_to_the_graveyard() {
    cr!("608.2n", "614.1a", "701.6a");
    ruling!(
        "Soulfire Grand Master",
        "A spell that instructs you to exile it (for example, Temporal Trespass) won’t be put into your hand because it’s not put into your graveyard as it resolves."
    );
    ruling!(
        "Soulfire Grand Master",
        "After you activate Soulfire Grand Master’s last ability, if the next instant or sorcery spell you cast from your hand doesn’t resolve, it won’t be put into your hand and the effect won’t be applied to the next spell you cast."
    );
    supported("Soulfire Grand Master");
    supported("Temporal Trespass");
    let activate = |t: &mut TestGame, sgm: ObjectId| {
        mana(t, P0, "UU2");
        activate_containing(t, P0, sgm, "next time").unwrap();
        t.resolve_all();
    };
    // Control: Lightning Bolt returns to the hand.
    let mut t = TestGame::new(2);
    let sgm = t.battlefield(P0, "Soulfire Grand Master");
    activate(&mut t, sgm);
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, "R");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    // Temporal Trespass: "Take an extra turn after this one. Exile Temporal Trespass."
    let mut t = TestGame::new(2);
    let sgm = t.battlefield(P0, "Soulfire Grand Master");
    activate(&mut t, sgm);
    let tt = t.hand(P0, "Temporal Trespass");
    mana(&mut t, P0, "UUU8");
    t.cast(P0, tt).go();
    t.resolve_all();
    assert!(t.in_exile("Temporal Trespass"));
    assert!(!t.in_hand(P0, "Temporal Trespass"));
    // Countered: it goes to the graveyard, and the next spell isn't affected.
    let mut t = TestGame::new(2);
    let sgm = t.battlefield(P0, "Soulfire Grand Master");
    activate(&mut t, sgm);
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0, "R");
    let spell = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    counter_spell(&mut t, spell);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, "R");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
}

#[test]
fn activate_only_once_each_turn_follows_the_permanent_across_controllers() {
    cr!("602.5b", "113.6");
    ruling!(
        "Setessan Griffin",
        "If Setessan Griffin's ability is activated by one player, then another player gains control of it during the same turn, the second player can't activate its ability that turn."
    );
    supported("Setessan Griffin");
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Setessan Griffin");
    mana(&mut t, P0, "GG2");
    activate_containing(&mut t, P0, griffin, "+2/+2").unwrap();
    t.resolve_all();
    give_control(&mut t, griffin, P1);
    assert_eq!(t.obj_now(griffin).controller, P1);
    mana(&mut t, P1, "GG2");
    t.set_step(P0, Step::PrecombatMain);
    assert!(activate_containing(&mut t, P1, griffin, "+2/+2").is_err());
    // Control: next turn it can.
    t.clear_answers();
    t.advance_to(P1, Step::PrecombatMain);
    mana(&mut t, P1, "GG2");
    activate_containing(&mut t, P1, griffin, "+2/+2").expect("a new turn");
}

#[test]
fn sisay_searches_by_its_last_known_power_and_x_is_zero() {
    cr!("608.2h", "202.3e", "701.23a");
    ruling!(
        "Sisay, Weatherlight Captain",
        "If Sisay leaves the battlefield after you've activated its last ability but before that ability resolves, use its last known existence on the battlefield to determine what cards you can find."
    );
    ruling!(
        "Sisay, Weatherlight Captain",
        "If a card in a library has {X} in its mana cost, X is considered to be 0."
    );
    supported("Sisay, Weatherlight Captain");
    supported("The Meathook Massacre");
    // Sisay with Isamaru (white, legendary) is 3/3: "Search your library for a legendary
    // permanent card with mana value less than Sisay's power". The Meathook Massacre
    // ({X}{B}{B}) has mana value 2 in the library; Squee ({2}{R}) has 3.
    let mut t = TestGame::new(2);
    let sisay = t.battlefield(P0, "Sisay, Weatherlight Captain");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    assert_eq!(t.pt(sisay), (3, 3));
    let meathook = t.library_top(P0, "The Meathook Massacre");
    let squee = t.library_top(P0, "Squee, Goblin Nabob");
    mana(&mut t, P0, "WUBRG");
    activate_containing(&mut t, P0, sisay, "Search").unwrap();
    kill(&mut t, sisay);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(meathook)]);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .flat_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => candidates.clone(),
            _ => vec![],
        })
        .collect();
    assert!(offered.contains(&obj(meathook)));
    assert!(!offered.contains(&obj(squee)));
    assert!(t.on_battlefield(meathook));
}
