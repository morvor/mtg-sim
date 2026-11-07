//! Rulings batch P141 — "return it to your hand" engines: delayed returns that outlive
//! their source but not the card's zone change (Rienne, Tiana, Feather), "unless target
//! opponent" punisher returns that need a target (Athreos, Pia's Revolution), intervening
//! "if" clauses and illegal targets (Golbez, Liliana), and replacement effects on draws
//! and discards (Necropotence, Enduring Renewal, Forbidden Crypt).

use crate::r_p141_common::*;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

#[test]
fn rienne_delayed_return_survives_rienne_but_not_the_card_leaving() {
    cr!("603.7c", "400.7");
    ruling!(
        "Rienne, Angel of Rebirth",
        "If Rienne leaves the battlefield after its ability has triggered but before the next end step, its delayed triggered ability will still return the card."
    );
    ruling!(
        "Rienne, Angel of Rebirth",
        "If a multicolored creature you control leaves the graveyard after it dies but before the delayed triggered ability resolves, it will not return to its owner's hand."
    );
    supported("Rienne, Angel of Rebirth");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rienne, Angel of Rebirth");
    let w = t.battlefield(P0, "Watchwolf");
    destroy(&mut t, w);
    t.resolve_all();
    destroy(&mut t, r);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Watchwolf"));
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Watchwolf"));
    // The card is exiled from the graveyard first: it stays in exile.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rienne, Angel of Rebirth");
    let w = t.battlefield(P0, "Watchwolf");
    destroy(&mut t, w);
    t.resolve_all();
    move_to(&mut t, w, Zone::Exile);
    end_step(&mut t, P0);
    assert!(t.in_exile("Watchwolf"));
    assert!(!t.in_hand(P0, "Watchwolf"));
}

#[test]
fn rienne_returns_a_creature_made_multicolored() {
    cr!("603.10a", "105.4");
    ruling!(
        "Rienne, Angel of Rebirth",
        "If an effect causes a non-multicolored creature to become multicolored and that creature dies, the non-multicolored card will return to its owner's hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rienne, Angel of Rebirth");
    let b = t.battlefield(P0, "Grizzly Bears");
    // Without the effect: no trigger.
    let b2 = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, b2);
    assert_eq!(stacked_triggers(&t), 0);
    crate::r_s26_common::modify_until_eot(
        &mut t,
        b,
        vec![Modification::AddColors(ColorSet::single(Color::White))],
    );
    destroy(&mut t, b);
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve_all();
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn tiana_end_step_deaths_wait_for_the_next_end_step_and_the_card_must_stay() {
    cr!("603.7c", "513.1", "400.7");
    ruling!(
        "Tiana, Ship's Caretaker",
        "If an Aura or Equipment is put into your graveyard during an end step, you'll be able to return it during the next end step, not the current one."
    );
    ruling!(
        "Tiana, Ship's Caretaker",
        "If an Aura or Equipment leaves the graveyard after triggering Tiana's last ability, it won't be returned to its owner's hand as the delayed triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tiana, Ship's Caretaker");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    destroy(&mut t, eq);
    yes(&mut t, P0);
    t.resolve_all();
    // Rest of this end step: still in the graveyard.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Bonesplitter"));
    yes(&mut t, P0);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Bonesplitter"));
    // Leaves the graveyard first: not returned.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tiana, Ship's Caretaker");
    let eq = t.battlefield(P0, "Bonesplitter");
    destroy(&mut t, eq);
    yes(&mut t, P0);
    t.resolve_all();
    move_to(&mut t, eq, Zone::Exile);
    yes(&mut t, P0);
    end_step(&mut t, P0);
    assert!(t.in_exile("Bonesplitter"));
}

/// P0 controls Feather, the Redeemed and a Grizzly Bears; casts Giant Growth on the Bears
/// and resolves Feather's trigger (the Growth stays on the stack). Returns (Feather,
/// Bears, Growth spell).
fn feather_growth(t: &mut TestGame) -> (ObjectId, ObjectId, ObjectId) {
    let f = t.battlefield(P0, "Feather, the Redeemed");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let g = t.hand(P0, "Giant Growth");
    let spell = t.cast(P0, g).target(b).go();
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    t.resolve();
    (f, b, spell)
}

