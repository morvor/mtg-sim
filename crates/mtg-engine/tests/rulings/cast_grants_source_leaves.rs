//! Abilities static abilities grant to spells as they're cast, when the source leaves
//! while the spell is cast or afterwards.
//!
//! "[Spells] you cast gain [ability] as you cast them" (Ashling, the Limitless; Zinnia,
//! Valley's Voice) is a one-shot effect applied as the spell is put on the stack (CR
//! 610.5, 601.2a): the spell keeps the ability. "[Spells] you cast have [ability]" (Wort,
//! Party Thrasher, Caetus, Eirdu, Herigast) is a continuous effect from a static ability
//! (CR 611.3a-b): once its source is gone, the spell no longer has it.

use crate::r_s01_common::supported;
use crate::r_s17_common::enter_transformed;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const ASHLING: &str = "Ashling, the Limitless";
const ZINNIA: &str = "Zinnia, Valley's Voice";

fn tap(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = true;
}

/// P0 casts Air Elemental for Ashling's evoke {4} with two Islands and Ashnod's Altar,
/// whose only creature to sacrifice is Ashling. Returns Air Elemental.
fn evoke_sacrificing_ashling(t: &mut TestGame) -> ObjectId {
    let ashling = t.battlefield(P0, ASHLING);
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Island", 2);
    let air = t.hand(P0, "Air Elemental");
    t.cast(P0, air)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    // Ashling was sacrificed to the Altar to pay {4}.
    assert_eq!(t.zone(t.g.current(ashling)), Zone::Graveyard(P0));
    air
}

#[test]
fn ashling_sacrificed_while_an_elemental_is_cast_for_evoke_still_evokes_it() {
    cr!("610.5", "601.2a", "702.74a");
    supported(ASHLING);
    let mut t = TestGame::new(2);
    let air = evoke_sacrificing_ashling(&mut t);
    // The spell gained evoke as it was put on the stack and keeps it.
    assert!(t.obj_now(air).has_keyword(KeywordKind::Evoke));
    t.resolve_all();
    // Evoke's "When this permanent enters, if its evoke cost was paid, its controller
    // sacrifices it." (Ashling's own trigger may make token copies of what's sacrificed.)
    assert!(t
        .g
        .player(P0)
        .graveyard
        .iter()
        .any(|o| t.obj(*o).chars.name == "Air Elemental"));
    assert!(t
        .named_on_battlefield("Air Elemental")
        .iter()
        .all(|o| t.obj(*o).is_token()));
}

#[test]
fn ashling_leaving_after_the_cast_still_evokes_the_permanent() {
    cr!("610.5", "400.7b", "702.74a");
    ruling!(
        "Ashling, the Limitless",
        "If Ashling leaves the battlefield after you've cast an Elemental permanent spell and paid its evoke cost but before that spell resolves, the permanent that spell becomes will still have evoke when it enters."
    );
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, ASHLING);
    t.lands(P0, "Island", 4);
    let air = t.hand(P0, "Air Elemental");
    let spell = t
        .cast(P0, air)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    t.g.move_object(ashling, Zone::Exile, MoveCause::Effect, None);
    t.recompute();
    assert!(t.obj_now(spell).has_keyword(KeywordKind::Evoke));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Air Elemental"));
    assert!(t.named_on_battlefield("Air Elemental").is_empty());
}

#[test]
fn zinnia_sacrificed_while_a_creature_is_cast_with_offspring_still_makes_the_token() {
    cr!("610.5", "601.2a", "702.175a", "400.7b");
    supported(ZINNIA);
    let mut t = TestGame::new(2);
    let zinnia = t.battlefield(P0, ZINNIA);
    t.battlefield(P0, "Ashnod's Altar");
    t.lands(P0, "Forest", 2);
    // Grizzly Bears {1}{G} plus offspring {2}: two Forests and Zinnia sacrificed to the
    // Altar.
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, bears).go();
    assert_eq!(t.zone(t.g.current(zinnia)), Zone::Graveyard(P0));
    t.resolve_all();
    let all = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(all.len(), 2);
    assert_eq!(all.iter().filter(|o| t.obj(**o).is_token()).count(), 1);
}

