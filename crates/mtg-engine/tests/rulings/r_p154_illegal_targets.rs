//! Rulings batch P154 — spells and abilities that scale with a mana value whose target
//! became illegal: with every target illegal the spell or ability doesn't resolve and
//! none of its effects happen (CR 608.2b), while a legal target that the effect fails to
//! affect (indestructible, "can't be countered") still gets the rest of the text applied
//! (CR 608.2c); with one of two targets illegal, the spell resolves and does what it can
//! (CR 608.2b).

use crate::r_p154_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn artifact_destroyers_gain_no_life_for_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Crumble",
        "If the target artifact becomes illegal before resolution, the player does not gain any life."
    );
    ruling!(
        "Divine Offering",
        "If the target artifact becomes illegal before resolution, you do not gain any life."
    );
    supported("Crumble");
    supported("Divine Offering");
    // Crumble: the artifact's controller (P1) would gain its mana value (2).
    for (card, cost, illegal) in [
        ("Crumble", "G", true),
        ("Crumble", "G", false),
        ("Divine Offering", "1W", true),
        ("Divine Offering", "1W", false),
    ] {
        let mut t = TestGame::new(2);
        let stone = t.battlefield(P1, "Mind Stone");
        let spell = cast(&mut t, P0, card, cost, &[obj(stone)]);
        if illegal {
            bounce(&mut t, stone);
        }
        t.resolve_all();
        assert_eq!(resolved(&t, spell), !illegal, "{card} illegal: {illegal}");
        let gainer = if card == "Crumble" { P1 } else { P0 };
        assert_eq!(t.life(gainer), if illegal { 20 } else { 22 }, "{card} illegal: {illegal}");
        assert!(t.in_graveyard(P0, card));
    }
}

#[test]
fn vengeful_rebirth_with_one_of_its_two_targets_illegal() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Vengeful Rebirth",
        "If the target card in your graveyard becomes an illegal target but the target permanent or player doesn’t, Vengeful Rebirth won’t return that card to your hand and it won’t deal any damage (because you didn’t return a nonland card to your hand this way). Vengeful Rebirth will still be exiled."
    );
    ruling!(
        "Vengeful Rebirth",
        "If the target permanent or player becomes an illegal target but the target card in your graveyard doesn’t, Vengeful Rebirth will still return that card from your graveyard to your hand, but it won’t deal any damage. Vengeful Rebirth will then be exiled."
    );
    supported("Vengeful Rebirth");
    // The card is exiled in response: no return, no damage, Rebirth exiled.
    let mut t = TestGame::new(2);
    let giant_card = t.graveyard(P0, "Hill Giant");
    let victim = t.battlefield(P1, "Indomitable Ancients");
    let spell = cast(&mut t, P0, "Vengeful Rebirth", "4RG", &[obj(giant_card), obj(victim)]);
    exile(&mut t, giant_card);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(t.in_exile("Hill Giant") && !t.in_hand(P0, "Hill Giant"));
    assert_eq!(dmg(&t, victim), 0);
    assert!(t.in_exile("Vengeful Rebirth"));
    // The creature leaves in response: the card is returned, no damage, Rebirth exiled.
    let mut t = TestGame::new(2);
    let giant_card = t.graveyard(P0, "Hill Giant");
    let victim = t.battlefield(P1, "Indomitable Ancients");
    let spell = cast(&mut t, P0, "Vengeful Rebirth", "4RG", &[obj(giant_card), obj(victim)]);
    bounce(&mut t, victim);
    t.resolve_all();
    assert!(resolved(&t, spell));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_exile("Vengeful Rebirth"));
    // Both legal: the card returns and the Ancients (2/10) are dealt 4.
    let mut t = TestGame::new(2);
    let giant_card = t.graveyard(P0, "Hill Giant");
    let victim = t.battlefield(P1, "Indomitable Ancients");
    cast_resolve(&mut t, P0, "Vengeful Rebirth", "4RG", &[obj(giant_card), obj(victim)]);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(dmg(&t, victim), 4);
    assert!(t.in_exile("Vengeful Rebirth"));
}

