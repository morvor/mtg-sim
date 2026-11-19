//! Rulings batch P164 — black synergies: Hellfire ("creatures that died this way"),
//! nongreen / nonblack and hybrid (Soul Reap, Soot Imp, Gravelgill Duo), "if you've cast
//! another black spell this turn" (Soul Reap), convoke with mana abilities (Vote Out, We
//! Ride at Dawn), devotion to black as a cost reduction (Marshmist Titan, Drag to the
//! Underworld) and as an X (Blight-Breath Catoblepas), revealing black cards (Nightshade
//! Assassin, Nightshade Seer, Scent of Nightshade), Vampire Nocturnus, Knight of Grace,
//! Nocturnal Raid, Shadow Alley Denizen, Urborg Stalker, Stain the Mind, Bloodline Bidding,
//! Liliana's Spoils and Hoarding Broodlord.

use crate::r_p164_common::*;
use crate::r_s02_common::{can_attack, create_token};
use crate::r_s21_common::castable;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts Hellfire (with lands for it) and resolves everything; returns `p`'s life
/// lost.
fn hellfire(t: &mut TestGame, p: PlayerId) -> i32 {
    t.lands(p, "Swamp", 5);
    let h = t.hand(p, "Hellfire");
    let before = t.life(p);
    t.cast(p, h).go();
    t.resolve_all();
    before - t.life(p)
}

#[test]
fn hellfire_counts_creatures_put_into_graveyards() {
    cr!("700.4", "701.19a", "614.6", "111.7");
    ruling!(
        "Hellfire",
        "A creature that regenerates instead of being destroyed doesn’t leave the battlefield, so it’s not put into a graveyard."
    );
    ruling!(
        "Hellfire",
        "A token creature that’s destroyed is put into its owner’s graveyard, and will be counted by Hellfire’s effect."
    );
    ruling!(
        "Hellfire",
        "Hellfire checks the number of creatures put into graveyards as the result of its effect, not the number of creatures destroyed as a result of its effect."
    );
    supported("Hellfire");
    supported("River Boa");
    supported("Rest in Peace");
    // A regenerating River Boa doesn't count: one died, 1 + 3.
    let mut t = TestGame::new(2);
    let boa = t.battlefield(P1, "River Boa");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Drudge Skeletons");
    t.lands(P1, "Forest", 1);
    t.activate(P1, boa, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(hellfire(&mut t, P0), 4);
    assert!(t.on_battlefield(boa));
    assert_eq!(t.named_on_battlefield("Drudge Skeletons").len(), 1);
    // A token counts: two died, 2 + 3.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let token = create_token(&mut t, P1, "Soldier");
    assert_eq!(hellfire(&mut t, P0), 5);
    assert!(!t.g.is_live(token) || !t.on_battlefield(token));
    // Rest in Peace exiles them instead: none died, 0 + 3.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Rest in Peace");
    assert_eq!(hellfire(&mut t, P0), 3);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
}

#[test]
fn hybrid_objects_of_a_color_arent_non_that_color() {
    cr!("105.4", "202.2", "205.4b");
    ruling!(
        "Soul Reap",
        "A hybrid creature that’s green and another color is *not* a nongreen creature. Since it’s green, it can’t be nongreen!"
    );
    ruling!(
        "Soot Imp",
        "A hybrid spell that’s black and another color is *not* a nonblack spell. Since it’s black, it can’t be nonblack!"
    );
    ruling!("Soot Imp", "This ability affects all players, including you.");
    supported("Soul Reap");
    supported("Soot Imp");
    supported("Kitchen Finks");
    supported("Gravelgill Duo");
    // Soul Reap can't target Kitchen Finks ({1}{G/W}{G/W}).
    let mut t = TestGame::new(2);
    let finks = t.battlefield(P1, "Kitchen Finks");
    t.lands(P0, "Swamp", 2);
    let reap = t.hand(P0, "Soul Reap");
    assert!(t.cast(P0, reap).target(finks).try_go().is_err());
    let bears = t.battlefield(P1, "Hill Giant");
    let reap = t.g.current(reap);
    assert!(t.cast(P0, reap).target(bears).try_go().is_ok());
    // Soot Imp: Gravelgill Duo ({2}{U/B}) is black: no life loss; Kitchen Finks is
    // nonblack: its caster loses 1, even the Imp's controller.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soot Imp");
    t.lands(P0, "Swamp", 3);
    let duo = t.hand(P0, "Gravelgill Duo");
    t.cast(P0, duo).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    t.lands(P0, "Plains", 3);
    let finks = t.hand(P0, "Kitchen Finks");
    t.cast(P0, finks).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "-1 from Soot Imp, +2 from the Finks");
}

