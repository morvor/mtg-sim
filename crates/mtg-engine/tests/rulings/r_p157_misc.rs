//! Rulings batch P157 — the rest of the batch: conditions checked as a spell resolves
//! (Triumphs, Invasive Maneuvers, Anoint with Affliction, Stitch Together, Golden Demise,
//! Pride of Conquerors), self-replacement effects (Dark Temper, CR 614.15), uncounterable
//! spells (Urza's Rage, Devious Cover-Up), last known information (Blade of the
//! Bloodchief, Adipose Offspring), base power and toughness (Behind the Mask), Auras
//! returned without targeting (Cleansing Meditation, CR 303.4f), and triggers on cards
//! put into graveyards (Sefris), dying together (Anax), being targeted (Fblthp),
//! entering (Emiel) and dredge (The Necrobloom).

use crate::r_p157_common::*;
use crate::r_p180_common::plus;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The number of damage events this turn.
fn damage_events(t: &TestGame) -> usize {
    crate::r_p194_common::damage_events(t).len()
}

#[test]
fn sefris_must_be_on_the_battlefield_not_its_own_death() {
    cr!("603.10a", "603.6c", "701.49a");
    ruling!(
        "Sefris of the Hidden Ways",
        "Sefris of the Hidden Ways must be on the battlefield for its first ability to trigger. It does not trigger when Sefris goes to the graveyard from the battlefield, even if other creature cards also went to the graveyard at the same time."
    );
    supported("Sefris of the Hidden Ways");
    let mut t = TestGame::new(2);
    let ids = [
        t.battlefield(P0, "Sefris of the Hidden Ways"),
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Savannah Lions"),
    ];
    // All three are destroyed at once.
    crate::r_p146_common::destroy_together(&mut t, &ids);
    assert!(t.in_graveyard(P0, "Sefris of the Hidden Ways"));
    assert_eq!(t.stack_len(), 0, "no venture");
    // On the battlefield: a creature card put into the graveyard triggers it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sefris of the Hidden Ways");
    let bears = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn sefris_looks_at_the_card_in_the_graveyard() {
    cr!("603.6c", "715.4", "400.7");
    ruling!(
        "Sefris of the Hidden Ways",
        "Sefris of the Hidden Ways' first ability looks only at the type the card has in the graveyard, not the type it had in any other zone. For example, animated lands dying won't cause the ability to trigger. The opposite is also true. For example, if a creature that was cast as an Adventure is countered, it's a creature card in the graveyard and the first ability will trigger."
    );
    supported("Mutavault");
    // An animated Mutavault dies: a land card in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sefris of the Hidden Ways");
    let mv = t.battlefield(P0, "Mutavault");
    act(&mut t, P0, mv, 1, &[]);
    assert!(t.obj_now(mv).is(mtg_engine::types::CardType::Creature));
    kill(&mut t, mv);
    assert!(t.in_graveyard(P0, "Mutavault"));
    assert_eq!(t.stack_len(), 0);
    // Stomp (the Adventure of Bonecrusher Giant) is countered: a creature card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sefris of the Hidden Ways");
    lots(&mut t, P0);
    let g = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.answer_targets(P0, &[pl(P1)]);
    let stomp = t.cast(P0, g).method(CastMethod::Half(1)).go();
    cast_at(&mut t, P1, "Counterspell", &[obj(stomp)]);
    t.resolve();
    assert!(!on_stack(&t, stomp));
    assert_eq!(t.graveyard_size(P0), 1, "the Giant card");
    assert_eq!(t.stack_len(), 1, "venture triggered");
}

#[test]
fn emeria_shepherd_nonland_permanent_cards() {
    cr!("110.4", "109.2");
    // The ruling predates battles: under today's rules (CR 110.4) a battle card is a
    // nonland permanent card too (`docs/RULING_DISCREPANCIES.md`, "Not yet
    // investigated").
    ruling!(
        "Emeria Shepherd",
        "A \"nonland permanent card\" is an artifact, creature, enchantment, or planeswalker card."
    );
    supported("Emeria Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Emeria Shepherd");
    let ok = [
        t.graveyard(P0, "Ornithopter"),
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P0, "Pacifism"),
        t.graveyard(P0, "Chandra Nalaar"),
        t.graveyard(P0, "Invasion of Zendikar"),
    ];
    let no = [
        t.graveyard(P0, "Forest"),
        t.graveyard(P0, "Shock"),
        t.graveyard(P0, "Divination"),
    ];
    let m = mark(&t);
    t.answer_targets(P0, &[obj(ok[0])]);
    t.answer_yes(P0, true);
    let f = t.hand(P0, "Island");
    t.play_land(P0, f).unwrap();
    t.resolve_all();
    let c = target_choices(&t, P0, m, "");
    assert!(!c.is_empty());
    for id in ok {
        assert!(c[0].contains(&obj(id)));
    }
    for id in no {
        assert!(!c[0].contains(&obj(id)));
    }
    assert!(t.in_hand(P0, "Ornithopter"));
}