/// P0 controls Wort, the Raidmother and three Raging Goblins and casts Fling at P1,
/// sacrificing `sac` (Wort or the third Goblin) and tapping the other two Goblins for
/// conspire. Returns P1's life after everything resolves.
fn fling_with_conspire(sacrifice_wort: bool) -> i32 {
    let mut t = TestGame::new(2);
    let wort = t.battlefield(P0, "Wort, the Raidmother");
    let goblins: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Raging Goblin")).collect();
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    let sac = if sacrifice_wort { wort } else { goblins[2] };
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(
        P0,
        &[Entity::Object(goblins[0]), Entity::Object(goblins[1])],
    );
    t.answer_choose(P0, &[Entity::Object(sac)]);
    t.cast(P0, fling).target(P1).go();
    assert_eq!(t.zone(t.g.current(sac)), Zone::Graveyard(P0));
    assert!(t.obj_now(goblins[0]).tapped && t.obj_now(goblins[1]).tapped);
    t.resolve_all();
    t.life(P1)
}

#[test]
fn wort_sacrificed_to_flings_cost_gives_it_no_conspire() {
    cr!("611.3b", "601.2i", "702.78a");
    ruling!(
        "Wort, the Raidmother",
        "If you sacrifice Wort to pay that cost, that spell won't have conspire at the moment it becomes cast, so conspire won't trigger, even if you tapped two creatures."
    );
    supported("Wort, the Raidmother");
    // Wort (3/3) sacrificed: Fling deals 3 and isn't copied.
    assert_eq!(fling_with_conspire(true), 17);
    // A Raging Goblin (1/1) sacrificed: conspire copies Fling (the copy uses the same
    // sacrificed creature's power).
    assert_eq!(fling_with_conspire(false), 18);
}

/// Whether P0 can cast `spell` when the only ways to pay are convoking with `convoker`
/// and sacrificing creatures to Ashnod's Altar ({C}{C}), the `granter` (tapped) being the
/// one that makes the spell have convoke; plus one Wastes if `wastes`.
fn castable_with_altar(t: &mut TestGame, granter: ObjectId, spell: ObjectId, wastes: bool) -> bool {
    tap(t, granter);
    t.battlefield(P0, "Ashnod's Altar");
    if wastes {
        t.lands(P0, "Wastes", 1);
    }
    t.cast(P0, spell).try_go().is_ok()
}

#[test]
fn eirdu_sacrificed_for_mana_while_a_creature_spell_is_cast_takes_convoke_away() {
    cr!("611.3b", "601.2g", "601.2h", "702.51a");
    ruling!(
        "Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight",
        "If you sacrifice Eirdu while casting a creature spell (to activate a mana ability, for example), the spell won't have convoke when you pay its costs unless it has convoke some other way."
    );
    for wastes in [false, true] {
        let mut t = TestGame::new(2);
        let eirdu = t.battlefield(P0, "Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight");
        let convoker = t.battlefield(P0, "Grizzly Bears");
        // Grizzly Bears {1}{G}: the green Bears convoke {G}; {1} comes from the Wastes,
        // or from sacrificing Eirdu to the Altar, which takes convoke away.
        let bears = t.hand(P0, "Grizzly Bears");
        assert_eq!(castable_with_altar(&mut t, eirdu, bears, wastes), wastes);
        assert_eq!(t.obj_now(convoker).tapped, wastes);
        assert_eq!(t.zone(t.g.current(eirdu)), Zone::Battlefield);
    }
}