#[test]
fn gravelgill_duo_triggers_both_for_a_blue_and_black_spell() {
    cr!("603.3b", "105.2");
    ruling!(
        "Gravelgill Duo",
        "An object that's both of the listed colors will cause both abilities to trigger. You can put them on the stack in either order."
    );
    let mut t = TestGame::new(2);
    let duo = t.battlefield(P0, "Gravelgill Duo");
    t.lands(P0, "Island", 3);
    let other = t.hand(P0, "Gravelgill Duo");
    t.cast(P0, other).go();
    t.settle();
    assert_eq!(triggers_of(&t, duo), 2);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
    t.resolve_all();
    assert_eq!(t.pt(duo), (3, 2));
    assert!(has_kw(&t, duo, KeywordKind::Fear));
}

#[test]
fn soul_reap_checks_at_resolution_whether_you_cast_another_black_spell() {
    cr!("608.2h", "601.2i");
    ruling!(
        "Soul Reap",
        "Soul Reap doesn’t check whether you’ve cast another black spell until Soul Reap resolves. If you cast Soul Reap, then cast a black instant in response, the creature’s controller will lose the life when Soul Reap resolves."
    );
    supported("Dark Ritual");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Swamp", 3);
        let reap = t.hand(P0, "Soul Reap");
        t.cast(P0, reap).target(bears).go();
        if respond {
            let ritual = t.hand(P0, "Dark Ritual");
            t.cast(P0, ritual).go();
        }
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Hill Giant"));
        assert_eq!(t.life(P1), if respond { 17 } else { 20 });
    }
}

#[test]
fn soul_reap_under_your_control_checks_your_black_spells_even_if_you_didnt_cast_it() {
    cr!("608.2h", "110.2");
    ruling!(
        "Soul Reap",
        "Although Soul Reap says it looks for “another black spell,” there’s no requirement that Soul Reap actually be cast (or be black) for this part of its ability to work."
    );
    supported("Commandeer");
    for ritual_first in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Swamp", 2);
        let reap = t.hand(P1, "Soul Reap");
        let spell = t.cast(P1, reap).target(giant).go();
        // P0 (who hasn't cast Soul Reap) gains control of it with Commandeer.
        t.lands(P0, "Island", 8);
        if ritual_first {
            t.lands(P0, "Swamp", 1);
            let ritual = t.hand(P0, "Dark Ritual");
            t.cast(P0, ritual).go();
            t.resolve();
        }
        let cmd = t.hand(P0, "Commandeer");
        t.answer_yes(P0, false);
        t.cast(P0, cmd).target(spell).go();
        t.resolve();
        assert_eq!(t.obj_now(spell).controller, P0);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Hill Giant"));
        // P1 cast Soul Reap, a black spell, but P1 isn't its controller now: only P0's
        // spells count.
        assert_eq!(t.life(P0), if ritual_first { 17 } else { 20 });
    }
}

