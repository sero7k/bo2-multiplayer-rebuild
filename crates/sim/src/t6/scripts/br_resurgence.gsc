// bo2mp: Resurgence (Warzone's small-map battle royale) on Black Ops II's
// free-for-all. OUR CODE. Squads, a gas circle that closes in phases,
// redeploys while a squadmate lives (off in the last circles), a squad
// with nobody alive is out, the last squad standing wins. Armor plates
// over Black Ops II's own health, cash, buy stations (loadout drop,
// self-revive kit, UAV, squad buyback, plates) and ground loot by rarity.
//
// Off unless `scr_br 1`. Every number is a dvar written by the engine
// from one table (`scr_br_*`, gamemode_iw4::resurgence); every sound is
// `scr_br_snd_<event>` (Modern Warfare III's alias once it can be read,
// Black Ops II's stand-in until then, "" for none).

main()
{
    if ( getdvarint( "scr_br" ) == 0 )
        return;

    br_read_rules();
    level.br = 1;
    level.br_squads = [];
    level.br_out_order = [];
    level.br_live = 0;
    level.br_over = 0;
    level.br_objid = 20;
    level.br_loot = [];
    level.br_stations = [];
    level.br_drops = [];
    level.killstreaksenabled = 0;

    precachemodel( "t6_wpn_supply_drop_ally" );
    precachemodel( "t6_wpn_supply_drop_axis" );

    // (Black Ops II's free-for-all would end the match on its last player
    // or on everyone dead; the squads decide here.)
    level.ononeleftevent = ::br_blank;
    level.ondeadevent = ::br_blank;
    level.playermayspawn = ::br_mayspawn;
    // (Black Ops II's damage callback with the squads and the plates in
    // front of it; its own hook for a game type when there is none yet.)
    if ( isdefined( level.callbackplayerdamage ) )
    {
        level.br_prev_damage = level.callbackplayerdamage;
        level.callbackplayerdamage = ::br_onplayerdamage;
    }
    else
        level.onplayerdamage = ::br_damage_hook;

    br_spots();
    level thread br_start_clock();
    level thread br_onconnect();

    foreach ( player in level.players )
        player thread br_player_init();

    level thread br_circle();
    level thread br_place_stations();
    level thread br_place_loot();
    level thread br_check_squads();
    level thread br_ambience();
}

br_blank( a, b, c, d )
{
}

br_dvar_int( name, fallback )
{
    if ( getdvar( name ) == "" )
        return fallback;

    return getdvarint( name );
}

br_read_rules()
{
    level.br_squad_size = br_dvar_int( "scr_br_squad", 4 );
    level.br_infil_ms = br_dvar_int( "scr_br_infil_ms", 15000 );
    level.br_health = br_dvar_int( "scr_br_health", 100 );
    level.br_plate_hp = br_dvar_int( "scr_br_plate_hp", 50 );
    level.br_plate_slots = br_dvar_int( "scr_br_plate_slots", 3 );
    level.br_plates_max = br_dvar_int( "scr_br_plates_max", 8 );
    level.br_plate_ms = br_dvar_int( "scr_br_plate_ms", 1200 );
    level.br_spawn_plates = br_dvar_int( "scr_br_spawn_plates", 1 );
    level.br_start_cash = br_dvar_int( "scr_br_start_cash", 0 );
    level.br_kill_cash = br_dvar_int( "scr_br_kill_cash", 400 );
    level.br_kill_cut_ms = br_dvar_int( "scr_br_kill_cut_ms", 5000 );
    level.br_station_count = br_dvar_int( "scr_br_stations", 3 );
    level.br_uav_ms = br_dvar_int( "scr_br_uav_ms", 30000 );
    level.br_loot_count = br_dvar_int( "scr_br_loot_spots", 48 );

    level.br_stacks = [];
    foreach ( s in strtok( getdvar( "scr_br_stacks" ), " " ) )
        level.br_stacks[level.br_stacks.size] = int( s );

    if ( level.br_stacks.size == 0 )
        level.br_stacks[0] = 100;

    // A phase: wait from, close from, closed at (ms from the start), end
    // radius share, gas damage a second, redeploy wait in ms (-1: none).
    level.br_phases = [];
    n = br_dvar_int( "scr_br_phases", 0 );

    for ( i = 0; i < n; i++ )
    {
        f = strtok( getdvar( "scr_br_phase_" + i ), " " );

        if ( f.size < 6 )
            continue;

        p = spawnstruct();
        p.wait_from = int( f[0] );
        p.close_from = int( f[1] );
        p.closed = int( f[2] );
        p.share = float( f[3] );
        p.dps = int( f[4] );
        p.redeploy = int( f[5] );
        level.br_phases[level.br_phases.size] = p;
    }

    level.br_buys = [];
    n = br_dvar_int( "scr_br_buys", 0 );

    for ( i = 0; i < n; i++ )
    {
        f = strtok( getdvar( "scr_br_buy_" + i ), "|" );

        if ( f.size < 3 )
            continue;

        b = spawnstruct();
        b.key = f[0];
        b.label = f[1];
        b.price = int( f[2] );
        level.br_buys[level.br_buys.size] = b;
    }

    level.br_tiers = [];
    level.br_tier_total = 0;
    n = br_dvar_int( "scr_br_tiers", 0 );

    for ( i = 0; i < n; i++ )
    {
        f = strtok( getdvar( "scr_br_tier_" + i ), "|" );

        if ( f.size < 4 )
            continue;

        t = spawnstruct();
        t.key = f[0];
        t.label = f[1];
        t.weight = int( f[2] );
        t.attachments = int( f[3] );
        level.br_tiers[level.br_tiers.size] = t;
        level.br_tier_total = level.br_tier_total + t.weight;
    }

    level.br_phase = -1;
    level.br_start_ms = gettime();
}

// The clock starts when everyone may move (the prematch countdown over).
br_start_clock()
{
    while ( isdefined( level.inprematchperiod ) && level.inprematchperiod )
        wait 0.05;

    level.br_start_ms = gettime();
    level.br_started = 1;
    level notify( "br_started" );
}

br_wait_start()
{
    if ( !isdefined( level.br_started ) )
        level waittill( "br_started" );
}