#[test]
fn self_exiling_spells_with_an_illegal_target_go_to_the_graveyard_and_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Sheoldred's Restoration",
        "If the target creature card is an illegal target as Sheoldred's Restoration tries to resolve (usually because the card has been removed from the graveyard in response), it won't do anything. You won't gain or lose life, and Sheoldred's Restoration will go to the graveyard rather than be exiled by its last ability."
    );
    ruling!(
        "Volcanic Vision",
        "If the target instant or sorcery card becomes an illegal target before Volcanic Vision resolves, the spell won’t resolve and none of its effects will happen. No damage will be dealt and Volcanic Vision won’t be exiled."
    );
    supported("Sheoldred's Restoration");
    supported("Volcanic Vision");
    // Sheoldred's Restoration, kicked and not: the Hill Giant card is exiled in response.
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        let giant_card = t.graveyard(P0, "Hill Giant");
        let r = t.hand(P0, "Sheoldred's Restoration");
        mana(&mut t, P0, "3B2W");
        let spell = t.cast(P0, r).kicked(kicked).target(obj(giant_card)).go();
        exile(&mut t, giant_card);
        t.resolve_all();
        assert!(!resolved(&t, spell), "kicked: {kicked}");
        assert_eq!(t.life(P0), 20, "kicked: {kicked}");
        assert!(t.in_graveyard(P0, "Sheoldred's Restoration"), "kicked: {kicked}");
        assert_eq!(count_named(&t, "Hill Giant"), 0);
    }
    // Both kinds of Restoration with a legal target: 4 life gained or lost, and exiled.
    for (kicked, life) in [(false, 16), (true, 24)] {
        let mut t = TestGame::new(2);
        let giant_card = t.graveyard(P0, "Hill Giant");
        let r = t.hand(P0, "Sheoldred's Restoration");
        mana(&mut t, P0, "3B2W");
        t.cast(P0, r).kicked(kicked).target(obj(giant_card)).go();
        t.resolve_all();
        assert_eq!(t.life(P0), life, "kicked: {kicked}");
        assert_eq!(count_named(&t, "Hill Giant"), 1);
        assert!(t.in_exile("Sheoldred's Restoration"));
    }
    // Volcanic Vision: the Fireball card is exiled in response.
    let mut t = TestGame::new(2);
    let fireball = t.graveyard(P0, "Fireball");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = cast(&mut t, P0, "Volcanic Vision", "5RR", &[obj(fireball)]);
    exile(&mut t, fireball);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(dmg(&t, giant), 0);
    assert!(t.in_graveyard(P0, "Volcanic Vision"));
}

#[test]
fn enters_triggers_with_an_illegal_target_dont_resolve() {
    cr!("608.2b");
    ruling!(
        "Niambi, Esteemed Speaker",
        "If the target creature is an illegal target by the time Niambi's triggered ability tries to resolve, the ability won't resolve. You won't return the card from the zone it moved to or gain any life."
    );
    ruling!(
        "Hoard-Smelter Dragon",
        "If the targeted artifact is an illegal target by the time the activated ability resolves, the ability doesn't resolve. Hoard-Smelter Dragon won't get the bonus."
    );
    supported("Niambi, Esteemed Speaker");
    supported("Hoard-Smelter Dragon");
    // Niambi's target (a Hill Giant) is exiled in response: it isn't returned from exile
    // and no life is gained.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Niambi, Esteemed Speaker");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    exile(&mut t, giant);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant") && !t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 20);
    // With a legal target: the Giant returns to hand and 4 life is gained.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.enter(P0, "Niambi, Esteemed Speaker");
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 24);
    // Hoard-Smelter Dragon's ability targeting a Mind Stone that leaves in response.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Hoard-Smelter Dragon");
    let stone = t.battlefield(P1, "Mind Stone");
    mana(&mut t, P0, "3R");
    act(&mut t, P0, dragon, "Destroy target artifact", &[obj(stone)]).expect("Dragon");
    bounce(&mut t, stone);
    t.resolve_all();
    assert_eq!(t.pt(dragon).0, 5, "no bonus");
}

