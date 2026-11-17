use std::collections::BTreeSet;

use lemonfiber_ports::media::preset::Preset;
use lemonfiber_ports::media::Kind;

use super::{guidance, holds, of_section, pointed, rewrite, section, PROFILES};

/// The `config.yml` the plugin ships: its curators are held to Balanced.
const SHIPPED: &str = include_str!("../../../config/configarr/config.yml");

/// Both curators held to `preset`.
fn everywhere(config: &str, preset: Preset) -> String {
    rewrite(&rewrite(config, Kind::Tv, preset), Kind::Movies, preset)
}

/// Every `- template:` entry of `config`, in order.
fn templates(config: &str) -> Vec<&str> {
    config
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- template: "))
        .collect()
}

#[test]
fn every_profile_is_a_trash_id_and_each_preset_resolves_to_one_of_them() {
    for known in PROFILES {
        assert_eq!(known.profile().len(), 32, "{known:?}");
        assert!(known.profile().bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    for kind in Kind::ALL {
        for preset in Preset::ALL {
            assert!(PROFILES.contains(&guidance(kind, preset)));
        }
    }
}

#[test]
fn television_takes_series_sizes_and_film_takes_film_sizes() {
    for preset in Preset::ALL {
        assert_eq!(guidance(Kind::Tv, preset).sizes(), "series");
        assert!(["movie", "sqp-streaming"].contains(&guidance(Kind::Movies, preset).sizes()));
    }
    assert_eq!(
        guidance(Kind::Movies, Preset::SpaceSaving).sizes(),
        "sqp-streaming"
    );
}

#[test]
fn the_three_1080p_presets_collapse_for_television_and_film_keeps_all_four() {
    assert_eq!(
        guidance(Kind::Tv, Preset::SpaceSaving),
        guidance(Kind::Tv, Preset::HighQuality)
    );
    assert_ne!(
        guidance(Kind::Tv, Preset::HighQuality),
        guidance(Kind::Tv, Preset::Maximum)
    );
    let film: BTreeSet<_> = Preset::ALL
        .iter()
        .map(|preset| guidance(Kind::Movies, *preset).profile())
        .collect();
    assert_eq!(film.len(), Preset::ALL.len());
}

#[test]
fn each_section_names_the_kind_it_holds_and_no_other_key_does() {
    assert_eq!(of_section("sonarr"), Some(Kind::Tv));
    assert_eq!(of_section("radarr"), Some(Kind::Movies));
    assert_eq!(of_section("prowlarr"), None);
    assert_eq!(section(Kind::Movies), "radarr");
}

#[test]
fn balanced_everywhere_reproduces_the_shipped_file() {
    assert_eq!(everywhere(SHIPPED, Preset::Balanced), SHIPPED);
    for kind in Kind::ALL {
        assert!(holds(SHIPPED, kind, Preset::Balanced));
        assert!(!holds(SHIPPED, kind, Preset::Maximum));
    }
}

#[test]
fn a_preset_reaches_only_the_section_of_its_curator() {
    let rewritten = rewrite(SHIPPED, Kind::Movies, Preset::SpaceSaving);
    let asked = guidance(Kind::Movies, Preset::SpaceSaving);

    assert_eq!(
        templates(&rewritten),
        [
            guidance(Kind::Tv, Preset::Balanced).profile(),
            asked.profile()
        ]
    );
    assert!(rewritten.contains("      type: series\n"));
    assert!(rewritten.contains("      type: sqp-streaming\n"));
    assert!(holds(&rewritten, Kind::Tv, Preset::Balanced));
    assert!(holds(&rewritten, Kind::Movies, Preset::SpaceSaving));
}

#[test]
fn every_preset_leaves_each_curator_with_one_entry_naming_trash_as_its_source() {
    for preset in Preset::ALL {
        let config = everywhere(SHIPPED, preset);
        assert_eq!(templates(&config).len(), 2, "{preset:?}");
        assert_eq!(config.matches("        source: TRASH\n").count(), 2);
        for kind in Kind::ALL {
            assert!(holds(&config, kind, preset), "{preset:?} {kind:?}");
        }
    }
}

#[test]
fn rewriting_is_idempotent_and_keeps_comments() {
    let once = everywhere(SHIPPED, Preset::HighQuality);

    assert_eq!(everywhere(&once, Preset::HighQuality), once);
    assert!(once.contains("# Configarr"));
    assert_eq!(once.lines().count(), SHIPPED.lines().count());
}

#[test]
fn an_operators_own_section_and_entries_are_left_untouched() {
    let config = "\
lidarr:
  main:
    include:
      - template: 72dae194fc92bf828f32cde7744e51a1
        source: TRASH
sonarr:
  main:
    include:
      - template: theirs
      - template: 72dae194fc92bf828f32cde7744e51a1
        source: TRASH
";
    let rewritten = rewrite(config, Kind::Tv, Preset::Maximum);

    assert!(rewritten.starts_with(
        "lidarr:\n  main:\n    include:\n      - template: 72dae194fc92bf828f32cde7744e51a1\n"
    ));
    assert_eq!(
        templates(&rewritten),
        [
            "72dae194fc92bf828f32cde7744e51a1",
            "theirs",
            guidance(Kind::Tv, Preset::Maximum).profile()
        ]
    );
}

#[test]
fn an_include_nested_beneath_another_key_is_not_the_instances() {
    let config = "\
sonarr:
  main:
    custom_format_groups:
      - trash_guide:
          - id: some-group
            include:
              - id: some-format
    include:
      - template: 72dae194fc92bf828f32cde7744e51a1
        source: TRASH
";
    let rewritten = rewrite(config, Kind::Tv, Preset::Maximum);

    assert!(rewritten.contains("            include:\n              - id: some-format\n"));
    assert_eq!(
        templates(&rewritten),
        [guidance(Kind::Tv, Preset::Maximum).profile()]
    );
}

#[test]
fn an_empty_include_and_a_typeless_definition_are_filled_at_the_derived_indent() {
    let config = "\
sonarr:
  main:
    quality_definition:
      preferred_ratio: 0.5
    include:
radarr:
";
    let rewritten = rewrite(config, Kind::Tv, Preset::Balanced);

    assert!(rewritten
        .contains("    quality_definition:\n      type: series\n      preferred_ratio: 0.5\n"));
    assert!(rewritten.contains(
        "    include:\n      - template: 72dae194fc92bf828f32cde7744e51a1\n        source: TRASH\nradarr:\n"
    ));
}

#[test]
fn no_stale_preset_survives_a_reshaped_include_block() {
    let config = "\
radarr:
  main:
    include:

      - template: d1d67249d3890e49bc12e275d989a7e9
        source: TRASH
      # the one I keep meaning to revisit
      - source: TRASH
        template: 90a3370d2d30cbaf08d9c23b856a12c8

    quality_definition:
      type: movie
      type: sqp-streaming
";
    let rewritten = rewrite(config, Kind::Movies, Preset::Maximum);

    assert_eq!(
        templates(&rewritten),
        [guidance(Kind::Movies, Preset::Maximum).profile()]
    );
    assert!(!rewritten.contains("90a3370d2d30cbaf08d9c23b856a12c8"));
    assert!(rewritten.contains("# the one I keep meaning to revisit"));
    assert_eq!(rewritten.matches("type: movie").count(), 1);
    assert!(!rewritten.contains("sqp-streaming"));
    assert!(rewritten.contains("\n\n    quality_definition:"));
}

#[test]
fn a_section_key_carrying_an_inline_comment_is_still_recognised() {
    let config = "\
sonarr: # primary television instance
  main:
    include: # profiles
      - template: 72dae194fc92bf828f32cde7744e51a1 # WEB-1080p
        source: TRASH
";
    let rewritten = rewrite(config, Kind::Tv, Preset::Maximum);

    assert_eq!(
        templates(&rewritten),
        [guidance(Kind::Tv, Preset::Maximum).profile()]
    );
    assert!(rewritten.contains("sonarr: # primary television instance"));
}

#[test]
fn a_curator_is_pointed_at_its_address_and_secret_and_enabled_and_nothing_else_moves() {
    let pointed = pointed(SHIPPED, Kind::Movies, "http://films:7000", "films_api_key");

    assert!(pointed.contains(
        "    enabled: true\n    base_url: http://films:7000\n    api_key: !secret films_api_key\n"
    ));
    let moved: Vec<(&str, &str)> = SHIPPED
        .lines()
        .zip(pointed.lines())
        .filter(|(shipped, now)| shipped != now)
        .collect();
    assert_eq!(moved.len(), 3, "{moved:?}");
    assert_eq!(pointed.lines().count(), SHIPPED.lines().count());
    assert!(pointed.contains("    enabled: false\n    base_url: http://sonarr:8989\n"));
}

#[test]
fn only_an_instances_own_keys_are_pointed() {
    let config = "\
radarr:
  main:
    delete_unmanaged_custom_formats:
      enabled: false
    enabled: false
";
    let pointed = pointed(config, Kind::Movies, "http://films:7000", "films_api_key");

    assert!(pointed.contains("      enabled: false\n    enabled: true\n"));
}

#[test]
fn an_empty_definition_takes_its_type_at_the_derived_indent() {
    let config = "\
radarr:
  main:
    quality_definition:
    # sizes come from the guides
";
    let rewritten = rewrite(config, Kind::Movies, Preset::Balanced);

    assert!(rewritten.contains(
        "    quality_definition:\n      type: movie\n    # sizes come from the guides\n"
    ));
}