// ---- sounds -----------------------------------------------------------------

br_snd_name( key )
{
    return getdvar( "scr_br_snd_" + key );
}

br_snd( key )
{
    name = br_snd_name( key );

    if ( name != "" )
        self playlocalsound( name );
}

br_snd_all( key )
{
    foreach ( player in level.players )
        player br_snd( key );
}

br_snd_squad( squad, key )
{
    foreach ( player in br_squad_members( squad ) )
        player br_snd( key );
}

br_ambience()
{
    level endon( "game_ended" );
    br_wait_start();
    br_snd_all( "ambience" );
    br_snd_all( "infil_music" );
}

// ---- squads -----------------------------------------------------------------

br_onconnect()
{
    for (;;)
    {
        level waittill( "connected", player );
        player thread br_player_init();
    }
}

br_player_init()
{
    self endon( "disconnect" );

    if ( isdefined( self.br_squad ) )
        return;

    self br_assign_squad();
    self.br_cash = level.br_start_cash;
    self.br_plates = 0;
    self.br_armor = 0;
    self.br_plates_carried = 0;
    self.br_out = 0;
    self.br_pending = 0;
    self.br_selfrevive = 0;
    self.br_buy_index = 0;
    self.br_kit = undefined;
    self br_hud_init();
    self thread br_life();
    self thread br_death_watch();
    self thread br_plates_input();
    self thread br_hud_think();

    if ( self istestclient() )
        self thread br_bot_think();
}

// Humans first, then bots, as they come in: squad after squad.
br_assign_squad()
{
    for ( i = 0; i < level.br_squads.size; i++ )
    {
        if ( level.br_squads[i].size < level.br_squad_size )
        {
            self br_join_squad( i );
            return;
        }
    }

    self br_join_squad( level.br_squads.size );
}

br_join_squad( i )
{
    if ( !isdefined( level.br_squads[i] ) )
        level.br_squads[i] = [];

    level.br_squads[i][level.br_squads[i].size] = self;
    self.br_squad = i;
}

br_squad_members( squad )
{
    out = [];

    if ( !isdefined( level.br_squads[squad] ) )
        return out;

    foreach ( p in level.br_squads[squad] )
    {
        if ( isdefined( p ) )
            out[out.size] = p;
    }

    return out;
}

br_same_squad( a, b )
{
    return isdefined( a.br_squad ) && isdefined( b.br_squad ) && a.br_squad == b.br_squad;
}

br_squad_alive( squad )
{
    n = 0;

    foreach ( p in br_squad_members( squad ) )
    {
        if ( isalive( p ) )
            n++;
    }

    return n;
}

br_squad_pending( squad )
{
    foreach ( p in br_squad_members( squad ) )
    {
        if ( p.br_pending )
            return 1;
    }

    return 0;
}

br_squad_out( squad )
{
    foreach ( s in level.br_out_order )
    {
        if ( s == squad )
            return 1;
    }

    return 0;
}

br_squads_standing()
{
    n = 0;

    for ( i = 0; i < level.br_squads.size; i++ )
    {
        if ( br_squad_members( i ).size > 0 && !br_squad_out( i ) )
            n++;
    }

    return n;
}

// A squad with nobody alive is out (its redeploys with it); the last one
// standing wins.
br_check_squads()
{
    level endon( "game_ended" );
    br_wait_start();
    wait level.br_infil_ms / 1000;
    level.br_live = 1;

    for (;;)
    {
        wait 0.25;

        for ( i = 0; i < level.br_squads.size; i++ )
        {
            if ( br_squad_out( i ) || br_squad_members( i ).size == 0 )
                continue;

            if ( br_squad_alive( i ) == 0 && !( level.br_squad_size == 1 && br_squad_pending( i ) ) )
                br_squad_wiped( i );
        }

        standing = br_squads_standing();

        if ( standing == 0 )
        {
            br_end( -1 );
            return;
        }

        if ( standing == 1 && level.br_squads.size > 1 )
        {
            winner = -1;

            for ( i = 0; i < level.br_squads.size; i++ )
            {
                if ( !br_squad_out( i ) && br_squad_members( i ).size > 0 )
                    winner = i;
            }

            br_end( winner );
            return;
        }
    }
}

br_squad_wiped( squad )
{
    level.br_out_order[level.br_out_order.size] = squad;

    foreach ( p in br_squad_members( squad ) )
    {
        p.br_out = 1;
        p.br_pending = 0;
        p notify( "br_redeploy_cancel" );
    }

    br_snd_squad( squad, "squad_wiped" );
    left = br_squads_standing();

    foreach ( player in level.players )
    {
        if ( !isdefined( player.br_squad ) || player.br_squad == squad )
            continue;

        player br_snd( "enemy_squad_wiped" );
        player br_hud_flash( "SQUAD ELIMINATED  -  " + left + " SQUADS LEFT" );
    }

    foreach ( p in br_squad_members( squad ) )
        p br_hud_flash( "YOUR SQUAD WAS ELIMINATED" );

    if ( left == 3 || left == 2 )
        br_snd_all( "top_squads" );
    else
        br_snd_all( "squads_remaining" );
}

br_end( winner )
{
    if ( level.br_over )
        return;

    level.br_over = 1;
    first = undefined;

    foreach ( p in br_squad_members( winner ) )
    {
        if ( !isdefined( first ) || isalive( p ) )
            first = p;
    }

    // (The free-for-all end screen: the winning squad first, then the
    // squads by how long they lasted; a row's score is its place.)
    num = -1;

    if ( isdefined( first ) )
        num = first getentitynumber();

    reason = "SQUAD " + ( winner + 1 ) + " WINS";

    if ( winner < 0 )
        reason = "NO SQUAD SURVIVED";

    s = "" + num + "|br|" + reason + "|";
    order = [];

    if ( winner >= 0 )
        order[0] = winner;

    for ( i = level.br_out_order.size - 1; i >= 0; i-- )
        order[order.size] = level.br_out_order[i];

    for ( k = 0; k < order.size; k++ )
    {
        foreach ( p in br_squad_members( order[k] ) )
            s = s + p getentitynumber() + "," + ( k + 1 ) + "," + p.name + ";";
    }

    level.bo2mp_outcome = s;

    foreach ( player in level.players )
    {
        if ( isdefined( player.br_squad ) && player.br_squad == winner )
        {
            player br_snd( "victory" );
            player br_snd( "music_exfil" );
        }
        else
            player br_snd( "defeat" );
    }

    thread maps\mp\gametypes\_globallogic::endgame( first, reason );
}

