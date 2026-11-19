//! Rulings batch P170 — copies of spells and abilities (CR 707.10): Repeated
//! Reverberation's delayed trigger, Feather's and Kitsa's copies (not cast, CR 707.10;
//! permanent spell copies become tokens without being created, CR 707.10f, 111.1), who
//! owns a copy of a spell and a token (CR 707.10, 111.2), and token copies of tokens
//! (CR 707.2, 111.3).

use crate::r_p170_common::*;
use crate::r_s25_common::{change_copy_targets, keep_copy_targets, spell_copies, x_of};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Repeated Reverberation and it resolves.
fn reverberation(t: &mut TestGame) -> ObjectId {
    supported("Repeated Reverberation");
    let rr = cast_card(t, P0, "Repeated Reverberation", &[]);
    t.resolve_all();
    rr
}

fn count_asked(t: &TestGame, from: usize, f: fn(&Decision) -> bool) -> usize {
    t.asked()[from..].iter().filter(|(_, d)| f(d)).count()
}

#[test]
fn repeated_reverberation_copies_even_if_the_spell_was_countered() {
    cr!("603.7a", "707.10", "405.2");
    ruling!(
        "Repeated Reverberation",
        "Copies are created even if the spell or ability that caused Repeated Reverberation’s ability to trigger has been countered by the time that ability resolves. The copies resolve before the original spell or ability."
    );
    ruling!(
        "Repeated Reverberation",
        "Repeated Reverberation’s ability will copy any instant or sorcery spell or loyalty ability, not just one with targets."
    );
    // Divination (no targets), countered: two copies all the same.
    let mut t = TestGame::new(2);
    reverberation(&mut t);
    let div = cast_card(&mut t, P0, "Divination", &[]);
    assert!(trigger_on_top(&t));
    leave(&mut t, div, P0, Leaves::Countered);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 2);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 26, "two copies drew two each");
    // Not countered: the copies are above the original and resolve first.
    let mut t = TestGame::new(2);
    reverberation(&mut t);
    let div = cast_card(&mut t, P0, "Divination", &[]);
    t.resolve();
    assert_eq!(t.g.stack.len(), 3);
    assert_eq!(t.g.stack[0], div);
    t.resolve();
    t.resolve();
    assert_eq!(t.library_size(P0), 26);
    assert_eq!(t.g.stack, vec![div]);
    t.resolve();
    assert_eq!(t.library_size(P0), 24);
    // A loyalty ability with no targets: Ral's +1 makes three Otters.
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral, Crackling Wit");
    reverberation(&mut t);
    t.activate(P0, ral, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Otter").len(), 3);
}

#[test]
fn repeated_reverberation_copies_keep_x_and_modes() {
    cr!("707.10", "700.2g");
    ruling!(
        "Repeated Reverberation",
        "If the spell or ability that’s copied has an X whose value was determined as it was cast, the copies will have the same value of X."
    );
    ruling!(
        "Repeated Reverberation",
        "If the spell that’s copied is modal (that is, it says “Choose one —” or the like), the copies will have the same mode or modes. You can’t choose different ones."
    );
    // Mind Spring with X = 2: each copy draws two.
    let mut t = TestGame::new(2);
    reverberation(&mut t);
    lands_for_cost(&mut t, P0, "Mind Spring");
    t.lands(P0, "Island", 2);
    let ms = t.hand(P0, "Mind Spring");
    t.cast(P0, ms).x(2).go();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert_eq!(x_of(&t, c), Some(2));
    }
    t.resolve_all();
    assert_eq!(t.library_size(P0), 24);
    // Boros Charm's first mode: the copies deal 4 each to P1, no mode asked for them.
    let mut t = TestGame::new(2);
    reverberation(&mut t);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_card(&mut t, P0, "Boros Charm", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::ChooseModes { .. })),
        1
    );
}