#[test]
fn urzas_rage_and_devious_cover_up_uncounterable_targets() {
    cr!("101.2", "701.6a", "608.2c", "115.1");
    ruling!(
        "Urza's Rage",
        "A spell or ability that counters spells can still target Urza's Rage. When that spell or ability resolves, Urza's Rage won't be countered, but any additional effects of that spell or ability will still happen."
    );
    ruling!(
        "Devious Cover-Up",
        "A spell that can't be countered is a legal target for Devious Cover-Up. The spell won't be countered when Devious Cover-Up resolves, but you'll still shuffle cards into your library."
    );
    supported("Urza's Rage");
    supported("Devious Cover-Up");
    let mut t = TestGame::new(2);
    let gy: Vec<_> = (0..4).map(|_| t.graveyard(P1, "Island")).collect();
    let rage = cast_at(&mut t, P0, "Urza's Rage", &[pl(P1)]);
    let lib = t.library_size(P1);
    t.answer_targets(P1, &[obj(rage)]);
    t.answer_targets(P1, &gy.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    lots(&mut t, P1);
    let dc = t.hand(P1, "Devious Cover-Up");
    t.cast(P1, dc).go();
    t.resolve();
    assert!(on_stack(&t, rage), "not countered");
    assert_eq!(t.library_size(P1), lib + 4, "the cards were shuffled in");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Countersquall: not countered, but its controller still loses 2 life.
    let mut t = TestGame::new(2);
    let rage = cast_at(&mut t, P0, "Urza's Rage", &[pl(P1)]);
    cast_at(&mut t, P1, "Countersquall", &[obj(rage)]);
    t.resolve();
    assert!(on_stack(&t, rage));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn devious_cover_up_cant_target_itself() {
    cr!("115.5");
    ruling!(
        "Devious Cover-Up",
        "Devious Cover-Up can't target itself at all. You can't try to counter it with itself and shuffle cards in, and you can't use its effect to shuffle itself into your library."
    );
    let mut t = TestGame::new(2);
    let shock = cast_at(&mut t, P1, "Shock", &[pl(P0)]);
    t.graveyard(P0, "Island");
    let m = mark(&t);
    lots(&mut t, P0);
    let dc = t.hand(P0, "Devious Cover-Up");
    t.answer_targets(P0, &[obj(shock)]);
    let spell = t.cast(P0, dc).go();
    for c in target_choices(&t, P0, m, "") {
        assert!(!c.contains(&obj(spell)) && !c.contains(&obj(dc)));
    }
}

#[test]
fn the_eagles_are_coming_counts_returned_tokens() {
    cr!("603.7a", "608.2h", "704.5d");
    ruling!(
        "The Eagles Are Coming!",
        "Any creature tokens that were returned to your hand are counted by the delayed triggered ability. You'll get a Bird Soldier token for each of them."
    );
    supported("The Eagles Are Coming!");
    let mut t = TestGame::new(2);
    cast_at(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let tk = tokens(&t, P0);
    lots(&mut t, P0);
    let c = t.hand(P0, "The Eagles Are Coming!");
    t.cast(P0, c)
        .kicked(true)
        .targets(&[obj(tk[0]), obj(tk[1])])
        .go();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Bird").len(), 2);
}

#[test]
fn anoint_with_affliction_checks_poison_on_resolution() {
    cr!("608.2c", "122.1f");
    ruling!(
        "Anoint with Affliction",
        "As Anoint with Affliction resolves, if the creature's controller has three or more poison counters, the creature will be exiled no matter what its mana value is. If the controller doesn't, the creature will be exiled only if its mana value is 3 or less."
    );
    for (poison, name, exiled) in [
        (0, "Craw Wurm", false),
        (0, "Grizzly Bears", true),
        (3, "Craw Wurm", true),
    ] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        let spell = cast_at(&mut t, P0, "Anoint with Affliction", &[obj(c)]);
        let _ = spell;
        if poison > 0 {
            t.g.players[1].counters.insert("poison".into(), poison);
        }
        t.resolve_all();
        assert_eq!(!t.on_battlefield(c), exiled, "{name} with {poison} poison");
    }
}

#[test]
fn everflame_eidolon_bonus_goes_where_it_is_on_resolution() {
    cr!("608.2c", "702.103b", "611.2c");
    ruling!(
        "Everflame Eidolon",
        "As Everflame Eidolon's ability resolves, the bonus is applied to either Everflame Eidolon (if it's a creature) or to the enchanted creature (if it's an Aura). If the bonus is applied to the enchanted creature, that bonus won't apply to Everflame Eidolon, even if it becomes a creature later that turn."
    );
    supported("Everflame Eidolon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lots(&mut t, P0);
    let e = t.hand(P0, "Everflame Eidolon");
    let method = crate::r_s08_common::legal_cast_methods(&mut t, P0, e)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Keyword(KeywordKind::Bestow)))
        .expect("bestow");
    t.cast(P0, e).method(method).target(bears).go();
    t.resolve_all();
    let e = t.g.current(e);
    assert_eq!(t.pt(bears), (3, 3));
    act(&mut t, P0, e, 0, &[]);
    assert_eq!(t.pt(bears), (4, 3));
    kill(&mut t, bears);
    let e = t.g.current(e);
    assert!(t.obj_now(e).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.pt(e), (1, 1), "the bonus stayed with the Bears");
}

#[test]
fn cleansing_meditation_auras_return_onto_permanents_already_there() {
    cr!("303.4f", "303.4a", "614.12");
    ruling!(
        "Cleansing Meditation",
        "Auras returning to the battlefield are placed on any legal permanent of your choice. Doing this does not target the permanent. If there is no legal permanent for an Aura, it remains in the graveyard."
    );
    ruling!(
        "Cleansing Meditation",
        "Auras returning to the battlefield can only enchant something that was already on the battlefield. An Enchant Enchantment returning to the battlefield can’t be placed on an enchantment that is also in the process of returning to the battlefield."
    );
    supported("Cleansing Meditation");
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pac = t.battlefield(P0, "Pacifism");
    t.g.attach(pac, obj(scout));
    let leak = t.battlefield(P0, "Power Leak");
    t.g.attach(leak, obj(pac));
    t.settle();
    for _ in 0..7 {
        t.graveyard(P0, "Island");
    }
    // Pacifism goes onto the hexproof Scout (not targeted).
    t.answer_choose_moved(P0, &[obj(scout)]);
    cast_at(&mut t, P0, "Cleansing Meditation", &[]);
    t.resolve_all();
    let p = t.named_on_battlefield("Pacifism");
    assert_eq!(p.len(), 1);
    assert_eq!(t.obj_now(p[0]).attached_to, Some(obj(scout)));
    // Power Leak had only the returning Pacifism: it stays in the graveyard.
    assert!(t.in_graveyard(P0, "Power Leak"));
    assert!(!named_here(&t, "Power Leak"));
}

#[test]
fn behind_the_mask_overwrites_earlier_setting_effects_only() {
    cr!("613.4", "613.4b", "613.4c");
    ruling!(
        "Behind the Mask",
        "Behind the Mask will overwrite any previous effects that set a creature or Vehicle's power and toughness to specific numbers. Effects that otherwise modify its power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Behind the Mask");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P0, "Turn to Frog", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (1, 1));
    cast_at(&mut t, P0, "Giant Growth", &[obj(wurm)]);
    t.resolve_all();
    t.g.objects[wurm.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::PLUS1.into(), 1);
    t.g.recompute();
    lots(&mut t, P0);
    let c = t.hand(P0, "Behind the Mask");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, c).target(wurm).go();
    t.resolve_all();
    // 4/3, +3/+3, +1/+1.
    assert_eq!(t.pt(wurm), (8, 7));
}