#[test]
fn convoke_cant_use_a_creature_already_used_for_mana() {
    cr!("702.51a", "601.2g", "601.2h");
    ruling!(
        "Vote Out",
        "If a creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell with convoke will result in the creature being tapped before you pay the spell’s costs. You won’t be able to tap it again for convoke. Similarly, if you sacrifice a creature to activate a mana ability while casting a spell with convoke, that creature won’t be on the battlefield when you pay the spell’s costs, so you won’t be able to tap it for convoke."
    );
    ruling!(
        "We Ride at Dawn",
        "If a creature you control has a mana ability with {T} in the cost, activating that ability while casting a spell with convoke will result in the creature being tapped before you pay the spell’s costs. You won’t be able to tap it again for convoke."
    );
    supported("Vote Out");
    supported("We Ride at Dawn");
    supported("Blood Pet");
    // Vote Out ({3}{B}): a Swamp, Llanowar Elves and Blood Pet are only three mana.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 1);
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Blood Pet");
    let target = t.battlefield(P1, "Grizzly Bears");
    let vote = t.hand(P0, "Vote Out");
    assert!(t.cast(P0, vote).target(target).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    let vote = t.g.current(vote);
    assert!(t.cast(P0, vote).target(target).try_go().is_ok());
    // We Ride at Dawn: Jedit Ojanen ({3}{G}{G}{G}) with four Forests and the Elves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "We Ride at Dawn");
    t.lands(P0, "Forest", 4);
    t.battlefield(P0, "Llanowar Elves");
    let jedit = t.hand(P0, "Jedit Ojanen of Efrava");
    assert!(t.cast(P0, jedit).try_go().is_err());
    t.battlefield(P0, "Grizzly Bears");
    let jedit = t.g.current(jedit);
    assert!(t.cast(P0, jedit).try_go().is_ok());
}

#[test]
fn devotion_cost_reductions_reduce_generic_mana_and_lock_in() {
    cr!("700.5", "601.2f", "601.2g", "202.3");
    ruling!(
        "Marshmist Titan",
        "If your devotion to black is greater than six, Marshmist Titan will cost {B} to cast. The colored mana requirement isn’t reduced."
    );
    ruling!(
        "Marshmist Titan",
        "Your devotion to black is calculated when you determine Marshmist Titan’s total cost, and that cost is locked in before any costs are paid."
    );
    ruling!(
        "Drag to the Underworld",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Drag to the Underworld). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Drag to the Underworld",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for. Notably, opponents can't try to change by how much Drag to the Underworld's cost is reduced."
    );
    supported("Marshmist Titan");
    supported("Drag to the Underworld");
    supported("Phyrexian Obliterator");
    // Devotion 8: the Titan costs {B}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Obliterator");
    t.battlefield(P0, "Phyrexian Obliterator");
    t.lands(P0, "Swamp", 1);
    let titan = t.hand(P0, "Marshmist Titan");
    assert!(t.cast(P0, titan).try_go().is_ok());
    // Devotion 5 (Obliterator and Blood Pet): {1}{B}, paid by sacrificing the Blood Pet
    // for {B} and a Swamp.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Obliterator");
    t.battlefield(P0, "Blood Pet");
    t.lands(P0, "Swamp", 1);
    let titan = t.hand(P0, "Marshmist Titan");
    assert!(t.cast(P0, titan).try_go().is_ok());
    assert!(t.in_graveyard(P0, "Blood Pet"));
    // Drag to the Underworld with devotion 4 costs {B}{B}; its mana value is still 4, and
    // the opponent has no chance to act while it's cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Phyrexian Obliterator");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let drag = t.hand(P0, "Drag to the Underworld");
    let from = t.asked().len();
    let spell = t.cast(P0, drag).target(target).go();
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn blight_breath_catoblepas_x_is_set_on_resolution() {
    cr!("700.5", "608.2h", "611.2c");
    ruling!(
        "Blight-Breath Catoblepas",
        "The value of X is determined only as the triggered ability resolves. Once that happens, the value of X won’t change later in the turn even if your devotion to black changes."
    );
    supported("Blight-Breath Catoblepas");
    // Devotion goes from 2 to 6 before it resolves: -6/-6 kills the Craw Wurm (6/4).
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[o(wurm)]);
    t.enter(P0, "Blight-Breath Catoblepas");
    t.settle();
    t.battlefield(P0, "Phyrexian Obliterator");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    // Devotion 2 as it resolves (-2/-2); it rising afterwards changes nothing.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[o(wurm)]);
    t.enter(P0, "Blight-Breath Catoblepas");
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    t.battlefield(P0, "Phyrexian Obliterator");
    t.settle();
    assert_eq!(t.pt(wurm), (4, 2));
}