// ---- lives and redeploys ----------------------------------------------------

br_mayspawn()
{
    if ( !isdefined( self.br_squad ) )
        return 1;

    if ( self.br_out )
        return 0;

    if ( !level.br_live )
        return 1;

    return isdefined( self.br_grant ) && self.br_grant;
}

br_life()
{
    self endon( "disconnect" );

    for (;;)
    {
        self waittill( "spawned_player" );
        self.br_grant = 0;
        self.br_pending = 0;
        self.br_last_hurt = 0;
        self thread br_spawned();
    }
}

br_spawned()
{
    self endon( "disconnect" );
    self endon( "death" );
    wait 0.05;

    // (His class as Black Ops II gave it, for the loadout drop; he starts
    // and redeploys with a pistol.)
    self.br_kit = self getweaponslist();
    self.br_kit_primaries = self getweaponslistprimaries();
    self takeallweapons();
    self giveweapon( "knife_mp" );
    self giveweapon( "fiveseven_mp" );
    self givemaxammo( "fiveseven_mp" );
    self switchtoweapon( "fiveseven_mp" );
    self.maxhealth = level.br_health;
    self.health = level.br_health;
    self.br_plates = level.br_spawn_plates;

    if ( self.br_plates > level.br_plate_slots )
        self.br_plates = level.br_plate_slots;

    self.br_armor = self.br_plates * level.br_plate_hp;
    spot = self br_spawn_spot();

    if ( isdefined( spot ) )
    {
        self setorigin( spot.origin );
        self setplayerangles( spot.angles );
    }

    if ( isdefined( self.br_redeployed ) && self.br_redeployed )
    {
        self.br_redeployed = 0;
        self br_snd( "redeployed" );
        self br_hud_flash( "REDEPLOYED" );
    }
    else if ( !level.br_live )
        self br_snd( "match_start" );
}

br_death_watch()
{
    self endon( "disconnect" );

    for (;;)
    {
        self waittill( "death", attacker );
        self br_onplayerkilled( attacker );
    }
}

br_onplayerkilled( attacker )
{
    self.br_plates = 0;
    self.br_armor = 0;

    if ( self.br_cash > 0 )
    {
        level thread br_cash_drop( self.origin, self.br_cash );
        self.br_cash = 0;
    }

    if ( isdefined( attacker ) && isplayer( attacker ) && attacker != self && !br_same_squad( attacker, self ) )
    {
        attacker.br_cash = attacker.br_cash + level.br_kill_cash;
        attacker br_snd( "cash_pickup" );

        // (His squad's dead come back sooner.)
        foreach ( mate in br_squad_members( attacker.br_squad ) )
        {
            if ( mate.br_pending )
                mate.br_redeploy_at = mate.br_redeploy_at - level.br_kill_cut_ms;
        }
    }

    foreach ( mate in br_squad_members( self.br_squad ) )
    {
        if ( mate != self )
            mate br_snd( "squadmate_down" );
    }

    if ( level.br_over || br_squad_out( self.br_squad ) )
        return;

    // (A fall before the drop is over: straight back in.)
    if ( !level.br_live )
    {
        self.br_pending = 1;
        self thread br_redeploy( 3000 );
        return;
    }

    // (A squad with nobody left alive is out; a solo comes back while
    // redeploys are on.)
    if ( level.br_squad_size > 1 && br_squad_alive( self.br_squad ) == 0 )
        return;

    wait_ms = br_redeploy_wait();

    if ( wait_ms < 0 )
    {
        self.br_out = 1;
        self br_snd( "redeploy_disabled" );
        self br_hud_flash( "REDEPLOY DISABLED  -  A SQUADMATE CAN BUY YOU BACK" );
        return;
    }

    self.br_pending = 1;
    self thread br_redeploy( wait_ms );
}

br_redeploy_wait()
{
    if ( level.br_phase < 0 )
    {
        if ( level.br_phases.size == 0 )
            return 10000;

        return level.br_phases[0].redeploy;
    }

    if ( level.br_phase >= level.br_phases.size )
        return -1;

    return level.br_phases[level.br_phase].redeploy;
}

br_redeploy( wait_ms )
{
    self endon( "disconnect" );
    self endon( "br_redeploy_cancel" );
    level endon( "game_ended" );
    self.br_redeploy_at = gettime() + wait_ms;
    last = -1;

    while ( gettime() < self.br_redeploy_at )
    {
        left = int( ( self.br_redeploy_at - gettime() + 999 ) / 1000 );

        if ( left != last && left <= 3 )
            self br_snd( "redeploy_tick" );

        last = left;
        self.br_hud_big settext( "REDEPLOYING IN " + left );
        self.br_hud_big.alpha = 1;

        // (No squadmate left alive: the squad is out, him with it.)
        if ( level.br_squad_size > 1 && br_squad_alive( self.br_squad ) == 0 )
        {
            self.br_hud_big.alpha = 0;
            self.br_pending = 0;
            return;
        }

        wait 0.1;
    }

    self.br_hud_big.alpha = 0;
    self br_respawn();
}

br_respawn()
{
    if ( isalive( self ) || level.br_over )
        return;

    self.br_pending = 0;
    self.br_out = 0;
    self.br_grant = 1;
    self.br_redeployed = 1;
    self.pers["lives"] = 1;
    self thread [[ level.spawnclient ]]();
}

// A redeploy lands near a squadmate inside the circle when one lives,
// anywhere inside it otherwise.
br_spawn_spot()
{
    mate = undefined;

    foreach ( p in br_squad_members( self.br_squad ) )
    {
        if ( p != self && isalive( p ) && br_inside( p.origin, 0 ) )
            mate = p;
    }

    if ( isdefined( mate ) && level.br_live )
        return br_spot_near( mate.origin, 1200 );

    if ( !level.br_live && isdefined( level.br_squad_spot ) && isdefined( level.br_squad_spot[self.br_squad] ) )
        return br_spot_near( level.br_squad_spot[self.br_squad].origin, 600 );

    return br_spot_in_circle();
}