#[test]
fn behind_the_mask_isnt_crewing_a_vehicle() {
    cr!("702.122a", "702.122c");
    ruling!(
        "Behind the Mask",
        "If Behind the Mask causes a Vehicle to become an artifact creature, it doesn't count as \"crewing\" that Vehicle for any ability that would trigger off of a Vehicle becoming crewed."
    );
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    t.battlefield(P0, "Smuggler's Copter");
    lots(&mut t, P0);
    let c = t.hand(P0, "Behind the Mask");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, c).target(mech).go();
    t.resolve_all();
    assert!(t.obj_now(mech).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.pt(mech), (4, 3));
    assert_eq!(t.stack_len(), 0, "no crewed trigger");
    let copter = t.named_on_battlefield("Smuggler's Copter")[0];
    assert!(!t.obj_now(copter).is(mtg_engine::types::CardType::Creature));
}

#[test]
fn blade_of_the_bloodchief_any_creature_and_its_last_attachment() {
    cr!("603.10a", "603.2", "608.2h");
    ruling!(
        "Blade of the Bloodchief",
        "Blade of the Bloodchief's ability triggers regardless of who controlled the creature and whose graveyard it was put into."
    );
    ruling!(
        "Blade of the Bloodchief",
        "If Blade of the Bloodchief leaves the battlefield before its triggered ability resolves, the counters are put on the creature it was attached to at the time it left the battlefield. If it wasn't attached to a creature at that time, nothing gets the counters."
    );
    supported("Blade of the Bloodchief");
    // An opponent's creature dies; the Blade leaves before the trigger resolves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Blade of the Bloodchief");
    t.g.attach(blade, obj(bears));
    let theirs = t.battlefield(P1, "Savannah Lions");
    kill(&mut t, theirs);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, blade);
    t.resolve_all();
    assert_eq!(plus(&t, bears), 1);
    // Unattached when it left: nothing gets them.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Blade of the Bloodchief");
    t.g.attach(blade, obj(bears));
    let theirs = t.battlefield(P1, "Savannah Lions");
    kill(&mut t, theirs);
    t.g.unattach(blade);
    t.g.recompute();
    kill(&mut t, blade);
    t.resolve_all();
    assert_eq!(plus(&t, bears), 0);
}

