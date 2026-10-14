// bo2mp: Multiplayer's custom guns (other games' guns, "guest guns") in the
// Zombies Mystery Box. OUR CODE (Black Ops II's box has only its own guns).
//
// Each gun is registered the way a map registers its box guns
// (_zm_weapons::include_zombie_weapon, then add_zombie_weapon): a box gun's
// price (50, as Black Ops II's box-only guns), its ammo price from it, the
// voice line of its kind, and no upgraded weapon, so the Pack-a-Punch
// refuses it (Black Ops II's can_upgrade_weapon: no upgrade_name). The
// engine gives them Zombies' damage and reserve ammo.
//
// Black Ops II's box picks evenly among the guns in it (marked is_in_box)
// that the player can take. The custom guns are many (well over a hundred)
// and the map's own about twenty: all in at once, the map's guns and its
// wonder weapon would hardly come up. So a few custom guns are in the box
// at a time (`bo2mp_zm_guest_slots`, 6), a new few every second: a custom
// gun comes up about as often as six of the map's guns together, and each
// of them is as likely as any other.
//
// The list comes from the engine (`bo2mp_zm_guest_list`: `name:vox`, the
// loaded custom guns, each kind of gun once and none the map already has);
// Custom Games' CUSTOM WEAPONS turns them off (`bo2mp_zm_guest_guns` 0).

main()
{
    if ( getdvarint( "bo2mp_zm_guest_guns" ) == 0 )
        return;

    list = strtok( getdvar( "bo2mp_zm_guest_list" ), " " );
    if ( list.size == 0 )
        return;

    // After the map has registered its own weapons.
    for ( i = 0; !isdefined( level.zombie_weapons ) && i < 200; i++ )
        wait 0.05;

    waittillframeend;
    if ( !isdefined( level.zombie_weapons ) )
        return;

    level.bo2mp_guest_guns = [];
    level.bo2mp_guest_gun = [];
    foreach ( entry in list )
    {
        parts = strtok( entry, ":" );
        name = parts[ 0 ];
        vox = "";
        if ( parts.size > 1 )
            vox = parts[ 1 ];

        if ( isdefined( level.zombie_weapons[ name ] ) )
            continue;

        if ( !register_gun( name, vox ) )
            continue;

        level.bo2mp_guest_guns[ level.bo2mp_guest_guns.size ] = name;
        level.bo2mp_guest_gun[ name ] = 1;
    }

    if ( level.bo2mp_guest_guns.size > 0 )
        level thread box_slots();
}

// One gun as a map registers a box gun; not in the box until box_slots
// puts it there. True when it is registered.
register_gun( name, vox )
{
    maps/mp/zombies/_zm_weapons::include_zombie_weapon( name, 0 );
    if ( !isdefined( level.zombie_include_weapons ) )
        level.zombie_include_weapons = [];

    level.zombie_include_weapons[ name ] = 0;
    if ( isdefined( level.weapon_weighting_funcs ) && !isdefined( level.weapon_weighting_funcs[ name ] ) )
        level.weapon_weighting_funcs[ name ] = ::guest_weight;

    maps/mp/zombies/_zm_weapons::add_zombie_weapon( name, undefined, &"ZOMBIE_WEAPONCOSTAMMO", 50, vox, "", undefined );

    // (The fields add_zombie_weapon sets, should it not have.)
    if ( !isdefined( level.zombie_weapons[ name ] ) )
    {
        s = spawnstruct();
        s.weapon_name = name;
        s.weapon_classname = "weapon_" + name;
        s.hint = &"ZOMBIE_WEAPONCOSTAMMO";
        s.cost = 50;
        s.vox = vox;
        s.vox_response = "";
        s.ammo_cost = 30;
        level.zombie_weapons[ name ] = s;
    }

    level.zombie_weapons[ name ].upgrade_name = undefined;
    level.zombie_weapons[ name ].is_in_box = 0;
    return 1;
}

// The box weighs each gun the same (a map's weighting function).
guest_weight()
{
    return 1;
}

// A few custom guns in the box at a time, a new few every second.
box_slots()
{
    level endon( "end_game" );
    slots = getdvarint( "bo2mp_zm_guest_slots" );
    if ( slots <= 0 )
        slots = 6;

    guns = level.bo2mp_guest_guns;
    if ( slots > guns.size )
        slots = guns.size;

    on = [];
    while ( 1 )
    {
        foreach ( name in on )
        {
            if ( isdefined( level.zombie_weapons[ name ] ) )
                level.zombie_weapons[ name ].is_in_box = 0;
        }

        on = [];
        for ( tries = 0; on.size < slots && tries < slots * 4; tries++ )
        {
            name = guns[ randomint( guns.size ) ];
            if ( isdefined( level.zombie_weapons[ name ] ) && !level.zombie_weapons[ name ].is_in_box )
            {
                level.zombie_weapons[ name ].is_in_box = 1;
                on[ on.size ] = name;
            }
        }

        wait 1;
    }
}
