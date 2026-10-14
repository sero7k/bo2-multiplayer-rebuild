// bo2mp: the map script of a Zombies map from another game (Infinite
// Warfare's cp_zmb, cp_rave...), linked as maps/mp/<map>. OUR CODE: such a
// map has no Black Ops II map script; this one sets the level up the way
// Black Ops II's own Zombies maps do (zm_buried.gsc, zm_nuked.gsc: its
// game mode and start location, _load, the perks and weapons it uses,
// _zm::init, its zones), on Buried's Zombies kit (its zombies, barriers,
// box, machines and guns), whose zones the map loads.
//
// The map's layout comes as entities and structs Black Ops II's scripts
// read (zones, spawners, exterior_goal, zombie_debris, zm_perk_machine,
// treasure_chest_use, weapon_upgrade: assets::lane::iw7_zombies), and
// these of ours:
//   iw7_zone_link     script_noteworthy "<zone> <zone>", script_flag: two
//                     zones joined once the flag is set (always_on: open)
//   iw7_start_zone    script_noteworthy: a zone open from the start
//   iw7_power_switch  a generator: any one turns the power on
//   iw7_portal        a portal (target: where it lands players); the main
//                     street's sends players to Pack-a-Punch's room
//                     (iw7_pap_room) for 30 s, the others to the main
//                     street's, once the power is on.

#include common_scripts\utility;
#include maps\mp\_utility;
#include maps\mp\zombies\_zm_utility;
#include maps\mp\gametypes_zm\_zm_gametype;
#include maps\mp\zombies\_zm_zonemgr;

gamemode_callback_setup()
{
    add_map_gamemode( "zclassic", ::zclassic_preinit, undefined, undefined );
    add_map_location_gamemode( "zclassic", "iw7", ::location_precache, ::location_main );
}

zclassic_preinit()
{
    level.precachecustomcharacters = maps\mp\zm_buried::precache_personality_characters;
    level.givecustomcharacters = maps\mp\zm_buried::give_personality_characters;
    level.setupcustomcharacterexerts = maps\mp\zm_buried::setup_personality_character_exerts;
}

main()
{
    level.default_game_mode = "zclassic";
    level.default_start_location = "iw7";
    setup_rex_starts();
    maps\mp\zombies\_zm::init_fx();
    maps\mp\animscripts\zm_death::precache_gib_fx();
    level.zombiemode = 1;
    level._no_water_risers = 1;
    maps\mp\zombies\_load::main();
    maps\mp\teams\_teamset_cdc::level_init();
    maps\mp\gametypes_zm\_spawning::level_use_unified_spawning( 1 );
    level.givecustomloadout = maps\mp\zm_buried::givecustomloadout;
    level.initial_round_wait_func = ::initial_round_wait_func;
    level.zombiemode_using_pack_a_punch = 1;
    level.zombiemode_reusing_pack_a_punch = 1;
    level.pap_interaction_height = 47;
    level.zombiemode_using_doubletap_perk = 1;
    level.zombiemode_using_juggernaut_perk = 1;
    level.zombiemode_using_revive_perk = 1;
    level.zombiemode_using_sleightofhand_perk = 1;
    level.zombiemode_using_additionalprimaryweapon_perk = 1;
    level.zombiemode_using_marathon_perk = 1;
    level.register_offhand_weapons_for_level_defaults_override = ::offhand_weapon_override;
    level.zombiemode_offhand_weapon_give_override = maps\mp\zm_buried::offhand_weapon_give_override;
    level._zombie_custom_add_weapons = maps\mp\zm_buried::custom_add_weapons;
    level._allow_melee_weapon_switching = 1;
    level.custom_ai_type = [];
    maps\mp\zm_buried::include_weapons();
    include_powerups();
    maps\mp\zm_buried::init_level_specific_wall_buy_fx();
    maps\mp\zombies\_zm::init();
    maps\mp\zombies\_zm_weap_bowie::init();
    maps\mp\zombies\_zm_weap_cymbal_monkey::init();
    maps\mp\zombies\_zm_weap_claymore::init();
    maps\mp\zombies\_zm_weap_ballistic_knife::init();
    maps\mp\zombies\_zm_weap_slowgun::init();
    maps\mp\zombies\_zm_weap_tazer_knuckles::init();
    precacheitem( "death_throe_zm" );
    level.zones = [];
    level.zone_manager_init_func = ::foreign_zone_init;
    init_zones = [];
    foreach ( s in getstructarray( "iw7_start_zone", "targetname" ) )
        init_zones[ init_zones.size ] = s.script_noteworthy;
    level thread maps\mp\zombies\_zm_zonemgr::manage_zones( init_zones );
    level.zombie_ai_limit = 24;
    level.speed_change_round = 15;
    level.speed_change_max = 5;
    level.calc_closest_player_using_paths = 1;
}

location_precache()
{
}

location_main()
{
    level.buildables_built[ "pap" ] = 1;
    maps\mp\gametypes_zm\_zm_gametype::setup_standard_objects( "iw7" );
    maps\mp\zombies\_zm_game_module::set_current_game_module( level.game_module_standard_index );
    level thread power_switches();
    level thread portals();
}

