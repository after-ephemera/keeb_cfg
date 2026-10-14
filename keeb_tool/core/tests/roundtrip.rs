use keeb_core::ferris::TargetBoard;
use keeb_core::{generate_main_py, parse_main_py, suggest_compact_layout};

/// The real config from this repo.
const REAL: &str = include_str!("../../../lily58/main.py");

#[test]
fn parses_real_config() {
    let km = parse_main_py(REAL).expect("should parse");
    assert_eq!(km.layers.len(), 8, "3 real + 5 placeholder layers");
    assert_eq!(km.key_count(), 58, "Lily58 has 58 keys");

    // Layer names come from the leading comment on each layer block.
    assert_eq!(km.layers[0].name.as_deref(), Some("qwerty"));
    assert_eq!(km.layers[1].name.as_deref(), Some("lower"));
    assert_eq!(km.layers[2].name.as_deref(), Some("raise"));

    // Base-layer spot checks (see the index map in ferris.rs).
    assert_eq!(km.layers[0].keys[0].raw, "KC.TAB");
    assert_eq!(km.layers[0].keys[13].raw, "KC.Q");
    assert_eq!(km.layers[0].keys[25].raw, "KC.A");

    // The complex tap-dance alias is preserved verbatim as a single token.
    assert_eq!(km.layers[0].keys[50].raw, "LOWER");
    assert!(km.layers[0].keys[50].is_alias);

    // Aliases from the preamble were collected.
    for a in ["LOWER", "RAISE", "BASE", "SKGUI", "LALT", "F", "J"] {
        assert!(km.aliases.contains(&a.to_string()), "missing alias {a}");
    }
}

#[test]
fn complex_expression_not_split_on_inner_commas() {
    let km = parse_main_py(REAL).unwrap();
    // Raise layer index 42/43 are KC.LCBR / KC.RCBR; index 12 (LALT) is an alias.
    // Verify a nested call token on the base layer survives whole:
    // index 51 = LCTLPP alias -> preamble defines KC.TD(KC.LCTL, KC.LCTL(KC.TAB)).
    let preamble = &km.preamble;
    assert!(preamble.contains("LCTLPP = KC.TD(KC.LCTL, KC.LCTL(KC.TAB))"));
}

#[test]
fn regenerates_parseable_config_with_same_keys() {
    let km = parse_main_py(REAL).unwrap();
    let regen = generate_main_py(&km);

    // Preamble + postamble preserved verbatim.
    assert!(regen.contains("modtap.tap_time = 200"));
    assert!(regen.contains("keyboard.go(hid_type=HIDModes.USB)"));

    // Re-parsing the generated file yields identical keys.
    let km2 = parse_main_py(&regen).expect("regenerated file must re-parse");
    assert_eq!(km2.layers.len(), km.layers.len());
    for (a, b) in km.layers.iter().zip(km2.layers.iter()) {
        let ra: Vec<_> = a.keys.iter().map(|k| &k.raw).collect();
        let rb: Vec<_> = b.keys.iter().map(|k| &k.raw).collect();
        assert_eq!(ra, rb, "keys must round-trip");
    }
}

#[test]
fn editing_a_key_changes_only_that_key() {
    let mut km = parse_main_py(REAL).unwrap();
    km.layers[0].keys[13] = keeb_core::Key::new("KC.SPC"); // Q -> Space
    let regen = generate_main_py(&km);
    let km2 = parse_main_py(&regen).unwrap();
    assert_eq!(km2.layers[0].keys[13].raw, "KC.SPC");
    assert_eq!(km2.layers[0].keys[14].raw, "KC.W"); // neighbor untouched
}

#[test]
fn ferris_suggestion_keeps_3x5_and_relocates_outer_column() {
    let km = parse_main_py(REAL).unwrap();
    let s = suggest_compact_layout(&km, TargetBoard::FerrisSweep);
    assert_eq!(s.total_keys, 34);
    assert_eq!(s.cols_per_half, 5);
    assert_eq!(s.left_rows.len(), 3);
    assert!(s.left_rows.iter().all(|r| r.len() == 5));
    assert!(s.right_rows.iter().all(|r| r.len() == 5));
    assert_eq!(s.left_thumbs.len(), 2);

    // Q W E R T kept on the left top alpha row.
    let top: Vec<_> = s.left_rows[0].iter().map(|k| k.raw.as_str()).collect();
    assert_eq!(top, vec!["KC.Q", "KC.W", "KC.E", "KC.R", "KC.T"]);

    // The dropped outer-column keys show up as relocations.
    let relocated: Vec<_> = s.relocations.iter().map(|r| r.raw.as_str()).collect();
    assert!(relocated.iter().any(|r| r.contains("ESC")) || relocated.contains(&"KC.ESC"));
}

#[test]
fn corne_keeps_all_six_columns() {
    let km = parse_main_py(REAL).unwrap();
    let s = suggest_compact_layout(&km, TargetBoard::Corne);
    assert_eq!(s.total_keys, 42);
    assert!(s.left_rows.iter().all(|r| r.len() == 6));
    assert_eq!(s.left_thumbs.len(), 3);
}
