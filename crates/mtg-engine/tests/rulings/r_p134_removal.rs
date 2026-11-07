//! Rulings batch P134 — destroying and exiling artifacts, creatures and enchantments
//! ("removal-destroy", "removal-enchantment", "removal-equipment", "removal-exile"):
//! illegal targets (CR 608.2b), regeneration and indestructible (CR 701.19, 702.12b),
//! "destroyed this way" counts, replacement effects that exile instead (CR 614), and
//! simultaneous zone changes (CR 603.6a, 603.10a).

use crate::r_p113_common::cast_modes;
use crate::r_p133_common::{fizzles, kill};
use crate::r_p134_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use crate::r_s07_common::{chosen_modes, resolved};
use crate::r_s25_common::{creature_tokens, lands_for_cost};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Illegal targets: the spell doesn't resolve and none of its effects happen.
// ---------------------------------------------------------------------------------------

#[test]
fn removal_spells_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!("Cursebreak", "If the enchantment is an illegal target when Cursebreak tries to resolve, it’ll be countered and none of its effects will happen. You won’t gain 2 life.");
    ruling!("Aura Blast", "You don’t draw a card if this spell is countered or if the target is illegal on resolution.");
    ruling!("Divine Gambit", "If the artifact, creature, or enchantment is an illegal target as Divine Gambit tries to resolve, Divine Gambit won't resolve and none of its effects will happen. No player will put any cards onto the battlefield.");
    ruling!("Blastfire Bolt", "If the target creature is an illegal target when Blastfire Bolt tries to resolve, it won’t resolve and none of its effects will happen. No Equipment will be destroyed.");
    // Cursebreak / Aura Blast at P1's Glorious Anthem, which leaves.
    for name in ["Cursebreak", "Aura Blast"] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Island");
        let anthem = t.battlefield(P1, "Glorious Anthem");
        let s = cast_modes(&mut t, P0, name, &[], &[ob(anthem)]);
        fizzles(&mut t, s, |t| kill(t, anthem));
    }
    // Aura Blast countered: no card.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let s = cast_modes(&mut t, P0, "Aura Blast", &[], &[ob(anthem)]);
    cast_modes(&mut t, P1, "Counterspell", &[], &[ob(s)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.on_battlefield(anthem));
    // Divine Gambit: P1 would have put Hill Giant from their hand onto the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Hill Giant");
    t.answer_yes(P1, true);
    let s = cast_modes(&mut t, P0, "Divine Gambit", &[], &[ob(bears)]);
    fizzles(&mut t, s, |t| kill(t, bears));
    assert!(t.in_hand(P1, "Hill Giant"));
    // Blastfire Bolt: the creature's Equipment stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let eq = attach_new(&mut t, P1, "Bonesplitter", bears);
    let s = cast_modes(&mut t, P0, "Blastfire Bolt", &[], &[ob(bears)]);
    fizzles(&mut t, s, |t| {
        let b = t.g.current(bears);
        t.g.move_object(b, Zone::Hand(P1), events::MoveCause::Effect, None);
        t.settle();
    });
    assert!(t.on_battlefield(eq));
}

#[test]
fn blastfire_bolt_can_target_a_creature_without_equipment() {
    cr!("115.1", "120.3");
    ruling!("Blastfire Bolt", "Blastfire Bolt can target any creature, not just one with Equipment attached to it. The damage will be dealt to the creature even if it doesn’t have any Equipment attached to it.");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P1, "Serra Angel");
    let eq = t.battlefield(P1, "Bonesplitter");
    cast_modes(&mut t, P0, "Blastfire Bolt", &[], &[ob(w)]);
    t.resolve_all();
    assert!(!t.on_battlefield(w));
    assert!(t.on_battlefield(eq), "unattached Equipment isn't destroyed");
}

#[test]
fn crush_doesnt_resolve_if_the_artifact_became_a_creature() {
    cr!("608.2b", "115.1");
    ruling!("Crush", "If the targeted artifact is also a creature when Crush tries to resolve, it won’t resolve.");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let s = cast_modes(&mut t, P0, "Crush", &[], &[ob(ring)]);
    fizzles(&mut t, s, |t| animate(t, ring));
    assert!(t.on_battlefield(ring));
}