setup_rex_starts()
{
    add_gametype( "zclassic", ::dummy, "zclassic", ::dummy );
    add_gameloc( "iw7", ::dummy, "iw7", ::dummy );
}

dummy()
{
}

initial_round_wait_func()
{
    flag_wait( "initial_blackscreen_passed" );
}

include_powerups()
{
    include_powerup( "nuke" );
    include_powerup( "insta_kill" );
    include_powerup( "double_points" );
    include_powerup( "full_ammo" );
    include_powerup( "carpenter" );
    include_powerup( "fire_sale" );
}

offhand_weapon_override()
{
    register_lethal_grenade_for_level( "frag_grenade_zm" );
    level.zombie_lethal_grenade_player_init = "frag_grenade_zm";
    register_tactical_grenade_for_level( "cymbal_monkey_zm" );
    register_placeable_mine_for_level( "claymore_zm" );
    register_melee_weapon_for_level( "knife_zm" );
    register_melee_weapon_for_level( "bowie_knife_zm" );
    register_melee_weapon_for_level( "tazer_knuckles_zm" );
    level.zombie_melee_weapon_player_init = "knife_zm";
    level.zombie_equipment_player_init = undefined;
}

// The zones and what joins them.
foreign_zone_init()
{
    flag_init( "always_on" );
    flag_set( "always_on" );
    foreach ( link in getstructarray( "iw7_zone_link", "targetname" ) )
    {
        zones = strtok( link.script_noteworthy, " " );
        if ( zones.size == 2 )
            add_adjacent_zone( zones[ 0 ], zones[ 1 ], link.script_flag );
    }
}

// The power: any generator turns it on, as a map's one switch does
// (zm_buried_power::electric_switch).
power_switches()
{
    switches = getstructarray( "iw7_power_switch", "targetname" );
    if ( switches.size == 0 )
    {
        flag_set( "power_on" );
        return;
    }
    trigs = [];
    foreach ( s in switches )
    {
        trig = spawn( "trigger_radius_use", s.origin + vectorscale( ( 0, 0, 1 ), 8 ), 0, 64, 96 );
        trig setcursorhint( "HINT_NOICON" );
        trig sethintstring( &"ZOMBIE_ELECTRIC_SWITCH" );
        trig triggerignoreteam();
        trig thread power_switch_think();
        trigs[ trigs.size ] = trig;
    }
    level waittill( "iw7_power_switch", user, where );
    foreach ( trig in trigs )
        trig delete();
    playsoundatposition( "zmb_switch_flip", where );
    playsoundatposition( "zmb_poweron", where );
    if ( isdefined( user ) )
        user thread maps\mp\zombies\_zm_audio::create_and_play_dialog( "power", "power_on" );
    level thread maps\mp\zombies\_zm_perks::perk_unpause_all_perks();
    level notify( "electric_door" );
    clientnotify( "power_on" );
    flag_set( "power_on" );
    level setclientfield( "zombie_power_on", 1 );
}

power_switch_think()
{
    level endon( "iw7_power_switch" );
    while ( 1 )
    {
        self waittill( "trigger", who );
        if ( !is_player_valid( who ) )
            continue;
        level notify( "iw7_power_switch", who, self.origin );
        return;
    }
}

// The portals: once the power is on, each sends players to the main
// street's; that one to Pack-a-Punch's room, and back after 30 seconds.
portals()
{
    list = getstructarray( "iw7_portal", "targetname" );
    if ( list.size == 0 )
        return;
    hub = list[ 0 ];
    foreach ( p in list )
    {
        p.dests = [];
        if ( isdefined( p.target ) )
            p.dests = getstructarray( p.target, "targetname" );
        if ( isdefined( p.script_noteworthy ) && p.script_noteworthy == "main_street" )
            hub = p;
        p.trig = spawn( "trigger_radius_use", p.origin, 0, 48, 72 );
        p.trig setcursorhint( "HINT_NOICON" );
        p.trig sethintstring( &"ZOMBIE_NEED_POWER" );
        p.trig triggerignoreteam();
    }
    level.iw7_pap_room = getstructarray( "iw7_pap_room", "targetname" );
    flag_wait( "power_on" );
    foreach ( p in list )
    {
        p.trig sethintstring( "Press ^3[{+activate}]^7 to use the portal" );
        p thread portal_think( hub );
    }
}

portal_think( hub )
{
    while ( 1 )
    {
        self.trig waittill( "trigger", who );
        if ( !is_player_valid( who ) )
            continue;
        if ( self == hub )
        {
            if ( level.iw7_pap_room.size > 0 )
                who thread pap_room_visit( hub );
        }
        else
            who portal_to( hub.dests );
        wait 1;
    }
}

portal_to( dests )
{
    if ( !isdefined( dests ) || dests.size == 0 )
        return;
    d = dests[ randomint( dests.size ) ];
    self setorigin( d.origin );
    self setplayerangles( d.angles );
}

pap_room_visit( hub )
{
    self endon( "disconnect" );
    self portal_to( level.iw7_pap_room );
    wait 30;
    if ( isalive( self ) )
        self portal_to( hub.dests );
}