// ---- damage and armor -------------------------------------------------------

br_onplayerdamage( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset, boneindex )
{
    idamage = self br_damage_hook( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset );

    if ( idamage <= 0 )
        return;

    self [[ level.br_prev_damage ]]( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset, boneindex );
}

// The damage he takes after his squad and his plates (0: none).
br_damage_hook( einflictor, eattacker, idamage, idflags, smeansofdeath, sweapon, vpoint, vdir, shitloc, timeoffset )
{
    if ( isdefined( eattacker ) && isplayer( eattacker ) && eattacker != self && br_same_squad( eattacker, self ) )
        return 0;

    if ( !level.br_live && smeansofdeath != "MOD_TRIGGER_HURT" && smeansofdeath != "MOD_FALLING" )
        return 0;

    // (The gas and falls go past the plates.)
    if ( self.br_armor > 0 && smeansofdeath != "MOD_TRIGGER_HURT" && smeansofdeath != "MOD_FALLING" )
    {
        took = idamage;

        if ( took > self.br_armor )
            took = self.br_armor;

        self.br_armor = self.br_armor - took;
        idamage = idamage - took;
        plates = int( ( self.br_armor + level.br_plate_hp - 1 ) / level.br_plate_hp );

        if ( plates < self.br_plates )
        {
            self.br_plates = plates;

            if ( isdefined( eattacker ) && isplayer( eattacker ) )
                eattacker br_snd( "armor_break" );
        }

        self.br_last_hurt = gettime();

        if ( idamage <= 0 )
        {
            if ( isdefined( eattacker ) && isplayer( eattacker ) )
                eattacker playlocalsound( "mpl_hit_alert" );

            return 0;
        }
    }

    self.br_last_hurt = gettime();

    // (The self-revive kit: a hit that would kill him leaves him up.)
    if ( idamage >= self.health && self.br_selfrevive )
    {
        self.br_selfrevive = 0;
        self.health = self.maxhealth;
        self br_snd( "self_revive" );
        self br_hud_flash( "SELF-REVIVE USED" );
        return 0;
    }

    return idamage;
}

br_insert_plate()
{
    if ( self.br_plates_carried <= 0 || self.br_armor >= level.br_plate_slots * level.br_plate_hp )
        return 0;

    self.br_plates_carried--;
    self.br_armor = self.br_armor + level.br_plate_hp;
    cap = level.br_plate_slots * level.br_plate_hp;

    if ( self.br_armor > cap )
        self.br_armor = cap;

    self.br_plates = int( ( self.br_armor + level.br_plate_hp - 1 ) / level.br_plate_hp );
    self br_snd( "plate_insert" );

    if ( self.br_armor >= cap )
        self br_snd( "plates_full" );

    return 1;
}

// Hold [Use] away from a station or loot: a plate goes in every
// `scr_br_plate_ms`.
br_plates_input()
{
    self endon( "disconnect" );
    level endon( "game_ended" );

    for (;;)
    {
        wait 0.05;

        if ( !isalive( self ) || self istestclient() || !self usebuttonpressed() || isdefined( self.br_near ) )
            continue;

        held = 0;

        while ( isalive( self ) && self usebuttonpressed() && !isdefined( self.br_near ) )
        {
            wait 0.05;
            held = held + 50;

            if ( held >= level.br_plate_ms )
            {
                if ( !self br_insert_plate() )
                    break;

                held = 0;
            }
        }

        while ( self usebuttonpressed() )
            wait 0.05;
    }
}

// ---- the circle -------------------------------------------------------------

// Where players can stand: the map's spawn points (any game type's).
br_spots()
{
    level.br_spot_list = [];

    foreach ( cls in array( "mp_dm_spawn", "mp_tdm_spawn", "mp_tdm_spawn_allies_start", "mp_tdm_spawn_axis_start", "mp_dom_spawn", "mp_sd_spawn_attacker", "mp_sd_spawn_defender", "mp_ctf_spawn_allies", "mp_ctf_spawn_axis" ) )
    {
        foreach ( s in getentarray( cls, "classname" ) )
            level.br_spot_list[level.br_spot_list.size] = s;
    }

    if ( level.br_spot_list.size == 0 && isdefined( level.spawnpoints ) )
        level.br_spot_list = level.spawnpoints;

    sum = ( 0, 0, 0 );

    foreach ( s in level.br_spot_list )
        sum = sum + s.origin;

    if ( level.br_spot_list.size > 0 )
        sum = sum / level.br_spot_list.size;

    level.br_center = sum;
    far = 1024;

    foreach ( s in level.br_spot_list )
    {
        d = distance2d( s.origin, sum );

        if ( d > far )
            far = d;
    }

    level.br_radius0 = far * 1.1 + 256;
    level.br_radius = level.br_radius0;

    // (Each squad starts at a spot of its own, as far apart as they come.)
    level.br_squad_spot = [];

    for ( i = 0; i < 16 && level.br_spot_list.size > 0; i++ )
    {
        best = undefined;
        bestd = -1;

        foreach ( s in level.br_spot_list )
        {
            nearest = 1000000;

            foreach ( t in level.br_squad_spot )
            {
                d = distance2d( s.origin, t.origin );

                if ( d < nearest )
                    nearest = d;
            }

            nearest = nearest + randomfloat( 64 );

            if ( nearest > bestd )
            {
                bestd = nearest;
                best = s;
            }
        }

        level.br_squad_spot[i] = best;
    }
}

br_spot_near( origin, within )
{
    best = undefined;
    bestd = 1000000000;

    foreach ( s in level.br_spot_list )
    {
        d = distance2d( s.origin, origin );

        if ( d > within || !br_inside( s.origin, 0 ) )
            continue;

        d = d + randomfloat( 256 );

        if ( d < bestd )
        {
            bestd = d;
            best = s;
        }
    }

    if ( !isdefined( best ) )
        return br_spot_in_circle();

    return best;
}