#[test]
fn repeated_reverberation_copies_may_change_any_number_of_targets() {
    cr!("707.10c");
    ruling!(
        "Repeated Reverberation",
        "The copies will have the same targets as the spell or ability they’re copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    reverberation(&mut t);
    // The first copy is retargeted to the Hill Giant, the second keeps P1.
    cast_card(&mut t, P0, "Shock", &[pl(P1)]);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.obj_now(giant).damage, 2);
    // A new target must be legal: a Shock copy can't target the Shock itself, nor a
    // creature with hexproof (Gladecover Scout); asked for one, P0 keeps the old target.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    reverberation(&mut t);
    cast_card(&mut t, P0, "Shock", &[pl(P1)]);
    change_copy_targets(&mut t, P0, &[Some(obj(scout))]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(scout));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn repeated_reverberations_compound() {
    // 603.7b: each delayed trigger copies only the next spell (the first Reverberation's
    // doesn't copy the Shock too).
    cr!("603.7b", "707.10");
    ruling!(
        "Repeated Reverberation",
        "If you cast one Repeated Reverberation after another, you’ll copy the second Repeated Reverberation twice. Each resolves, creating three delayed triggered abilities. The next spell you cast or loyalty ability you activate after that will be copied six times."
    );
    // Two Reverberations: the Shock is copied six times (7 Shocks).
    let mut t = TestGame::new(2);
    t.g.players[1].life = 100;
    reverberation(&mut t);
    reverberation(&mut t);
    cast_card(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 100 - 14);
    // Three: the third is copied six times, so the Shock is copied fourteen times.
    let mut t = TestGame::new(2);
    t.g.players[1].life = 100;
    reverberation(&mut t);
    reverberation(&mut t);
    reverberation(&mut t);
    cast_card(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 100 - 30);
}

/// P0 controls Feather; P0 will choose `others` for copies and pay {2} for each (lands
/// added). Returns Feather.
fn feather(t: &mut TestGame, others: &[ObjectId]) -> ObjectId {
    supported("Feather, Radiant Arbiter");
    let f = t.battlefield(P0, "Feather, Radiant Arbiter");
    let es: Vec<Entity> = others.iter().map(|o| obj(*o)).collect();
    t.answer_choose(P0, &es);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.lands(P0, "Wastes", 2 * others.len());
    f
}

#[test]
fn feathers_copies_are_not_cast() {
    cr!("707.10", "702.108a");
    ruling!(
        "Feather, Radiant Arbiter",
        "The copies created by Feather, Radiant Arbiter's last ability are created on the stack, so they're not cast. Abilities that trigger when a player casts a spell (like Feather's own ability) won't trigger."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    let f = feather(&mut t, &[bears]);
    cast_card(&mut t, P0, "Giant Growth", &[obj(f)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5), "the copy resolved");
    assert_eq!(crate::r_p189_common::triggers_of(&t, f), 1);
    // Prowess triggered once, for the original only.
    assert_eq!(crate::r_p189_common::triggers_of(&t, spear), 1);
    assert_eq!(t.pt(spear), (2, 3));
}

#[test]
fn a_token_from_a_feather_copy_of_a_permanent_spell_is_not_created() {
    cr!("707.10f", "608.3f", "614.1a");
    ruling!(
        "Feather, Radiant Arbiter",
        "The token that a resolving copy of a permanent spell becomes isn't \"created\" and won't interact with abilities that care about tokens being created, such as that of Case of the Pilfered Proof."
    );
    // Doubling Season ("If an effect would create one or more tokens under your control,
    // it creates twice that many of those tokens instead") doesn't apply to it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = feather(&mut t, &[bears]);
    cast_card(&mut t, P0, "Holy Strength", &[obj(f)]);
    t.resolve_all();
    let auras: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.chars.name == "Holy Strength")
            .map(|o| o.id)
            .collect();
    assert_eq!(auras.len(), 2, "the card and one token");
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn feathers_copies_copy_kicker_without_paying_it() {
    cr!("707.10", "702.33d");
    ruling!(
        "Feather, Radiant Arbiter",
        "You can't choose to pay any alternative or additional costs for the copies. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    // Gift of Growth kicked on Feather: the copy gives the Bears +4/+4 too, and P0 is
    // asked about kicker only for the original.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = feather(&mut t, &[bears]);
    lands_for_cost(&mut t, P0, "Gift of Growth");
    t.lands(P0, "Wastes", 2);
    let gift = t.hand(P0, "Gift of Growth");
    let from = t.asked().len();
    t.cast(P0, gift).target(f).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.pt(f), (8, 7));
    assert_eq!(t.pt(bears), (6, 6));
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::OptionalCost { .. })),
        1,
        "kicker offered for the original only"
    );
    // Not kicked: the copy isn't either.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let f = feather(&mut t, &[bears]);
    lands_for_cost(&mut t, P0, "Gift of Growth");
    t.lands(P0, "Wastes", 2);
    let gift = t.hand(P0, "Gift of Growth");
    t.cast(P0, gift).target(f).kicked(false).go();
    t.resolve_all();
    assert_eq!(t.pt(f), (6, 5));
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn feathers_copies_are_yours_above_the_original() {
    cr!("707.10", "405.2", "405.3");
    ruling!(
        "Feather, Radiant Arbiter",
        "You control all the copies, and you choose the order the copies are put on the stack. The original spell will be on the stack beneath those copies and will resolve last."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let f = feather(&mut t, &[bears, giant]);
    let gg = cast_card(&mut t, P0, "Giant Growth", &[obj(f)]);
    // P0 orders the copies: the Hill Giant's first, so the Bears' is on top.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.resolve();
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::Order { .. })),
        1
    );
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2);
    assert_eq!(t.g.stack[0], gg);
    for c in &copies {
        assert_eq!(t.obj_now(*c).controller, P0);
    }
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(giant), (3, 3));
    t.resolve();
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(f), (4, 3), "the original resolves last");
    assert_eq!(t.g.stack, vec![gg]);
    t.resolve();
    assert_eq!(t.pt(f), (7, 6));
}