#[test]
fn nightshade_reveals_happen_on_resolution() {
    cr!("608.2b", "608.2h", "701.20a");
    ruling!(
        "Nightshade Assassin",
        "If the target creature becomes an illegal target, you don't reveal any cards."
    );
    ruling!(
        "Nightshade Assassin",
        "You choose how many cards to reveal and which as Nightshade Assassin's triggered ability resolves."
    );
    ruling!(
        "Scent of Nightshade",
        "You reveal cards, and thereby the value of X, on resolution."
    );
    ruling!(
        "Nightshade Seer",
        "You can reveal zero cards to give -0/-0 to a creature."
    );
    supported("Nightshade Assassin");
    supported("Scent of Nightshade");
    supported("Nightshade Seer");
    // Assassin: a black card added to hand after it triggers can be revealed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    let a = t.hand(P0, "Duress");
    t.answer_targets(P0, &[o(giant)]);
    t.enter(P0, "Nightshade Assassin");
    t.settle();
    let b = t.hand(P0, "Doom Blade");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(a), o(b)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 2));
    // The target leaves: nothing is asked.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Duress");
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Nightshade Assassin");
    t.settle();
    bounce(&mut t, bears);
    let from = t.asked().len();
    t.resolve_all();
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        Decision::ChooseEntities { .. } | Decision::YesNo { .. } | Decision::ChooseX { .. }
    )));
    // Scent of Nightshade: X counts the cards revealed as it resolves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Swamp", 2);
    let a = t.hand(P0, "Duress");
    let scent = t.hand(P0, "Scent of Nightshade");
    t.cast(P0, scent).target(giant).go();
    let b = t.hand(P0, "Doom Blade");
    t.answer_choose(P0, &[o(a), o(b)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 2));
    // Nightshade Seer: zero cards, -0/-0.
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Nightshade Seer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Duress");
    t.lands(P0, "Swamp", 3);
    t.answer_choose(P0, &[]);
    t.activate(P0, seer, 0, &[o(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert!(t.obj_now(seer).tapped);
}

#[test]
fn vampire_nocturnus_pumps_itself_and_flying_needs_to_be_there_at_blocks() {
    cr!("509.1b", "702.9b", "509.1h", "611.3a");
    ruling!(
        "Vampire Nocturnus",
        "If the top card of your library is black, Vampire Nocturnus will give itself +2/+1 and flying even if it's somehow not a Vampire."
    );
    ruling!(
        "Vampire Nocturnus",
        "To work as an evasion ability, an attacking creature must already have flying when the declare blockers step begins. Once a creature has become blocked, giving it flying won't change that."
    );
    supported("Vampire Nocturnus");
    supported("Imagecrafter");
    // Imagecrafter makes it an Elf: still +2/+1 and flying.
    let mut t = TestGame::new(2);
    let noct = t.battlefield(P0, "Vampire Nocturnus");
    let crafter = t.battlefield(P0, "Imagecrafter");
    set_library(&mut t, P0, &["Duress", "Grizzly Bears"]);
    crate::r_s35_common::pick_options(&mut t, P0, &["Elf"]);
    t.activate(P0, crafter, 0, &[o(noct)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(noct).chars.has_subtype("Vampire"));
    assert!(t.obj_now(noct).chars.has_subtype("Elf"));
    assert_eq!(t.pt(noct), (5, 4));
    assert!(has_kw(&t, noct, KeywordKind::Flying));
    // Blocked while the top card is nonblack; then it gains flying: still blocked.
    let mut t = TestGame::new(2);
    let noct = t.battlefield(P0, "Vampire Nocturnus");
    set_library(&mut t, P0, &["Grizzly Bears", "Duress"]);
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(noct, pl(P1))], &[(wall, noct)]);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(noct));
    t.library_top(P0, "Doom Blade");
    t.settle();
    assert!(has_kw(&t, noct, KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(noct));
    // Flying when blockers are declared: the Wall can't block it.
    let mut t = TestGame::new(2);
    let noct = t.battlefield(P0, "Vampire Nocturnus");
    set_library(&mut t, P0, &["Duress"]);
    let wall = t.battlefield(P1, "Wall of Stone");
    attack_with(&mut t, &[(noct, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(wall, noct)]));
}

#[test]
fn knight_of_grace_and_nocturnal_raid_restatements() {
    cr!("611.2c", "613.4c");
    ruling!(
        "Knight of Grace",
        "Knight of Grace only gets +1/+0 if any player controls a black permanent, no matter how many black permanents players control."
    );
    ruling!(
        "Nocturnal Raid",
        "On resolution, all black creatures on the battlefield get the bonus. Ones that enter later in the turn do not get it."
    );
    supported("Knight of Grace");
    supported("Nocturnal Raid");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Grace");
    assert_eq!(t.pt(knight), (2, 2));
    t.battlefield(P1, "Drudge Skeletons");
    t.battlefield(P0, "Phyrexian Obliterator");
    t.battlefield(P1, "Swamp");
    assert_eq!(t.pt(knight), (3, 2));
    // Nocturnal Raid: both players' black creatures, not later ones.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Drudge Skeletons");
    let theirs = t.battlefield(P1, "Drudge Skeletons");
    let white = t.battlefield(P0, "Savannah Lions");
    t.lands(P0, "Swamp", 4);
    let raid = t.hand(P0, "Nocturnal Raid");
    t.cast(P0, raid).go();
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 1));
    assert_eq!(t.pt(theirs), (3, 1));
    assert_eq!(t.pt(white), (2, 1));
    let later = t.battlefield(P0, "Drudge Skeletons");
    assert_eq!(t.pt(later), (1, 1));
}

