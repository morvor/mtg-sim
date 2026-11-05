//! Rulings batch P131 — interactions of abilities that exile or return cards with zone
//! changes: leaves-the-battlefield abilities that resolve before the exiling ability,
//! simultaneous deaths, "dies" triggers whose source has left the graveyard, abilities that
//! look back at enchantments put into a graveyard, and effects that replace dying.

use crate::r_p131_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let new = t.g.move_object(id, zone, MoveCause::Effect, None);
    t.settle();
    new.unwrap_or(id)
}

// --- Exile until it leaves: the leaves ability resolves first ------------------------------

#[test]
fn angel_of_serenity_leaving_before_its_enters_ability_exiles_forever() {
    cr!("603.6c", "607.2a");
    ruling!(
        "Angel of Serenity",
        "If Angel of Serenity leaves the battlefield before its \"enters\" ability has resolved, its leaves-the-battlefield ability will trigger and do nothing."
    );
    supported("Angel of Serenity");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(giant), obj(bears)]);
    t.answer_yes(P0, true);
    let angel = t.enter(P0, "Angel of Serenity");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, angel);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant") && t.in_exile("Grizzly Bears"));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn angel_of_serenity_doesnt_exile_a_targeted_creature_that_died() {
    cr!("608.2b", "400.7");
    ruling!(
        "Angel of Serenity",
        "If a creature targeted by the \"enters\" ability dies before that ability resolves, it will become an illegal target even though it may be a creature card in a graveyard when the ability resolves. It won't be exiled."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Angel of Serenity");
    t.settle();
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(!t.in_exile("Hill Giant"));
}

/// P0's face-down Aurelia's Vindicator is turned face up with X = 1, targeting `target`;
/// the "turned face up" trigger is on the stack.
fn vindicator_turned_up(t: &mut TestGame, target: ObjectId) -> ObjectId {
    use mtg_engine::mana::ManaType;
    supported("Aurelia's Vindicator");
    let fd = crate::r_p180_common::disguise(t, P0, "Aurelia's Vindicator");
    mana(t, P0, ManaType::W, 1);
    mana(t, P0, ManaType::C, 4);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[obj(target)]);
    assert!(crate::r_s11_common::turn_face_up(t, P0, fd));
    t.settle();
    assert_eq!(t.stack_len(), 1);
    fd
}