#[test]
fn killing_glare_checks_power_on_cast_and_on_resolution() {
    cr!("608.2b", "601.2c", "107.3");
    ruling!("Killing Glare", "The power of the target creature is checked both as you target it and as Killing Glare resolves. If its power is greater than the value chosen for X when Killing Glare tries to resolve, it won't resolve and none of its effects will happen.");
    supported("Killing Glare");
    // X = 2: Hill Giant (power 3) can't be targeted.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Killing Glare");
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Killing Glare");
    let from = t.asked().len();
    let s = t.cast(P0, card).x(2).target(bears).go();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands[0].contains(&ob(bears)) && !cands[0].contains(&ob(giant)));
    // The Bears gets +1/+0 in response: it doesn't resolve.
    fizzles(&mut t, s, |t| {
        modify(
            t,
            P1,
            bears,
            vec![mtg_engine::ability::Modification::ModifyPT(
                mtg_engine::ability::Value::Const(1),
                mtg_engine::ability::Value::Const(0),
            )],
        )
    });
}

#[test]
fn dawn_to_dusk_affects_only_legal_targets() {
    cr!("608.2b", "700.2");
    ruling!("Dawn to Dusk", "Dawn to Dusk won’t affect any target that is illegal when it tries to resolve. If you chose to use both modes and both targets are illegal at that time, Dawn to Dusk won’t resolve.");
    supported("Dawn to Dusk");
    // (graveyard card leaves, battlefield enchantment leaves)
    for (gy_gone, bf_gone) in [(true, false), (false, true), (true, true)] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P0, "Bad Moon");
        let anthem = t.battlefield(P1, "Glorious Anthem");
        let s = cast_modes(&mut t, P0, "Dawn to Dusk", &[0, 1], &[ob(card), ob(anthem)]);
        if gy_gone {
            t.g.move_object(card, Zone::Exile, events::MoveCause::Effect, None);
        }
        if bf_gone {
            let a = t.g.current(anthem);
            t.g.move_object(a, Zone::Hand(P1), events::MoveCause::Effect, None);
        }
        t.resolve_all();
        assert_eq!(resolved(&t, s), !(gy_gone && bf_gone));
        assert_eq!(t.in_hand(P0, "Bad Moon"), !gy_gone, "{gy_gone} {bf_gone}");
        assert_eq!(
            t.in_graveyard(P1, "Glorious Anthem"),
            !bf_gone,
            "{gy_gone} {bf_gone}"
        );
    }
}

#[test]
fn two_target_exile_with_one_target_gone_affects_the_other() {
    cr!("608.2b", "115.3");
    ruling!("Dust to Dust", "If one target is removed or becomes invalid after declaration, the other target is still affected.");
    ruling!(
        "Into the Core",
        "You must be able to target two different artifacts in order to cast Into the Core."
    );
    ruling!("Peace and Quiet", "Must target two different enchantments.");
    for name in ["Dust to Dust", "Into the Core"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Sol Ring");
        let b = t.battlefield(P1, "Bonesplitter");
        cast_modes(&mut t, P0, name, &[], &[ob(a), ob(b)]);
        kill(&mut t, a);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Sol Ring"), "{name}");
        assert!(t.in_exile("Bonesplitter"), "{name}");
    }
    // One artifact / enchantment isn't enough.
    for (name, perm) in [
        ("Into the Core", "Sol Ring"),
        ("Peace and Quiet", "Glorious Anthem"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, perm);
        lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        assert!(
            t.cast(P0, card).targets(&[ob(a), ob(a)]).try_go().is_err(),
            "{name}"
        );
        t.clear_answers();
        let b = t.battlefield(
            P1,
            if perm == "Sol Ring" {
                "Bonesplitter"
            } else {
                "Bad Moon"
            },
        );
        t.cast(P0, card).targets(&[ob(a), ob(b)]).go();
        t.resolve_all();
        assert!(!t.on_battlefield(a) && !t.on_battlefield(b), "{name}");
    }
}

