//! Rulings batch P026 — permanents that become copies (CR 707.2, 707.3, 613.1a): they copy
//! the printed values of what they copy plus the copy effects applied to it, never its
//! counters, status or other effects; the creature to copy is chosen as the effect
//! resolves, so it isn't targeted and may be one of the targets or the copier itself.

use crate::r_p026_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p`'s Clone copying a Hill Giant, then dressed up (tapped, equipped, counters, +3/+3,
/// green-blue). Anything copying it should be a red 3/3 Hill Giant.
fn dressed_clone(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let giant = t.battlefield(p, "Hill Giant");
    let clone = clone_of(t, p, giant);
    dress(t, p, clone);
    assert_eq!(t.pt(clone), (10, 8));
    clone
}

/// Asserts `id` copies the printed Hill Giant (a red 3/3), keeping its own untapped
/// state, with no counters or attachments.
fn is_printed_giant(t: &TestGame, id: ObjectId) {
    assert_printed_giant(t, id, Some((3, 3)), false);
}

#[test]
fn becoming_a_copy_copies_printed_values_plus_copy_effects() {
    cr!("707.2", "707.3", "613.1a");
    ruling!(
        "Cytoshape",
        "The creature copies the printed values of the chosen creature, plus any copy effects that have been applied to it. It won’t copy counters on that creature. It won’t copy effects that have changed the creature’s power, toughness, types, color, or so on."
    );
    ruling!(
        "Polymorphous Rush",
        "The target creatures copy the printed values of the chosen creature, plus any copy effects that have been applied to it. They won't copy counters on that creature or effects that have changed the creature's power, toughness, types, color, or so on."
    );
    ruling!(
        "Shapesharer",
        "The targeted Shapeshifter copies the printed values of the targeted creature, plus any copy effects that have been applied to it. It won’t copy counters on that creature. It won’t copy effects that have changed the creature’s power, toughness, types, color, and so on."
    );
    // Cytoshape: "Choose a nonlegendary creature on the battlefield. Target creature
    // becomes a copy of that creature until end of turn."
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    let clone = dressed_clone(&mut t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{1}{G}{U}");
    let c = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(clone)]);
    t.cast(P0, c).target(obj(bears)).go();
    t.resolve_all();
    is_printed_giant(&t, bears);
    // Polymorphous Rush, on two of P0's creatures.
    supported("Polymorphous Rush");
    let mut t = TestGame::new(2);
    let clone = dressed_clone(&mut t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    lands_for(&mut t, P0, "{2}{U}{1}{U}");
    let c = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(clone)]);
    t.cast(P0, c).targets(&[obj(bears), obj(elves)]).go();
    t.resolve_all();
    is_printed_giant(&t, bears);
    is_printed_giant(&t, elves);
    // Shapesharer: "{2}{U}: Target Shapeshifter becomes a copy of target creature until
    // your next turn." It targets itself (a changeling) and the Clone.
    supported("Shapesharer");
    let mut t = TestGame::new(2);
    let clone = dressed_clone(&mut t, P1);
    let sharer = t.battlefield(P0, "Shapesharer");
    lands_for(&mut t, P0, "{2}{U}");
    t.activate(P0, sharer, 0, &[obj(sharer), obj(clone)])
        .expect("activation");
    t.resolve_all();
    is_printed_giant(&t, sharer);
}

#[test]
fn permeating_mass_copy_copies_only_its_printed_values() {
    cr!("707.2");
    ruling!(
        "Permeating Mass",
        "The damaged creature copies the printed values of Permeating Mass. It won’t copy counters on Permeating Mass or effects that have changed its power, toughness, types, color, or so on. Notably, this means that a creature that becomes a copy of Permeating Mass has the ability to permeate other creatures as well."
    );
    // Permeating Mass (a green 1/3) has a +1/+1 counter and has been made blue too; it
    // attacks and deals 2 combat damage to the blocking Hill Giant.
    supported("Permeating Mass");
    let mut t = TestGame::new(2);
    let mass = t.battlefield(P0, "Permeating Mass");
    plus1(&mut t, mass, 1);
    modify(
        &mut t,
        mass,
        vec![mtg_engine::ability::Modification::SetColors(
            ColorSet::single(Color::Blue),
        )],
    );
    assert_eq!(t.pt(mass), (2, 4));
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(mass, Entity::Player(P1))], &[(giant, mass)]);
    assert!(t.on_battlefield(giant));
    assert_eq!(name_of(&t, giant), "Permeating Mass");
    assert_eq!(
        t.obj_now(giant).chars.colors,
        ColorSet::single(Color::Green)
    );
    assert_eq!(t.pt(giant), (1, 3));
    assert_eq!(t.counters(giant, counters::PLUS1), 0);
    // It can permeate other creatures: it attacks next turn and is blocked by Grizzly
    // Bears, which becomes a Permeating Mass too.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(abilities_with(&t, giant, "becomes a copy of"), 1);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[(bears, giant)]);
    assert_eq!(name_of(&t, bears), "Permeating Mass");
}

#[test]
fn fleeting_reflection_copies_only_the_printed_creature() {
    cr!("707.2", "613.1a");
    ruling!(
        "Fleeting Reflection",
        "The first target creature copies exactly what was printed on the other target creature and nothing else (unless that creature is copying something else or is a token; see below)."
    );
    // "Target creature you control gains hexproof until end of turn. Untap that creature.
    // Until end of turn, it becomes a copy of up to one other target creature."
    supported("Fleeting Reflection");
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{1}{U}");
    let c = t.hand(P0, "Fleeting Reflection");
    t.cast(P0, c).target(obj(bears)).target(obj(giant)).go();
    t.resolve_all();
    is_printed_giant(&t, bears);
    assert!(has_kw(&t, bears, KeywordKind::Hexproof));
}

#[test]
fn mystic_reflection_entering_permanents_copy_only_the_printed_creature() {
    cr!("707.2", "614.1c", "707.5");
    ruling!(
        "Mystic Reflection",
        "The entering permanents don’t copy whether the target creature is tapped or untapped, whether it has any counters on it, whether it has any Auras and/or Equipment attached to it, or any non-copy effects that changed its power, toughness, types, color, and so on."
    );
    supported("Mystic Reflection");
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P1);
    lands_for(&mut t, P0, "{1}{U}");
    let c = t.hand(P0, "Mystic Reflection");
    t.cast(P0, c).target(obj(giant)).go();
    t.resolve_all();
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    let bears = t.g.current(bears);
    assert_printed_giant(&t, bears, Some((3, 3)), true);
}

#[test]
fn march_from_velis_vel_lands_copy_only_the_printed_creature() {
    cr!("707.2", "613.1a");
    ruling!(
        "March from Velis Vel",
        "The lands copy exactly what was printed on the original creature and nothing else (unless that permanent is copying something else; see below)."
    );
    // "Choose a nonbasic land type. Each land you control of that type becomes a copy of
    // target creature you control until end of turn and gains haste until end of turn."
    supported("March from Velis Vel");
    let mut t = TestGame::new(2);
    let giant = dressed_giant(&mut t, P0);
    let gate = t.battlefield(P0, "Azorius Guildgate");
    lands_for(&mut t, P0, "{2}{U}");
    let m = t.hand(P0, "March from Velis Vel");
    t.cast(P0, m).target(obj(giant)).go();
    // Choose "Gate".
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(2),
    );
    t.resolve_all();
    assert_printed_giant(&t, gate, Some((3, 3)), true);
    assert!(has_kw(&t, gate, KeywordKind::Haste));
}

#[test]
fn the_animus_copies_only_the_exiled_card() {
    cr!("707.2", "400.7");
    ruling!(
        "The Animus",
        "The target legendary creature copies exactly what was printed on the card in exile and nothing else. It doesn’t copy any information about the object the card was before it was exiled."
    );
    // "At the beginning of your end step, exile up to one target legendary creature card
    // from a graveyard with a memory counter on it. / {T}: Until your next turn, target
    // legendary creature you control becomes a copy of target creature card in exile with
    // a memory counter on it." Isamaru (a legendary white 2/2) was dressed up, and was a
    // copy of Grizzly Bears, when it died.
    supported("The Animus");
    let mut t = TestGame::new(2);
    let animus = t.battlefield(P0, "The Animus");
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    dress(&mut t, P1, isamaru);
    disguise_as_bears(&mut t, isamaru);
    kill(&mut t, isamaru);
    let card = t.g.current(isamaru);
    t.set_step(P0, Step::PostcombatMain);
    t.answer_targets(P0, &[obj(card)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let exiled = t.g.current(isamaru);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, "memory"), 1);
    // On P0's next turn, P0's legendary creature becomes a copy of it.
    let kiki = t.battlefield(P0, "Kiki-Jiki, Mirror Breaker");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, animus, 0, &[obj(kiki), obj(exiled)])
        .expect("activation");
    t.resolve_all();
    assert_printed(
        &t,
        kiki,
        "Isamaru, Hound of Konda",
        ColorSet::single(Color::White),
        Some((2, 2)),
        false,
    );
    assert!(legendary(&t, kiki));
}

#[test]
fn polymorphous_rush_chooses_the_copied_creature_on_resolution() {
    cr!("608.2d", "115.1", "707.2");
    ruling!(
        "Polymorphous Rush",
        "The creature that's copied isn't targeted by Polymorphous Rush. You choose that creature as the spell resolves. It can be one of the targeted creatures. A creature can become a copy of itself this way, though this usually won't have any noticeable effect."
    );
    supported("Polymorphous Rush");
    // The opponent's hexproof Gladecover Scout can be chosen.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{U}");
    let c = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(scout)]);
    t.cast(P0, c).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Gladecover Scout");
    // It's chosen as the spell resolves: Serra Angel, which entered after the spell was
    // cast, can be chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for(&mut t, P0, "{2}{U}");
    let c = t.hand(P0, "Polymorphous Rush");
    t.cast(P0, c).target(obj(bears)).go();
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_choose(P0, &[obj(angel)]);
    t.resolve_all();
    assert_eq!(name_of(&t, bears), "Serra Angel");
    // One of the targets is chosen: both become Grizzly Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    lands_for(&mut t, P0, "{2}{U}{1}{U}");
    let c = t.hand(P0, "Polymorphous Rush");
    t.answer_choose(P0, &[obj(bears)]);
    t.cast(P0, c).targets(&[obj(bears), obj(giant)]).go();
    t.resolve_all();
    assert_eq!(name_of(&t, giant), "Grizzly Bears");
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(name_of(&t, bears), "Grizzly Bears");
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn shapesharer_can_make_a_creature_a_copy_of_itself() {
    cr!("115.3", "707.2");
    ruling!(
        "Shapesharer",
        "This ability can cause a creature to become a copy of itself. This will usually have no visible effect."
    );
    supported("Shapesharer");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let other = t.battlefield(P0, "Shapesharer");
    lands_for(&mut t, P0, "{2}{U}");
    // The other Shapesharer targets the first one twice.
    t.activate(P0, other, 0, &[obj(sharer), obj(sharer)])
        .expect("the same creature can be both targets");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(name_of(&t, sharer), "Shapesharer");
    assert_eq!(t.pt(sharer), (1, 1));
    assert_eq!(
        abilities_with(&t, sharer, "becomes a copy of target creature"),
        1
    );
}

#[test]
fn sakashimas_will_second_mode_chooses_on_resolution() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Sakashima's Will",
        "The second mode doesn't target any creature. You choose the creature that will be copied as Sakashima's Will resolves. If you chose both modes, the creature you gained control of because of the first mode can be chosen for your other creatures to copy."
    );
    // "Choose one. If you control a commander as you cast this spell, you may choose both
    // instead. • Target opponent chooses a creature they control. You gain control of it.
    // • Choose a creature you control. Each other creature you control becomes a copy of
    // that creature until end of turn."
    supported("Sakashima's Will");
    let mut t = crate::r_s13_common::commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmdr = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[0]
        .commander_names
        .push("Isamaru, Hound of Konda".into());
    let bears = t.battlefield(P0, "Grizzly Bears");
    // The opponent's only creature has hexproof.
    let scout = t.battlefield(P1, "Gladecover Scout");
    lands_for(&mut t, P0, "{3}{U}");
    let c = t.hand(P0, "Sakashima's Will");
    t.answer_choose(P1, &[obj(scout)]);
    t.answer_choose(P0, &[obj(scout)]);
    t.cast(P0, c).modes(&[0, 1]).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(scout).controller, P0);
    assert_eq!(name_of(&t, bears), "Gladecover Scout");
    assert_eq!(name_of(&t, cmdr), "Gladecover Scout");
    assert_eq!(name_of(&t, scout), "Gladecover Scout");
}

#[test]
fn the_myriad_pools_target_copies_the_spells_casting_choices() {
    cr!("707.2", "718.2a", "718.3b", "613.1a");
    ruling!(
        "The Everflowing Well // The Myriad Pools",
        "The target permanent copies what is printed on the original card, plus any choices that were made when casting the spell."
    );
    // Combat Thresher (an artifact Construct 3/3) cast for its prototype {2}{W} — 1/1:
    // the Hill Giant becomes a copy of the prototyped spell, a white 1/1.
    supported("The Everflowing Well // The Myriad Pools");
    supported("Combat Thresher");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let well = t.battlefield(P0, "The Everflowing Well // The Myriad Pools");
    assert!(mtg_engine::dfc::transform(&mut t.g, well));
    t.g.recompute();
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 1);
    let thresher = t.hand(P0, "Combat Thresher");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, thresher)
        .method(mtg_engine::object::CastMethod::Keyword(
            KeywordKind::Prototype,
        ))
        .go();
    t.settle();
    t.resolve();
    t.g.recompute();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(name_of(&t, giant), "Combat Thresher");
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(
        t.obj_now(giant).chars.colors,
        ColorSet::single(Color::White)
    );
    assert_eq!(crate::r_s26_common::mv(&mut t, giant), 3);
    assert!(has_kw(&t, giant, KeywordKind::DoubleStrike));
}

#[test]
fn exiling_a_graveyard_card_with_counters_puts_them_on_it() {
    cr!("122.2", "400.7");
    // Doom's Time Platform: "Whenever you attack, exile target nonland card from your
    // graveyard with two time counters on it. If it doesn't have suspend, it gains
    // suspend." (as The Animus's "exile ... from a graveyard with a memory counter on it"
    // above: cards in graveyards have no counters, CR 122.2).
    supported("Doom's Time Platform");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doom's Time Platform");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(card)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, counters::TIME), 2);
    assert!(has_kw(&t, exiled, KeywordKind::Suspend));
    // Altaïr Ibn-La'Ahad: "Whenever Altaïr attacks, exile up to one target Assassin
    // creature card from your graveyard with a memory counter on it. Then for each
    // creature card you own in exile with a memory counter on it, create a tapped and
    // attacking token that's a copy of it." Royal Assassin is a Human Assassin.
    supported("Altaïr Ibn-La'Ahad");
    let mut t = TestGame::new(2);
    let altair = t.battlefield(P0, "Altaïr Ibn-La'Ahad");
    let card = t.graveyard(P0, "Royal Assassin");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(card)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(altair, Entity::Player(P1))], &[]);
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, "memory"), 1);
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Royal Assassin");
}