#[test]
fn aurelias_vindicator_leaving_before_its_turned_up_ability_exiles_forever() {
    cr!("603.6c", "607.2a");
    ruling!(
        "Aurelia's Vindicator",
        "If Aurelia's Vindicator leaves the battlefield before its \"turned face up\" ability has resolved, its leaves the battlefield ability will trigger and do nothing."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let v = vindicator_turned_up(&mut t, giant);
    destroy(&mut t, v);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn aurelias_vindicator_doesnt_exile_a_targeted_creature_that_died() {
    cr!("608.2b", "400.7");
    ruling!(
        "Aurelia's Vindicator",
        "If a creature targeted by the \"turned face up\" ability dies before that ability resolves, it will become an illegal target even though it may be a creature card in a graveyard when the ability resolves. It won't be exiled."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    vindicator_turned_up(&mut t, giant);
    destroy(&mut t, giant);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn the_spot_as_a_token_cant_return_the_exiled_cards() {
    cr!("111.7", "609.3");
    ruling!(
        "The Spot, Living Portal",
        "If The Spot is a token, you can't move him from the graveyard when he dies, so the exiled cards won't be returned."
    );
    supported("The Spot, Living Portal");
    supported("Quasiduplicate");
    for token in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let spot = if token {
            let card = t.battlefield(P0, "The Spot, Living Portal");
            // The legend rule: keep the token.
            crate::r_s03_common::respond(&mut t, P0, |g, d| match d {
                Decision::ChooseEntities {
                    prompt, candidates, ..
                } if prompt.contains("Legend rule") => {
                    let tok = candidates
                        .iter()
                        .copied()
                        .find(|e| matches!(e, Entity::Object(o) if g.obj(*o).is_token()))?;
                    Some(Answer::Entities(vec![tok]))
                }
                _ => None,
            });
            cast(&mut t, P0, "Quasiduplicate", &[obj(card)]);
            t.answer_targets(P0, &[obj(giant)]);
            t.resolve_all();
            let toks = tokens(&t, P0);
            assert_eq!(toks.len(), 1);
            toks[0]
        } else {
            t.answer_targets(P0, &[obj(giant)]);
            let s = t.enter(P0, "The Spot, Living Portal");
            t.resolve_all();
            s
        };
        assert!(t.in_exile("Hill Giant"), "token: {token}");
        destroy(&mut t, spot);
        t.resolve_all();
        assert_eq!(t.in_hand(P1, "Hill Giant"), !token, "token: {token}");
        assert_eq!(t.in_exile("Hill Giant"), token, "token: {token}");
    }
}

// --- Simultaneous deaths and "dies" triggers ----------------------------------------------

#[test]
fn two_underworld_cerberuses_dying_together() {
    cr!("603.3b", "101.4", "603.10a", "608.2c");
    ruling!(
        "Underworld Cerberus",
        "If two of these die at the same time, the last ability of each will trigger. If each player controlled one, the nonactive player's ability will resolve first."
    );
    supported("Underworld Cerberus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Cerberus");
    t.battlefield(P1, "Underworld Cerberus");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let dj = cast(&mut t, P0, "Day of Judgment", &[]);
    t.resolve();
    assert!(resolved(&t, dj));
    assert_eq!(t.stack_len(), 2);
    // The nonactive player's ability (on top) resolves first: their Cerberus is exiled,
    // every other creature card (P0's Cerberus included) goes to its owner's hand.
    t.resolve();
    assert_eq!(
        t.g.find_in_zone(Zone::Exile, "Underworld Cerberus").len(),
        1
    );
    assert_eq!(
        t.g.obj(t.g.find_in_zone(Zone::Exile, "Underworld Cerberus")[0])
            .owner,
        P1
    );
    assert!(t.in_hand(P0, "Underworld Cerberus"));
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
    // A creature dies in response to the active player's ability: it's returned too, and
    // the Cerberus in P0's hand isn't exiled.
    let elves = t.battlefield(P1, "Llanowar Elves");
    destroy(&mut t, elves);
    t.resolve_all();
    assert!(t.in_hand(P1, "Llanowar Elves"));
    assert!(t.in_hand(P0, "Underworld Cerberus"));
    assert_eq!(
        t.g.find_in_zone(Zone::Exile, "Underworld Cerberus").len(),
        1
    );
}

#[test]
fn underworld_cerberus_returns_creature_cards_even_if_it_cant_be_exiled() {
    cr!("608.2c", "609.3");
    ruling!(
        "Underworld Cerberus",
        "If you can't exile Underworld Cerberus when its last ability resolves, perhaps because it's been exiled by another spell or ability, each player will still return all creature cards from their graveyard to their hand."
    );
    let mut t = TestGame::new(2);
    let cerb = t.battlefield(P0, "Underworld Cerberus");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    destroy(&mut t, cerb);
    assert_eq!(t.stack_len(), 1);
    let card = in_yard(&t, P0, "Underworld Cerberus");
    move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P1, "Hill Giant"));
    assert!(t.in_exile("Underworld Cerberus"));
}

#[test]
fn carrion_thrash_can_return_a_creature_that_died_with_it() {
    cr!("603.10a", "603.3d");
    ruling!(
        "Carrion Thrash",
        "If Carrion Thrash and another creature are put into a graveyard from the battlefield at the same time, you may target the other creature with Carrion Thrash’s ability."
    );
    supported("Carrion Thrash");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carrion Thrash");
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast(&mut t, P0, "Day of Judgment", &[]);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 1, "only the Bears: another target");
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Carrion Thrash"));
}

#[test]
fn sylvan_hierophant_targets_another_card_as_it_triggers() {
    cr!("603.3d");
    ruling!(
        "Sylvan Hierophant",
        "It targets a creature card in the graveyard. This target is selected when putting the triggered ability on the stack. The wording specifies it has to be a card other than the Hierophant."
    );
    supported("Sylvan Hierophant");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Sylvan Hierophant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, h);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(bears)]]);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(bears)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_exile("Sylvan Hierophant"));
}