#[test]
fn brittle_effigy_stays_exiled_if_its_target_is_illegal() {
    cr!("602.2b", "608.2b", "118.3");
    ruling!("Brittle Effigy", "Brittle Effigy is exiled as a cost to activate its ability, not as part of its effect. If the targeted creature is an illegal target by the time the ability would resolve, the ability doesn’t resolve, but Brittle Effigy remains exiled.");
    supported("Brittle Effigy");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Brittle Effigy");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    t.activate(P0, e, 0, &[ob(bears)]).expect("activate");
    assert!(t.in_exile("Brittle Effigy"), "exiled as a cost");
    t.g.move_object(bears, Zone::Hand(P1), events::MoveCause::Effect, None);
    t.resolve_all();
    assert!(t.in_exile("Brittle Effigy"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn mortify_destroys_a_target_that_changed_between_creature_and_enchantment() {
    cr!("608.2b", "115.1");
    ruling!("Mortify", "This spell isn't modal. When it resolves, it will destroy the target if it's a creature or an enchantment, even if it changed from one to the other between targeting and resolution.");
    supported("Mortify");
    // An enchantment targeted, then animated (a creature enchantment): destroyed.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    cast_modes(&mut t, P0, "Mortify", &[], &[ob(anthem)]);
    animate(&mut t, anthem);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // An enchantment creature that stops being a creature: still an enchantment.
    let mut t = TestGame::new(2);
    let ram = t.battlefield(P1, "Nyx-Fleece Ram");
    cast_modes(&mut t, P0, "Mortify", &[], &[ob(ram)]);
    modify(
        &mut t,
        P1,
        ram,
        vec![mtg_engine::ability::Modification::RemoveTypes(vec![
            types::CardType::Creature,
        ])],
    );
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Nyx-Fleece Ram"));
}

// ---------------------------------------------------------------------------------------
// Regeneration and indestructible.
// ---------------------------------------------------------------------------------------

#[test]
fn shatter_and_aura_blast_regeneration() {
    cr!("701.19a", "701.19c");
    ruling!(
        "Shatter",
        "Regenerating artifacts can regenerate from this."
    );
    ruling!(
        "Aura Blast",
        "You still draw a card if the enchantment regenerates."
    );
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let s = cast_modes(&mut t, P0, "Shatter", &[], &[ob(ring)]);
    regen_shield(&mut t, ring);
    t.resolve_all();
    assert!(resolved(&t, s));
    assert!(t.on_battlefield(ring));
    assert!(t.obj_now(ring).tapped);
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    cast_modes(&mut t, P0, "Aura Blast", &[], &[ob(anthem)]);
    regen_shield(&mut t, anthem);
    t.resolve_all();
    assert!(t.on_battlefield(anthem));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn destroyed_this_way_counts_only_permanents_that_were_destroyed() {
    cr!("701.8a", "701.8b", "701.19a", "702.12b", "614.6");
    ruling!("Fracturing Gust", "If an artifact or enchantment remains on the battlefield because it was regenerated or has indestructible, you won’t gain life for it.");
    ruling!("Bane of Progress", "If an artifact or enchantment isn't destroyed (perhaps because it has indestructible or it regenerated), it won't count toward the number of +1/+1 counters put on Bane of Progress. However, if an artifact or enchantment is destroyed but doesn't go to its owner's graveyard due to a replacement effect (like the one Rest in Peace creates), it will count.");
    ruling!("Bane of Progress", "Bane of Progress's ability destroys all artifacts and enchantments, including those you control.");
    ruling!("Rampage of the Clans", "If a permanent has indestructible, it isn’t destroyed this way and its controller won’t create a Centaur token for it. If it is destroyed but put into a zone other than a graveyard, its controller does create a Centaur token for it.");
    supported("Fracturing Gust");
    supported("Bane of Progress");
    supported("Rampage of the Clans");
    /// P0: Sol Ring, Bad Moon; P1: Darksteel Relic (indestructible), a regenerating
    /// Bonesplitter, Glorious Anthem, and (with `rip`) Rest in Peace.
    fn setup(t: &mut TestGame, rip: bool) {
        t.battlefield(P0, "Sol Ring");
        t.battlefield(P0, "Bad Moon");
        t.battlefield(P1, "Darksteel Relic");
        let b = t.battlefield(P1, "Bonesplitter");
        regen_shield(t, b);
        t.battlefield(P1, "Glorious Anthem");
        if rip {
            t.battlefield(P1, "Rest in Peace");
        }
    }
    for rip in [false, true] {
        // Fracturing Gust: 2 life per permanent destroyed.
        let mut t = TestGame::new(2);
        setup(&mut t, rip);
        cast_modes(&mut t, P0, "Fracturing Gust", &[], &[]);
        t.resolve_all();
        let destroyed = if rip { 4 } else { 3 };
        assert_eq!(t.life(P0), 20 + 2 * destroyed, "rip {rip}");
        assert_eq!(t.named_on_battlefield("Darksteel Relic").len(), 1);
        assert_eq!(t.named_on_battlefield("Bonesplitter").len(), 1);
        assert_eq!(t.named_on_battlefield("Sol Ring").len(), 0, "yours too");
        // Bane of Progress: a counter per permanent destroyed.
        let mut t = TestGame::new(2);
        setup(&mut t, rip);
        let bane = t.enter(P0, "Bane of Progress");
        t.resolve_all();
        assert_eq!(t.counters(bane, "+1/+1"), destroyed as u32, "rip {rip}");
        assert!(t
            .g
            .permanents()
            .all(|o| o.chars.name != "Sol Ring" && o.chars.name != "Bad Moon"));
        if rip {
            assert!(t.in_exile("Glorious Anthem"));
        }
        // Rampage of the Clans: each controller gets a Centaur per permanent of theirs
        // destroyed.
        let mut t = TestGame::new(2);
        setup(&mut t, rip);
        cast_modes(&mut t, P0, "Rampage of the Clans", &[], &[]);
        t.resolve_all();
        assert_eq!(creature_tokens(&t, P0), 2, "rip {rip}");
        assert_eq!(
            creature_tokens(&t, P1),
            if rip { 2 } else { 1 },
            "rip {rip}"
        );
    }
}

#[test]
fn rampage_tokens_arrive_after_the_destroyed_permanents_left() {
    cr!("603.6a", "603.10a", "608.2c");
    ruling!("Rampage of the Clans", "All of the artifacts and enchantments that are destroyed leave the battlefield before any Centaurs are created, and their abilities can’t affect or trigger on these tokens entering the battlefield.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impact Tremors");
    t.battlefield(P0, "Glorious Anthem");
    cast_modes(&mut t, P0, "Rampage of the Clans", &[], &[]);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0), 2);
    assert_eq!(t.life(P1), 20, "Impact Tremors didn't see the Centaurs");
    let c =
        t.g.permanents()
            .find(|o| o.controller == P0 && o.is_token())
            .map(|o| o.id)
            .unwrap();
    assert_eq!(t.pt(c), (3, 3), "Glorious Anthem doesn't pump them");
}

#[test]
fn magus_of_the_disk_survives_its_ability_if_indestructible_or_regenerating() {
    cr!("701.8b", "701.19a");
    ruling!("Magus of the Disk", "If an effect gives Magus of the Disk indestructible or regenerates it, it won’t be destroyed by its ability.");
    supported("Magus of the Disk");
    for regen in [false, true] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Magus of the Disk");
        t.g.untap(m);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Wastes", 1);
        t.activate(P0, m, 0, &[]).expect("activate");
        if regen {
            regen_shield(&mut t, m);
        } else {
            indestructible(&mut t, m);
        }
        t.resolve_all();
        assert!(t.on_battlefield(m), "regen {regen}");
        assert!(!t.on_battlefield(bears));
    }
}

#[test]
fn serenity_destroys_itself() {
    cr!("701.8a", "503.1a");
    ruling!(
        "Serenity",
        "Serenity's triggered ability destroys itself too."
    );
    supported("Serenity");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    t.battlefield(P0, "Serenity");
    let ring = t.battlefield(P1, "Sol Ring");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Serenity"));
    assert!(!t.on_battlefield(ring));
}

// ---------------------------------------------------------------------------------------
// "Exile instead" replacement effects.
// ---------------------------------------------------------------------------------------

#[test]
fn exile_instead_of_dying_lasts_the_whole_turn() {
    cr!("614.1a", "614.6", "611.2a");
    ruling!("Bleed Dry", "If Bleed Dry resolves, its effect causes the creature to be exiled any time it would die that turn, not just if it would die as a result of having toughness 0 or less.");
    ruling!("Ob Nixilis's Cruelty", "The replacement effect from Ob Nixilis’s Cruelty will exile the target creature if it would die this turn for any reason, not just immediately after Ob Nixilis’s Cruelty resolves.");
    for (spell, victim) in [
        ("Bleed Dry", "Worldspine Wurm"),
        ("Ob Nixilis's Cruelty", "Colossal Dreadmaw"),
    ] {
        supported(spell);
        let mut t = TestGame::new(2);
        let v = t.battlefield(P1, victim);
        cast_modes(&mut t, P0, spell, &[], &[ob(v)]);
        t.resolve_all();
        assert!(t.on_battlefield(v), "{spell}: it survived the -N/-N");
        destroy(&mut t, v);
        t.resolve_all();
        assert!(t.in_exile(victim), "{spell}");
        assert!(!t.in_graveyard(P1, victim), "{spell}");
        assert_eq!(creature_tokens(&t, P1), 0, "{spell}: it didn't die");
    }
}

#[test]
fn malicious_malfunction_exiles_every_creature_that_would_die_this_turn() {
    cr!("611.2c", "614.1a");
    ruling!("Malicious Malfunction", "Creatures that would die for any reason this turn are exiled instead, not just creatures that would die after getting -2/-2.");
    ruling!("Malicious Malfunction", "While the -2/-2 applies only to creatures that are on the battlefield as Malicious Malfunction resolves, the replacement effect applies to all creatures that turn, including ones that enter the battlefield after Malicious Malfunction resolves or noncreatures that become creatures after Malicious Malfunction resolves.");
    supported("Malicious Malfunction");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Craw Wurm");
    let ring = t.battlefield(P1, "Sol Ring");
    cast_modes(&mut t, P0, "Malicious Malfunction", &[], &[]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.pt(giant), (4, 2));
    // Destroyed later: exiled.
    destroy(&mut t, giant);
    assert!(t.in_exile("Craw Wurm"));
    // A creature that enters later isn't -2/-2, but is exiled if it would die.
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    destroy(&mut t, late);
    assert!(t.in_exile("Hill Giant"));
    // A noncreature that becomes a creature afterwards.
    animate(&mut t, ring);
    destroy(&mut t, ring);
    assert!(t.in_exile("Sol Ring"));
}

#[test]
fn unnatural_aggression_style_exile_and_false_prophet() {
    cr!("603.10a", "406.1");
    ruling!("False Prophet", "False Prophet only exiles creatures, which means only creature permanents which are on the battlefield. It won’t exile itself since it is already in the graveyard.");
    supported("False Prophet");
    let mut t = TestGame::new(2);
    let fp = t.battlefield(P0, "False Prophet");
    t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    destroy(&mut t, fp);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "False Prophet"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(
        t.in_graveyard(P1, "Hill Giant"),
        "creature cards in graveyards stay"
    );
}

// ---------------------------------------------------------------------------------------
// "Until this leaves the battlefield" exile.
// ---------------------------------------------------------------------------------------

/// P1's Grizzly Bears (with a +1/+1 counter, P1's Holy Strength and Bonesplitter attached)
/// targeted by `name`'s enters trigger. Returns (game, source, bears, aura, equipment).
fn jail(name: &str) -> (TestGame, ObjectId, ObjectId, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, bears, "+1/+1", 1);
    let aura = attach_new(&mut t, P1, "Holy Strength", bears);
    let eq = attach_new(&mut t, P1, "Bonesplitter", bears);
    t.answer_targets(P0, &[ob(bears)]);
    let src = t.enter(P0, name);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{name}");
    (t, src, bears, aura, eq)
}