/// Kitsa with a +1/+1 counter and an Opt on the stack whose prowess trigger has resolved
/// (Kitsa is 3/5). Returns (Kitsa, Opt).
fn kitsa_with_opt(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Kitsa, Otterball Elite");
    let kitsa = t.battlefield(P0, "Kitsa, Otterball Elite");
    add_counters(t, kitsa, "+1/+1", 1);
    let opt = cast_card(t, P0, "Opt", &[]);
    t.resolve();
    assert_eq!(t.pt(kitsa), (3, 5));
    t.lands(P0, "Wastes", 2);
    (kitsa, opt)
}

#[test]
fn kitsas_copy_ability_resolves_even_if_her_power_drops() {
    // 602.1b: "Activate only if ..." is an activation instruction, not part of the effect;
    // 113.7a: once activated, the ability is independent of its source.
    cr!("602.1b", "113.7a");
    ruling!(
        "Kitsa, Otterball Elite",
        "Once you’ve activated Kitsa’s last ability, any changes to Kitsa’s power won’t stop the ability from resolving."
    );
    let mut t = TestGame::new(2);
    let (kitsa, opt) = kitsa_with_opt(&mut t);
    crate::r_s06_common::activate_containing(&mut t, P0, kitsa, "Copy target")
        .expect("Kitsa's ability can be activated");
    // In response, Kitsa loses the counter: power 2.
    t.g.remove_counters(Entity::Object(kitsa), "+1/+1", 1);
    t.g.flush_events();
    assert_eq!(t.pt(kitsa), (2, 4));
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    assert_eq!(t.g.stack[0], opt);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn kitsas_copy_is_not_cast() {
    cr!("707.10", "702.108a");
    ruling!(
        "Kitsa, Otterball Elite",
        "The copy created by Kitsa’s last ability is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    let mut t = TestGame::new(2);
    let (kitsa, _) = kitsa_with_opt(&mut t);
    crate::r_s06_common::activate_containing(&mut t, P0, kitsa, "Copy target")
        .expect("Kitsa's ability can be activated");
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    // Prowess triggered only for the Opt that was cast.
    assert_eq!(crate::r_p189_common::triggers_of(&t, kitsa), 1);
    assert_eq!(t.pt(kitsa), (3, 5));
    assert_eq!(t.library_size(P0), 28);
}

#[test]
fn don_andres_ownership_of_spell_copies_and_tokens() {
    cr!("707.10", "111.2");
    ruling!(
        "Don Andres, the Renegade",
        "The owner of a copy of a spell is the player under whose control it was put onto the stack."
    );
    ruling!(
        "Don Andres, the Renegade",
        "The player who creates a token is its owner."
    );
    supported("Don Andres, the Renegade");
    // P0 copies P1's Lightning Bolt with Reverberate: P0 owns the copy (so it isn't a
    // spell P0 doesn't own), and a copy isn't cast anyway: no Treasures.
    let mut t = TestGame::new(2);
    let don = t.battlefield(P0, "Don Andres, the Renegade");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bolt = cast_card(&mut t, P1, "Lightning Bolt", &[pl(P0)]);
    t.answer_targets(P0, &[obj(bolt)]);
    keep_copy_targets(&mut t, P0);
    cast_card(&mut t, P0, "Reverberate", &[]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(t.obj_now(copy).owner, P0);
    assert_eq!(t.obj_now(copy).controller, P0);
    assert_eq!(t.obj_now(bolt).owner, P1);
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Treasure").is_empty());
    // P1 creates Soldier tokens (P1 owns them); P0 gains control of one: Don Andres
    // gives it +2/+2, menace and deathtouch.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Don Andres, the Renegade");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_card(&mut t, P1, "Raise the Alarm", &[]);
    t.resolve_all();
    let soldiers = tokens_with_subtype(&t, P1, "Soldier");
    assert_eq!(soldiers.len(), 2);
    assert_eq!(t.obj_now(soldiers[0]).owner, P1);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    cast_card(&mut t, P0, "Act of Treason", &[obj(soldiers[0])]);
    t.resolve_all();
    let s = soldiers[0];
    assert_eq!(t.obj_now(s).controller, P0);
    assert_eq!(t.obj_now(s).owner, P1);
    assert_eq!(t.pt(s), (3, 3));
    assert!(t.obj_now(s).has_keyword(KeywordKind::Deathtouch));
    assert!(t.obj_now(s).chars.has_subtype("Pirate"));
    let _ = don;
}

#[test]
fn capricious_hellraiser_exiles_at_random_and_may_find_nothing_to_copy() {
    cr!("701.13a", "707.12");
    ruling!(
        "Capricious Hellraiser",
        "The three cards you exile are chosen at random from among all cards in your graveyard. If all three cards you exile this way are creature and/or land cards, you won't be able to choose one of them to copy and cast."
    );
    supported("Capricious Hellraiser");
    let mut t = TestGame::new(2);
    for n in [
        "Grizzly Bears",
        "Forest",
        "Hill Giant",
        "Plains",
        "Llanowar Elves",
    ] {
        t.graveyard(P0, n);
    }
    let from = t.asked().len();
    t.enter(P0, "Capricious Hellraiser");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 2);
    let exiled = t.g.exile.len();
    assert_eq!(exiled, 3);
    // No choice of the cards to exile, and nothing to copy: no spell was cast.
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::ChooseEntities { .. })),
        0
    );
    assert!(t.g.stack.is_empty());
    // Over several seeds, different cards are exiled (they're chosen at random).
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..12u64 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        for n in [
            "Grizzly Bears",
            "Forest",
            "Hill Giant",
            "Plains",
            "Llanowar Elves",
        ] {
            t.graveyard(P0, n);
        }
        t.enter(P0, "Capricious Hellraiser");
        t.resolve_all();
        let mut left: Vec<String> = t.g.players[0]
            .graveyard
            .iter()
            .map(|id| t.g.obj(*id).chars.name.to_string())
            .collect();
        left.sort();
        seen.insert(left);
    }
    assert!(seen.len() > 1, "always the same cards: {seen:?}");
}