#[test]
fn invasive_maneuvers_spacecraft_checked_on_resolution() {
    cr!("608.2c");
    ruling!(
        "Invasive Maneuvers",
        "Check whether you control a Spacecraft at the time Invasive Maneuvers resolves to determine how much damage it deals."
    );
    supported("Invasive Maneuvers");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P0, "Invasive Maneuvers", &[obj(wurm)]);
    t.battlefield(P0, "Wedgelight Rammer");
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "5 damage");
}

#[test]
fn stitch_together_threshold_checked_as_it_starts_resolving() {
    cr!("608.2c", "207.2c");
    ruling!(
        "Stitch Together",
        "Checks the number of cards in your graveyard when it starts resolving. You may drop below seven cards during resolution, but that won't stop it from resolving."
    );
    supported("Stitch Together");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    for _ in 0..6 {
        t.graveyard(P0, "Island");
    }
    cast_at(&mut t, P0, "Stitch Together", &[obj(bears)]);
    t.resolve_all();
    assert!(named_here(&t, "Grizzly Bears"));
}

#[test]
fn paragon_of_modernity_colorless_isnt_a_color() {
    cr!("105.1", "105.2c", "106.1b");
    ruling!(
        "Paragon of Modernity",
        "Colorless is not a color. Similarly, colorless mana isn't counted when determining the number of colors that were spent on an ability."
    );
    supported("Paragon of Modernity");
    for (types, counter) in [
        ([ManaType::W, ManaType::U, ManaType::C], false),
        ([ManaType::W, ManaType::U, ManaType::B], true),
    ] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Paragon of Modernity");
        let (pw, th) = t.pt(p);
        for ty in types {
            mana(&mut t, P0, ty, 1);
        }
        t.activate(P0, p, 0, &[]).expect("activated");
        t.resolve_all();
        assert_eq!(plus(&t, p) == 1, counter, "{types:?}");
        // Either way it's +1/+1 (until end of turn, or a counter instead).
        assert_eq!(t.pt(p), (pw + 1, th + 1));
    }
}