#[test]
fn until_leaves_exile_does_nothing_if_the_source_left_first() {
    cr!("610.3c");
    ruling!("Isolation Zone", "If Isolation Zone leaves the battlefield before its triggered ability resolves, the target creature or enchantment won’t be exiled.");
    ruling!("Earth Kingdom Jailer", "If Earth Kingdom Jailer leaves the battlefield before its triggered ability resolves, the target artifact, creature, or enchantment won't be exiled.");
    for name in ["Isolation Zone", "Earth Kingdom Jailer"] {
        let (mut t, src, bears, _, _) = jail(name);
        kill(&mut t, src);
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{name}");
    }
}

#[test]
fn isolation_zone_exile_and_return() {
    cr!("610.3a", "704.5m", "122.2", "400.7", "301.5c");
    ruling!("Isolation Zone", "Auras attached to the exiled creature or enchantment will be put into their owners’ graveyards. Any Equipment will become unattached and remain on the battlefield. Any counters on the exiled permanent will cease to exist.");
    ruling!("Isolation Zone", "The exiled card returns to the battlefield immediately after Isolation Zone leaves the battlefield. Nothing happens between the two events, including state-based actions.");
    let (mut t, zone, bears, aura, eq) = jail("Isolation Zone");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert!(!t.on_battlefield(aura));
    assert!(t.on_battlefield(eq));
    assert!(t.obj_now(eq).attached_to.is_none());
    // The Zone leaves: the Bears is back at once, without a counter, before anything
    // else happens (nothing on the stack, no priority).
    kill(&mut t, zone);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], bears);
    assert_eq!(t.counters(back[0], "+1/+1"), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn isolation_zone_returns_the_card_if_its_owner_leaves_the_game() {
    cr!("800.4a", "610.3a");
    ruling!("Isolation Zone", "In a multiplayer game, if Isolation Zone’s owner leaves the game, the exiled card will return to the battlefield.");
    supported("Isolation Zone");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[ob(bears)]);
    t.enter(P0, "Isolation Zone");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.g.lose_game(P0);
    t.settle();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj_now(back[0]).controller, P1);
}