#[test]
fn shadow_alley_denizen_targets_itself_or_the_new_creature() {
    cr!("115.1", "302.6", "603.2");
    ruling!(
        "Shadow Alley Denizen",
        "Shadow Alley Denizen can be chosen as the target of its own ability."
    );
    ruling!(
        "Shadow Alley Denizen",
        "The black creature that entered can be chosen as the target of Shadow Alley Denizen's ability, but it won't be able to attack that turn unless it also has haste."
    );
    supported("Shadow Alley Denizen");
    let mut t = TestGame::new(2);
    let denizen = t.battlefield(P0, "Shadow Alley Denizen");
    t.answer_targets(P0, &[o(denizen)]);
    t.enter(P0, "Drudge Skeletons");
    t.resolve_all();
    assert!(has_kw(&t, denizen, KeywordKind::Intimidate));
    // The new creature: intimidate, but summoning sick.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Alley Denizen");
    t.lands(P0, "Swamp", 2);
    let sk = t.hand(P0, "Drudge Skeletons");
    t.cast(P0, sk).go();
    t.g.resolve_top();
    let sk = t.named_on_battlefield("Drudge Skeletons")[0];
    t.answer_targets(P0, &[o(sk)]);
    t.resolve_all();
    assert!(has_kw(&t, sk, KeywordKind::Intimidate));
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, sk));
}