#[test]
fn sylvan_hierophant_returns_the_target_even_if_it_left_the_graveyard() {
    cr!("603.10a", "608.2c");
    ruling!(
        "Sylvan Hierophant",
        "If Sylvan Hierophant is removed from the graveyard before the ability resolves, you still return the targeted creature card from your graveyard to your hand."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Sylvan Hierophant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, h);
    let card = in_yard(&t, P0, "Sylvan Hierophant");
    move_to(&mut t, card, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Sylvan Hierophant"));
}

#[test]
fn sylvan_hierophant_you_controlled_works_from_its_owners_graveyard() {
    cr!("603.3a");
    ruling!(
        "Sylvan Hierophant",
        "If you control Sylvan Hierophant when it goes to the graveyard, you exile the Hierophant and return a creature card from your graveyard to your hand. It doesn't matter whose graveyard the Hierophant goes to."
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P1, "Sylvan Hierophant");
    give_control(&mut t, h, P0);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, h);
    // P0's ability, targeting a card in P0's graveyard only.
    assert_eq!(t.g.obj(top(&t)).controller, P0);
    assert_eq!(target_candidates(&t, P0, from), vec![vec![obj(bears)]]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_exile("Sylvan Hierophant"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn fell_horseman_stays_put_once_it_left_the_graveyard() {
    cr!("400.7", "603.10a");
    ruling!(
        "Fell Horseman // Deathly Ride",
        "Fell Horseman will be put on the bottom of its owner's library only if it's still in the graveyard when its ability resolves."
    );
    supported("Fell Horseman // Deathly Ride");
    // Still there: to the bottom.
    let mut t = TestGame::new(2);
    let fh = t.battlefield(P0, "Fell Horseman // Deathly Ride");
    destroy(&mut t, fh);
    t.resolve_all();
    let lib = &t.g.player(P0).library;
    assert_eq!(t.g.obj(lib[0]).chars.name, "Fell Horseman");
    // To hand and back to the graveyard in response: a new object, it stays.
    let mut t = TestGame::new(2);
    let fh = t.battlefield(P0, "Fell Horseman // Deathly Ride");
    destroy(&mut t, fh);
    assert_eq!(t.stack_len(), 1);
    let card = t.g.player(P0).graveyard[0];
    let in_hand = move_to(&mut t, card, Zone::Hand(P0));
    move_to(&mut t, in_hand, Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn eternal_witness_can_return_the_spell_that_put_it_onto_the_battlefield() {
    cr!("608.2n", "603.3");
    ruling!(
        "Eternal Witness",
        "If a resolving spell puts Eternal Witness onto the battlefield, Eternal Witness's ability can target that card if it's put into your graveyard as it resolves."
    );
    supported("Eternal Witness");
    let mut t = TestGame::new(2);
    let ew = t.graveyard(P0, "Eternal Witness");
    cast(&mut t, P0, "Zombify", &[obj(ew)]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0]
        .iter()
        .any(|e| matches!(e, Entity::Object(o) if t.g.obj(*o).chars.name == "Zombify")));
    assert!(t.in_hand(P0, "Zombify"));
    assert_eq!(t.named_on_battlefield("Eternal Witness").len(), 1);
}

// --- Neva: enchantments put into a graveyard ------------------------------------------------

fn neva(t: &mut TestGame) -> ObjectId {
    supported("Neva, Stalked by Nightmares");
    t.battlefield(P0, "Neva, Stalked by Nightmares")
}

#[test]
fn neva_triggers_for_a_role_token_put_into_the_graveyard() {
    cr!("111.7", "603.6c");
    ruling!(
        "Neva, Stalked by Nightmares",
        "Enchantment tokens (such as Roles) that are sacrificed, destroyed, or would otherwise go to the graveyard are put into their owner's graveyard before ceasing to exist. If you controlled the token, Neva's ability will trigger."
    );
    let mut t = TestGame::new(2);
    let n = neva(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast(&mut t, P0, "Monstrous Rage", &[obj(bears)]);
    t.resolve_all();
    let role = tokens(&t, P0);
    assert_eq!(role.len(), 1);
    destroy(&mut t, role[0]);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(n, counters::PLUS1), 1);
}

#[test]
fn neva_sees_only_auras_destroyed_by_the_same_effect() {
    cr!("603.10a", "603.6c", "704.5m");
    ruling!(
        "Neva, Stalked by Nightmares",
        "If Neva leaves the battlefield while it has an Aura you control attached to it or at the same time as another creature with an Aura you control attached to it, its ability won't trigger for those enchantments unless they were destroyed by the same effect that caused Neva to leave the battlefield."
    );
    // Neva and an enchanted creature die to Day of Judgment: the Auras go to the
    // graveyard afterwards (state-based actions), when Neva is gone.
    let mut t = TestGame::new(2);
    let n = neva(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", n);
    attach_new(&mut t, P0, "Holy Strength", bears);
    cast(&mut t, P0, "Day of Judgment", &[]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.stack_len(), 0);
    // Akroma's Vengeance destroys them all at once: Neva sees both Auras go.
    let mut t = TestGame::new(2);
    let n = neva(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", n);
    attach_new(&mut t, P0, "Holy Strength", bears);
    cast(&mut t, P0, "Akroma's Vengeance", &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
}

// --- Possessed Skaab ------------------------------------------------------------------------

#[test]
fn possessed_skaab_is_exiled_however_it_would_die() {
    cr!("614.6", "700.4");
    ruling!(
        "Possessed Skaab",
        "Possessed Skaab’s last ability will apply no matter what caused it to die (lethal damage, being sacrificed, enough -1/-1 counters, and so on)."
    );
    supported("Possessed Skaab");
    for how in 0..3 {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Blood Artist");
        let skaab = t.battlefield(P0, "Possessed Skaab");
        match how {
            0 => {
                let giant = t.battlefield(P1, "Hill Giant");
                damage(&mut t, giant, 2, skaab);
            }
            1 => {
                t.g.sacrifice(skaab, P0);
                t.settle();
            }
            _ => {
                put_counters(&mut t, skaab, counters::MINUS1, 2);
                t.settle();
            }
        }
        assert!(t.in_exile("Possessed Skaab"), "case {how}");
        assert!(!t.in_graveyard(P0, "Possessed Skaab"), "case {how}");
        // Blood Artist ("whenever Blood Artist or another creature dies") didn't trigger.
        assert_eq!(t.stack_len(), 0, "case {how}");
    }
}

// --- Vile Redeemer: nontoken creatures that died under your control ------------------------

/// P0 casts Vile Redeemer and pays {C} for its cast trigger: the Scions created.
fn redeemer_scions(t: &mut TestGame) -> usize {
    use mtg_engine::mana::ManaType;
    supported("Vile Redeemer");
    mana(t, P0, ManaType::G, 1);
    mana(t, P0, ManaType::C, 3);
    let c = t.hand(P0, "Vile Redeemer");
    t.answer_yes(P0, true);
    t.cast(P0, c).go();
    t.resolve_all();
    tokens_with(t, P0, "Scion")
}

#[test]
fn vile_redeemer_counts_creatures_you_controlled_wherever_they_are() {
    cr!("700.4");
    ruling!(
        "Vile Redeemer",
        "Vile Redeemer's last ability will count any nontoken creature that died while under your control, even if you weren't the creature's owner."
    );
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, theirs, P0);
    let mine = t.battlefield(P0, "Hill Giant");
    destroy(&mut t, theirs);
    destroy(&mut t, mine);
    // One of them is no longer in a graveyard.
    let card = in_yard(&t, P1, "Grizzly Bears");
    move_to(&mut t, card, Zone::Exile);
    // A token and an opponent's creature don't count.
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    destroy(&mut t, tok);
    let other = t.battlefield(P1, "Llanowar Elves");
    destroy(&mut t, other);
    assert_eq!(redeemer_scions(&mut t), 2);
}

#[test]
fn vile_redeemer_doesnt_count_creatures_exiled_instead() {
    cr!("700.4", "614.6");
    ruling!(
        "Vile Redeemer",
        "Vile Redeemer's last ability won't count nontoken creatures that didn't go to a graveyard because of a replacement effect"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(redeemer_scions(&mut t), 0);
}

#[test]
fn soul_transfer_keeps_both_modes_if_the_artifact_leaves() {
    cr!("601.2b", "700.2");
    ruling!(
        "Soul Transfer",
        "Once you cast Soul Transfer and choose both modes, it doesn't matter what happens to the artifact or enchantment you control in response."
    );
    supported("Soul Transfer");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Pacifism");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Soul Transfer");
    let c = t.hand(P0, "Soul Transfer");
    t.cast(P0, c)
        .modes(&[0, 1])
        .targets(&[obj(bears)])
        .targets(&[obj(giant)])
        .go();
    destroy(&mut t, ring);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P0, "Hill Giant"));
    // Without both permanents, only one mode can be chosen.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    lands_for_cost(&mut t, P0, "Soul Transfer");
    let c = t.hand(P0, "Soul Transfer");
    let r = t
        .cast(P0, c)
        .modes(&[0, 1])
        .targets(&[obj(bears)])
        .targets(&[obj(giant)])
        .try_go();
    let one_mode = match r {
        Err(_) => true,
        Ok(s) => t.g.obj(s).stack.as_deref().unwrap().chosen.len() == 1,
    };
    assert!(one_mode);
}