#[test]
fn caetus_sacrificed_for_mana_while_a_noncreature_spell_is_cast_takes_convoke_away() {
    cr!("611.3b", "601.2g", "601.2h", "702.51a");
    ruling!(
        "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        "If you sacrifice Caetus while casting a noncreature spell (say, to activate a mana ability), the spell won't have convoke when you pay its costs unless it has convoke some other way."
    );
    for wastes in [false, true] {
        let mut t = TestGame::new(2);
        let caetus = enter_transformed(
            &mut t,
            P0,
            "Invasion of Segovia // Caetus, Sea Tyrant of Segovia",
        );
        assert_eq!(
            t.obj_now(caetus).chars.name,
            "Caetus, Sea Tyrant of Segovia"
        );
        let goblin = t.battlefield(P0, "Raging Goblin");
        // Searing Spear {1}{R}: the red Goblin convokes {R}.
        let spear = t.hand(P0, "Searing Spear");
        tap(&mut t, caetus);
        t.battlefield(P0, "Ashnod's Altar");
        if wastes {
            t.lands(P0, "Wastes", 1);
        }
        let ok = t.cast(P0, spear).target(P1).try_go().is_ok();
        assert_eq!(ok, wastes);
        assert_eq!(t.obj_now(goblin).tapped, wastes);
        assert_eq!(t.zone(t.g.current(caetus)), Zone::Battlefield);
    }
}

/// Foretells `card` from `p`'s hand and returns it in exile.
fn foretell(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    t.g.players[p.idx()].mana_pool.add_type(ManaType::C, 2);
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(SpecialAction::Foretell { card }));
    t.g.flush_events();
    let new = t.g.current(card);
    assert_eq!(t.obj(new).zone, Zone::Exile);
    new
}

#[test]
fn party_thrasher_sacrificed_for_mana_while_a_spell_is_cast_from_exile_takes_convoke_away() {
    cr!("611.3b", "601.2g", "601.2h", "702.51a");
    ruling!(
        "Party Thrasher",
        "If you sacrifice Party Thrasher while casting a noncreature spell from exile (to activate a mana ability, for example), the spell won't have convoke when you pay its costs unless it has convoke some other way."
    );
    supported("Party Thrasher");
    for wastes in [false, true] {
        let mut t = TestGame::new(2);
        let thrasher = t.battlefield(P0, "Party Thrasher");
        let target = t.battlefield(P1, "Grizzly Bears");
        // Poison the Cup, foretold: "Destroy target creature." for {1}{B} from exile.
        let cup = t.hand(P0, "Poison the Cup");
        let foretold = foretell(&mut t, P0, cup);
        t.advance_to(P1, Step::Upkeep);
        // The black Walking Corpse convokes {B}.
        let corpse = t.battlefield(P0, "Walking Corpse");
        tap(&mut t, thrasher);
        t.battlefield(P0, "Ashnod's Altar");
        if wastes {
            t.lands(P0, "Wastes", 1);
        }
        let ok = t
            .cast(P0, foretold)
            .method(CastMethod::Keyword(KeywordKind::Foretell))
            .target(target)
            .try_go()
            .is_ok();
        assert_eq!(ok, wastes);
        assert_eq!(t.obj_now(corpse).tapped, wastes);
        assert_eq!(t.zone(t.g.current(thrasher)), Zone::Battlefield);
    }
}

#[test]
fn herigast_can_be_sacrificed_for_the_emerge_cost_it_grants() {
    cr!("601.2f", "702.119a");
    ruling!(
        "Herigast, Erupting Nullkite",
        "Once you begin to cast a spell using an emerge ability granted by Herigast, losing control of Herigast won't affect it. You can finish casting it as normal."
    );
    supported("Herigast, Erupting Nullkite");
    let mut t = TestGame::new(2);
    let herigast = t.battlefield(P0, "Herigast, Erupting Nullkite");
    t.lands(P0, "Forest", 2);
    // Craw Wurm {4}{G}{G}, its emerge cost reduced by Herigast's mana value 9: {G}{G}.
    let wurm = t.hand(P0, "Craw Wurm");
    t.answer_choose(P0, &[Entity::Object(herigast)]);
    t.cast(P0, wurm)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    assert!(t.in_graveyard(P0, "Herigast, Erupting Nullkite"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}