// ---------------------------------------------------------------------------------------
// Library placement.
// ---------------------------------------------------------------------------------------

#[test]
fn owner_chooses_second_from_top_or_bottom() {
    cr!("401.4");
    ruling!("Deem Inferior", "The permanent's owner chooses whether to put it second from the top or on the bottom of their library.");
    ruling!("Lost Days", "The permanent's owner chooses whether to put it second from the top or on the bottom of their library.");
    for name in ["Deem Inferior", "Lost Days"] {
        supported(name);
        for bottom in [false, true] {
            let mut t = TestGame::new(2);
            stack_library(&mut t, P1, &["Island", "Forest", "Swamp"]);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let from = t.asked().len();
            t.answer_yes(P1, !bottom);
            t.answer(
                P1,
                DecisionKind::Option,
                mtg_engine::decision::Answer::Index(bottom as usize),
            );
            cast_modes(&mut t, P0, name, &[], &[ob(bears)]);
            t.resolve_all();
            let lib = &t.g.players[1].library;
            let card = t.g.current(bears);
            let pos = lib.iter().position(|c| *c == card).expect("in the library");
            if bottom {
                assert_eq!(pos, 0, "{name}: bottom");
            } else {
                assert_eq!(pos, lib.len() - 2, "{name}: second from the top");
            }
            assert!(
                t.asked()[from..].iter().any(|(p, d)| *p == P1
                    && !matches!(d, mtg_engine::decision::Decision::Priority { .. })),
                "{name}: the owner chose"
            );
        }
    }
}