br_spot_in_circle()
{
    inside = [];

    foreach ( s in level.br_spot_list )
    {
        if ( br_inside( s.origin, 0 ) )
            inside[inside.size] = s;
    }

    if ( inside.size > 0 )
        return inside[randomint( inside.size )];

    // (None inside: the one nearest the middle.)
    best = undefined;
    bestd = 1000000000;

    foreach ( s in level.br_spot_list )
    {
        d = distance2d( s.origin, level.br_center );

        if ( d < bestd )
        {
            bestd = d;
            best = s;
        }
    }

    return best;
}

br_inside( origin, margin )
{
    return distance2d( origin, level.br_center ) <= level.br_radius - margin;
}

// The circles' middles, picked at the start: each next one inside the one
// before, all closing toward a final spot.
br_plan_circles()
{
    level.br_plan = [];
    c = level.br_center;
    r = level.br_radius0;
    final = c;

    if ( level.br_spot_list.size > 0 )
    {
        pick = level.br_spot_list[randomint( level.br_spot_list.size )];
        final = pick.origin;
    }

    for ( i = 0; i < level.br_phases.size; i++ )
    {
        nr = level.br_radius0 * level.br_phases[i].share;
        room = r - nr;
        to = final - c;
        to = ( to[0], to[1], 0 );
        d = length( to );

        if ( d > room && d > 0 )
            to = to * ( room / d );

        c = c + to;
        p = spawnstruct();
        p.center = c;
        p.radius = nr;
        level.br_plan[i] = p;
        r = nr;
    }
}

br_circle()
{
    level endon( "game_ended" );
    br_wait_start();
    br_plan_circles();
    from_center = level.br_center;
    from_radius = level.br_radius0;
    objective_add( level.br_objid, "active", level.br_center, "compass_waypoint_target" );

    for ( i = 0; i < level.br_phases.size; i++ )
    {
        p = level.br_phases[i];
        plan = level.br_plan[i];

        while ( br_elapsed() < p.wait_from )
            wait 0.25;

        level.br_phase = i;
        level.br_next_center = plan.center;
        level.br_next_radius = plan.radius;
        objective_position( level.br_objid, plan.center );
        level.br_closing = 0;

        if ( i == level.br_phases.size - 1 || p.redeploy < 0 && ( i == 0 || level.br_phases[i - 1].redeploy >= 0 ) )
        {
            br_snd_all( "final_circle" );

            if ( p.redeploy < 0 )
            {
                foreach ( player in level.players )
                    player br_hud_flash( "REDEPLOYS ARE OFF" );
            }
        }

        while ( br_elapsed() < p.close_from )
            wait 0.25;

        level.br_closing = 1;
        br_snd_all( "circle_collapsing" );
        br_snd_all( br_circle_music( i ) );

        foreach ( player in level.players )
            player br_hud_flash( "THE CIRCLE IS CLOSING" );

        span = p.closed - p.close_from;

        if ( span < 1 )
            span = 1;

        while ( br_elapsed() < p.closed )
        {
            t = ( br_elapsed() - p.close_from ) / span;

            if ( t > 1 )
                t = 1;

            level.br_center = from_center + ( plan.center - from_center ) * t;
            level.br_radius = from_radius + ( plan.radius - from_radius ) * t;
            wait 0.1;
        }

        level.br_center = plan.center;
        level.br_radius = plan.radius;
        from_center = plan.center;
        from_radius = plan.radius;
        level.br_closing = 0;
        br_snd_all( "circle_closed" );
    }

    level.br_phase = level.br_phases.size;
}

// The circle's music by how far the match is: low, med, high, then the
// final loop for the last circle.
br_circle_music( i )
{
    n = level.br_phases.size;

    if ( i >= n - 1 )
        return "music_circle_final";

    if ( i * 3 < n )
        return "music_circle_low";

    if ( i * 3 < n * 2 )
        return "music_circle_med";

    return "music_circle_high";
}

br_elapsed()
{
    return gettime() - level.br_start_ms;
}

br_gas_dps()
{
    if ( level.br_phase < 0 )
        return 0;

    i = level.br_phase;

    if ( i >= level.br_phases.size )
        i = level.br_phases.size - 1;

    return level.br_phases[i].dps;
}

// Once a second outside: the gas's damage (past the plates).
br_gas( dps )
{
    if ( !isalive( self ) || br_inside( self.origin, 0 ) || dps <= 0 )
    {
        if ( isdefined( self.br_in_gas ) && self.br_in_gas )
            self.br_in_gas = 0;

        return;
    }

    if ( !isdefined( self.br_in_gas ) || !self.br_in_gas )
    {
        self.br_in_gas = 1;
        self br_snd( "gas_enter" );
    }

    self br_snd( "gas_tick" );
    self dodamage( dps, self.origin, undefined, undefined, "none", "MOD_TRIGGER_HURT" );
}

// ---- cash, loot, buy stations -----------------------------------------------

br_cash_drop( origin, amount )
{
    item = spawnstruct();
    item.kind = "cash";
    item.amount = amount;
    item.origin = origin + ( 0, 0, 8 );
    item.model = spawn( "script_model", item.origin );
    item.model setmodel( getweaponmodel( "briefcase_bomb_mp" ) );
    level.br_loot[level.br_loot.size] = item;
    level thread br_loot_think( item );
}