#[test]
fn megatrons_reflexive_trigger_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b", "603.12");
    ruling!(
        "Megatron, Tyrant // Megatron, Destructive Force",
        "If the target creature is an illegal target by the time the reflexive triggered ability tries to resolve, the ability won't resolve. It won't deal damage to any player."
    );
    supported("Megatron, Tyrant // Megatron, Destructive Force");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Megatron, Tyrant // Megatron, Destructive Force");
    mtg_engine::dfc::transform(&mut t.g, m);
    t.g.dirty = true;
    let stone = t.battlefield(P0, "Mind Stone");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(stone)]);
    t.answer_targets(P0, &[obj(elves)]);
    attack_with(&mut t, &[(m, pl(P1))]);
    // The attack trigger resolves: the Stone is sacrificed and the reflexive trigger
    // targets the Elves (2 damage, 1 of it excess to P1).
    t.resolve();
    assert!(t.in_graveyard(P0, "Mind Stone"));
    assert_eq!(t.stack_len(), 1, "the reflexive trigger");
    kill(&mut t, elves);
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "no excess damage to P1");
    assert_eq!(t.obj_now(m).face, mtg_engine::object::FaceState::Back, "not converted");
}

#[test]
fn death_mutation_and_blast_of_genius_do_nothing_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Death Mutation",
        "If the target is illegal on resolution, the entire spell doesn’t resolve. As such, you will not put any tokens onto the battlefield."
    );
    ruling!(
        "Blast of Genius",
        "If the target permanent or player becomes an illegal target for Blast of Genius, the spell doesn’t resolve. You don’t draw or discard any cards."
    );
    supported("Death Mutation");
    supported("Blast of Genius");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = cast(&mut t, P0, "Death Mutation", "6BG", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(tokens(&t, P0).is_empty());
    // Legal: four Saprolings.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_resolve(&mut t, P0, "Death Mutation", "6BG", &[obj(giant)]);
    assert_eq!(tokens(&t, P0).len(), 4);
    // Blast of Genius: no draw, no discard.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.hand(P0, "Craw Wurm");
    let hand = t.hand_size(P0);
    let spell = cast(&mut t, P0, "Blast of Genius", "4UR", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn feed_the_swarm_loses_life_only_if_the_target_was_legal() {
    cr!("608.2b", "608.2c", "702.12b");
    ruling!(
        "Feed the Swarm",
        "If the target permanent is an illegal target by the time Feed the Swarm tries to resolve, the spell doesn't resolve. You don't lose any life. If the target is legal but not destroyed (most likely because it has indestructible), you do lose life."
    );
    supported("Feed the Swarm");
    // Illegal: no life lost.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = cast(&mut t, P0, "Feed the Swarm", "1B", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.life(P0), 20);
    // Indestructible Darksteel Myr (mana value 3): not destroyed, 3 life lost.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Darksteel Myr");
    cast_resolve(&mut t, P0, "Feed the Swarm", "1B", &[obj(myr)]);
    assert!(t.on_battlefield(myr));
    assert_eq!(t.life(P0), 17);
}

/// P1 casts the card `theirs` on their turn (its spell is returned) with P0 ready to
/// respond: P0 has the mana of `cost` in their pool.
fn p1_casts(t: &mut TestGame, theirs: &str, their_cost: &str, cost: &str) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    let c = t.hand(P1, theirs);
    mana(t, P1, their_cost);
    let spell = t.cast(P1, c).go();
    mana(t, P0, cost);
    spell
}

#[test]
fn counterspells_with_an_illegal_target_spell_do_nothing_else() {
    cr!("608.2b");
    ruling!(
        "Counterpoint",
        "If the target spell is an illegal target as Counterpoint tries to resolve, it won’t resolve and none of its effects will happen."
    );
    ruling!(
        "Mystic Genesis",
        "If the target spell is an illegal target when Mystic Genesis tries to resolve, Mystic Genesis won’t resolve and none of its effects will happen. You won’t get an Ooze token."
    );
    supported("Counterpoint");
    supported("Mystic Genesis");
    // Counterpoint: P1's Hill Giant is countered by P1's own Cancel first; no free cast is
    // offered from P0's graveyard.
    let mut t = TestGame::new(2);
    let shock = t.graveyard(P0, "Shock");
    let giant = p1_casts(&mut t, "Hill Giant", "3R", "3UB");
    let cp = cast(&mut t, P0, "Counterpoint", "", &[obj(giant)]);
    mana(&mut t, P1, "1UU");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(giant)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let from = t.asked().len();
    t.resolve_all();
    assert!(!resolved(&t, cp));
    assert!(!was_offered(&t, P0, from, shock));
    assert!(t.in_graveyard(P0, "Shock"));
    // Mystic Genesis: no Ooze.
    let mut t = TestGame::new(2);
    let giant = p1_casts(&mut t, "Hill Giant", "3R", "2GUU");
    let mg = cast(&mut t, P0, "Mystic Genesis", "", &[obj(giant)]);
    mana(&mut t, P1, "1UU");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(giant)).go();
    t.resolve();
    t.resolve_all();
    assert!(!resolved(&t, mg));
    assert!(tokens(&t, P0).is_empty());
    // Mystic Genesis with a legal target: a 4/4 Ooze.
    let mut t = TestGame::new(2);
    let giant = p1_casts(&mut t, "Hill Giant", "3R", "2GUU");
    cast(&mut t, P0, "Mystic Genesis", "", &[obj(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    let ooze = tokens(&t, P0);
    assert_eq!(ooze.len(), 1);
    assert_eq!(t.pt(ooze[0]), (4, 4));
}

#[test]
fn draining_whelk_gets_counters_only_for_a_legal_target_even_one_it_cant_counter() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Draining Whelk",
        "If the target spell is an illegal target by the time Draining Whelk's ability tries to resolve, the spell doesn't resolve. You don't put any counters on Draining Whelk. If the target is legal but the spell can't be countered, you do put counters on Draining Whelk."
    );
    supported("Draining Whelk");
    // P1's Hill Giant is countered by P1's Cancel in response: no counters.
    let mut t = TestGame::new(2);
    let giant = p1_casts(&mut t, "Hill Giant", "3R", "4UU");
    let whelk = t.hand(P0, "Draining Whelk");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, whelk).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 2, "the Giant and the Whelk's trigger");
    mana(&mut t, P1, "1UU");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(giant)).go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.counters(whelk, counters::PLUS1), 0);
    // P1's Abrupt Decay (can't be countered, mana value 2): two counters.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let decay = cast(&mut t, P1, "Abrupt Decay", "BG", &[obj(mine)]);
    mana(&mut t, P0, "4UU");
    let whelk = t.hand(P0, "Draining Whelk");
    t.answer_targets(P0, &[obj(decay)]);
    t.cast(P0, whelk).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "Abrupt Decay resolved");
    assert_eq!(t.counters(whelk, counters::PLUS1), 2);
}

