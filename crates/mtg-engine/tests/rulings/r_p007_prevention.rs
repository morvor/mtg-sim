//! Rulings batch P007 — prevention and replacement effects around burn: "damage can't be
//! prevented" (CR 615.12), "players can't gain life" (CR 119.7), Vengeful Archon's
//! prevention shield with an additional effect (CR 615.7, 616.1), "if it would die,
//! exile it instead" (CR 614.6), and doubled combat damage with trample (CR 702.19b).

use crate::r_p007_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::destroy;
use crate::r_s05_common::run_from;
use crate::r_s06_common::damage;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 activates Vengeful Archon's ability with X = `x` targeting `target`; it resolves.
fn archon_shield(t: &mut TestGame, archon: ObjectId, x: i64, target: Entity) {
    t.lands(P0, "Wastes", x as usize);
    t.answer(P0, DecisionKind::X, Answer::Number(x));
    t.activate(P0, archon, 0, &[target])
        .expect("Vengeful Archon");
    t.resolve_all();
}

/// A source P1 controls deals `n` noncombat damage to P0 (as a resolving effect would).
fn hit_p0(t: &mut TestGame, src: ObjectId, n: u32) {
    t.g.deal_damage(src, pl(P0), n, false);
    t.g.flush_events();
    t.settle();
}

#[test]
fn vengeful_archon_deals_damage_as_part_of_the_prevention() {
    cr!("615.7", "608.2", "704.3");
    ruling!(
        "Vengeful Archon",
        "If Vengeful Archon’s ability prevents damage, Vengeful Archon deals its damage immediately afterward as part of that same prevention effect. This happens before state-based actions are checked, and before any player can cast spells or activate abilities."
    );
    ruling!(
        "Vengeful Archon",
        "If the amount of damage that would be dealt to you is in excess of the amount of damage that Vengeful Archon’s ability would prevent, that source deals its excess damage to you at the same time that the rest of it is prevented. Then Vengeful Archon deals its damage."
    );
    supported("Vengeful Archon");
    // P1 casts Shock at P0: as it resolves, the 2 damage is prevented and the Archon
    // deals 2 to P1 — no trigger waits on the stack.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    archon_shield(&mut t, archon, 3, pl(P1));
    cast_new(&mut t, P1, "Shock", &[pl(P0)]);
    t.g.resolve_top();
    assert_eq!(t.stack_len(), 0, "no trigger");
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    // 5 damage against a 2 shield: P0 is dealt 3, then the Archon deals 2.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let src = t.battlefield(P1, "Grizzly Bears");
    archon_shield(&mut t, archon, 2, pl(P1));
    hit_p0(&mut t, src, 5);
    assert_eq!((t.life(P0), t.life(P1)), (17, 18));
    // The rest of the shield is used up.
    hit_p0(&mut t, src, 1);
    assert_eq!((t.life(P0), t.life(P1)), (16, 18));
}

#[test]
fn vengeful_archon_deals_nothing_if_nothing_is_prevented() {
    cr!("615.12", "616.1");
    ruling!(
        "Vengeful Archon",
        "If Vengeful Archon’s ability doesn’t prevent any damage (perhaps because a different prevention effect is applied to the damage that would be dealt to you, or because the damage is unpreventable), Vengeful Archon won’t deal any damage itself."
    );
    // Unpreventable: Skullcrack resolved this turn.
    let mut t = TestGame::new(2);
    let archon = t.battlefield(P0, "Vengeful Archon");
    let src = t.battlefield(P1, "Grizzly Bears");
    archon_shield(&mut t, archon, 3, pl(P1));
    cast_new(&mut t, P0, "Skullcrack", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    hit_p0(&mut t, src, 2);
    assert_eq!((t.life(P0), t.life(P1)), (18, 17));
    // Another prevention shield (Healing Salve's 3) applied first absorbs the 2 damage.
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(2);
        let archon = t.battlefield(P0, "Vengeful Archon");
        let src = t.battlefield(P1, "Grizzly Bears");
        archon_shield(&mut t, archon, 3, pl(P1));
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
        cast_new(&mut t, P0, "Healing Salve", &[pl(P0)]);
        t.resolve_all();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        hit_p0(&mut t, src, 2);
        outcomes.push((t.life(P0), t.life(P1)));
    }
    outcomes.sort();
    // The Salve first: the Archon prevents nothing and deals nothing. The Archon first:
    // it prevents 2 and deals 2.
    assert_eq!(outcomes, vec![(20, 18), (20, 20)]);
}