br_place_loot()
{
    level endon( "game_ended" );
    wait 1;
    guns = array( "fiveseven_mp", "fnp45_mp", "beretta93r_mp", "kard_mp", "mp7_mp", "pdw57_mp", "vector_mp", "insas_mp", "qcw05_mp", "evoskorpion_mp", "peacekeeper_mp", "tar21_mp", "type95_mp", "sig556_mp", "sa58_mp", "hk416_mp", "scar_mp", "saritch_mp", "xm8_mp", "an94_mp", "mk48_mp", "qbb95_mp", "lsat_mp", "hamr_mp", "870mcs_mp", "saiga12_mp", "ksg_mp", "srm1216_mp", "svu_mp", "dsr50_mp", "ballista_mp" );
    atts = array( "reflex", "extclip", "fastads", "silencer", "fmj", "holo", "acog", "grip", "rf", "dualclip" );
    spots = level.br_spot_list;

    for ( i = 0; i < level.br_loot_count && spots.size > 0; i++ )
    {
        s = spots[randomint( spots.size )];
        origin = s.origin + ( randomfloatrange( -96, 96 ), randomfloatrange( -96, 96 ), 12 );
        roll = randomint( 100 );
        item = spawnstruct();
        item.origin = origin;

        if ( roll < 20 )
        {
            item.kind = "cash";
            item.amount = level.br_stacks[randomint( level.br_stacks.size )];
            model = getweaponmodel( "briefcase_bomb_mp" );
        }
        else if ( roll < 45 )
        {
            item.kind = "plates";
            item.amount = 1 + randomint( 3 );
            model = getweaponmodel( "riotshield_mp" );
        }
        else
        {
            item.kind = "gun";
            tier = br_roll_tier();
            name = guns[randomint( guns.size )];
            base = name;

            for ( k = 0; k < tier.attachments; k++ )
                name = name + "+" + atts[randomint( atts.size )];

            item.weapon = name;
            item.tier = tier;
            model = getweaponmodel( base );
        }

        item.model = spawn( "script_model", origin );

        if ( isdefined( model ) && model != "" )
            item.model setmodel( model );

        level.br_loot[level.br_loot.size] = item;
        level thread br_loot_think( item );
    }
}

br_roll_tier()
{
    roll = randomint( level.br_tier_total + 1 );
    acc = 0;

    foreach ( t in level.br_tiers )
    {
        acc = acc + t.weight;

        if ( roll < acc )
            return t;
    }

    return level.br_tiers[level.br_tiers.size - 1];
}

// Cash and plates are taken by walking over them; a gun with [Use] (it
// takes the place of the one in his hands when he has two).
br_loot_think( item )
{
    level endon( "game_ended" );

    for (;;)
    {
        wait 0.1;

        foreach ( player in level.players )
        {
            if ( !isalive( player ) || distancesquared( player.origin, item.origin ) > 64 * 64 )
            {
                if ( isdefined( player.br_near ) && player.br_near == item )
                {
                    player.br_near = undefined;
                    player br_hud_hint( "" );
                }

                continue;
            }

            if ( item.kind == "cash" )
            {
                player.br_cash = player.br_cash + item.amount;
                player br_snd( "cash_pickup" );
                item.model delete();
                return;
            }

            if ( item.kind == "plates" )
            {
                if ( player.br_plates_carried >= level.br_plates_max )
                    continue;

                player.br_plates_carried = player.br_plates_carried + item.amount;

                if ( player.br_plates_carried > level.br_plates_max )
                    player.br_plates_carried = level.br_plates_max;

                player br_snd( "plate_pickup" );
                item.model delete();
                return;
            }

            player.br_near = item;
            player br_hud_hint( "[USE]  " + item.tier.label + " WEAPON" );

            if ( player usebuttonpressed() && !player istestclient() || player istestclient() && !isdefined( player.br_bot_gun ) )
            {
                player br_take_gun( item.weapon );

                if ( item.tier.key == "legendary" || item.tier.key == "epic" )
                    player br_snd( "loot_legendary" );
                else if ( item.tier.key == "rare" )
                    player br_snd( "loot_rare" );
                else
                    player br_snd( "loot_common" );

                player.br_near = undefined;
                player br_hud_hint( "" );
                item.taken = 1;
                item.model delete();
                return;
            }
        }
    }
}

br_take_gun( weapon )
{
    primaries = self getweaponslistprimaries();

    if ( primaries.size >= 2 )
    {
        held = self getcurrentweapon();

        if ( held == "none" || !self hasweapon( held ) )
            held = primaries[0];

        self takeweapon( held );
    }

    self giveweapon( weapon );
    self givemaxammo( weapon );
    self switchtoweapon( weapon );

    if ( self istestclient() )
        self.br_bot_gun = 1;
}

br_place_stations()
{
    level endon( "game_ended" );
    wait 0.5;
    used = [];

    for ( i = 0; i < level.br_station_count && level.br_spot_list.size > 0; i++ )
    {
        best = undefined;
        bestd = -1;

        foreach ( s in level.br_spot_list )
        {
            nearest = 1000000;

            foreach ( u in used )
            {
                d = distance2d( s.origin, u.origin );

                if ( d < nearest )
                    nearest = d;
            }

            if ( distance2d( s.origin, level.br_center ) > level.br_radius0 * 0.75 )
                continue;

            if ( nearest > bestd )
            {
                bestd = nearest;
                best = s;
            }
        }

        if ( !isdefined( best ) )
            break;

        used[used.size] = best;
        station = spawn( "script_model", best.origin );
        station setmodel( "t6_wpn_supply_drop_ally" );
        station.br_id = level.br_objid + 1 + i;
        objective_add( station.br_id, "active", best.origin, "compass_supply_drop_green" );
        level.br_stations[level.br_stations.size] = station;
        level thread br_buy_station( station );
    }
}

// Near a station: [Use] steps through what it sells, held it buys.
br_buy_station( station )
{
    level endon( "game_ended" );

    for (;;)
    {
        wait 0.05;

        foreach ( player in level.players )
        {
            near = isalive( player ) && distancesquared( player.origin, station.origin ) < 96 * 96;

            if ( !near )
            {
                if ( isdefined( player.br_near ) && player.br_near == station )
                {
                    player.br_near = undefined;
                    player br_hud_hint( "" );
                }

                continue;
            }

            if ( !isdefined( player.br_near ) || player.br_near != station )
            {
                player.br_near = station;
                player br_snd( "buy_open" );
            }

            if ( player istestclient() )
                continue;

            b = level.br_buys[player.br_buy_index % level.br_buys.size];
            player br_hud_hint( "BUY STATION   " + b.label + "  $" + b.price + "\n[USE] NEXT   HOLD [USE] BUY" );

            if ( !player usebuttonpressed() )
                continue;

            held = 0;

            while ( player usebuttonpressed() && held < 600 )
            {
                wait 0.05;
                held = held + 50;
            }

            if ( held >= 600 )
                player br_buy( b );
            else
                player.br_buy_index++;

            while ( isdefined( player ) && player usebuttonpressed() )
                wait 0.05;
        }
    }
}