#[test]
fn feather_effects_outlive_feather() {
    cr!("603.7c", "608.2n");
    ruling!(
        "Feather, the Redeemed",
        "The replacement effect that exiles that spell and the delayed triggered ability that returns it to your hand both take effect even if Feather leaves the battlefield after this point."
    );
    supported("Feather, the Redeemed");
    let mut t = TestGame::new(2);
    let (f, b, _) = feather_growth(&mut t);
    destroy(&mut t, f);
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
    assert!(t.in_exile("Giant Growth"));
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Giant Growth"));
}

#[test]
fn feather_doesnt_exile_spells_that_dont_resolve() {
    cr!("608.2b", "701.6a");
    ruling!(
        "Feather, the Redeemed",
        "If an instant or sorcery spell you cast that targets your creature doesn't resolve for any reason (either because another spell or ability counters it or because all its targets are illegal as it tries to resolve), it won't be exiled. You won't return it to your hand."
    );
    // Countered.
    let mut t = TestGame::new(2);
    let (_, _, spell) = feather_growth(&mut t);
    crate::r_p146_common::counter(&mut t, spell);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Giant Growth"));
    // Its target is gone.
    let mut t = TestGame::new(2);
    let (_, b, _) = feather_growth(&mut t);
    move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    end_step(&mut t, P0);
    assert!(!t.in_hand(P0, "Giant Growth"));
}

#[test]
fn feather_ignores_spells_that_exile_themselves() {
    cr!("608.2n");
    ruling!(
        "Feather, the Redeemed",
        "If an instant or sorcery spell's own instructions tell you to exile it or put it anywhere else, it won't try to be put into your graveyard or exiled with Feather's effect, so you won't return it to your hand."
    );
    supported("Morningtide's Light");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feather, the Redeemed");
    let b = t.battlefield(P0, "Grizzly Bears");
    let light = cast_new(&mut t, P0, "Morningtide's Light", &[obj(b)]);
    let _ = light;
    t.resolve_all();
    assert!(t.in_exile("Morningtide's Light"));
    end_step(&mut t, P0);
    assert!(t.in_exile("Morningtide's Light"));
    assert!(!t.in_hand(P0, "Morningtide's Light"));
    // Positive control: the Bears came back from its effect.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn feather_can_apply_before_flashback() {
    cr!("616.1", "702.34a");
    ruling!(
        "Feather, the Redeemed",
        "If another replacement effect instructs you to exile an instant or sorcery spell, such as that of Dreadhorde Arcanist or the flashback keyword, you may choose to apply Feather's replacement effect first. If you do, Feather's delayed triggered ability will return that card to your hand."
    );
    supported("Firebolt");
    for (pick, returns) in [(0, true), (1, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Feather, the Redeemed");
        let b = t.battlefield(P0, "Hill Giant");
        let fb = t.graveyard(P0, "Firebolt");
        t.lands(P0, "Mountain", 5);
        t.cast(P0, fb)
            .method(mtg_engine::object::CastMethod::Keyword(
                mtg_engine::keywords::KeywordKind::Flashback,
            ))
            .target(b)
            .go();
        let from = n_asked(&t);
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.resolve_all();
        assert_eq!(t.obj_now(b).damage, 2);
        let offered = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, decision::Decision::ChooseReplacement { .. }))
            .count();
        assert_eq!(offered, 1, "Feather's and flashback's effects both apply");
        assert!(t.in_exile("Firebolt"));
        end_step(&mut t, P0);
        assert_eq!(t.in_hand(P0, "Firebolt"), returns, "pick {pick}");
        assert_eq!(t.in_exile("Firebolt"), !returns, "pick {pick}");
    }
}

#[test]
fn feather_doesnt_exile_a_spell_you_dont_own() {
    cr!("608.2n", "400.3");
    ruling!(
        "Feather, the Redeemed",
        "If you cast an instant or sorcery spell that you don't own, it won't try to be put into your graveyard, so you won't exile it with Feather's effect or return it to your hand."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Feather, the Redeemed");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let g = t.custom(
        P1,
        (*mtg_engine::card::card("Giant Growth")).clone(),
        Zone::Hand(P0),
    );
    t.cast(P0, g).target(b).go();
    t.resolve_all();
    assert_eq!(t.pt(b), (5, 5));
    assert!(t.in_graveyard(P1, "Giant Growth"));
    end_step(&mut t, P0);
    assert!(t.in_graveyard(P1, "Giant Growth"));
}