#[test]
fn the_player_dealt_damage_orders_vengeful_archon_shields() {
    cr!("616.1", "615.7");
    ruling!(
        "Vengeful Archon",
        "If multiple effects modify how damage will be dealt, the player who would be dealt damage chooses the order to apply the effects."
    );
    // Shields X = 3 at P1 and X = 1 at P2; P0 is dealt 2.
    let mut outcomes = vec![];
    for pick in 0..2 {
        let mut t = TestGame::new(3);
        let archon = t.battlefield(P0, "Vengeful Archon");
        let src = t.battlefield(P1, "Grizzly Bears");
        archon_shield(&mut t, archon, 3, pl(P1));
        archon_shield(&mut t, archon, 1, pl(P2));
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        hit_p0(&mut t, src, 2);
        let first = (t.life(P0), t.life(P1), t.life(P2));
        // The unused portions remain: another 2 damage.
        hit_p0(&mut t, src, 2);
        outcomes.push((first, (t.life(P0), t.life(P1), t.life(P2))));
    }
    outcomes.sort();
    // The X = 1 shield first: 1 to P2, then 1 to P1 (2 left of the X = 3 shield, used
    // by the next 2). The X = 3 shield first: 2 to P1 (1 left of it and the X = 1 shield,
    // used by the next 2 in some order).
    assert_eq!(outcomes[0].0, (20, 18, 20), "{outcomes:?}");
    assert_eq!(outcomes[1], ((20, 19, 19), (20, 17, 19)), "{outcomes:?}");
    assert_eq!(outcomes[0].1, (20, 17, 19), "{outcomes:?}");
}