br_buy( b )
{
    if ( self.br_cash < b.price )
    {
        self br_snd( "purchase_denied" );
        self br_hud_flash( "NOT ENOUGH CASH" );
        return 0;
    }

    ok = 0;

    switch ( b.key )
    {
        case "loadout":
            ok = self br_loadout_drop();
            break;
        case "selfrevive":
            ok = !self.br_selfrevive;

            if ( ok )
                self.br_selfrevive = 1;

            break;
        case "uav":
            self thread br_uav();
            ok = 1;
            break;
        case "buyback":
            ok = self br_buyback();
            break;
        case "plates":
            ok = self.br_plates_carried < level.br_plates_max;

            if ( ok )
            {
                self.br_plates_carried = self.br_plates_carried + 3;

                if ( self.br_plates_carried > level.br_plates_max )
                    self.br_plates_carried = level.br_plates_max;
            }

            break;
    }

    if ( !ok )
    {
        self br_snd( "purchase_denied" );
        return 0;
    }

    self.br_cash = self.br_cash - b.price;
    self br_snd( "purchase" );
    self br_hud_flash( b.label );
    return 1;
}

// The crate comes down by him; at it, [Use] gives him his class.
br_loadout_drop()
{
    if ( !isdefined( self.br_kit ) )
        return 0;

    origin = self.origin + anglestoforward( self.angles ) * 96;
    trace = bullettrace( origin + ( 0, 0, 64 ), origin - ( 0, 0, 512 ), 0, self );
    ground = trace["position"];
    self br_snd( "loadout_incoming" );
    level thread br_loadout_crate( self, ground );
    return 1;
}

br_loadout_crate( owner, ground )
{
    level endon( "game_ended" );
    crate = spawn( "script_model", ground + ( 0, 0, 1200 ) );
    crate setmodel( "t6_wpn_supply_drop_axis" );
    crate moveto( ground, 3, 1, 0.5 );
    wait 3;

    foreach ( player in br_squad_members( owner.br_squad ) )
        player br_snd( "loadout_landed" );

    claimed = [];

    for ( t = 0; t < 1200; t++ )
    {
        wait 0.1;

        foreach ( player in br_squad_members( owner.br_squad ) )
        {
            if ( !isalive( player ) || distancesquared( player.origin, ground ) > 80 * 80 )
                continue;

            if ( isdefined( claimed[player getentitynumber()] ) )
                continue;

            player br_hud_hint( "[USE]  LOADOUT" );

            if ( player usebuttonpressed() || player istestclient() )
            {
                claimed[player getentitynumber()] = 1;
                player br_give_kit();
                player br_hud_hint( "" );
            }
        }
    }

    crate delete();
}

br_give_kit()
{
    self takeallweapons();

    foreach ( w in self.br_kit )
    {
        self giveweapon( w );
        self givemaxammo( w );
    }

    if ( isdefined( self.br_kit_primaries ) && self.br_kit_primaries.size > 0 )
        self switchtoweapon( self.br_kit_primaries[0] );
}

br_uav()
{
    self endon( "disconnect" );
    self br_snd( "uav_online" );
    until = gettime() + level.br_uav_ms;

    while ( gettime() < until )
    {
        self setclientuivisibilityflag( "g_compassShowEnemies", 1 );
        wait 3;
    }

    self setclientuivisibilityflag( "g_compassShowEnemies", 0 );
}

br_buyback()
{
    foreach ( mate in br_squad_members( self.br_squad ) )
    {
        if ( mate == self || isalive( mate ) || mate.br_pending )
            continue;

        mate.br_out = 0;
        mate br_respawn();
        br_snd_squad( self.br_squad, "buyback" );
        return 1;
    }

    return 0;
}

// ---- the HUD ----------------------------------------------------------------

br_hud_elem( x, y, alignx, aligny, scale )
{
    e = newclienthudelem( self );
    e.horzalign = alignx;
    e.vertalign = aligny;
    e.alignx = alignx;
    e.aligny = aligny;
    e.x = x;
    e.y = y;
    e.fontscale = scale;
    e.font = "default";
    e.alpha = 1;
    e.hidewheninmenu = 1;
    e.sort = 2;
    return e;
}

br_hud_init()
{
    self.br_hud_circle = self br_hud_elem( 0, 18, "center", "top", 1.6 );
    self.br_hud_squad = self br_hud_elem( 12, 160, "left", "top", 1.3 );
    self.br_hud_cash = self br_hud_elem( -12, -118, "right", "bottom", 1.6 );
    self.br_hud_cash.color = ( 0.55, 0.95, 0.5 );
    self.br_hud_armor = self br_hud_elem( -12, -96, "right", "bottom", 1.3 );
    self.br_hud_gas = self br_hud_elem( 0, -150, "center", "middle", 1.6 );
    self.br_hud_gas.color = ( 1, 0.35, 0.2 );
    self.br_hud_gas.alpha = 0;
    self.br_hud_big = self br_hud_elem( 0, -40, "center", "middle", 2.4 );
    self.br_hud_big.alpha = 0;
    self.br_hud_note = self br_hud_elem( 0, 64, "center", "top", 1.4 );
    self.br_hud_note.alpha = 0;
    self.br_hud_use = self br_hud_elem( 0, 90, "center", "middle", 1.3 );
    self.br_hud_use.alpha = 0;
}

br_hud_hint( text )
{
    if ( !isdefined( self.br_hud_use ) )
        return;

    if ( text == "" )
    {
        self.br_hud_use.alpha = 0;
        return;
    }

    self.br_hud_use settext( text );
    self.br_hud_use.alpha = 1;
}

br_hud_flash( text )
{
    if ( !isdefined( self.br_hud_note ) )
        return;

    self notify( "br_flash" );
    self.br_hud_note settext( text );
    self.br_hud_note.alpha = 1;
    self thread br_hud_flash_off();
}

br_hud_flash_off()
{
    self endon( "br_flash" );
    self endon( "disconnect" );
    wait 3;
    self.br_hud_note fadeovertime( 0.5 );
    self.br_hud_note.alpha = 0;
}