#[test]
fn urborg_stalker_triggers_in_your_upkeep_too() {
    cr!("603.4", "503.1a");
    ruling!(
        "Urborg Stalker",
        "Urborg Stalker’s ability will trigger at the beginning of your upkeep as well."
    );
    supported("Urborg Stalker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urborg Stalker");
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn stain_the_mind_may_leave_cards_with_that_name() {
    cr!("701.23b", "701.23d");
    ruling!(
        "Stain the Mind",
        "You can leave any cards with that name in the zone they're in. You don't have to exile them."
    );
    supported("Stain the Mind");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Lightning Bolt");
    let in_yard = t.graveyard(P1, "Lightning Bolt");
    t.library_top(P1, "Lightning Bolt");
    t.lands(P0, "Swamp", 5);
    let stain = t.hand(P0, "Stain the Mind");
    t.answer(P0, DecisionKind::Name, Answer::Text("Lightning Bolt".into()));
    // P0 exiles only the one in the graveyard.
    t.answer_choose(P0, &[o(in_yard)]);
    t.cast(P0, stain).target(pl(P1)).go();
    t.resolve_all();
    let bolts_exiled = t
        .g
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Exile && o.chars.name == "Lightning Bolt")
        .count();
    assert_eq!(bolts_exiled, 1);
    // The ones in hand and library stay where they are.
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
    assert!(t
        .g
        .player(P1)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name == "Lightning Bolt"));
    assert_eq!(t.zone(in_yard), Zone::Exile);
}

#[test]
fn bloodline_bidding_chooses_the_type_as_it_resolves() {
    cr!("608.2c", "205.3m");
    ruling!(
        "Bloodline Bidding",
        "You choose the creature type as Bloodline Bidding resolves. Once Bloodline Bidding starts to resolve, players can't respond to the choice or take any actions until Bloodline Bidding finishes resolving."
    );
    supported("Bloodline Bidding");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 8);
    let bb = t.hand(P0, "Bloodline Bidding");
    let from = t.asked().len();
    t.cast(P0, bb).go();
    assert!(crate::r_s35_common::creature_type_options(&t, P0, from).is_empty());
    crate::r_s35_common::pick_options(&mut t, P0, &["Bear"]);
    let from = t.asked().len();
    t.resolve_all();
    // The choice is made during resolution, and nobody gets priority until it's done.
    let asked = t.asked()[from..].to_vec();
    assert_eq!(crate::r_s35_common::creature_type_options(&t, P0, from).len(), 1);
    assert!(!asked
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn lilianas_spoils_targets_an_opponent_with_an_empty_hand() {
    cr!("115.1", "608.2c");
    ruling!(
        "Liliana's Spoils",
        "You may cast Liliana’s Spoils targeting any opponent, even one who has no cards in hand. You’ll perform the actions of its second ability even if the target opponent can’t discard a card."
    );
    supported("Liliana's Spoils");
    let mut t = TestGame::new(2);
    crate::r_p163_common::empty_hand(&mut t, P1);
    assert_eq!(t.hand_size(P1), 0);
    let duress = set_library(
        &mut t,
        P0,
        &["Grizzly Bears", "Duress", "Hill Giant", "Forest", "Island", "Swamp"],
    )[1];
    t.lands(P0, "Swamp", 4);
    let spoils = t.hand(P0, "Liliana's Spoils");
    t.answer_choose(P0, &[o(duress)]);
    t.cast(P0, spoils).target(pl(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Duress"));
}

#[test]
fn hoarding_broodlords_card_stays_playable_by_you_after_a_control_change() {
    cr!("611.2c", "109.4");
    ruling!(
        "Hoarding Broodlord",
        "If another player gains control of Hoarding Broodlord, it's still you who can look at and play the exiled card. That player won't be able to look at or play the exiled card."
    );
    supported("Hoarding Broodlord");
    let mut t = TestGame::new(2);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.answer_choose(P0, &[o(bolt)]);
    let brood = t.enter(P0, "Hoarding Broodlord");
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    give_control(&mut t, brood, P1);
    assert_eq!(t.obj_now(brood).controller, P1);
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    assert!(castable(&mut t, P0, bolt));
    assert!(!castable(&mut t, P1, bolt));
    assert!(mtg_engine::zones::may_look(&t.g, P0, bolt));
    assert!(!mtg_engine::zones::may_look(&t.g, P1, bolt));
}