#[test]
fn athreos_token_trigger_lets_the_opponent_pay_but_nothing_returns() {
    cr!("603.10a", "111.7", "118.12");
    ruling!(
        "Athreos, God of Passage",
        "Athreos's last ability will trigger if a token creature you own dies. The target opponent has the option to pay 3 life, although the token can't return to your hand."
    );
    supported("Athreos, God of Passage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Athreos, God of Passage");
    let tok = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, tok);
    assert_eq!(stacked_triggers(&t), 1);
    t.answer_yes(P1, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn punisher_returns_need_a_target_opponent() {
    cr!("603.3d", "702.11b");
    ruling!(
        "Athreos, God of Passage",
        "If there are no legal targets for Athreos's last ability (perhaps because each of your opponents has hexproof), it will be removed from the stack with no effect. No one may pay 3 life and you won't return the creature card to your hand."
    );
    ruling!(
        "Pia's Revolution",
        "If there are no legal targets for Pia’s Revolution’s ability (perhaps because each of your opponents has hexproof), it will be removed from the stack with no effect. No one may choose to be dealt damage and you won’t return the artifact card to your hand."
    );
    supported("Pia's Revolution");
    for (engine, victim) in [
        ("Athreos, God of Passage", "Grizzly Bears"),
        ("Pia's Revolution", "Ornithopter"),
    ] {
        // Positive control: without hexproof, the card returns (the opponent declines).
        let mut t = TestGame::new(2);
        t.battlefield(P0, engine);
        let v = t.battlefield(P0, victim);
        t.answer_yes(P1, false);
        destroy(&mut t, v);
        t.resolve_all();
        assert!(t.in_hand(P0, victim), "{engine}");
        // The opponent has hexproof.
        let mut t = TestGame::new(2);
        t.battlefield(P0, engine);
        t.battlefield(P1, "Leyline of Sanctity");
        let v = t.battlefield(P0, victim);
        destroy(&mut t, v);
        t.resolve_all();
        assert!(t.in_graveyard(P0, victim), "{engine}");
        assert_eq!(t.life(P1), 20, "{engine}");
    }
}

#[test]
fn golbez_checks_four_artifacts_on_trigger_and_resolution() {
    cr!("603.4", "608.2b");
    ruling!(
        "Golbez, Crystal Collector",
        "Golbez, Crystal Collector's last ability checks at the moment it would trigger to see if you control four or more artifacts."
    );
    ruling!(
        "Golbez, Crystal Collector",
        "If the target creature card is an illegal target as Golbez, Crystal Collector's last ability tries to resolve, it won't resolve and none of its effects will happen. Opponents won't lose life even if you control eight or more artifacts."
    );
    supported("Golbez, Crystal Collector");
    // Three artifacts: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golbez, Crystal Collector");
    t.lands(P0, "Ornithopter", 3);
    t.graveyard(P0, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stacked_triggers(&t), 0);
    // Four, one leaves before resolution: nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golbez, Crystal Collector");
    let arts = t.lands(P0, "Ornithopter", 4);
    t.graveyard(P0, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    destroy(&mut t, arts[0]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Positive control: eight artifacts, the card returns and the opponent loses 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golbez, Crystal Collector");
    t.lands(P0, "Ornithopter", 8);
    t.graveyard(P0, "Hill Giant");
    end_step(&mut t, P0);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
    // Eight artifacts, the target leaves: no life loss.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Golbez, Crystal Collector");
    t.lands(P0, "Ornithopter", 8);
    let g = t.graveyard(P0, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
    move_to(&mut t, g, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn liliana_death_mage_needs_a_legal_target_for_the_life_loss() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Liliana, Death Mage",
        "If the target creature is an illegal target by the time Liliana's second ability tries to resolve, the ability won't resolve. No player loses 2 life. If the target is legal but not destroyed (most likely because it has indestructible), its controller does lose 2 life."
    );
    supported("Liliana, Death Mage");
    // Illegal target: no life loss.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, Death Mage");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, l, 1, &[obj(b)]).unwrap();
    move_to(&mut t, b, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Indestructible: survives, but its controller loses 2.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, Death Mage");
    let m = t.battlefield(P1, "Darksteel Myr");
    t.activate(P0, l, 1, &[obj(m)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(m));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn liliana_the_necromancer_returns_creatures_chosen_after_the_destruction() {
    cr!("608.2c", "603.3");
    ruling!(
        "Liliana, the Necromancer",
        "For Liliana’s last ability, the creatures you’ll return to the battlefield are chosen after the target creatures are destroyed."
    );
    ruling!(
        "Liliana, the Necromancer",
        "If any abilities trigger on the target creatures being destroyed, those abilities won’t be put onto the stack until after you’ve put creature cards onto the battlefield from graveyards."
    );
    supported("Liliana, the Necromancer");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Liliana, the Necromancer");
    put_counters(&mut t, l, "loyalty", 4);
    let d = t.battlefield(P1, "Doomed Traveler");
    t.activate(P0, l, 2, &[obj(d)]).unwrap();
    t.answer_choose_moved(P0, &[obj(d)]);
    t.g.resolve_top();
    // During the resolution the Traveler went to the graveyard and came back under P0's
    // control; its dies trigger waits until now.
    assert_eq!(t.g.stack.len(), 0);
    let back = t.named_on_battlefield("Doomed Traveler");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P0);
    t.settle();
    assert_eq!(stacked_triggers(&t), 1);
}

#[test]
fn necropotence_only_exiles_cards_still_in_your_graveyard() {
    cr!("603.10a", "400.7");
    ruling!(
        "Necropotence",
        "If a discarded card isn't put into your graveyard (due to an effect such as that of Obstinate Baloth) or leaves your graveyard (perhaps because another effect returned it to your hand), it won't be exiled."
    );
    supported("Necropotence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Necropotence");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    discard(&mut t, P0, a);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    discard(&mut t, P0, b);
    move_to(&mut t, b, Zone::Hand(P0));
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn koll_counts_any_aura_or_equipment_once() {
    cr!("613.4c", "301.5");
    ruling!(
        "Koll, the Forgemaster",
        "A creature token you control is enchanted or equipped if there’s an Aura or an Equipment attached to it. You don’t have to control that Aura or Equipment."
    );
    ruling!(
        "Koll, the Forgemaster",
        "A creature token you control that’s enchanted or equipped gets only +1/+1 from Koll’s last ability, no matter how many Auras and Equipment are attached to that creature."
    );
    supported("Koll, the Forgemaster");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Koll, the Forgemaster");
    let a = create_token(&mut t, P0, "Soldier");
    assert_eq!(t.pt(a), (1, 1));
    attach_new(&mut t, P1, "Holy Strength", a);
    assert_eq!(t.pt(a), (3, 4));
    let b = create_token(&mut t, P0, "Soldier");
    attach_new(&mut t, P0, "Holy Strength", b);
    attach_new(&mut t, P0, "Bonesplitter", b);
    assert_eq!(t.pt(b), (5, 4));
}

#[test]
fn enduring_renewal_only_replaces_draws_and_returns_animated_creatures() {
    cr!("121.6", "614.1a", "603.10a");
    ruling!(
        "Enduring Renewal",
        "Enduring Renewal only affects creature cards that are “drawn”. It doesn’t affect cards that are put into your hand from your library, or from anywhere else."
    );
    ruling!(
        "Enduring Renewal",
        "If the creature is only a creature due to an effect, it still goes to your hand. This includes any way to animate a card."
    );
    supported("Enduring Renewal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enduring Renewal");
    t.library_top(P0, "Grizzly Bears");
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let h = t.library_top(P0, "Hill Giant");
    move_to(&mut t, h, Zone::Hand(P0));
    assert!(t.in_hand(P0, "Hill Giant"));
    // An animated Mutavault dies: it's returned.
    let mv = t.battlefield(P0, "Mutavault");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, mv, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(mv).is(types::CardType::Creature));
    destroy(&mut t, mv);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mutavault"));
}

#[test]
fn forbidden_crypt_replaces_each_draw_separately() {
    cr!("121.2", "614.11");
    ruling!(
        "Forbidden Crypt",
        "If more than one card is being drawn due to a single effect, apply the replacement as if the cards were being drawn one at a time."
    );
    supported("Forbidden Crypt");
    // Two cards in the graveyard, draw two: both returned.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forbidden Crypt");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let lib = t.library_size(P0);
    t.g.draw_cards(P0, 2);
    t.settle();
    assert!(t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.library_size(P0), lib);
    assert!(!t.has_lost(P0));
    // One card in the graveyard, draw two: the second draw finds nothing and P0 loses.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forbidden Crypt");
    t.graveyard(P0, "Grizzly Bears");
    t.g.draw_cards(P0, 2);
    t.settle();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.has_lost(P0));
}