#[test]
fn harmonic_convergence_owner_orders_their_enchantments() {
    cr!("401.4");
    ruling!("Harmonic Convergence", "If any player needs to put more than one enchantment on top of their library, they choose the order.");
    supported("Harmonic Convergence");
    // P1 orders their two enchantments one way, then the other: the order they chose is
    // the order in their library.
    let mut tops = Vec::new();
    for reversed in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Glorious Anthem");
        t.battlefield(P1, "Bad Moon");
        let order = if reversed { vec![1, 0] } else { vec![0, 1] };
        t.answer(
            P1,
            DecisionKind::Order,
            mtg_engine::decision::Answer::Indices(order),
        );
        cast_modes(&mut t, P0, "Harmonic Convergence", &[], &[]);
        t.resolve_all();
        let lib = &t.g.players[1].library;
        let names: Vec<String> = lib[lib.len() - 2..]
            .iter()
            .map(|c| t.obj(*c).chars.name.to_string())
            .collect();
        assert!(
            names.contains(&"Glorious Anthem".to_string())
                && names.contains(&"Bad Moon".to_string()),
            "{names:?}"
        );
        let asked_p1 = t
            .asked()
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Order { .. }));
        assert!(asked_p1, "P1 orders their own enchantments");
        tops.push(names[1].clone());
    }
    assert_ne!(tops[0], tops[1], "the owner's order is used");
}