#[test]
fn unstable_footing_stops_prevention_but_not_other_damage_replacements() {
    cr!("615.12", "614.1a", "615.1");
    ruling!(
        "Unstable Footing",
        "Effects that replace or redirect damage without using the word \"prevent\" are not affected by Unstable Footing."
    );
    ruling!(
        "Unstable Footing",
        "If damage can't be prevented, damage prevention shields don't have any effect. If a prevention effect has an additional effect, the additional effect will still work (if possible). Spells that create prevention effects can still be cast, and abilities that create prevention effects can still be activated."
    );
    ruling!(
        "Unstable Footing",
        "If damage can't be prevented, static abilities that prevent damage (including protection abilities) don't do so."
    );
    supported("Unstable Footing");
    // Benevolent Unicorn ("deals that much damage minus 1 instead") still applies.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Benevolent Unicorn");
    cast_new(&mut t, P0, "Unstable Footing", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Withstand can be cast; its shield does nothing, but its "draw a card" happens.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Unstable Footing", &[]);
    t.resolve_all();
    t.library_top(P1, "Island");
    let hand = t.hand_size(P1);
    cast_new(&mut t, P1, "Withstand", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(
        t.hand_size(P1),
        hand + 1,
        "the additional effect still works"
    );
    cast_new(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "the shield doesn't prevent anything");
    // Protection from red doesn't prevent Shock's damage.
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P1, "Kor Firewalker");
    cast_new(&mut t, P0, "Unstable Footing", &[]);
    t.resolve_all();
    let src = t.battlefield(P0, "Hill Giant");
    damage(&mut t, src, 2, kor);
    assert!(
        !t.on_battlefield(kor),
        "red damage isn't prevented by protection"
    );
}

#[test]
fn skullcrack_stops_life_gain_and_its_replacements() {
    cr!("119.7", "119.5", "614.1a");
    ruling!(
        "Skullcrack",
        "Effects that would replace gaining life with another effect won't apply because it's impossible for players to gain life."
    );
    ruling!(
        "Skullcrack",
        "If an effect says to set a player's life total to a certain number and that number is higher than the player's current life total, that part of the effect won't do anything. (If the number is lower than the player's current life total, the effect will work as normal.)"
    );
    // Tainted Remedy ("if an opponent would gain life, that player loses that much life
    // instead") doesn't apply: P1 can't gain life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tainted Remedy");
    cast_new(&mut t, P0, "Skullcrack", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.g.gain_life(P1, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P1), 17);
    // Blessed Wind ("target player's life total becomes 20"): up does nothing, down works.
    for (start, end) in [(10, 7), (30, 20)] {
        let mut t = TestGame::new(2);
        t.g.players[1].life = start;
        cast_new(&mut t, P0, "Skullcrack", &[pl(P1)]);
        t.resolve_all();
        cast_new(&mut t, P0, "Blessed Wind", &[pl(P1)]);
        t.resolve_all();
        assert_eq!(t.life(P1), end, "from {start}");
    }
}

#[test]
fn exile_instead_of_dying_replacements() {
    cr!("614.6", "615.1");
    ruling!(
        "Blitz of the Thunder-Raptor",
        "Blitz of the Thunder-Raptor’s replacement effect will exile the target creature or planeswalker if it would die this turn for any reason, not just due to lethal damage. It applies to the target creature or planeswalker even if Blitz of the Thunder-Raptor deals no damage to it"
    );
    ruling!(
        "Chandra, Awakened Inferno",
        "If Chandra's last ability doesn't deal damage (most likely because the damage has been prevented in some way), the target won't be exiled if it would die."
    );
    supported("Chandra, Awakened Inferno");
    // Blitz with no instants or sorceries in the graveyard: 0 damage, then destroyed:
    // exiled.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Blitz of the Thunder-Raptor", &[obj(giant)]);
    t.resolve_all();
    destroy(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
    // Chandra −1 at a protection-from-red creature: prevented, so destroyed normally.
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P1, "Kor Firewalker");
    let c = pw(&mut t, P0, "Chandra, Awakened Inferno", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    loyalty(&mut t, P0, c, 2, &[obj(kor)]);
    t.resolve_all();
    destroy(&mut t, kor);
    assert!(t.in_graveyard(P1, "Kor Firewalker"));
    // Control: dealt 1, then destroyed: exiled.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let c = pw(&mut t, P0, "Chandra, Awakened Inferno", 6);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    loyalty(&mut t, P0, c, 2, &[obj(giant)]);
    t.resolve_all();
    destroy(&mut t, giant);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn insult_trample_assigns_unmodified_lethal_damage() {
    cr!("702.19b", "510.1c", "614.1a");
    ruling!(
        "Insult // Injury",
        "If a creature with trample you control would deal combat damage to a blocking creature this turn, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can only have 1 damage assigned to the defending player. It will then deal 4 damage to the blocking creature and 2 to the defending player."
    );
    supported("Insult // Injury");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Trample))],
            duration: Duration::EndOfTurn,
        },
        &[obj(giant)],
    );
    t.lands(P0, "Mountain", 3);
    let card = t.hand(P0, "Insult // Injury");
    t.cast(P0, card).method(CastMethod::Half(0)).go();
    t.resolve_all();
    attack_with(&mut t, &[(giant, pl(P1))]);
    // Trying to assign only 1 (half of the doubled lethal) to the blocker isn't allowed.
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![1, 2]));
    block_and_finish(&mut t, P1, &[(bears, giant)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}