#[test]
fn good_king_mogs_token_copy_uses_the_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Summon: Good King Mog XII",
        "The new token copies the original characteristics of the copied token as stated by the effect that created that token."
    );
    supported("Summon: Good King Mog XII");
    let mut t = TestGame::new(2);
    let mog = t.enter(P0, "Summon: Good King Mog XII");
    t.resolve_all();
    let moogles = tokens_with_subtype(&t, P0, "Moogle");
    let m = moogles[0];
    // The Moogle has a +1/+1 counter and Holy Strength.
    add_counters(&mut t, m, "+1/+1", 1);
    cast_card(&mut t, P0, "Holy Strength", &[obj(m)]);
    t.resolve_all();
    assert_eq!(t.pt(m), (3, 5));
    add_counters(&mut t, mog, "lore", 1);
    t.resolve_all();
    t.answer_choose(P0, &[obj(m)]);
    cast_card(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    let new: Vec<ObjectId> = tokens_with_subtype(&t, P0, "Moogle")
        .into_iter()
        .filter(|id| !moogles.contains(id))
        .collect();
    assert_eq!(new.len(), 1);
    let c = new[0];
    assert_eq!(t.pt(c), (1, 2));
    assert_eq!(t.counters(c, "+1/+1"), 0);
    assert!(t.obj_now(c).has_keyword(KeywordKind::Lifelink));
    assert_eq!(t.obj_now(c).chars.name, t.obj_now(m).chars.name);
}