// ---------------------------------------------------------------------------------------
// Modes and costs.
// ---------------------------------------------------------------------------------------

#[test]
fn modes_are_chosen_on_cast_and_performed_in_printed_order() {
    cr!("700.2a", "700.2", "601.2b");
    ruling!(
        "Merciless Eviction",
        "You choose the mode as you cast the spell."
    );
    ruling!("Farewell", "If you choose more than one mode for Farewell, you perform the actions in the order written.");
    supported("Merciless Eviction");
    supported("Farewell");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    let s = cast_modes(&mut t, P0, "Merciless Eviction", &[1], &[]);
    assert_eq!(chosen_modes(&t, s), vec![1]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(ring));
    // Farewell: creatures and graveyards chosen (answered out of order) are exiled in
    // printed order.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let s = cast_modes(&mut t, P0, "Farewell", &[3, 1], &[]);
    assert_eq!(chosen_modes(&t, s), vec![1, 3]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
}

#[test]
fn righteous_confluence_each_life_mode_is_a_separate_life_gain() {
    cr!("700.2d", "119.9", "603.2c");
    ruling!("Righteous Confluence", "If the third mode is chosen multiple times, each of those modes represents a separate life-gaining event.");
    supported("Righteous Confluence");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Karlov of the Ghost Council");
    cast_modes(&mut t, P0, "Righteous Confluence", &[2, 2, 2], &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 35);
    assert_eq!(t.counters(k, "+1/+1"), 6);
}

#[test]
fn march_of_otherworldly_light_x_and_exiled_white_cards() {
    cr!("601.2b", "601.2f", "202.3e", "118.7");
    ruling!("March of Otherworldly Light", "For example, say you wanted to choose a target with mana value 6. You need X to be at least 6.");
    ruling!("March of Otherworldly Light", "The mana value of March of Otherworldly Light while it's on the stack is the value chosen for X plus 1, no matter how much mana you actually paid to cast it.");
    ruling!("March of Otherworldly Light", "You may exile more cards than necessary for March of Otherworldly Light's first ability, but you can't reduce the mana it costs to less than {W} this way.");
    supported("March of Otherworldly Light");
    // X = 6 at Colossal Dreadmaw (mana value 6), exiling four white cards from hand: it
    // costs {W} (not less), and its mana value is 7.
    let mut t = TestGame::new(2);
    let dm = t.battlefield(P1, "Colossal Dreadmaw");
    let whites: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Glorious Anthem")).collect();
    t.lands(P0, "Plains", 1);
    let card = t.hand(P0, "March of Otherworldly Light");
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Number(4),
    );
    let s = t.cast(P0, card).x(6).target(dm).go();
    assert!(
        whites.iter().all(|w| t.zone(*w) == Zone::Exile),
        "four exiled"
    );
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 1);
    assert_eq!(crate::r_s08_common::mana_value(&t, s), 7);
    t.resolve_all();
    assert!(t.in_exile("Colossal Dreadmaw"));
    // Without exiling: {6}{W}; X = 5 can't target it.
    let mut t = TestGame::new(2);
    let dm = t.battlefield(P1, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 5);
    let card = t.hand(P0, "March of Otherworldly Light");
    let from = t.asked().len();
    t.cast(P0, card).x(5).target(bears).go();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!cands[0].contains(&ob(dm)));
}

#[test]
fn erase_exiles_without_going_to_the_graveyard_first() {
    cr!("406.2", "603.6c");
    ruling!("Erase", "The card does not go to the graveyard first.");
    supported("Erase");
    // Rancor: "When this Aura is put into a graveyard from the battlefield, return it to
    // its owner's hand." It never goes there, so it isn't returned.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let r = attach_new(&mut t, P1, "Rancor", bears);
    cast_modes(&mut t, P0, "Erase", &[], &[ob(r)]);
    t.resolve_all();
    assert!(t.in_exile("Rancor"));
    assert!(!t.in_hand(P1, "Rancor"));
    assert_eq!(t.stack_len(), 0);
}
