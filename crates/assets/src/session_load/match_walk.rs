use super::*;

static NEXT_PRODUCTS_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(super) async fn walk_prepared_match(
    zone_ff: Result<PathBuf, String>,
    common_mp: Result<PathBuf, String>,
    progress: LoadProgress,
) -> (MatchLoadOutcome, Option<Arc<CommonSet>>) {
    let zone_name = zone_ff
        .as_ref()
        .ok()
        .and_then(|path| path.file_stem())
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();

    let pool = load_pool();
    // bo2mp: a Call of Duty 4 (`iw3`) or World at War (`t4`) map: read by
    // its own reader below, not opened as one of the four envelopes.
    let legacy = zone_ff
        .as_ref()
        .ok()
        .and_then(|path| asset_transport::legacy_game_for_path(path));
    let map_open = {
        let zone_ff = if legacy.is_some() {
            Err("legacy map zone: read by its own reader".to_owned())
        } else {
            zone_ff.clone()
        };
        let progress = progress.clone();
        pool.spawn(async move {
            let path = match zone_ff {
                Ok(path) => path,
                Err(error) => {
                    progress.record_skipped_scoped(StageId::MapAssets, "open");
                    return Err(format!("zone not found: {error}"));
                }
            };
            let stage = progress.begin_scoped(StageId::MapAssets, "open", None);
            progress.begin_zone_open();
            let opened = open_zone_shared(&path);
            finish_zone_open(stage, &opened);
            let image = opened.map_err(|error| format!("open zone: {error}"))?;
            progress.record_zone_image_bytes(image.bytes.len());
            Ok::<_, String>((path, image))
        })
    };

    let mut donor_report = Vec::new();
    let key = CommonKey::for_match(
        zone_ff.as_ref().ok().map(PathBuf::as_path),
        common_mp.as_ref().ok().map(PathBuf::as_path),
        &mut donor_report,
    );
    let waiting = progress.begin_scoped(StageId::CommonAssets, "shared common", None);
    let (common, reach) = ensure_common(key).await;
    waiting.done();
    if progress.is_canceled() {
        diag::info!(
            World,
            "match walk: canceled while the common set was prepared — load retargeted"
        );
        return (MatchLoadOutcome::Canceled, None);
    }

    let cloning = std::time::Instant::now();
    let CommonProducts {
        scripts: mut iw4_scripts,
        mut t5_scene_models,
        material_seed,
        shared_surfaces,
        scene_models: mut common_scene_models,
        light_defs: common_light_defs,
        pen_table: mut common_pen_table,
        pen_table_loaded: mut common_pen_loaded,
        lochit_table: common_lochit_table,
        tracers: mut common_tracers,
        xmodel_walk,
        s1_common_bytes,
        teamsets,
        film_visions: mut common_film_visions,
        mut weapons,
        mut fpv_meshes,
        mut world_weapons,
        mut projectile_meshes,
        mut xanims,
        mut player_anim_sources,
        fx: common_fx,
        fx_models: common_fx_models,
        impact_fx: common_impact,
        t5_xanims,
        t5_fx,
        t5_impact_fx,
        iw5_materials,
        mut iw5_scene_models,
        iw5_shared_surfaces,
        strings,
        counts:
            CommonCounts {
                startup_count,
                t5_mat_count,
                t5_reuse_mat,
                t5_xanim_n,
                foreign_count,
                seed_mat,
                seed_img,
                common_reuse_mat,
                common_reuse_img,
            },
        report: mut common_report,
        localize_report,
    } = common.products.clone();
    let clone_ms = cloning.elapsed().as_secs_f32() * 1000.0;
    let iw5_mat_n = iw5_materials.materials.len();
    common_report.append(&mut donor_report);
    common_report.push(format!(
        "common set: {reach} for {} (prepared in {:.0}ms, {:.1}s ago); match copy {clone_ms:.0}ms; {} donor image batches and {} kept payloads ({:.1}MiB) shared, not decoded again",
        common.key,
        common.prepared_ms,
        common.ready_at.elapsed().as_secs_f32(),
        common.donor_batches(),
        common.retained_payloads(),
        common.retained_bytes() as f64 / (1024.0 * 1024.0),
    ));
    let common_images = hold_image_plan(
        "common_mp FPV",
        common.fpv_plan.clone(),
        load_jobs::open(JobKind::ImageDecode).namespace("iw4"),
    );

    let opened_map = map_open.await;
    if progress.is_canceled() {
        drop(opened_map);
        diag::info!(
            World,
            "match walk: canceled before the map walk — load retargeted"
        );
        return (MatchLoadOutcome::Canceled, None);
    }
    let (loaded, map_namespace) = match (legacy, zone_ff.as_ref(), opened_map) {
        (Some(game), Ok(path), _) => {
            crate::set_map_game(asset_core::AssetNamespace::parse(game));
            drop(shared_surfaces);
            drop(iw5_shared_surfaces);
            let mut loaded = match game {
                "iw3" => crate::lane::load_iw3_world(path, &progress, material_seed),
                "t4" => crate::lane::load_t4_world(path, &progress, material_seed),
                // bo2mp: Ghosts' maps.
                "iw6" => crate::lane::load_iw6_world(path, &progress, material_seed),
                // bo2mp: Infinite Warfare's maps.
                "iw7" => crate::lane::load_iw7_world(path, &progress, material_seed),
                // bo2mp: Black Ops III's maps.
                "t7" => crate::lane::load_t7_world(path, &progress, material_seed),
                // bo2mp: Modern Warfare III's (2023) maps.
                "jup" => crate::lane::load_jup_world(path, &progress, material_seed),
                _ => {
                    drop(material_seed);
                    LoadedWorld::with_gap(
                        WorldDrawPolicy::t5(),
                        PreparedCapability::PreparedWorld,
                        format!("{game} maps are not readable yet"),
                        Some("assets::session_load::load_prepared_match/legacy"),
                    )
                }
            };
            if crate::t6_rules() {
                let stage = progress.begin_scoped(StageId::MapAssets, "black ops ii rules", None);
                // (Ghosts' surface list is Infinity Ward's, as CoD4's.)
                crate::lane::t6_surface_list(&mut loaded, &zone_name, matches!(game, "iw3" | "iw6"));
                // bo2mp: an Infinite Warfare Zombies map plays Black Ops II
                // Zombies.
                // (bo2mp: and a Black Ops III Zombies map, which its lane
                // registers there.)
                if (game == "iw7" || game == "t7") && crate::lane::is_iw7_zombies_map(&zone_name) {
                    crate::lane::attach_t6_zm_rules(&mut loaded, &zone_name);
                } else {
                    // bo2mp: Black Ops III's entities are Treyarch's (Black
                    // Ops II's class names).
                    let source = if game == "t7" {
                        asset_world::ForeignEntityGame::Treyarch
                    } else {
                        asset_world::ForeignEntityGame::InfinityWard
                    };
                    crate::lane::attach_t6_rules(&mut loaded, &zone_name, source);
                }
                stage.done();
            }
            (loaded, Some(asset_core::AssetNamespace::T6))
        }
        (_, _, Ok((path, image))) => {
            let game = image.game;
            crate::set_map_game(Some(asset_core::AssetNamespace::from_zone_game(game)));
            let shared_surfaces = match game {
                asset_core::ZoneGame::Iw5 => iw5_shared_surfaces,
                _ => shared_surfaces,
            };
            let mut loaded = lane(game).load_world(
                &path,
                &image,
                &progress,
                shared_surfaces,
                material_seed,
                &mut common_film_visions,
            );
            // bo2mp: a map from another game under Black Ops II's rules.
            if game != asset_core::ZoneGame::T6 && crate::t6_rules() {
                let stage = progress.begin_scoped(StageId::MapAssets, "black ops ii rules", None);
                let source = match game {
                    asset_core::ZoneGame::T5 => asset_world::ForeignEntityGame::Treyarch,
                    _ => asset_world::ForeignEntityGame::InfinityWard,
                };
                crate::lane::t6_surface_list(
                    &mut loaded,
                    &zone_name,
                    matches!(game, asset_core::ZoneGame::Iw4 | asset_core::ZoneGame::Iw5),
                );
                crate::lane::attach_t6_rules(&mut loaded, &zone_name, source);
                stage.done();
            }
            (loaded, Some(asset_core::AssetNamespace::from_zone_game(game)))
        }
        (_, _, Err(gap)) => {
            crate::set_map_game(None);
            drop(material_seed);
            (
                LoadedWorld::with_gap(
                    WorldDrawPolicy::default(),
                    PreparedCapability::PreparedWorld,
                    gap,
                    Some("assets::session_load::load_prepared_match/zone_open"),
                ),
                None,
            )
        }
    };

    let LoadedWorld {
        scripts: map_scripts,
        mut world,
        mut materials,
        collision: clip,
        spawns: dm_spawns,
        mut bodies,
        fpv_meshes: map_fpv,
        xanims: map_xanims,
        weapons: map_weapons,
        world_weapons: map_world_weapons,
        tracers: map_tracers,
        pen_table: map_pen_table,
        mut facts,
        arena_bytes: s1_map_bytes,
        sound,
        mut report,
        gaps,
    } = loaded;
    report.append(&mut common_report);
    // bo2zm: a Black Ops II map brings its own penetration table (it has no
    // IW4 common zone).
    let (mut common_pen_table, mut common_pen_loaded) = match facts.t6_pen_table.take() {
        Some(table) => (table, true),
        None => (common_pen_table, common_pen_loaded),
    };

    if facts.team_settings.allies.is_none() && facts.team_settings.axis.is_none() {
        if let Some(name) = facts.t5_teamset.as_ref() {
            if let Some(icons) = teamsets.get(name) {
                facts.team_settings = icons.clone();
            }
        }
    }
    if facts.t5_teamset.is_some() {
        facts.script_sound.attackers = facts
            .script_sound
            .attackers
            .or_else(|| facts.team_settings.attackers.clone());
        facts.script_sound.defenders = facts
            .script_sound
            .defenders
            .or_else(|| facts.team_settings.defenders.clone());
    }
    if map_namespace == Some(asset_core::AssetNamespace::T5) {
        facts.objective_visuals = asset_game::ObjectiveVisuals::t5(
            facts.team_settings.allies_charset.as_deref(),
            facts.team_settings.axis_charset.as_deref(),
        );
    }
    match (
        facts.t5_teamset.as_deref(),
        facts.team_settings.allies.as_ref(),
        facts.team_settings.axis.as_ref(),
    ) {
        (Some(ts), Some(a), Some(x)) => {
            report.push(format!("team icons: teamset={ts} allies={a} axis={x}"));
        }
        (Some(ts), ..) => {
            report.push(format!(
                "team icons gap: teamset={ts} but common_mp had no matching _teamset_*.gsc icons"
            ));
        }
        _ => {}
    }

    if matches!(
        map_namespace,
        Some(asset_core::AssetNamespace::T5 | asset_core::AssetNamespace::Iw5)
    ) && !foreign_faction_rows(&mut iw4_scripts, &facts.team_settings)
    {
        report.push(format!(
            "team rows gap: {FACTION_TABLE} lacks {IW4_ALLIES_CHARSET}/{IW4_AXIS_CHARSET}; IW4 scripts show IW4 teams"
        ));
    }
    let map_scripts = match map_namespace {
        Some(asset_core::AssetNamespace::T5) if !crate::t6_rules() => {
            let scripts = t5_map_under_iw4_rules(&map_scripts, &zone_name, &facts);
            report.push(format!(
                "map script: maps/mp/{zone_name} written from the T5 map's declarations; T5 map scripts left out"
            ));
            scripts
        }
        _ => map_scripts,
    };
    let mut scripts = iw4_scripts;
    scripts.overlay(map_scripts);
    report.push(format!(
        "GSC source assets: {} (map overrides common_mp)",
        scripts.len()
    ));

    report.push(format!(
        "map teams: allies={:?} axis={:?} attackers={:?} defenders={:?}",
        facts.team_settings.allies_name,
        facts.team_settings.axis_name,
        facts.script_sound.attackers,
        facts.script_sound.defenders,
    ));

    report.push(format!(
        "s1 pool walked: common={s1_common_bytes} map={s1_map_bytes} total={} rss={}",
        s1_common_bytes.saturating_add(s1_map_bytes),
        asset_transport::process_resident_bytes().unwrap_or(0),
    ));

    // bo2zm: weapons the map zone defines (Black Ops II maps carry their
    // own) join the catalog before anything resolves against it.
    if !map_world_weapons.is_empty() {
        world_weapons.absorb(map_world_weapons);
    }
    // bo2mp: Black Ops II's penetration depths come with the map's zones.
    if !common_pen_loaded {
        if let Some(table) = map_pen_table {
            common_pen_table = table;
            common_pen_loaded = true;
        }
    }
    if !map_tracers.is_empty() {
        let before = common_tracers.len();
        common_tracers.absorb(map_tracers);
        report.push(format!("map tracers: catalog {before} -> {}", common_tracers.len()));
    }
    if !map_weapons.is_empty() {
        let before = weapons.len();
        weapons.absorb(map_weapons);
        report.push(format!(
            "map weapons: catalog {before} -> {} ({} map namespace)",
            weapons.len(),
            map_namespace.map_or("?", |ns| ns.as_str())
        ));
    }
    // bo2mp: other games' guns as extra weapons in a Black Ops II match:
    // BO2's own rows keep every plain name, the guests are `bo1_<gun>_mp`.
    if crate::t6_rules() {
        weapons.set_guest_names(true);
        let guests: Vec<String> = (1..weapons.len() as u32)
            .filter(|&i| weapons.namespace_of(i).is_some_and(|ns| ns != asset_core::AssetNamespace::T6))
            .map(|i| weapons.script_name_of(i))
            .filter(|n| !n.contains('+'))
            .collect();
        // (A game's guns are listed in Create-a-Class only when they loaded.)
        for (game, ns) in [("t5", asset_core::AssetNamespace::T5), ("iw4", asset_core::AssetNamespace::Iw4), ("iw5", asset_core::AssetNamespace::Iw5), ("t4", asset_core::AssetNamespace::T4), ("iw3", asset_core::AssetNamespace::Iw3), ("iw6", asset_core::AssetNamespace::Iw6), ("s1", asset_core::AssetNamespace::S1), ("t7", asset_core::AssetNamespace::T7)] {
            let wanted = bo2_profile::guest::all().filter(|g| g.game == game).count();
            let loaded = bo2_profile::guest::all()
                .filter(|g| g.game == game)
                .filter(|g| {
                    asset_core::AssetKey::new(ns, asset_core::AssetKind::Weapon, format!("{}_mp", g.gun))
                        .ok()
                        .and_then(|k| weapons.resolve_key(&k).ok().flatten())
                        .is_some()
                })
                .count();
            if wanted > 0 {
                bo2_profile::guest::set_available(game, crate::guest_guns() && loaded > 0);
                report.push(format!("bo2mp guest guns: {game}: {loaded} of {wanted} listed guns loaded"));
            }
        }
        // (The mod's guns, `IW4L_T6_MOD`: Black Ops II weapons from its zone.)
        if asset_transport::t6_mod::zone().is_some() {
            let wanted = bo2_profile::guest::MOD.len();
            let loaded = bo2_profile::guest::MOD
                .iter()
                .filter(|g| weapons.resolve_index(&bo2_profile::guest::script_name(g)).ok().flatten().is_some_and(|i| i != 0))
                .count();
            bo2_profile::guest::set_available(bo2_profile::guest::MOD_GAME, loaded > 0);
            report.push(format!("bo2mp mod guns: {loaded} of {wanted} listed guns loaded"));
        } else {
            bo2_profile::guest::set_available(bo2_profile::guest::MOD_GAME, false);
        }
        report.push(format!(
            "bo2mp guest guns: {} other games' weapons named for BO2's scripts ({} base guns): {}",
            (1..weapons.len() as u32)
                .filter(|&i| weapons.namespace_of(i).is_some_and(|ns| ns != asset_core::AssetNamespace::T6))
                .count(),
            guests.len(),
            guests.iter().take(80).cloned().collect::<Vec<_>>().join(" ")
        ));
    }
    // bo2mp: on a Zombies map, the Mystery Box's guest guns' Pack-a-Punch
    // upgrades (their own games' upgraded weapons, else Black Ops II-style
    // copies), before anything resolves against the catalog.
    let zm_upgrades = if crate::t6_rules() && crate::guest_guns() && (zone_name.starts_with("zm_") || crate::lane::is_iw7_zombies_map(&zone_name)) {
        guest_zm_upgrades(&mut weapons, &mut report)
    } else {
        Vec::new()
    };
    bo2_profile::guest::set_zm_upgrades(zm_upgrades);
    let common_xanim_count = xanims.len();
    let map_xanim_count = map_xanims.len();
    let t5_xanim_added = xanims.absorb(t5_xanims);
    xanims.absorb_local(map_xanims);
    weapons.resolve_sz_xanim_edges(&xanims);
    let weapon_clip_indices = weapons.bound_weapon_xanim_indices();
    let clip_prewarm_started = std::time::Instant::now();
    let failed_weapon_clips: Vec<_> = weapon_clip_indices
        .iter()
        .copied()
        .filter(|&index| xanims.clip_at(index).is_none())
        .collect();
    report.push(format!(
        "weapon XAnim CPU prewarm: linked={} decoded={} failed={} elapsed_ms={:.1}",
        weapon_clip_indices.len(),
        weapon_clip_indices.len() - failed_weapon_clips.len(),
        failed_weapon_clips.len(),
        clip_prewarm_started.elapsed().as_secs_f64() * 1000.0,
    ));
    for index in failed_weapon_clips.iter().take(16) {
        report.push(format!(
            "weapon XAnim decode gap: index={index} name={}",
            xanims.name_at(*index).unwrap_or("<unknown>")
        ));
    }
    let (note_actions, inline_note_actions) = weapons.resolve_notetrack_actions(&xanims);
    report.push(format!(
        "weapon notetrack actions linked: {note_actions} ({inline_note_actions} T5 inline)"
    ));
    let sz_xanims = weapons.sz_xanim_edge_census();
    report.push(format!(
        "XAnim generation: common={common_xanim_count} t5={t5_xanim_n} t5_keys={t5_xanim_added} collide={} map={map_xanim_count} merged={}",
        xanims.collide_name_count(),
        xanims.len()
    ));
    report.push(format!(
        "weapon szXAnims after absorb: bound={} unresolved={} absent={}",
        sz_xanims.bound, sz_xanims.unresolved, sz_xanims.absent,
    ));
    player_anim_sources.bind_leaves(&xanims);
    report.push(player_anim_sources.bind_report_line());
    let gap_lines: Vec<String> = gaps
        .iter()
        .map(|gap| match gap.addr {
            Some(addr) => format!("lane gap [{addr}]: {}", gap.reason),
            None => format!("lane gap: {}", gap.reason),
        })
        .collect();
    report.extend(gap_lines.iter().cloned());
    report.extend(bodies.report_lines());
    let map_fpv_n = map_fpv.len();
    let map_fpv_added = fpv_meshes.absorb(map_fpv);
    fpv_meshes.set_map_namespace(map_namespace);
    weapons.resolve_fpv_mesh_edges(&fpv_meshes);
    let hands = weapons.resolve_fpv_hands(&fpv_meshes, &bodies);
    if hands.t6_linked + hands.t6_unresolved > 0 {
        let line = format!(
            "bo2 fpv hands: map_ns={map_namespace:?} t6 gun sides linked={} unresolved={} arms={}",
            hands.t6_linked,
            hands.t6_unresolved,
            hands.t6_sample.as_deref().unwrap_or("<none>"),
        );
        diag::info!(Fpv, "{line}");
        report.push(line);
    }
    let assembly_started = std::time::Instant::now();
    let assemblies = weapons.resolve_fpv_assemblies(&fpv_meshes, &xanims);
    report.push(format!(
        "FPV assemblies: built={} kit sides linked={} refused={} clip track tables={} elapsed_ms={:.1}",
        assemblies.built,
        assemblies.linked,
        assemblies.refused,
        assemblies.clip_tables,
        assembly_started.elapsed().as_secs_f64() * 1000.0,
    ));

    world_weapons.seal_identity();
    weapons.resolve_world_model_edges(&world_weapons);
    let world_model_edges = weapons.world_model_edge_census();
    report.push(format!(
        "world gun generation: catalog={}; worldModel edges bound={} unresolved={} absent={}",
        world_weapons.len(),
        world_model_edges.bound,
        world_model_edges.unresolved,
        world_model_edges.absent,
    ));
    let dependency_gaps = weapons.dependency_gaps();
    let selectable_gap_ids: std::collections::BTreeSet<_> = dependency_gaps
        .iter()
        .map(|gap| gap.id)
        .filter(|&id| weapons.describe_configuration(id).is_some())
        .collect();
    report.push(format!(
        "weapon dependency audit: {} gaps in {} definitions; selectable={}",
        dependency_gaps.len(),
        dependency_gaps
            .iter()
            .map(|gap| gap.id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        selectable_gap_ids.len(),
    ));
    for gap in &dependency_gaps {
        report.push(format!(
            "weapon dependency gap: {} {} `{}`",
            weapons.name_of(gap.id),
            gap.kind,
            gap.name
        ));
    }
    report.push(format!(
        "FPV map fanout: map={map_fpv_n} added={map_fpv_added} merged={} map_ns={map_namespace:?}",
        fpv_meshes.len()
    ));

    let before_unrouted = materials.unrouted_material_count();
    let promoted = materials.promote_iw5_fallback_tables();
    let t5_alias_map = materials.absorb_t5_feature_token_donors();
    let stub_routed = materials.reroute_stub_materials();
    let after_unrouted = materials.unrouted_material_count();
    let takes_ml = materials
        .materials
        .iter()
        .filter(|m| materials.takes_model_lighting(m) == Some(true))
        .count();
    let t5_fb = materials
        .technique_set_facts()
        .iter()
        .filter(|facts| facts.t5_fallback_table.is_some())
        .count();
    report.push(format!(
        "material route: iw5_promoted={promoted} \
             t5_tech_alias={t5_alias_map} t5_fallback={t5_fb} stub_routed={stub_routed} \
             unrouted {before_unrouted}→{after_unrouted}; takes_model_lighting={takes_ml}"
    ));

    let map_reuse_mat = materials.link_reused_materials;
    let map_reuse_img = materials.link_reused_images;
    let pool_mat = materials.materials.len();
    let pool_img = materials.images.len();
    report.push(format!(
            "s2 pool: seed_mat={seed_mat} seed_img={seed_img} common_reuse_mat={common_reuse_mat} common_reuse_img={common_reuse_img} map_reuse_mat={map_reuse_mat} map_reuse_img={map_reuse_img} pool_mat={pool_mat} pool_img={pool_img}"
        ));
    let map_new = pool_mat.saturating_sub(seed_mat);

    let mut global = materials;
    let iw5_linked = global.absorb_asset_population_host_materials_win(iw5_materials);
    let provisional_map_ids: Vec<usize> = (0..global.materials.len()).collect();
    if iw5_mat_n > 0 {
        iw5_scene_models.remap_walk_materials(&iw5_linked);
    }
    let mut scene_names = scripts.asset_names();
    scene_names.extend(facts.objective_visuals.model_names().map(str::to_owned));
    common_scene_models.retain_names(&scene_names);
    t5_scene_models.retain_names(&scene_names);
    iw5_scene_models.retain_names(&scene_names);
    match map_namespace {
        Some(asset_core::AssetNamespace::T5) => world
            .map_xmodel_scene_assets
            .absorb_captured(t5_scene_models),
        Some(asset_core::AssetNamespace::Iw5) => world
            .map_xmodel_scene_assets
            .absorb_captured(iw5_scene_models),
        _ => {}
    }
    world
        .map_xmodel_scene_assets
        .absorb_captured(common_scene_models);
    report.push(format!(
        "iw5 leftover materials: donor={iw5_mat_n} linked={} pool={}",
        iw5_linked.len(),
        global.materials.len(),
    ));
    let promoted = global.promote_iw5_fallback_tables();
    let t5_alias = global.absorb_t5_feature_token_donors();
    global.reroute_stub_materials();
    let mat_refs = global.material_ref_census();
    let img_refs = global.image_ref_census();
    let shader_refs = global.shader_ref_census();
    let decl_refs = global.vertex_decl_ref_census();
    report.push(format!(
            "asset_ref before finalize: materials n={} real={} reference={}; images n={} real={} reference={}; shaders n={} real={} reference={}; decls n={} real={} reference={}",
            mat_refs.n, mat_refs.real, mat_refs.reference,
            img_refs.n, img_refs.real, img_refs.reference,
            shader_refs.n, shader_refs.real, shader_refs.reference,
            decl_refs.n, decl_refs.real, decl_refs.reference,
        ));
    let (mut global, finalized_ids) = global.publish();
    let map_ids = provisional_map_ids
        .into_iter()
        .map(|id| finalized_ids.get(id).copied().flatten())
        .collect::<Vec<_>>();
    let t5_fb = global
        .technique_set_facts()
        .iter()
        .filter(|facts| facts.t5_fallback_table.is_some())
        .count();
    report.push(format!(
            "material generation: startup={startup_count} t5={t5_mat_count} t5_reuse={t5_reuse_mat} foreign={foreign_count} common_pool={seed_mat} map_new={map_new} pooled={pool_mat} global={} common_reuse={common_reuse_mat} map_reuse={map_reuse_mat} unresolved_aliases={} iw5_promoted={promoted} t5_tech_alias={t5_alias} t5_fallback={t5_fb}",
            global.materials.len(),
            finalized_ids.iter().filter(|id| id.is_none()).count(),
        ));
    let mat_refs = global.material_ref_census();
    let img_refs = global.image_ref_census();
    let shader_refs = global.shader_ref_census();
    let decl_refs = global.vertex_decl_ref_census();
    report.push(format!(
            "asset_ref after finalize: materials n={} real={} reference={}; images n={} real={} reference={}; shaders n={} real={} reference={}; decls n={} real={} reference={}",
            mat_refs.n, mat_refs.real, mat_refs.reference,
            img_refs.n, img_refs.real, img_refs.reference,
            shader_refs.n, shader_refs.real, shader_refs.reference,
            decl_refs.n, decl_refs.real, decl_refs.reference,
        ));

    let global_memory = global.image_memory();
    report.push(global_memory.report_row("image memory global generation"));
    report.push(format!(
        "canonical materials after absorb: n={} images={} decoded={}",
        global.materials.len(),
        global_memory.images,
        global_memory.decoded_images,
    ));
    let shader_census = global.shader_source_census();
    report.push(format!(
        "shader source corpus: programs={} unresolved_aliases={} byteless={}",
        shader_census.programs, shader_census.unresolved_aliases, shader_census.byteless,
    ));
    let decl_streams = global.vertex_decl_stream_census();
    report.push(format!(
        "vertex decls: n={} stream0={} ppcc0t0t0nn n={} streams={}",
        decl_streams.n,
        decl_streams.stream0,
        decl_streams.ppcc_n,
        decl_streams
            .ppcc_stream_count
            .map(|n| n.to_string())
            .unwrap_or_else(|| "missing".into()),
    ));

    // The donors first, and only then the plan that claims the same names
    // they do. Its claims are resolved against the catalog they leave behind
    // rather than against the one it was built from, which is why it waited.
    for kept in common.donor_images().await {
        let job = load_jobs::open(JobKind::ImageDecode)
            .namespace(kept.namespace)
            .canonical(kept.label)
            .depends_on(kept.job)
            .plan_ready()
            .enqueued()
            .started()
            .finished()
            .joined();
        merge_image_batch(
            &mut global,
            kept.label,
            kept.batch.share(),
            job,
            &mut report,
        );
    }
    if let Some(held) = common_images
        && let Some(pending) = held.prune_then_enqueue(&mut global, &progress, &mut report)
    {
        let job = pending.job;
        let (label, batch) = pending.join().await;
        let kept = common.retain(&batch);
        report.push(format!(
            "{label} payloads kept for the next map: +{:.1}MiB (common set holds {} payloads, {:.1}MiB)",
            kept as f64 / (1024.0 * 1024.0),
            common.retained_payloads(),
            common.retained_bytes() as f64 / (1024.0 * 1024.0),
        ));
        merge_image_batch(&mut global, label, batch, job, &mut report);
    }
    let mut merged_image_gaps = Vec::new();
    if let Ok(path) = &zone_ff {
        let stage = progress.begin_scoped(StageId::Images, "merged", None);
        let decoded =
            asset_material::decode_material_color_maps(path, &mut global, &stage, load_pool());
        stage.finish_from(&decoded);
        match decoded {
            Ok(stats) => {
                report.push(format!(
                    "merged material images: {}/{} decoded, {} missing, {} unsupported",
                    stats.decoded, stats.requested, stats.missing, stats.unsupported
                ));
                if let Some(gap) = &stats.first_unsupported {
                    report.push(format!("merged material images: first unsupported: {gap}"));
                }
                merged_image_gaps = stats.gaps;
            }
            Err(error) => report.push(format!("merged material images gap: {error}")),
        }
    }
    if let Some(draw) = world.draw.as_mut() {
        world.dynamic_light = resolve_world_lights(
            draw,
            &mut global,
            &common_light_defs,
            map_namespace,
            &zone_ff,
            &progress,
            &mut report,
        );
    }
    let builtins = asset_material::decode_in_zone_builtin_images(&mut global);
    if builtins != 0 {
        report.push(format!(
            "in-zone builtin images: decoded {builtins} leftover $ 2D loadDefs after absorb"
        ));
    }
    // bo2mp: name, per map, every material of the map's own game whose images
    // this load could not produce. Those surfaces draw through the untextured
    // fallback (neutral grey); the line is what the next run fixes from.
    if let Some(namespace) = map_namespace {
        let map = zone_ff
            .as_ref()
            .ok()
            .and_then(|path| path.file_stem())
            .map_or_else(|| "?".to_owned(), |stem| stem.to_string_lossy().into_owned());
        let rows =
            asset_material::unresolved_material_images(&global, namespace, &merged_image_gaps);
        report.push(asset_material::unresolved_material_images_line(
            &format!("{map}, {}", namespace.as_str()),
            &rows,
        ));
    }
    if let Ok(path) = &zone_ff {
        let stage = progress.begin_scoped(StageId::Images, "tracers", None);
        let decoded = asset_material::material_images::decode_color_or_2d_for_keys(
            path,
            &mut global,
            common_tracers.material_keys(),
            &stage,
            load_pool(),
        );
        stage.finish_from(&decoded);
        match decoded {
            Ok(n) => report.push(format!(
                "tracer beam images after absorb: {n} TS_COLOR_MAP/TS_2D decoded"
            )),
            Err(error) => report.push(format!("tracer beam images after absorb: {error}")),
        }
    }
    let unique: std::collections::BTreeSet<String> = common_tracers
        .named_materials()
        .map(str::to_owned)
        .collect();
    for name in &unique {
        let bind = asset_game::material_bind_name(name);
        let twins: Vec<&str> = global
            .materials
            .iter()
            .filter(|m| m.name.as_str() == bind)
            .map(|m| m.name.as_str())
            .collect();
        report.push(format!("tracer material `{name}` global twins={twins:?}"));
    }
    report.push(format!(
        "tracer color maps after absorb: Bound into global ({} unique names; no clone sidecar)",
        unique.len()
    ));

    world.fx.absorb(common_fx);
    let common_fx_model_n = common_fx_models.len();
    let map_fx_model_n = world.fx_models.len();
    let common_fx_model_added = world.fx_models.absorb(common_fx_models);
    let leftover_t5_fx_n = t5_fx.len();
    let leftover_t5_fx_gaps = t5_fx.capture_gaps;
    world.fx.absorb_missing(t5_fx);
    report.push(format!(
        "leftover t5 fx absorb_missing: donor={leftover_t5_fx_n} gaps={leftover_t5_fx_gaps} host now {}",
        world.fx.len()
    ));
    let fx_model_walked_n = world.fx_models.len();
    world.fx_models.keep_referenced(&world.fx.model_hints());
    world.fx.resolve_model_edges(&world.fx_models);
    let fx_model_edges = world.fx.model_edge_census();
    report.push(format!(
        "FX model generation: map={map_fx_model_n} common={common_fx_model_n} added={common_fx_model_added} walked={fx_model_walked_n} retained={} edges bound={} unresolved={} absent={}",
        world.fx_models.len(),
        fx_model_edges.bound,
        fx_model_edges.unresolved,
        fx_model_edges.absent,
    ));
    weapons.resolve_combat_fx(&world.fx, &common_tracers);
    weapons.resolve_projectile_fx_edges(&world.fx);
    let projectile_fx = weapons.projectile_fx_edge_census();
    report.push(format!(
        "weapon projectile FX after absorb: bound={} unresolved={} absent={}",
        projectile_fx.bound, projectile_fx.unresolved, projectile_fx.absent,
    ));
    link_materials_after_absorb(
        &global,
        &mut world,
        &mut weapons,
        &mut common_tracers,
        &mut bodies,
        &mut world_weapons,
        &mut fpv_meshes,
        &mut projectile_meshes,
        &mut report,
    );
    report.push(
        "fx color maps handoff: n=0 bytes=0 (Bound GPU bind at spawn; no CPU clone sidecar; stub_aliases=0)"
            .into(),
    );
    decode_fx_colour_maps(&world.fx, &mut global, &zone_ff, &progress, &mut report);
    if world.impact_fx.is_none() {
        world.impact_fx = if map_namespace == Some(asset_core::AssetNamespace::T5) {
            t5_impact_fx.or(common_impact)
        } else {
            common_impact
        };
    } else if let Some(common_table) = common_impact {
        report.push(format!(
            "impactfx: map table kept; common_mp table `{}` discarded",
            common_table.name
        ));
    }
    if let Some(ref table) = world.impact_fx {
        report.push(format!(
            "impactfx handoff: {:?}:`{}` rows={} (fx catalog now {} defs)",
            table.namespace(),
            table.name,
            table.row_count(),
            world.fx.len()
        ));
    } else {
        report.push("impactfx handoff: missing — combat play_oriented will miss cells".into());
    }
    report.push(format!(
        "tracer catalog handoff: {} named ({} bound, {} unresolved)",
        common_tracers.len(),
        common_tracers.bound_count(),
        common_tracers.unresolved_count()
    ));

    let prepared_map = PreparedMap {
        zone: zone_name,
        namespace: map_namespace,
        spawns: dm_spawns,
        facts,
        gaps: PreparedGaps { lines: gap_lines },
    };
    if prepared_map.facts.minimap_corners.is_some() {
        report.push("compass: minimap_corner pair from MapEnts".into());
    } else {
        report.push("compass gap: minimap_corner missing — no world-to-map frame".into());
    }
    match prepared_map.facts.north_yaw {
        Some(yaw) => report.push(format!("compass: worldspawn northyaw {yaw}")),
        None => {
            report.push("compass gap: worldspawn has no northyaw — map is drawn north-up".into())
        }
    }

    report.extend(localize_report);
    let (directory_ms, opens, inflate_ms) = asset_material::iwd_read_cost();
    report.push(format!(
        "IWD read cost: central_dir={directory_ms:.0}ms over {opens} opens, inflate={inflate_ms:.0}ms (summed over worker threads, not wall)"
    ));
    let (mip_hit, mip_miss, mip_io_ms) = asset_material::mip_cache_cost();
    report.push(format!(
        "mip cache: hit={mip_hit} miss={mip_miss} io={mip_io_ms:.0}ms"
    ));
    let (payload_reads, header_reads) = asset_material::iwd_entry_reads();
    report.push(format!(
        "IWD entry reads: payload={payload_reads} header-only={header_reads} (a header answers whether an image is a cubemap; a payload read is the whole entry inflated)"
    ));
    if let Some(draw) = world.draw.as_ref() {
        census_image_working_set(
            draw,
            &world,
            &global,
            &map_ids,
            &fpv_meshes,
            &common_tracers,
            &mut report,
        );
    }
    let mut fx = std::mem::take(&mut world.fx).publish();
    fx.set_map_namespace(map_namespace.unwrap_or(asset_core::AssetNamespace::Iw4));
    let xanims = xanims.publish();
    let destructible_death =
        crate::stamp_match_destructible_death(&xanims, &world.map_xmodel_scene_assets);
    for row in &destructible_death {
        report.push(format!(
            "destructible death {}: clip={} husk={}",
            row.kind,
            row.clip.edge_kind(),
            row.husk.edge_kind()
        ));
    }
    let prepared = PreparedMatch {
        scripts,
        fx,
        world,
        materials: crate::MatchMaterials {
            population: Arc::new(global),
            map_ids,
            common_profile_id: common.id,
            products_id: NEXT_PRODUCTS_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        },
        clip: clip.map(Arc::new),
        weapons: Arc::new(weapons.publish()),
        fpv_meshes: fpv_meshes.publish(),
        bodies: Arc::new(bodies.publish()),
        world_weapons: world_weapons.publish(),
        projectile_meshes: projectile_meshes.publish(),
        xanims,
        destructible_death,
        player_anim_sources,
        tracers: common_tracers.publish(),
        strings,
        report,
        prepared_map,
        pen_table: common_pen_table,
        pen_table_loaded: common_pen_loaded,
        lochit_table: common_lochit_table,
        xmodel_walk,
        sound,
    };
    (MatchLoadOutcome::Ready(prepared), Some(common))
}

fn resolve_world_lights(
    draw: &mut WorldDraw,
    global: &mut asset_material::MaterialDefinitions,
    common_light_defs: &[asset_world::CapturedLightDef],
    map_namespace: Option<asset_core::AssetNamespace>,
    zone_ff: &Result<PathBuf, String>,
    progress: &LoadProgress,
    report: &mut Vec<String>,
) -> Option<asset_world::ResolvedLightDef> {
    asset_world::resolve_primary_light_attenuation(draw, global, common_light_defs);
    let dynamic_light_name =
        (map_namespace == Some(asset_core::AssetNamespace::Iw4)).then_some("light_dynamic");
    let dynamic_light = dynamic_light_name.and_then(|name| {
        asset_world::resolve_named_light_def(name, &draw.light_defs, common_light_defs, global)
    });
    let (ordinal, source) = asset_world::resolve_outdoor_image(
        draw.outdoor_image_name.as_deref(),
        map_namespace.unwrap_or(asset_core::AssetNamespace::Iw4),
        global,
    );
    draw.outdoor_image = ordinal;
    if source == "$outdoor" && draw.outdoor_image_name.is_none() {
        draw.outdoor_image_name = Some("$outdoor".into());
    }
    report.push(format!(
        "outdoorImage: source={source} name={} global_ordinal={} lookup_m00={:.6e} lookup_m30={:.4}",
        draw.outdoor_image_name.as_deref().unwrap_or("-"),
        draw.outdoor_image
            .map(|i| i.to_string())
            .unwrap_or_else(|| "NONE".into()),
        f32::from_bits(draw.outdoor_lookup[0]),
        f32::from_bits(draw.outdoor_lookup[12]),
    ));
    let named = draw
        .primary_lights
        .iter()
        .filter(|light| light.def_name.as_ref().is_some_and(|name| !name.is_empty()))
        .count();
    let atten = draw
        .primary_lights
        .iter()
        .filter(|light| light.attenuation_image.is_some())
        .count();
    report.push(format!(
        "GfxLightDef resolve: common_defs={} map_defs={} named_lights={named} atten_image={atten} (light-def name → global catalog; missing stays None)",
        common_light_defs.len(),
        draw.light_defs.len(),
    ));
    report.push(format!(
        "GfxLightDef names: common={:?} map={:?} lights={:?} images_common={:?} images_map={:?}",
        common_light_defs
            .iter()
            .map(|def| def.name.as_str())
            .collect::<Vec<_>>(),
        draw.light_defs
            .iter()
            .map(|def| def.name.as_str())
            .collect::<Vec<_>>(),
        draw.primary_lights
            .iter()
            .filter_map(|light| light.def_name.as_deref())
            .collect::<Vec<_>>(),
        common_light_defs
            .iter()
            .map(|def| def.attenuation_image_name.as_deref())
            .collect::<Vec<_>>(),
        draw.light_defs
            .iter()
            .map(|def| def.attenuation_image_name.as_deref())
            .collect::<Vec<_>>(),
    ));
    if let Ok(path) = &zone_ff {
        let mut requested: Vec<(usize, u8)> = draw
            .primary_lights
            .iter()
            .filter_map(|light| Some((light.attenuation_image?, light.attenuation_sampler)))
            .collect();
        if let Some(dynamic) = dynamic_light
            && let Some(image) = dynamic.attenuation_image
        {
            requested.push((image, dynamic.attenuation_sampler));
        }
        let want: std::collections::BTreeSet<usize> =
            requested.iter().map(|(index, _)| *index).collect();
        let want = want.len();
        let stage = progress.begin_scoped(StageId::Images, "attenuation", None);
        let decoded = asset_material::decode_catalog_images_from_iwd(
            path,
            global,
            requested,
            &stage,
            load_pool(),
        );
        stage.finish_from(&decoded);
        match decoded {
            Ok(n) => report.push(format!(
                "IWD light attenuation: decoded {n} of {want} GfxLightDef images (empty payload is not a host ramp)"
            )),
            Err(error) => report.push(format!("IWD light attenuation: {error}")),
        }
    }

    asset_world::resolve_primary_light_attenuation(draw, global, common_light_defs);
    let resolved_dynamic = dynamic_light_name.and_then(|name| {
        asset_world::resolve_named_light_def(name, &draw.light_defs, common_light_defs, global)
    });
    let dynamic_decoded = resolved_dynamic
        .and_then(|light| light.attenuation_image)
        .is_some_and(|index| {
            global
                .images
                .get(index)
                .is_some_and(|image| image.decoded.is_some())
        });
    report.push(format!(
        "FX light_dynamic: image={:?} decoded={} width={:?} sampler={}",
        resolved_dynamic.and_then(|light| light.attenuation_image),
        dynamic_decoded,
        resolved_dynamic.and_then(|light| light.falloff_image_width),
        resolved_dynamic.map_or(0, |light| light.attenuation_sampler),
    ));
    let dynamic_light =
        resolved_dynamic.filter(|light| dynamic_decoded && light.falloff_image_width.is_some());
    if dynamic_light_name.is_some() && dynamic_light.is_none() {
        report.push(
            "FX light_dynamic GAP: light definition or decoded attenuation image missing; additional FX lights unavailable"
                .into(),
        );
    }
    dynamic_light
}

/// bo2mp: the Pack-a-Punch upgrade of each guest gun the Mystery Box can
/// give on this Zombies map (`bo2_profile::guest::zm_box_guns`, as the
/// box's list is made): its own game's upgraded weapon when that game has
/// a Pack-a-Punch and the weapon is loaded (shown by our key for its
/// name), else a Black Ops II-style copy of the gun
/// (`WeaponBuild::add_upgrade_copies`; its damage and ammo are the combat
/// table's, `session::combat_table::zombies_guest_upgrades`).
fn guest_zm_upgrades(weapons: &mut asset_game::WeaponBuild, report: &mut Vec<String>) -> Vec<bo2_profile::guest::ZmUpgrade> {
    use asset_core::AssetNamespace as Ns;
    use bo2_profile::guest;
    let n = weapons.len() as u32;
    let zm: std::collections::HashSet<String> = (1..n)
        .filter(|&i| weapons.namespace_of(i) == Some(Ns::T6))
        .map(|i| weapons.script_name_of(i))
        .filter(|name| {
            let name = name.split('+').next().unwrap_or(name);
            name.strip_suffix("_mp").unwrap_or(name).ends_with("_zm")
        })
        .map(|name| guest::family_of_zm_weapon(&name))
        .collect();
    let index = |w: &asset_game::WeaponBuild, name: &str| w.resolve_index(name).ok().flatten().unwrap_or(0);
    let guns = guest::zm_box_guns(&zm, |g| index(weapons, &guest::script_name(g)) != 0);
    let mut out = Vec::new();
    let mut copies: Vec<(u32, String, String)> = Vec::new();
    let mut copied: Vec<&guest::GuestGun> = Vec::new();
    let (mut own, mut keyed) = (Vec::new(), Vec::new());
    for gun in guns {
        let base = guest::script_name(gun);
        let base_id = index(weapons, &base);
        // (The mod's guns are Black Ops II weapons.)
        let ns = if gun.game == guest::MOD_GAME { Some(Ns::T6) } else { Ns::parse(gun.game) };
        let real = ns
            .filter(|_| guest::has_own_pack_a_punch(gun.game))
            .and_then(|ns| {
                guest::zm_upgrade_assets(gun.game, gun.gun).into_iter().find_map(|name| {
                    let key = asset_core::AssetKey::new(ns, asset_core::AssetKind::Weapon, name).ok()?;
                    weapons.resolve_key(&key).ok().flatten().filter(|&id| id != 0 && id != base_id)
                })
            });
        match real {
            Some(id) => {
                keyed.push((id, guest::zm_upgrade_key(gun)));
                let upgraded = weapons.script_name_of(id).to_ascii_lowercase();
                own.push(upgraded.clone());
                out.push(guest::ZmUpgrade {
                    base,
                    upgraded,
                    made: false,
                    bo2_camo: guest::own_upgrade_wears_bo2_camo(gun.game),
                });
            }
            None => {
                copies.push((base_id, guest::zm_upgrade_copy_name(weapons.name_of(base_id)), guest::zm_upgrade_key(gun)));
                copied.push(gun);
            }
        }
    }
    for (id, key) in keyed {
        weapons.set_display_name_key(id, &key);
    }
    let ids = weapons.add_upgrade_copies(&copies);
    for (gun, id) in copied.into_iter().zip(ids) {
        if id == 0 {
            continue;
        }
        out.push(guest::ZmUpgrade {
            base: guest::script_name(gun),
            upgraded: weapons.script_name_of(id).to_ascii_lowercase(),
            made: true,
            bo2_camo: true,
        });
    }
    report.push(format!(
        "bo2mp zombies Pack-a-Punch: {} guest guns: {} own games' upgrades ({}), {} made Black Ops II-style",
        out.len(),
        own.len(),
        own.join(" "),
        out.iter().filter(|u| u.made).count()
    ));
    out
}

#[allow(clippy::too_many_arguments)]

/// bo2mp: the other games' guns' camos as material swaps (see
/// `asset_game::weapon_camo_models`): a camo model with the gun model's
/// surfaces (vertex ranges and index count alike) gives, per surface whose
/// material differs, (camo slot, gun's material) -> the camo's material.
fn guest_camo_swaps(fpv: &FpvMeshBuild, world: &WorldWeaponBuild, report: &mut Vec<String>) {
    use asset_core::weapon_camos::MaterialCamo;
    let mut swaps: std::collections::HashMap<(u32, u32), MaterialCamo> = std::collections::HashMap::new();
    let (mut guns, mut camos, mut refused) = (0usize, 0usize, 0usize);
    let mut pair = |slot: u8, base: Option<(&asset_model::ModelSkel, &[asset_core::AssetEdge<asset_model::MaterialSpace>])>,
                    camo: Option<(&asset_model::ModelSkel, &[asset_core::AssetEdge<asset_model::MaterialSpace>])>|
     -> bool {
        let (Some((bs, be)), Some((cs, ce))) = (base, camo) else { return false };
        if bs.surface_vertex_ranges != cs.surface_vertex_ranges || bs.indices.len() != cs.indices.len() || be.len() != ce.len() {
            return false;
        }
        let mut any = false;
        for (b, c) in be.iter().zip(ce) {
            if let (Some(b), Some(c)) = (b.bound_index(), c.bound_index())
                && b != c
            {
                swaps.insert((u32::from(slot), b as u32), MaterialCamo { material: c as u32, tile: 1.0 });
                any = true;
            }
        }
        any
    };
    for (ns, gun, gun_world, list) in asset_game::weapon_camo_models::all() {
        guns += 1;
        let view_base = fpv.get(ns, &gun).map(|e| (&*e.skel, e.material_edges.as_slice()));
        let world_base = gun_world.as_deref().and_then(|w| world.get(ns, w)).map(|e| (&*e.skel, e.material_edges.as_slice()));
        for (slot, view, world_name) in list {
            let v = view.as_deref().and_then(|n| fpv.get(ns, n)).map(|e| (&*e.skel, e.material_edges.as_slice()));
            let w = world_name.as_deref().and_then(|n| world.get(ns, n)).map(|e| (&*e.skel, e.material_edges.as_slice()));
            let a = pair(slot, view_base, v);
            let b = pair(slot, world_base, w);
            if a || b {
                camos += 1;
            } else {
                refused += 1;
            }
        }
    }
    report.push(format!("guest camos: {camos} camos of {guns} guns as {} material swaps ({refused} not paired)", swaps.len()));
    asset_core::weapon_camos::install_guest(swaps);

    // The other games' guns with no camos of their own wear Black Ops II's
    // material camos: every material of their gun models but glass, sights
    // and hands.
    use asset_core::AssetNamespace as Ns;
    let wears = |ns: Ns| matches!(ns, Ns::T5 | Ns::T4 | Ns::Iw3 | Ns::Iw6 | Ns::S1);
    let skip = |name: &str| {
        let n = name.to_ascii_lowercase();
        ["lens", "glass", "reticle", "reflex", "holo", "scope_overlay", "red_dot", "dot_", "hand", "glove", "sleeve", "arm_", "_arms", "emissive", "_glow", "screen", "display", "decal", "sticker", "laser", "flash", "muzzle"]
            .iter()
            .any(|w| n.contains(w))
    };
    let mut wrap = std::collections::HashSet::new();
    let mut take = |keys: &[Option<asset_core::MaterialKey>], edges: &[asset_core::AssetEdge<asset_model::MaterialSpace>]| {
        for (k, e) in keys.iter().zip(edges) {
            if let (Some(k), Some(i)) = (k, e.bound_index())
                && !skip(&k.name)
            {
                wrap.insert(i as u32);
            }
        }
    };
    let mut models = 0usize;
    for i in 0..fpv.len() {
        if let Some(e) = fpv.get_at(i).filter(|e| wears(e.namespace)) {
            take(&e.material_keys, &e.material_edges);
            models += 1;
        }
    }
    for i in 0..world.len() {
        if let Some(e) = world.get_at(i).filter(|e| wears(e.namespace)) {
            take(&e.material_keys, &e.material_edges);
            models += 1;
        }
    }
    report.push(format!("guest camos: Black Ops II's material camos on {} materials of {models} other games' gun models", wrap.len()));
    asset_core::weapon_camos::install_wrap(wrap);
    // bo2mp: Zombies' Pack-a-Punch camo on the guest guns' upgrades draws
    // over every other game's gun, Modern Warfare 2's and 3's as well.
    let mut pap = std::collections::HashSet::new();
    let mut take_all = |keys: &[Option<asset_core::MaterialKey>], edges: &[asset_core::AssetEdge<asset_model::MaterialSpace>]| {
        for (k, e) in keys.iter().zip(edges) {
            if let (Some(k), Some(i)) = (k, e.bound_index())
                && !skip(&k.name)
            {
                pap.insert(i as u32);
            }
        }
    };
    if !bo2_profile::guest::zm_upgrades().is_empty() {
        for i in 0..fpv.len() {
            if let Some(e) = fpv.get_at(i).filter(|e| e.namespace != Ns::T6) {
                take_all(&e.material_keys, &e.material_edges);
            }
        }
        for i in 0..world.len() {
            if let Some(e) = world.get_at(i).filter(|e| e.namespace != Ns::T6) {
                take_all(&e.material_keys, &e.material_edges);
            }
        }
        report.push(format!("guest Pack-a-Punch camo: drawn over {} materials of other games' gun models", pap.len()));
    }
    asset_core::weapon_camos::install_pap_wrap(pap);
}

fn link_materials_after_absorb(
    materials: &asset_material::MaterialDefinitions,
    world: &mut PreparedWorld,
    weapons: &mut WeaponBuild,
    tracers: &mut asset_game::TracerCatalog,
    bodies: &mut asset_model::BodyMeshBuild,
    world_weapons: &mut WorldWeaponBuild,
    fpv_meshes: &mut FpvMeshBuild,
    projectile_meshes: &mut ProjectileMeshBuild,
    report: &mut Vec<String>,
) {
    let fx_model_materials = world.fx_models.resolve_materials(materials);
    report.push(format!(
        "FX model materialHandles after absorb: bound={} unresolved={} absent={}",
        fx_model_materials.bound, fx_model_materials.unresolved, fx_model_materials.absent,
    ));
    weapons.resolve_hud_material_edges(materials);
    if let Some(glass) = world.fx_glass.as_mut() {
        glass.resolve_material_edges(materials);
        let census = glass.material_edge_census();
        report.push(format!(
            "glass Material* after absorb: bound={} unresolved={} absent={}",
            census.bound, census.unresolved, census.absent
        ));
    }
    let hud_materials = weapons.hud_material_edge_census();
    let technique_sets = materials.technique_set_edge_census();
    report.push(format!(
        "material pointer graph after finalize: weapon_hud bound={} unresolved={} absent={}; technique_set bound={} unresolved={} absent={}",
        hud_materials.bound,
        hud_materials.unresolved,
        hud_materials.absent,
        technique_sets.bound,
        technique_sets.unresolved,
        technique_sets.absent,
    ));
    let graph = crate::resolve_after_absorb(
        materials,
        tracers,
        &mut world.fx,
        None,
        Some(bodies),
        Some(world_weapons),
        None,
        Some(fpv_meshes),
        Some(projectile_meshes),
    );
    report.push(format!(
        "asset graph after absorb: tracer_mat bound={} unresolved={} absent={}; fx_elem bound={} unresolved={} absent={}; fx_child bound={} unresolved={} absent={}; fx_runner bound={} unresolved={} absent={}; body materialHandles bound={} unresolved={} absent={}; world-gun materialHandles bound={} unresolved={} absent={}; FPV materialHandles bound={} unresolved={} absent={} (WeaponDef.tracerType stamped at common_mp walk)",
        graph.tracer_materials.bound,
        graph.tracer_materials.unresolved,
        graph.tracer_materials.absent,
        graph.fx_elem_materials.bound,
        graph.fx_elem_materials.unresolved,
        graph.fx_elem_materials.absent,
        graph.fx_nested_children.bound,
        graph.fx_nested_children.unresolved,
        graph.fx_nested_children.absent,
        graph.fx_runner_children.bound,
        graph.fx_runner_children.unresolved,
        graph.fx_runner_children.absent,
        graph.xmodel_body_materials.bound,
        graph.xmodel_body_materials.unresolved,
        graph.xmodel_body_materials.absent,
        graph.xmodel_gun_materials.bound,
        graph.xmodel_gun_materials.unresolved,
        graph.xmodel_gun_materials.absent,
        graph.xmodel_fpv_materials.bound,
        graph.xmodel_fpv_materials.unresolved,
        graph.xmodel_fpv_materials.absent,
    ));
    // bo2mp: Modern Warfare 2's and 3's camos: each camo model of a gun
    // against the gun's own (the same shape), its differing materials as
    // swaps for that camo slot.
    guest_camo_swaps(fpv_meshes, world_weapons, report);
    report.push(format!(
        "Material stamp: iw4={} t5={} iw5={} (name-link identity; colliding T5 names are the later-zone row)",
        materials.namespace_count(asset_core::AssetNamespace::Iw4),
        materials.namespace_count(asset_core::AssetNamespace::T5),
        materials.namespace_count(asset_core::AssetNamespace::Iw5),
    ));
    report.push(format!(
        "fx elem material edges after absorb: {} bound ({} unique), {} unresolved (temp={}, catalog_miss={}), {} absent of {} material visuals; {} decal mark arms ({} Bound slots, {} unresolved, {} temp, {} array-unpatched)",
        world.fx.material_visual_bound_count(),
        world.fx.material_visual_unique_bound_count(),
        world.fx.material_visual_unresolved_count(),
        world.fx.material_visual_unresolved_temp_count(),
        world.fx.material_visual_unresolved_miss_count(),
        world.fx.material_visual_absent_count(),
        world.fx.material_visual_count(),
        world.fx.material_visual_decal_count(),
        world.fx.decal_mark_bound_slot_count(),
        world.fx.decal_mark_unresolved_slot_count(),
        world.fx.decal_mark_temp_slot_count(),
        world.fx.material_visual_decal_unpatched_count()
    ));
    report.push(format!(
        "fx elem decal unique Bound after absorb: {} (mc={}, wc={}); decoded color/2D in catalog {} of {} (not sprite Plan)",
        world.fx.unique_decal_mark_count(),
        world.fx.unique_decal_mark_mc_count(),
        world.fx.unique_decal_mark_wc_count(),
        world.fx.unique_decal_mark_decoded_color_count(materials),
        world.fx.unique_decal_mark_count()
    ));
    let miss = world
        .fx
        .unique_decal_mark_decoded_miss_samples(materials, 8);
    if !miss.is_empty() {
        report.push(format!(
            "fx elem decal catalog nocolor sample: {}",
            miss.join("; ")
        ));
    }
    let samples = world.fx.unresolved_material_samples(8);
    if !samples.is_empty() {
        report.push(format!(
            "fx elem unresolved Material* sample: {}",
            samples.join("; ")
        ));
    }
    let mark_samples = world.fx.decal_mark_samples(8);
    if !mark_samples.is_empty() {
        report.push(format!(
            "fx elem decal mark sample: {}",
            mark_samples.join("; ")
        ));
    }
}

fn decode_fx_colour_maps(
    fx: &FxCatalog,
    global: &mut asset_material::MaterialDefinitions,
    zone_ff: &Result<PathBuf, String>,
    progress: &LoadProgress,
    report: &mut Vec<String>,
) {
    let missing: Vec<asset_core::MaterialKey> = fx
        .unique_bound_hints()
        .into_iter()
        .chain(fx.unique_decal_mark_hints())
        .filter(|(index, _)| !asset_game::color_decoded_in_catalog(global, *index))
        .filter_map(|(index, _)| {
            let material = global.materials.get(index)?;
            Some(asset_core::MaterialKey {
                namespace: material.namespace,
                name: material.name.as_str().to_owned(),
            })
        })
        .collect();
    if !missing.is_empty() {
        if let Ok(path) = zone_ff {
            let stage = progress.begin_scoped(StageId::Images, "fx_elem", None);
            let decoded = asset_material::material_images::decode_color_or_2d_for_keys(
                path,
                global,
                missing,
                &stage,
                load_pool(),
            );
            stage.finish_from(&decoded);
            match decoded {
                Ok(n) => report.push(format!(
                    "fx elem 2d images after absorb: {n} TS_COLOR_MAP/TS_2D decoded"
                )),
                Err(error) => report.push(format!("fx elem 2d images after absorb: {error}")),
            }
        }
    }
    let nocolor: Vec<(usize, String)> = fx
        .unique_bound_hints()
        .into_iter()
        .filter(|(index, _)| !asset_game::color_decoded_in_catalog(global, *index))
        .map(|(index, hint)| (index, hint.to_owned()))
        .collect();
    if !nocolor.is_empty() {
        let (distortion, other): (Vec<_>, Vec<_>) = nocolor
            .into_iter()
            .partition(|(_, hint)| hint.contains("distortion"));
        report.push(format!(
            "fx elem Bound without decoded color: {} of {} unique (distortion={}, other={}) other_sample: {}",
            distortion.len() + other.len(),
            fx.material_visual_unique_bound_count(),
            distortion.len(),
            other.len(),
            if other.is_empty() {
                "-".to_string()
            } else {
                other
                    .iter()
                    .map(|(index, hint)| format!("{index}:{hint}"))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        ));
        for (index, hint) in &other {
            let Some(mat) = global.materials.get(*index) else {
                report.push(format!(
                    "fx elem Bound `{index}:{hint}` has no global material row"
                ));
                continue;
            };
            let sem: Vec<u8> = mat.textures.iter().map(|t| t.semantic).collect();
            let decoded = mat.textures.iter().any(|t| {
                t.image
                    .and_then(|i| global.images.get(i))
                    .is_some_and(|img| img.decoded.is_some())
            });
            report.push(format!(
                "fx elem Bound `{index}:{}` techset={} camera_region={} tex={} sem={sem:?} decoded={decoded}",
                mat.name,
                mat.technique_set,
                mat.camera_region,
                mat.textures.len(),
            ));
        }
    }
}

fn census_image_working_set(
    draw: &WorldDraw,
    world: &PreparedWorld,
    global: &asset_material::MaterialDefinitions,
    map_ids: &[Option<usize>],
    fpv_meshes: &FpvMeshBuild,
    tracers: &asset_game::TracerCatalog,
    report: &mut Vec<String>,
) {
    let world_mats = draw.batches.iter().filter_map(|batch| {
        let local = batch.material?;
        map_ids.get(local).copied().flatten().or(Some(local))
    });

    let smodel_mats = world.static_model_meshes.iter().flat_map(|mesh| {
        mesh.lod_surfaces.iter().flatten().filter_map(|surface| {
            let local = surface.material?;
            map_ids.get(local).copied().flatten()
        })
    });
    let fpv_mats = fpv_meshes.bound_material_indices();
    let fx_mats = world
        .fx
        .unique_bound_hints()
        .into_iter()
        .map(|(index, _)| index)
        .chain(
            world
                .fx
                .unique_decal_mark_hints()
                .into_iter()
                .map(|(index, _)| index),
        )
        .chain(tracers.defs().filter_map(|def| def.material.bound_index()));
    let mut set = asset_material::material_images::census_image_working_set(
        global,
        world_mats,
        smodel_mats,
        fpv_mats,
        fx_mats,
    );
    let (probe_n, probe_bytes) = asset_material::material_images::cpu_image_census(
        world.reflection_probe_images.iter().flatten(),
    );
    let (lightmap_n, lightmap_bytes) = match &draw.lightmap {
        Ok(pages) => asset_material::material_images::cpu_image_census(
            pages.iter().flatten().flat_map(|page| {
                [
                    page.primary_image.as_ref(),
                    page.secondary_image.as_ref(),
                    Some(&page.ambient_image),
                    Some(&page.directional_image),
                    Some(&page.sun_mask_image),
                ]
                .into_iter()
                .flatten()
            }),
        ),
        Err(_) => (0, 0),
    };
    set.probe_n = probe_n;
    set.probe_bytes = probe_bytes;
    set.lightmap_n = lightmap_n;
    set.lightmap_bytes = lightmap_bytes;
    asset_material::material_images::store_image_working_set(set);
    report.push(format!(
        "image working set: decoded={} ({:.1}MiB) world-batch={} ({:.1}MiB) smodel={} ({:.1}MiB) fpv={} ({:.1}MiB) probe={} ({:.1}MiB) lightmap={} ({:.1}MiB) fx={} ({:.1}MiB); subsets, not a skipped load",
        set.decoded_n,
        set.decoded_bytes as f64 / (1024.0 * 1024.0),
        set.world_n,
        set.world_bytes as f64 / (1024.0 * 1024.0),
        set.smodel_n,
        set.smodel_bytes as f64 / (1024.0 * 1024.0),
        set.fpv_n,
        set.fpv_bytes as f64 / (1024.0 * 1024.0),
        set.probe_n,
        set.probe_bytes as f64 / (1024.0 * 1024.0),
        set.lightmap_n,
        set.lightmap_bytes as f64 / (1024.0 * 1024.0),
        set.fx_n,
        set.fx_bytes as f64 / (1024.0 * 1024.0),
    ));
}

const FACTION_TABLE: &str = "mp/factionTable.csv";
const IW4_ALLIES_CHARSET: &str = "us_army";
const IW4_AXIS_CHARSET: &str = "opforce_composite";

fn foreign_faction_rows(
    scripts: &mut crate::ScriptSources,
    teams: &asset_game::MapTeamSettings,
) -> bool {
    let allies = faction_cells(
        &teams.allies_strings,
        teams.allies_name.as_ref(),
        teams.allies.as_ref(),
        teams.allies_color,
    );
    let axis = faction_cells(
        &teams.axis_strings,
        teams.axis_name.as_ref(),
        teams.axis.as_ref(),
        teams.axis_color,
    );
    scripts.set_table_cells(FACTION_TABLE, IW4_ALLIES_CHARSET, &allies)
        & scripts.set_table_cells(FACTION_TABLE, IW4_AXIS_CHARSET, &axis)
}

fn faction_cells(
    strings: &asset_game::FactionStrings,
    short_name: Option<&asset_core::AssetKey>,
    icon: Option<&asset_core::AssetKey>,
    color: Option<[f32; 3]>,
) -> Vec<(usize, String)> {
    let mut cells: Vec<(usize, String)> = [
        (1, strings.name.clone()),
        (2, short_name.map(|k| k.logical_name().to_owned())),
        (3, strings.eliminated.clone()),
        (4, strings.forfeited.clone()),
        (5, icon.map(asset_core::AssetKey::display)),
    ]
    .into_iter()
    .filter_map(|(column, value)| Some((column, value?)))
    .collect();
    if let Some(rgb) = color {
        cells.extend((14..).zip(rgb.map(|c| c.to_string())));
    }
    cells
}

fn t5_map_under_iw4_rules(
    map_scripts: &crate::ScriptSources,
    zone_name: &str,
    facts: &crate::MapFacts,
) -> crate::ScriptSources {
    let module = format!("maps/mp/{zone_name}");
    let main = map_scripts
        .read(&module)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();
    let mut declarations = asset_world::MapDeclarations::from_t5_map_main(&main);
    declarations.set_teams(
        facts.script_sound.attackers.as_deref().unwrap_or("allies"),
        facts.script_sound.defenders.as_deref().unwrap_or("axis"),
    );
    let entities = map_scripts
        .entities()
        .map(asset_world::t5_entities_for_iw4_rules)
        .unwrap_or_default();
    let mut scripts = crate::ScriptSources::default();
    let mut map_main = declarations.map_script(&entities);
    if zone_name == "mp_radiation" {
        let end = map_main.rfind('}').expect("generated map main");
        map_main.insert_str(end, "\tthread iw4l_maps\\radiation::main();\n");
        scripts.insert_source(
            "iw4l_maps/radiation",
            crate::map_scripts::RADIATION.to_owned(),
        );
    }
    scripts.insert_source(&module, map_main);
    scripts.set_entities(entities);
    scripts
}
