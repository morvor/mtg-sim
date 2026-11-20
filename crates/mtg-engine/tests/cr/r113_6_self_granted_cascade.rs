//! A card's own "This spell has cascade as long as ..." (Bloodbraid Marauder's delirium
//! ability) functions only on the stack. CR 113.6: "Abilities of an instant or sorcery
//! spell usually function only while that object is on the stack. Abilities of all other
//! objects usually function only while that object is on the battlefield. The exceptions
//! are as follows:" / 113.6b "An ability that states which zones it functions in functions
//! only from those zones." / 112.1 "A spell is a card on the stack. ..." / 702.85a
//! "Cascade is a triggered ability that functions only while the spell with cascade is on
//! the stack. ..." The ability speaks of the card as a spell, so it gives the card cascade
//! only on the stack, as it's cast (the card's ruling: "The delirium ability will check
//! your graveyard at the moment you finish casting Bloodbraid Marauder to determine if it
//! has cascade."). Before small-fixes-6 it functioned in every zone: Bloodbraid Marauder
//! had cascade in a hand, a library, a graveyard, exile and on the battlefield.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Four card types in `p`'s graveyard (land, creature, instant, artifact): delirium.
fn delirium(t: &mut TestGame, p: PlayerId) {
    for n in ["Forest", "Grizzly Bears", "Lightning Bolt", "Ornithopter"] {
        t.graveyard(p, n);
    }
}

#[test]
fn a_self_granted_cascade_functions_only_on_the_stack() {
    cr!("113.6", "113.6b", "702.85a");
    interp!("self-granted-cascade-functions-on-the-stack");
    ruling!(
        "Bloodbraid Marauder",
        "The delirium ability will check your graveyard at the moment you finish casting Bloodbraid Marauder to determine if it has cascade."
    );
    // With delirium, Bloodbraid Marauder doesn't have cascade outside the stack.
    let mut t = TestGame::new(2);
    delirium(&mut t, P0);
    let cards = [
        t.hand(P0, "Bloodbraid Marauder"),
        t.library_top(P0, "Bloodbraid Marauder"),
        t.graveyard(P0, "Bloodbraid Marauder"),
        t.exile(P0, "Bloodbraid Marauder"),
        t.battlefield(P0, "Bloodbraid Marauder"),
    ];
    t.g.recompute();
    for c in cards {
        assert!(
            !t.obj(c).chars.has_keyword(KeywordKind::Cascade),
            "in {:?}",
            t.zone(c)
        );
    }
    // Cast, it has cascade and cascades into Ornithopter.
    t.library_top(P0, "Ornithopter");
    t.lands(P0, "Mountain", 3);
    let spell = t.cast(P0, cards[0]).go();
    t.settle();
    t.g.recompute();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(t.obj(spell).chars.has_keyword(KeywordKind::Cascade));
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    // On the battlefield it no longer has cascade.
    let m = t.g.current(spell);
    assert_eq!(t.zone(m), Zone::Battlefield);
    assert!(!t.obj(m).chars.has_keyword(KeywordKind::Cascade));
}

/// The abilities `text` compiles to on a creature card, or `None` if any of it isn't
/// supported.
fn compiled(text: &str) -> Option<Vec<mtg_engine::ability::Ability>> {
    use mtg_engine::oracle::{self, CompileContext};
    let tl = mtg_engine::types::TypeLine::parse("Creature — Human Berserker");
    let ctx = CompileContext {
        card_name: "Probe",
        full_name: "Probe",
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: Some("3"),
        toughness: Some("2"),
    };
    let c = oracle::compile(text, &ctx);
    c.unsupported.is_empty().then_some(c.abilities)
}

#[test]
fn a_cards_own_keyword_is_compiled_only_where_it_can_function() {
    // The zone the compiler gives a creature card's own "has [keyword] as long as ...":
    // the stack for cascade and flash (CR 702.85a; 113.6e), the battlefield for keywords
    // that function there (CR 113.6). A grant of another keyword that doesn't function on
    // the battlefield, or of cascade or flash with another keyword, isn't accepted: no card
    // has one, and it would do nothing on the battlefield (in review: "This spell has storm
    // as long as ..." compiled to an ability of the permanent).
    interp!("self-granted-cascade-functions-on-the-stack");
    use mtg_engine::ability::{AbilityKind, FunctionZone};
    let zone = |kw: &str| -> Option<FunctionZone> {
        let abilities = compiled(&format!(
            "This spell has {kw} as long as you control a Wizard."
        ))?;
        match abilities.as_slice() {
            [a] => match &a.kind {
                AbilityKind::Static(s) => Some(s.zone),
                other => panic!("{kw}: {other:?}"),
            },
            v => panic!("{kw}: {v:?}"),
        }
    };
    assert_eq!(zone("cascade"), Some(FunctionZone::Stack));
    assert_eq!(zone("flash"), Some(FunctionZone::Stack));
    assert_eq!(zone("flash and cascade"), Some(FunctionZone::Stack));
    assert_eq!(zone("flying"), Some(FunctionZone::Battlefield));
    assert_eq!(
        zone("first strike and trample"),
        Some(FunctionZone::Battlefield)
    );
    for kw in [
        "storm",
        "split second",
        "replicate {1}",
        "casualty 2",
        "convoke",
        "cycling {2}",
        "flash and haste",
        "cascade and trample",
    ] {
        assert_eq!(zone(kw), None, "{kw}");
    }
}