#[test]
fn triumphs_two_planeswalkers_are_no_better_than_one() {
    cr!("608.2c");
    ruling!("Chandra's Triumph", "Controlling more than one planeswalker of the appropriate type won’t have any more of a benefit than just controlling one.");
    ruling!("Gideon's Triumph", "Controlling more than one planeswalker of the appropriate type won’t have any more of a benefit than just controlling one.");
    ruling!("Jace's Triumph", "Controlling more than one planeswalker of the appropriate type won’t have any more of a benefit than just controlling one.");
    ruling!("Nissa's Triumph", "Controlling more than one planeswalker of the appropriate type won’t have any more of a benefit than just controlling one.");
    supported("Chandra's Triumph");
    supported("Jace's Triumph");
    supported("Nissa's Triumph");
    // Chandra's Triumph: 5 damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chandra, Torch of Defiance");
    t.battlefield(P0, "Chandra Nalaar");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P0, "Chandra's Triumph", &[obj(wurm)]);
    t.resolve_all();
    let dealt: Vec<u32> = crate::r_p194_common::damage_events(&t)
        .iter()
        .map(|d| d.2)
        .collect();
    assert_eq!(dealt, vec![5]);
    // Jace's Triumph: three cards.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace Beleren");
    t.battlefield(P0, "Jace, Memory Adept");
    let hand = t.hand_size(P0);
    cast_at(&mut t, P0, "Jace's Triumph", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Nissa's Triumph: up to three lands.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nissa, Who Shakes the World");
    t.battlefield(P0, "Nissa Revane");
    let lands: Vec<_> = (0..4).map(|_| t.library_top(P0, "Forest")).collect();
    let m = mark(&t);
    t.answer_choose_moved(P0, &lands[..3].iter().map(|l| obj(*l)).collect::<Vec<_>>());
    cast_at(&mut t, P0, "Nissa's Triumph", &[]);
    t.resolve_all();
    let maxes: Vec<u32> = t.asked()[m..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes.first(), Some(&3));
    assert_eq!(t.hand_size(P0), 3);
    // Gideon's Triumph: two creatures.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gideon, Ally of Zendikar");
    t.battlefield(P0, "Gideon Blackblade");
    let ats: Vec<_> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(
        &mut t,
        &ats.iter().map(|a| (*a, pl(P0))).collect::<Vec<_>>(),
    );
    cast_at(&mut t, P0, "Gideon's Triumph", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(creatures(&t, P1).len(), 1);
}

#[test]
fn dark_temper_destroys_without_dealing_damage() {
    cr!("614.15", "608.2c");
    ruling!(
        "Dark Temper",
        "Dark Temper has a self-replacement effect, which applies before other prevention or replacement effects would. If you control a black permanent as Dark Temper resolves, it doesn’t deal damage at all (so damage prevention or redirection effects would have no effect); it simply destroys the targeted creature."
    );
    supported("Dark Temper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P1, "Mending Hands", &[obj(wurm)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Dark Temper", &[obj(wurm)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(damage_events(&t), 0);
    // Without a black permanent: 2 damage, prevented by the shield.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P1, "Mending Hands", &[obj(wurm)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Dark Temper", &[obj(wurm)]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn the_necrobloom_dredge_any_draw_one_dredge_per_draw() {
    cr!("702.52a", "702.52b", "121.6");
    ruling!(
        "The Necrobloom",
        "Dredge can replace any card draw, not just the one during your draw step. One card draw can't be replaced by multiple dredge abilities."
    );
    supported("The Necrobloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Necrobloom");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Mountain");
    let lib = t.library_size(P0);
    let hand = t.hand_size(P0);
    // One draw from an effect in the main phase: dredge once (a land returns, two
    // cards milled), the other land stays.
    t.answer(P0, DecisionKind::Replacement, Answer::Index(0));
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.library_size(P0), lib - 2);
    let lands_in_hand = t
        .g
        .player(P0)
        .hand
        .iter()
        .filter(|c| t.g.obj(**c).is(mtg_engine::types::CardType::Land))
        .count();
    assert_eq!(lands_in_hand, 1);
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn emiel_triggers_for_creatures_it_flickers() {
    cr!("603.6a", "400.7");
    ruling!(
        "Emiel the Blessed",
        "Emiel's second ability triggers whenever any creature other than itself enters the battlefield under your control, including those returned by its first ability."
    );
    supported("Emiel the Blessed");
    let mut t = TestGame::new(2);
    let emiel = t.battlefield(P0, "Emiel the Blessed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    act(&mut t, P0, emiel, 0, &[obj(bears)]);
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(b.len(), 1);
    assert_eq!(plus(&t, b[0]), 1);
    assert_eq!(plus(&t, emiel), 0);
}

#[test]
fn fblthp_shuffles_before_the_spell_even_if_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Fblthp, the Lost",
        "Fblthp's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Fblthp, the Lost");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fblthp, the Lost");
    let shock = cast_at(&mut t, P1, "Shock", &[obj(f)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_at(&mut t, P0, "Counterspell", &[obj(shock)]);
    t.resolve();
    assert!(!on_stack(&t, shock));
    t.resolve_all();
    assert!(!t.on_battlefield(f));
    assert_eq!(t.zone(t.g.current(f)), Zone::Library(P0));
}

#[test]
fn fblthp_must_be_on_the_battlefield_to_trigger() {
    cr!("603.2", "113.6");
    ruling!(
        "Fblthp, the Lost",
        "Fblthp's last ability triggers only if it's on the battlefield when it becomes the target of a spell."
    );
    let mut t = TestGame::new(2);
    let f = t.graveyard(P0, "Fblthp, the Lost");
    cast_at(&mut t, P0, "Raise Dead", &[obj(f)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Fblthp, the Lost"));
}

#[test]
fn adipose_offspring_uses_the_sacrificed_creatures_last_toughness() {
    cr!("702.119a", "608.2h");
    ruling!(
        "Adipose Offspring",
        "For Adipose Offspring's last ability, use the toughness of the sacrificed creature as it existed when it was last on the battlefield, which may be different from its toughness in the graveyard."
    );
    supported("Adipose Offspring");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    lots(&mut t, P0);
    let a = t.hand(P0, "Adipose Offspring");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, a)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    t.resolve_all();
    // Five Alien tokens (Giant Growth made the Bears 5/5).
    assert_eq!(tokens(&t, P0).len(), 5);
}

#[test]
fn golden_demise_and_pride_of_conquerors_lock_in_on_resolution() {
    cr!("611.2c", "702.131b", "608.2c");
    ruling!(
        "Golden Demise",
        "Golden Demise affects only creatures that are on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won’t get -2/-2. Similarly, if you have the city’s blessing, creatures that come under your opponents’ control later in the turn won’t get -2/-2."
    );
    ruling!(
        "Pride of Conquerors",
        "How Pride of Conquerors affects your creatures is determined at the time it resolves. If you don’t get the city’s blessing until later in the turn, your creatures still only get +1/+1."
    );
    supported("Golden Demise");
    supported("Pride of Conquerors");
    // Golden Demise without the blessing: a creature entering later is unaffected.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P0, "Golden Demise", &[]);
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    // With the blessing: P0's own creature, taken by P1 later, doesn't get -2/-2.
    let mut t = TestGame::new(2);
    t.g.players[0].has_citys_blessing = true;
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    cast_at(&mut t, P0, "Golden Demise", &[]);
    t.resolve_all();
    assert_eq!(t.pt(mine), (3, 3));
    assert_eq!(t.pt(theirs), (4, 2));
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Act of Treason", &[obj(mine)]);
    t.resolve_all();
    assert_eq!(ctl(&t, mine), P1);
    assert_eq!(t.pt(mine), (3, 3));
    // Pride of Conquerors: the blessing later doesn't upgrade it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Pride of Conquerors", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    t.g.players[0].has_citys_blessing = true;
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn haunting_voyage_chooses_on_resolution_without_targeting() {
    cr!("608.2c", "115.1");
    ruling!(
        "Haunting Voyage",
        "Haunting Voyage doesn't target any creature cards. You choose the creature type as Haunting Voyage resolves, then you choose up to two creature cards of that type to return (or all of them if Haunting Voyage was foretold)."
    );
    supported("Haunting Voyage");
    let mut t = TestGame::new(2);
    let z: Vec<_> = (0..3).map(|_| t.graveyard(P0, "Walking Corpse")).collect();
    t.graveyard(P0, "Grizzly Bears");
    let seen = crate::r_p198_common::choose_words(&mut t, P0, &["Zombie"]);
    let m = mark(&t);
    t.answer_choose_moved(P0, &[obj(z[0]), obj(z[1])]);
    cast_at(&mut t, P0, "Haunting Voyage", &[]);
    assert!(seen.lock().unwrap().is_empty(), "no type chosen as it's cast");
    t.resolve_all();
    assert!(!seen.lock().unwrap().is_empty(), "the type was chosen on resolution");
    assert!(target_choices(&t, P0, m, "").is_empty(), "no targets");
    assert_eq!(t.named_on_battlefield("Walking Corpse").len(), 2);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn anax_triggers_for_each_creature_dying_with_it() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Anax, Hardened in the Forge",
        "If Anax dies at the same time as another nontoken creature you control, its ability triggers for each of them."
    );
    supported("Anax, Hardened in the Forge");
    let mut t = TestGame::new(2);
    let ids = [
        t.battlefield(P0, "Anax, Hardened in the Forge"),
        t.battlefield(P0, "Grizzly Bears"),
        t.battlefield(P0, "Savannah Lions"),
    ];
    crate::r_p146_common::destroy_together(&mut t, &ids);
    assert!(t.in_graveyard(P0, "Anax, Hardened in the Forge"));
    assert_eq!(t.stack_len(), 3);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Satyr").len(), 3);
}