br_clock( ms )
{
    s = int( ( ms + 999 ) / 1000 );

    if ( s < 0 )
        s = 0;

    m = int( s / 60 );
    s = s % 60;

    if ( s < 10 )
        return "" + m + ":0" + s;

    return "" + m + ":" + s;
}

br_hud_think()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    tick = 0;

    for (;;)
    {
        wait 0.25;
        tick++;
        now = br_elapsed();

        if ( level.br_phase < 0 )
            self.br_hud_circle settext( "INFIL  " + br_clock( level.br_infil_ms - now ) );
        else if ( level.br_phase >= level.br_phases.size )
            self.br_hud_circle settext( "FINAL CIRCLE" );
        else
        {
            p = level.br_phases[level.br_phase];
            label = "CIRCLE " + ( level.br_phase + 1 );

            if ( now < p.close_from )
                self.br_hud_circle settext( label + "  CLOSES IN " + br_clock( p.close_from - now ) );
            else
                self.br_hud_circle settext( label + "  CLOSING " + br_clock( p.closed - now ) );
        }

        mates = "";

        foreach ( p in br_squad_members( self.br_squad ) )
        {
            if ( p == self )
                continue;

            state = "";

            if ( !isalive( p ) )
            {
                if ( p.br_pending )
                    state = "  REDEPLOYING";
                else
                    state = "  DOWN";
            }

            mates = mates + p.name + state + "\n";
        }

        redeploy = "REDEPLOY ON";

        if ( br_redeploy_wait() < 0 )
            redeploy = "REDEPLOY OFF";

        self.br_hud_squad settext( "SQUAD " + ( self.br_squad + 1 ) + "   " + br_squads_standing() + " SQUADS LEFT   " + redeploy + "\n" + mates );
        self.br_hud_cash settext( "$" + self.br_cash );
        bars = "";

        for ( k = 0; k < level.br_plate_slots; k++ )
        {
            if ( k < self.br_plates )
                bars = bars + "[#]";
            else
                bars = bars + "[ ]";
        }

        kit = "";

        if ( self.br_selfrevive )
            kit = "  SELF-REVIVE";

        self.br_hud_armor settext( "ARMOR " + bars + "  PLATES " + self.br_plates_carried + kit );

        if ( isalive( self ) && !br_inside( self.origin, 0 ) && level.br_phase >= 0 )
        {
            self.br_hud_gas settext( "YOU ARE IN THE GAS  -  GET TO THE SAFE ZONE  " + int( distance2d( self.origin, level.br_center ) - level.br_radius ) + "" );
            self.br_hud_gas.alpha = 1;
        }
        else
            self.br_hud_gas.alpha = 0;

        if ( tick % 4 == 0 )
            self br_gas( br_gas_dps() );

        if ( isdefined( self.br_near ) && ( !isalive( self ) || isdefined( self.br_near.taken ) ) )
        {
            self.br_near = undefined;
            self br_hud_hint( "" );
        }
    }
}

// ---- bots -------------------------------------------------------------------

// A bot keeps inside the next circle, sticks with his squad, insert plates
// when out of a fight and buys a downed squadmate back.
br_bot_think()
{
    self endon( "disconnect" );
    level endon( "game_ended" );

    for (;;)
    {
        wait 1;

        if ( !isalive( self ) )
        {
            self.br_bot_gun = undefined;
            continue;
        }

        center = level.br_center;
        radius = level.br_radius;

        if ( isdefined( level.br_next_center ) && level.br_phase >= 0 )
        {
            center = level.br_next_center;
            radius = level.br_next_radius;
        }

        if ( distance2d( self.origin, center ) > radius * 0.8 || !br_inside( self.origin, 64 ) )
        {
            spot = br_spot_near( center, radius * 0.6 );
            goal = center;

            if ( isdefined( spot ) )
                goal = spot.origin;

            if ( !self hasgoal( "br_circle" ) )
                self addgoal( goal, radius * 0.4 + 64, 4, "br_circle" );
        }
        else if ( self hasgoal( "br_circle" ) )
            self cancelgoal( "br_circle" );

        lead = undefined;

        foreach ( p in br_squad_members( self.br_squad ) )
        {
            if ( p == self || !isalive( p ) )
                continue;

            if ( !isdefined( lead ) || !p istestclient() )
                lead = p;
        }

        if ( isdefined( lead ) && distance2d( self.origin, lead.origin ) > 900 )
            self addgoal( lead.origin, 300, 2, "br_regroup" );
        else if ( self hasgoal( "br_regroup" ) )
            self cancelgoal( "br_regroup" );

        if ( gettime() - self.br_last_hurt > 4000 )
            self br_insert_plate();

        self br_bot_shop();
    }
}

br_bot_shop()
{
    want = undefined;
    need_buyback = 0;

    foreach ( mate in br_squad_members( self.br_squad ) )
    {
        if ( mate != self && !isalive( mate ) && !mate.br_pending && !br_squad_out( self.br_squad ) )
            need_buyback = 1;
    }

    foreach ( b in level.br_buys )
    {
        if ( b.key == "buyback" && need_buyback && self.br_cash >= b.price )
            want = b;
        else if ( !isdefined( want ) && b.key == "loadout" && !isdefined( self.br_bot_kit ) && self.br_cash >= b.price )
            want = b;
    }

    if ( !isdefined( want ) || level.br_stations.size == 0 )
    {
        if ( self hasgoal( "br_buy" ) )
            self cancelgoal( "br_buy" );

        return;
    }

    best = undefined;
    bestd = 1000000000;

    foreach ( s in level.br_stations )
    {
        d = distancesquared( self.origin, s.origin );

        if ( d < bestd && br_inside( s.origin, 0 ) )
        {
            bestd = d;
            best = s;
        }
    }

    if ( !isdefined( best ) )
        return;

    if ( bestd > 96 * 96 )
    {
        self addgoal( best.origin, 48, 3, "br_buy" );
        return;
    }

    self cancelgoal( "br_buy" );

    if ( self br_buy( want ) && want.key == "loadout" )
        self.br_bot_kit = 1;
}