#[test]
fn mana_drain_adds_mana_only_for_a_legal_target_even_one_it_cant_counter() {
    cr!("608.2b", "608.2c", "603.7a");
    ruling!(
        "Mana Drain",
        "If the target spell is an illegal target by the time Mana Drain tries to resolve, Mana Drain doesn't resolve. You don't add mana at the beginning of your next main phase. If the target is legal but not countered (most likely because an effect says that the spell can't be countered), you do add mana."
    );
    supported("Mana Drain");
    // Illegal: no mana at P0's next main phase.
    let mut t = TestGame::new(2);
    let giant = p1_casts(&mut t, "Hill Giant", "3R", "UU");
    let drain = cast(&mut t, P0, "Mana Drain", "", &[obj(giant)]);
    mana(&mut t, P1, "1UU");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(giant)).go();
    t.resolve();
    t.resolve_all();
    assert!(!resolved(&t, drain));
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    // Abrupt Decay (can't be countered, mana value 2): {C}{C} at P0's next main phase.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let decay = cast(&mut t, P1, "Abrupt Decay", "BG", &[obj(mine)]);
    cast(&mut t, P0, "Mana Drain", "UU", &[obj(decay)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
}

#[test]
fn library_revealers_with_an_illegal_target_reveal_nothing() {
    cr!("608.2b");
    ruling!(
        "Cerebral Eruption",
        "If the targeted opponent is an illegal target by the time Cerebral Eruption resolves, the spell doesn’t resolve. No cards will be revealed and no damage will be dealt."
    );
    ruling!(
        "Explosive Revelation",
        "If the targeted permanent or player is an illegal target by the time Explosive Revelation resolves, the spell doesn’t resolve. You won’t reveal any cards from your library."
    );
    ruling!(
        "Heretic's Punishment",
        "If the targeted permanent or player is an illegal target by the time the ability resolves, the entire ability won't resolve. No cards will be put into your graveyard, and no damage will be dealt."
    );
    supported("Cerebral Eruption");
    supported("Explosive Revelation");
    supported("Heretic's Punishment");
    // Cerebral Eruption: P1 gains hexproof (Leyline of Sanctity) in response. Its library
    // isn't revealed (the Hill Giant stays on top, unknown), the Bears live, and the spell
    // goes to the graveyard (it isn't returned to hand: no land was revealed).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P1, &["Hill Giant", "Forest"]);
    let spell = cast(&mut t, P0, "Cerebral Eruption", "2RR", &[pl(P1)]);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Cerebral Eruption"));
    assert_eq!(t.library_size(P1), 32, "nothing revealed");
    // Explosive Revelation targeting a creature that leaves: nothing revealed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    stack_library(&mut t, P0, &["Forest", "Craw Wurm"]);
    let spell = cast(&mut t, P0, "Explosive Revelation", "3RR", &[obj(giant)]);
    bounce(&mut t, giant);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.library_size(P0), 32, "nothing revealed or drawn");
    assert!(!t.in_hand(P0, "Craw Wurm"));
    // Heretic's Punishment: no mill, no damage.
    let mut t = TestGame::new(2);
    let hp = t.battlefield(P0, "Heretic's Punishment");
    let giant = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, "3R");
    act(&mut t, P0, hp, "mill three cards", &[obj(giant)]).expect("Punishment");
    bounce(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn reinterpret_still_lets_you_cast_when_the_target_cant_be_countered() {
    cr!("608.2c", "701.6b");
    ruling!(
        "Reinterpret",
        "If you target a spell that can’t be countered, like Heated Debate, you won’t counter the spell, but you will get to cast a spell with an equal or lesser mana value."
    );
    supported("Reinterpret");
    supported("Heated Debate");
    // P1's Heated Debate ({2}{R}, can't be countered) at P0's Hill Giant; Reinterpret
    // doesn't counter it, but P0 casts Grizzly Bears (2 <= 3) for free.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let debate = cast(&mut t, P1, "Heated Debate", "2R", &[obj(giant)]);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast(&mut t, P0, "Reinterpret", "2UR", &[obj(debate)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"), "Heated Debate resolved: 4 damage");
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    let _ = (CastMethod::Normal, Zone::Stack);
}
