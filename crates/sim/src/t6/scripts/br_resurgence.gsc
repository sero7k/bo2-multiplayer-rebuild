// bo2mp: Resurgence (Warzone's small-map battle royale) on Black Ops II's
// free-for-all. OUR CODE. Squads, a gas circle that closes in phases,
// redeploys while a squadmate lives (off in the last circles), a squad
// with nobody alive is out, the last squad standing wins. Armor plates
// over Black Ops II's own health, cash, buy stations (loadout drop,
// self-revive kit, UAV, squad buyback, plates) and ground loot by rarity.
//
// Off unless `scr_br 1`. Every number is a dvar written by the engine
// from one table (`scr_br_*`, gamemode_iw4::resurgence); every sound is
// `scr_br_snd_<event>`: Modern Warfare III's aliases for it (one picked at
// random each time), Black Ops II's stand-in without that game, "" for
// none; `scr_br_sndnext_<event>` "<event> <ms>" is what follows it (music:
// an intro into its main part).

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
    level.br_contract = [];
    level.br_contract_taker = [];
    level.br_contract_end = [];
    level.br_hurried = [];
    level.br_bounty_hit = [];
    level.br_scav_open = [];
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
    level thread br_place_contracts();
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
    level.br_armor_cut_ms = br_dvar_int( "scr_br_armor_cut_ms", 2000 );
    level.br_cache_cut_ms = br_dvar_int( "scr_br_cache_cut_ms", 2000 );
    level.br_bounty_cut_ms = br_dvar_int( "scr_br_bounty_cut_ms", 10000 );
    level.br_cache_count = br_dvar_int( "scr_br_caches", 16 );
    level.br_mask_ms = br_dvar_int( "scr_br_mask_ms", 20000 );
    level.br_discount = br_dvar_int( "scr_br_discount", 80 );
    level.br_bounty_live_cash = br_dvar_int( "scr_br_bounty_live_cash", 1000 );
    level.br_recon_ms = br_dvar_int( "scr_br_recon_ms", 30000 );

    // Heavy Armor: now and then a match gives everyone one more plate slot.
    level.br_heavy = 0;

    if ( randomint( 100 ) < br_dvar_int( "scr_br_heavy_pct", 0 ) )
    {
        level.br_heavy = 1;
        level.br_plate_slots++;
    }

    level.br_contract_kinds = [];
    n = br_dvar_int( "scr_br_contracts", 0 );

    for ( i = 0; i < n; i++ )
    {
        f = strtok( getdvar( "scr_br_contract_" + i ), "|" );

        if ( f.size < 5 )
            continue;

        c = spawnstruct();
        c.key = f[0];
        c.label = f[1];
        c.reward = int( f[2] );
        c.time = int( f[3] );
        c.count = int( f[4] );
        level.br_contract_kinds[level.br_contract_kinds.size] = c;
    }

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
    list = strtok( getdvar( "scr_br_snd_" + key ), " " );

    if ( list.size == 0 )
        return "";

    return list[randomint( list.size )];
}

br_snd( key )
{
    name = br_snd_name( key );

    if ( name != "" )
        self playlocalsound( name );

    next = strtok( getdvar( "scr_br_sndnext_" + key ), " " );

    if ( next.size == 2 )
        self thread br_snd_then( next[0], int( next[1] ) );
}

br_snd_then( key, ms )
{
    self endon( "disconnect" );

    // (Music gives way to the next piece; anything else stops with the match.)
    if ( issubstr( key, "music_" ) )
        self endon( "br_music" );
    else
        level endon( "game_ended" );

    wait ms / 1000;
    self br_snd( key );
}

// A piece of music: what was playing does not carry on into its next part.
br_music( key )
{
    self notify( "br_music" );
    self br_snd( key );
}

br_music_all( key )
{
    foreach ( player in level.players )
        player br_music( key );
}

// An announcer line: one at a time for each player, the newest wins a
// clash only when the one before has had a second.
br_vo( key )
{
    if ( br_snd_name( key ) == "" )
        return;

    now = gettime();

    if ( isdefined( self.br_vo_at ) && now - self.br_vo_at < 1200 )
        return;

    self.br_vo_at = now;
    self br_snd( key );
}

br_vo_all( key )
{
    foreach ( player in level.players )
        player br_vo( key );
}

br_vo_squad( squad, key )
{
    foreach ( player in br_squad_members( squad ) )
        player br_vo( key );
}

// A spoken number (the circle's and the redeploy's last seconds): `vo_count_<n>`.
br_vo_count( n )
{
    self br_snd( "vo_count_" + n );
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

// Before the drop: the island's pre-lobby music; at the drop the plane goes
// over, the infil music starts and the island's own strikes land; its
// emitters hum at spots around the map all match.
br_ambience()
{
    level endon( "game_ended" );
    br_music_all( "music_prelobby" );
    br_wait_start();
    br_music_all( "music_infil" );
    br_snd_all( "infil_flyby" );
    br_vo_all( "vo_prelobby_over" );
    wait 2;
    br_vo_all( "vo_infil" );
    br_snd_all( "infil_jump" );
    br_snd_all( "ambience" );
    level thread br_emitters();

    if ( level.br_heavy )
    {
        br_snd_all( "heavy_armor" );
        br_vo_all( "vo_heavy_armor" );

        foreach ( player in level.players )
            player br_hud_flash( "HEAVY ARMOR  -  ONE MORE PLATE THIS MATCH" );
    }

    wait 2;
    br_snd_all( "infil_strike" );
    wait level.br_infil_ms / 1000 - 4;
    br_vo_all( "vo_landed" );
    wait 3;
    br_vo_all( "vo_rules" );
}

// Rebirth Island's emitter loops (steam, vents, generators, fans, pipes),
// each on a spot of the map.
br_emitters()
{
    list = strtok( getdvar( "scr_br_snd_amb_loops" ), " " );

    if ( list.size == 0 || level.br_spot_list.size == 0 )
        return;

    for ( i = 0; i < 6; i++ )
    {
        s = level.br_spot_list[randomint( level.br_spot_list.size )];
        e = spawn( "script_origin", s.origin + ( 0, 0, 48 ) );
        e playloopsound( list[randomint( list.size )] );
    }
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
    self.br_mask_ms = 0;
    self.br_mask_on = 0;
    self.br_wall_at = 0;
    self.br_discount = 0;
    self.br_buy_index = 0;
    self.br_kit = undefined;
    self br_hud_init();
    self thread br_life();
    self thread br_death_watch();
    self thread br_plates_input();
    self thread br_hud_think();

    if ( self istestclient() )
        self thread br_bot_think();

    if ( !isdefined( level.br_started ) )
    {
        self br_music( "music_prelobby" );
        self thread br_prelobby_vo();
    }
}

br_prelobby_vo()
{
    self endon( "disconnect" );
    level endon( "br_started" );
    wait 1;
    self br_vo( "vo_mode_name" );
    wait 4;
    self br_vo( "vo_prelobby" );
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

    // (Players left, said at the counts the announcer has.)
    players = 0;

    foreach ( p in level.players )
    {
        if ( isalive( p ) || p.br_pending )
            players++;
    }

    if ( !isdefined( level.br_players_said ) )
        level.br_players_said = 1000;

    foreach ( n in array( 50, 25, 10 ) )
    {
        if ( players <= n && level.br_players_said > n )
        {
            level.br_players_said = n;
            br_vo_all( "vo_players_left_" + n );
        }
    }

    if ( left == 3 || left == 2 )
        br_snd_all( "top_squads" );
    else
        br_snd_all( "squads_remaining" );

    // (How many are left, said when it is a number the announcer has.)
    foreach ( player in level.players )
    {
        if ( isdefined( player.br_squad ) && player.br_squad != squad && !player.br_out )
        {
            player br_vo( "vo_squads_left_" + left );
        }
    }
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
            player br_snd( "victory_amb" );
            player br_vo( "vo_victory" );
            player br_music( "music_exfil" );
        }
        else
        {
            player br_snd( "defeat" );

            // (A squad out in the last three hears where it finished.)
            place = br_place_of( player.br_squad );

            if ( place <= 3 )
                player br_vo( "vo_third" );
            else
                player br_vo( "vo_defeat" );

            player br_music( "music_circle_last_exit" );
        }
    }

    thread maps\mp\gametypes\_globallogic::endgame( first, reason );
}

// 1 for the last squad standing, 2 for the one out last, ...
br_place_of( squad )
{
    for ( i = 0; i < level.br_out_order.size; i++ )
    {
        if ( level.br_out_order[i] == squad )
            return level.br_out_order.size - i + 1;
    }

    return 1;
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
        self br_snd( "redeploy_flare" );
        self br_vo( "vo_redeployed" );
        self br_hud_flash( "REDEPLOYED" );

        foreach ( mate in br_squad_members( self.br_squad ) )
        {
            if ( mate != self )
                mate br_vo( "vo_mate_redeployed" );
        }
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
        level.br_last_killer_squad = attacker.br_squad;
        attacker.br_kills++;
        attacker.br_ev_kill++;
        attacker.br_ev_victim = self.name;
        attacker br_hud_publish();

        if ( randomint( 4 ) == 0 )
            attacker br_vo( "vo_kill" );

        // (His squad's dead come back sooner.)
        br_cut_redeploys( attacker.br_squad, level.br_kill_cut_ms );
        level thread br_contract_kill( attacker, self );
    }
    else
        level thread br_contract_kill( undefined, self );

    alive = br_squad_alive( self.br_squad );

    foreach ( mate in br_squad_members( self.br_squad ) )
    {
        if ( mate == self )
            continue;

        mate br_snd( "squadmate_down" );

        // (The last one standing is told so.)
        if ( alive == 1 && isalive( mate ) )
            mate br_vo( "vo_last_alive" );
        else
            mate br_vo( "vo_mate_down" );
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
        self br_vo( "vo_redeploy_off" );
        self br_hud_flash( "REDEPLOY DISABLED  -  A SQUADMATE CAN BUY YOU BACK" );
        return;
    }

    self.br_pending = 1;
    self br_vo( "vo_redeploy_pending" );
    self thread br_redeploy( wait_ms );
    level thread br_mates_hear( self, "vo_mate_pending", 2.5 );
}

// After `delay` seconds, his living squadmates hear `key`.
br_mates_hear( who, key, delay )
{
    wait delay;

    if ( !isdefined( who ) )
        return;

    foreach ( mate in br_squad_members( who.br_squad ) )
    {
        if ( mate != who && isalive( mate ) )
            mate br_vo( key );
    }
}

// The squad's dead come back `ms` sooner.
br_cut_redeploys( squad, ms )
{
    foreach ( mate in br_squad_members( squad ) )
    {
        if ( mate.br_pending && isdefined( mate.br_redeploy_at ) )
        {
            mate.br_redeploy_at = mate.br_redeploy_at - ms;
            mate br_vo( "vo_redeploy_cut" );
        }
    }
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

        if ( left != last && left <= 5 )
        {
            if ( left == 1 )
                self br_snd( "redeploy_go" );
            else
                self br_snd( "redeploy_tick" );

            self br_vo_count( left );
        }

        last = left;
        self.br_hud_big settext( "REDEPLOYING IN " + left );
        self.br_hud_big.alpha = 1;

        if ( isdefined( self.br_wzhud ) )
            self.br_hud_big.alpha = 0;

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
            {
                eattacker br_snd( "armor_break" );
                eattacker.br_ev_break++;
                eattacker br_hud_publish();

                if ( self.br_plates == 0 )
                    br_cut_redeploys( eattacker.br_squad, level.br_armor_cut_ms );
            }
        }

        self.br_last_hurt = gettime();

        if ( idamage <= 0 )
        {
            if ( isdefined( eattacker ) && isplayer( eattacker ) )
            {
                eattacker br_snd( "armor_hit" );
                eattacker.br_ev_hit++;
                eattacker br_hud_publish();
            }

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

// The map's own places, where its entities have them (Rebirth Island's):
// its intel upload stations stand where the buy stations go, its loadout
// drop spots (script_noteworthy "dropBagLocation") hold the ground loot and
// the supply boxes.
br_map_places()
{
    level.br_station_spots = [];
    level.br_loot_spots = [];

    if ( !isdefined( level.struct ) )
        return;

    foreach ( s in level.struct )
    {
        if ( !isdefined( s.origin ) )
            continue;

        if ( isdefined( s.targetname ) && s.targetname == "intel_upload_station" )
            level.br_station_spots[level.br_station_spots.size] = s;
        else if ( isdefined( s.script_noteworthy ) && s.script_noteworthy == "dropBagLocation" )
            level.br_loot_spots[level.br_loot_spots.size] = s;
    }
}

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

    // (And the map's own loadout drop spots, where it has them.)
    br_map_places();

    foreach ( s in level.br_loot_spots )
        level.br_spot_list[level.br_spot_list.size] = s;

    println( "resurgence: " + level.br_spot_list.size + " spots, " + level.br_loot_spots.size + " of the map's drop spots, " + level.br_station_spots.size + " of its stations" );
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
        last = i == level.br_phases.size - 1;
        off = p.redeploy < 0 && ( i == 0 || level.br_phases[i - 1].redeploy >= 0 );

        if ( last || off )
        {
            br_snd_all( "final_circle" );

            if ( off )
            {
                br_vo_all( "vo_redeploy_off" );

                foreach ( player in level.players )
                    player br_hud_flash( "REDEPLOYS ARE OFF" );
            }
            else
                br_vo_all( "vo_final_circle" );
        }

        // (Redeploys go off with the next circle: said now.)
        if ( p.redeploy >= 0 && i + 1 < level.br_phases.size && level.br_phases[i + 1].redeploy < 0 )
        {
            br_vo_all( "vo_redeploy_closing" );

            foreach ( player in level.players )
                player br_hud_flash( "REDEPLOYS END WITH THIS CIRCLE" );
        }

        said_soon = 0;
        said = -1;

        while ( br_elapsed() < p.close_from )
        {
            left = int( ( p.close_from - br_elapsed() + 999 ) / 1000 );

            if ( !said_soon && left <= 30 && p.close_from - p.wait_from > 40000 )
            {
                said_soon = 1;
                br_vo_all( "vo_circle_soon" );
            }

            if ( left <= 5 && left != said )
            {
                said = left;
                br_snd_all( "circle_tick" );

                foreach ( player in level.players )
                    player br_vo_count( left );
            }

            wait 0.1;
        }

        level.br_closing = 1;
        br_snd_all( "circle_collapsing" );

        if ( !last )
            br_vo_all( "vo_circle_closing" );

        br_music_all( br_circle_music( i ) );

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
        br_squads_inside();
    }

    // (The last circle is shut: its loop ends, the second loop plays to the end.)
    br_music_all( "music_circle_final_exit" );
    wait 10;
    br_music_all( "music_circle_last" );

    level.br_phase = level.br_phases.size;
}

// A circle has closed: a squad all inside hears so; one with someone
// outside is told to get him in.
br_squads_inside()
{
    for ( i = 0; i < level.br_squads.size; i++ )
    {
        out = 0;

        foreach ( p in br_squad_members( i ) )
        {
            if ( isalive( p ) && !br_inside( p.origin, 0 ) )
                out++;
        }

        foreach ( p in br_squad_members( i ) )
        {
            if ( !isalive( p ) )
                continue;

            if ( out == 0 )
                p br_vo( "vo_squad_safe" );
            else if ( br_inside( p.origin, 0 ) )
                p br_vo( "vo_team_outside" );
        }
    }
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
        {
            self.br_in_gas = 0;

            if ( self.br_mask_on )
            {
                self.br_mask_on = 0;
                self br_snd( "gas_mask_off" );
            }
        }

        // (By the wall: its hum.)
        if ( isalive( self ) && level.br_phase >= 0 && !br_inside( self.origin, 400 ) && gettime() - self.br_wall_at > 8000 )
        {
            self.br_wall_at = gettime();
            self br_snd( "gas_wall" );
        }

        return;
    }

    if ( !isdefined( self.br_in_gas ) || !self.br_in_gas )
    {
        self.br_in_gas = 1;
        self br_snd( "gas_enter" );

        if ( distance2d( self.origin, level.br_center ) > level.br_radius * 1.5 )
            self br_vo( "vo_safe_far" );
        else
            self br_vo( "vo_in_gas" );

        if ( self.br_mask_ms > 0 )
        {
            self.br_mask_on = 1;
            self br_snd( "gas_mask_on" );
        }
    }

    // (A gas mask holds the gas off while it lasts.)
    if ( self.br_mask_on )
    {
        self.br_mask_ms = self.br_mask_ms - 1000;

        if ( self.br_mask_ms > 0 )
            return;

        self.br_mask_ms = 0;
        self.br_mask_on = 0;
        self br_snd( "gas_mask_break" );
        self br_hud_flash( "GAS MASK BROKE" );
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

    foreach ( player in level.players )
    {
        if ( isalive( player ) && distancesquared( player.origin, origin ) < 1024 * 1024 )
            player br_snd( "cash_drop" );
    }
}

br_place_loot()
{
    level endon( "game_ended" );
    wait 1;
    spots = level.br_spot_list;
    // (Around the map's own drop spots, close: they stand by walls.)
    spread = 96;

    if ( level.br_loot_spots.size > 0 )
    {
        spots = level.br_loot_spots;
        spread = 32;
    }

    for ( i = 0; i < level.br_loot_count && spots.size > 0; i++ )
    {
        s = spots[randomint( spots.size )];
        origin = s.origin + ( randomfloatrange( 0 - spread, spread ), randomfloatrange( 0 - spread, spread ), 12 );
        roll = randomint( 100 );

        br_spawn_loot( origin, roll );
    }

    level thread br_place_caches();
}

br_guns()
{
    return array( "fiveseven_mp", "fnp45_mp", "beretta93r_mp", "kard_mp", "mp7_mp", "pdw57_mp", "vector_mp", "insas_mp", "qcw05_mp", "evoskorpion_mp", "peacekeeper_mp", "tar21_mp", "type95_mp", "sig556_mp", "sa58_mp", "hk416_mp", "scar_mp", "saritch_mp", "xm8_mp", "an94_mp", "mk48_mp", "qbb95_mp", "lsat_mp", "hamr_mp", "870mcs_mp", "saiga12_mp", "ksg_mp", "srm1216_mp", "svu_mp", "dsr50_mp", "ballista_mp" );
}

// One item on the ground: `roll` (0..99) picks cash, plates, a gas mask or
// a gun of a rolled rarity.
br_spawn_loot( origin, roll )
{
    atts = array( "reflex", "extclip", "fastads", "silencer", "fmj", "holo", "acog", "grip", "rf", "dualclip" );
    item = spawnstruct();
    item.origin = origin;

    if ( roll < 20 )
    {
        item.kind = "cash";
        item.amount = level.br_stacks[randomint( level.br_stacks.size )];
        model = getweaponmodel( "briefcase_bomb_mp" );
    }
    else if ( roll < 42 )
    {
        item.kind = "plates";
        item.amount = 1 + randomint( 3 );
        model = getweaponmodel( "riotshield_mp" );
    }
    else if ( roll < 47 )
    {
        item.kind = "mask";
        model = getweaponmodel( "tactical_insertion_mp" );
    }
    else
    {
        item.kind = "gun";
        tier = br_roll_tier();
        guns = br_guns();
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
    return item;
}

// Supply boxes: [Use] opens one (cash and a few items spill out, the
// squad's dead come back sooner).
br_place_caches()
{
    // (On the map's own drop spots, each box on a spot of its own.)
    spots = level.br_loot_spots;

    for ( i = 0; i < level.br_cache_count && spots.size > 0; i++ )
    {
        k = randomint( spots.size );
        level thread br_cache( spots[k].origin, 0, undefined );
        last = spots.size - 1;
        spots[k] = spots[last];
        spots[last] = undefined;
    }

    if ( level.br_loot_spots.size > 0 )
        return;

    for ( i = 0; i < level.br_cache_count && level.br_spot_list.size > 0; i++ )
    {
        s = level.br_spot_list[randomint( level.br_spot_list.size )];
        level thread br_cache( s.origin + ( randomfloatrange( -64, 64 ), randomfloatrange( -64, 64 ), 0 ), 0, undefined );
    }
}

// A supply box at `origin`; `contract`: a Scavenger box only that squad
// may open (it ends when it is opened).
br_cache( origin, contract, squad )
{
    level endon( "game_ended" );
    box = spawn( "script_model", origin );
    box setmodel( "t6_wpn_supply_drop_axis" );

    for (;;)
    {
        wait 0.1;

        foreach ( player in level.players )
        {
            if ( !isalive( player ) || distancesquared( player.origin, origin ) > 72 * 72 )
                continue;

            if ( contract && player.br_squad != squad )
                continue;

            player br_hud_hint( "[USE]  OPEN SUPPLY BOX" );

            if ( !player usebuttonpressed() && !player istestclient() )
                continue;

            player br_hud_hint( "" );

            if ( contract )
                player br_snd( "supply_chest_open" );
            else
                player br_snd( "cache_open" );

            player.br_cash = player.br_cash + level.br_stacks[randomint( level.br_stacks.size )];
            player br_snd( "cash_pickup" );
            br_cut_redeploys( player.br_squad, level.br_cache_cut_ms );

            for ( k = 0; k < 2; k++ )
                br_spawn_loot( origin + ( randomfloatrange( -48, 48 ), randomfloatrange( -48, 48 ), 12 ), 20 + randomint( 80 ) );

            box delete();

            if ( contract )
                level.br_scav_open[squad] = 1;

            return;
        }
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

            if ( item.kind == "mask" )
            {
                if ( player.br_mask_ms > 0 )
                    continue;

                player.br_mask_ms = level.br_mask_ms;
                player br_snd( "mask_pickup" );
                player br_hud_flash( "GAS MASK" );
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
    // (Where the map has its own stations, among them; else among the
    // spots in the middle three quarters.)
    spots = level.br_spot_list;
    reach = level.br_radius0 * 0.75;

    if ( level.br_station_spots.size > 0 )
    {
        spots = level.br_station_spots;
        reach = level.br_radius0 * 2;
    }

    for ( i = 0; i < level.br_station_count && spots.size > 0; i++ )
    {
        best = undefined;
        bestd = -1;

        foreach ( s in spots )
        {
            nearest = 1000000;

            foreach ( u in used )
            {
                d = distance2d( s.origin, u.origin );

                if ( d < nearest )
                    nearest = d;
            }

            if ( distance2d( s.origin, level.br_center ) > reach || nearest < 1 )
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
                    player br_snd( "buy_close" );
                }

                continue;
            }

            if ( !isdefined( player.br_near ) || player.br_near != station )
            {
                player.br_near = station;
                player br_snd( "buy_open" );
                player br_vo( "vo_buy_station" );
                level notify( "br_at_station", player );
            }

            if ( player istestclient() )
                continue;

            b = level.br_buys[player.br_buy_index % level.br_buys.size];
            player br_hud_hint( "BUY STATION   " + b.label + "  $" + player br_price( b ) + "\n[USE] NEXT   HOLD [USE] BUY" );

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

// What `b` costs him (a Supply Run's discount off his next one).
br_price( b )
{
    if ( self.br_discount > 0 )
        return int( b.price * ( 100 - self.br_discount ) / 100 );

    return b.price;
}

br_buy( b )
{
    price = self br_price( b );

    if ( self.br_cash < price )
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

    self.br_cash = self.br_cash - price;
    self.br_discount = 0;
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
    self br_vo( "vo_loadout_incoming" );
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
    {
        player br_snd( "loadout_landed" );
        player br_vo( "vo_loadout_landed" );
    }

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
                player br_snd( "loadout_claim" );
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
    self br_vo( "vo_uav_online" );

    foreach ( player in level.players )
    {
        if ( isdefined( player.br_squad ) && player.br_squad != self.br_squad )
        {
            player br_snd( "enemy_uav" );
            player br_vo( "vo_enemy_uav" );
        }
    }

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
        mate br_vo( "vo_bought_back" );
        return 1;
    }

    return 0;
}

// ---- contracts --------------------------------------------------------------

// Tablets on the map: [Use] takes its contract for his squad (one at a
// time). A taken tablet's map marker becomes the contract's.
br_place_contracts()
{
    level endon( "game_ended" );
    br_wait_start();
    wait 2;
    k = 0;

    foreach ( c in level.br_contract_kinds )
    {
        for ( i = 0; i < c.count && level.br_spot_list.size > 0 && k < 8; i++ )
        {
            s = level.br_spot_list[randomint( level.br_spot_list.size )];
            t = spawn( "script_model", s.origin + ( randomfloatrange( -48, 48 ), randomfloatrange( -48, 48 ), 8 ) );
            t setmodel( getweaponmodel( "satchel_charge_mp" ) );
            t.br_kind = c;
            t.br_id = level.br_objid + 4 + k;
            objective_add( t.br_id, "active", t.origin, "compass_waypoint_defend" );
            k++;
            level thread br_tablet( t );
        }
    }
}

br_tablet( t )
{
    level endon( "game_ended" );

    for (;;)
    {
        wait 0.1;

        foreach ( player in level.players )
        {
            if ( !isalive( player ) || !isdefined( player.br_squad ) || distancesquared( player.origin, t.origin ) > 64 * 64 )
                continue;

            if ( isdefined( level.br_contract[player.br_squad] ) )
            {
                player br_hud_hint( "YOUR SQUAD HAS A CONTRACT" );
                continue;
            }

            player br_hud_hint( "[USE]  " + t.br_kind.label + " CONTRACT" );

            if ( !player usebuttonpressed() && !player istestclient() )
                continue;

            player br_hud_hint( "" );
            id = t.br_id;
            kind = t.br_kind;
            t delete();
            level thread br_contract_run( player, kind, id );
            return;
        }
    }
}

// The squad's marker, seen by it alone.
br_contract_marker( id, squad, origin )
{
    objective_position( id, origin );
    objective_setinvisibletoall( id );

    foreach ( p in br_squad_members( squad ) )
        objective_setvisibletoplayer( id, p );
}

br_contract_run( player, c, id )
{
    squad = player.br_squad;
    level.br_contract[squad] = c.key;
    level.br_contract_taker[squad] = player;
    player br_snd( "contract_tablet" );

    // (Most Wanted is the elite one: its own splash.)
    elite = "";

    if ( c.key == "mostwanted" )
        elite = "_elite";

    br_snd_squad( squad, "contract_take" + elite );
    br_vo_squad( squad, "vo_contract_take" );

    foreach ( p in br_squad_members( squad ) )
        p br_hud_flash( c.label + " CONTRACT" );

    ok = 0;

    switch ( c.key )
    {
        case "bounty":
            ok = br_bounty( squad, c, id );
            break;
        case "scavenger":
            ok = br_scavenger( squad, c, id );
            break;
        case "recon":
            ok = br_recon( squad, c, id );
            break;
        case "supplyrun":
            ok = br_supply_run( squad, c, id );
            break;
        case "mostwanted":
            ok = br_most_wanted( squad, c, id );
            break;
    }

    objective_delete( id );
    level.br_contract[squad] = undefined;
    level.br_contract_end[squad] = undefined;

    if ( isdefined( player ) )
        player br_snd( "biometric_close" );

    if ( level.br_over || br_squad_out( squad ) )
        return;

    foreach ( p in br_squad_members( squad ) )
    {
        if ( ok )
        {
            p.br_cash = p.br_cash + c.reward;
            p br_snd( "contract_reward" + elite );
            p br_snd( "reward_cash" );
            p br_vo( "vo_contract_done" );
            p br_hud_flash( "CONTRACT COMPLETE  +$" + c.reward );
        }
        else
        {
            p br_snd( "contract_failed" + elite );
            p br_vo( "vo_contract_failed" );
            p br_hud_flash( "CONTRACT FAILED" );
        }
    }
}

// Who leads the squad: a living human first, else a living bot.
br_squad_lead( squad )
{
    lead = undefined;

    foreach ( p in br_squad_members( squad ) )
    {
        if ( !isalive( p ) )
            continue;

        if ( !isdefined( lead ) || !p istestclient() )
            lead = p;
    }

    return lead;
}

// Bounty: the nearest living enemy is marked; kill him in time. A target
// who lives through it is paid.
br_bounty( squad, c, id )
{
    lead = br_squad_lead( squad );

    if ( !isdefined( lead ) )
        return 0;

    target = undefined;
    bestd = 1000000000;

    foreach ( p in level.players )
    {
        if ( !isalive( p ) || !isdefined( p.br_squad ) || p.br_squad == squad || br_squad_out( p.br_squad ) )
            continue;

        d = distancesquared( p.origin, lead.origin );

        if ( d < bestd )
        {
            bestd = d;
            target = p;
        }
    }

    if ( !isdefined( target ) )
        return 0;

    target.br_bounty_by = squad;
    level.br_bounty_hit[squad] = 0;
    br_vo_squad( squad, "vo_bounty_start" );
    br_snd_squad( target.br_squad, "bounty_marked" );
    br_vo_squad( target.br_squad, "vo_bounty_on_you" );
    target br_hud_flash( "THERE IS A BOUNTY ON YOU" );
    end = gettime() + c.time;

    while ( gettime() < end )
    {
        if ( br_squad_out( squad ) || level.br_over || !isdefined( target ) )
            return 0;

        if ( !isalive( target ) )
        {
            target.br_bounty_by = undefined;

            if ( level.br_bounty_hit[squad] )
            {
                br_snd_squad( squad, "target_down" );
                br_vo_squad( squad, "vo_bounty_done" );
                br_cut_redeploys( squad, level.br_bounty_cut_ms );
                return 1;
            }

            return 0;
        }

        br_contract_marker( id, squad, target.origin );
        br_contract_hurry( squad, end );
        wait 0.5;
    }

    target.br_bounty_by = undefined;

    foreach ( p in br_squad_members( target.br_squad ) )
    {
        p.br_cash = p.br_cash + level.br_bounty_live_cash;
        p br_vo( "vo_bounty_survived" );
        p br_snd( "reward_cash" );
    }

    br_vo_squad( squad, "vo_bounty_failed" );
    return 0;
}

// Scavenger: three supply boxes, each marked once the one before is open;
// the last one fills everyone's plates (the armor satchel).
br_scavenger( squad, c, id )
{
    br_vo_squad( squad, "vo_scav_start" );

    for ( n = 0; n < 3; n++ )
    {
        lead = br_squad_lead( squad );

        if ( !isdefined( lead ) )
            return 0;

        spot = br_spot_near( lead.origin, 2500 );

        if ( !isdefined( spot ) )
            return 0;

        if ( n > 0 )
            br_vo_squad( squad, "vo_scav_next" );

        br_contract_marker( id, squad, spot.origin );
        level.br_scav_open[squad] = 0;
        level thread br_cache( spot.origin, 1, squad );

        while ( !level.br_scav_open[squad] )
        {
            if ( br_squad_out( squad ) || level.br_over )
                return 0;

            wait 0.25;
        }
    }

    foreach ( p in br_squad_members( squad ) )
        p.br_plates_carried = level.br_plates_max;

    br_vo_squad( squad, "vo_scav_done" );
    return 1;
}

// Recon: hold a spot (more of the squad on it uploads faster); the next
// circle's middle is then shown to the squad for a while.
br_recon( squad, c, id )
{
    lead = br_squad_lead( squad );

    if ( !isdefined( lead ) )
        return 0;

    spot = br_spot_near( lead.origin, 2500 );

    if ( !isdefined( spot ) )
        return 0;

    br_contract_marker( id, squad, spot.origin );
    br_vo_squad( squad, "vo_recon_start" );
    done = 0;
    uploading = 0;

    while ( done < level.br_recon_ms )
    {
        if ( br_squad_out( squad ) || level.br_over )
            return 0;

        n = 0;

        foreach ( p in br_squad_members( squad ) )
        {
            if ( isalive( p ) && distance2d( p.origin, spot.origin ) < 160 )
            {
                n++;
                p br_hud_hint( "UPLOADING DATA  " + int( done * 100 / level.br_recon_ms ) + "%" );
            }
        }

        if ( n > 0 && !uploading )
        {
            uploading = 1;
            br_snd_squad( squad, "recon_upload" );
            br_vo_squad( squad, "vo_recon_upload" );
        }
        else if ( n == 0 && uploading )
        {
            uploading = 0;
            br_snd_squad( squad, "recon_exit" );
            br_vo_squad( squad, "vo_recon_lost" );
        }

        done = done + 250 * n;
        wait 0.25;
    }

    br_snd_squad( squad, "recon_exit" );

    foreach ( p in br_squad_members( squad ) )
        p br_hud_hint( "" );

    br_vo_squad( squad, "vo_recon_done" );
    next = level.br_phase + 1;

    if ( next < 0 )
        next = 0;

    if ( isdefined( level.br_plan ) && isdefined( level.br_plan[next] ) )
    {
        br_contract_marker( id, squad, level.br_plan[next].center );

        foreach ( p in br_squad_members( squad ) )
            p br_hud_flash( "NEXT CIRCLE MARKED" );

        wait 20;
    }

    return 1;
}

// Supply Run: reach the marked buy station in time; the squad's next
// purchase is cheaper.
br_supply_run( squad, c, id )
{
    lead = br_squad_lead( squad );

    if ( !isdefined( lead ) || level.br_stations.size == 0 )
        return 0;

    st = level.br_stations[0];
    bestd = 1000000000;

    foreach ( s in level.br_stations )
    {
        d = distancesquared( s.origin, lead.origin );

        if ( d < bestd && br_inside( s.origin, 0 ) )
        {
            bestd = d;
            st = s;
        }
    }

    br_contract_marker( id, squad, st.origin );
    end = gettime() + c.time;

    while ( gettime() < end )
    {
        if ( br_squad_out( squad ) || level.br_over )
            return 0;

        foreach ( p in br_squad_members( squad ) )
        {
            if ( isalive( p ) && distancesquared( p.origin, st.origin ) < 96 * 96 )
            {
                foreach ( m in br_squad_members( squad ) )
                    m.br_discount = level.br_discount;

                br_vo_squad( squad, "vo_supply_done" );
                return 1;
            }
        }

        br_contract_hurry( squad, end );
        wait 0.25;
    }

    return 0;
}

// Most Wanted: the squad is on everyone's map; when it lives through it,
// its dead come back.
br_most_wanted( squad, c, id )
{
    taker = level.br_contract_taker[squad];

    foreach ( p in br_squad_members( squad ) )
    {
        if ( isdefined( taker ) && p == taker )
            p br_vo( "vo_wanted_you" );
        else
            p br_vo( "vo_wanted_mate" );
    }

    foreach ( p in level.players )
    {
        if ( isdefined( p.br_squad ) && p.br_squad != squad )
            p br_vo( "vo_wanted_enemy" );
    }

    level.br_wanted = squad;
    objective_setvisibletoall( id );
    end = gettime() + c.time;

    while ( gettime() < end )
    {
        if ( br_squad_out( squad ) || level.br_over )
        {
            level.br_wanted = undefined;
            br_vo_squad( squad, "vo_wanted_failed" );
            return 0;
        }

        lead = br_squad_lead( squad );

        if ( isdefined( lead ) )
            objective_position( id, lead.origin );

        wait 0.5;
    }

    level.br_wanted = undefined;

    foreach ( p in br_squad_members( squad ) )
    {
        if ( isalive( p ) )
            continue;

        if ( p.br_pending )
            p.br_redeploy_at = gettime();
        else
        {
            p.br_out = 0;
            p br_respawn();
        }
    }

    br_vo_squad( squad, "vo_wanted_done" );
    return 1;
}

// Fifteen seconds left on a timed contract: said once.
br_contract_hurry( squad, end )
{
    level.br_contract_end[squad] = end;

    if ( end - gettime() > 15000 )
        return;

    if ( isdefined( level.br_hurried[squad] ) && level.br_hurried[squad] == end )
        return;

    level.br_hurried[squad] = end;
    br_vo_squad( squad, "vo_contract_hurry" );
}

// A kill and the contracts: a bounty's target, a most-wanted squad's man.
br_contract_kill( attacker, victim )
{
    if ( !isdefined( victim ) )
        return;

    if ( isdefined( victim.br_bounty_by ) && isdefined( attacker ) && isdefined( attacker.br_squad ) && attacker.br_squad == victim.br_bounty_by )
        level.br_bounty_hit[victim.br_bounty_by] = 1;

    if ( isdefined( level.br_wanted ) && victim.br_squad == level.br_wanted && isdefined( attacker ) && isdefined( attacker.br_squad ) && attacker.br_squad != level.br_wanted )
    {
        attacker.br_cash = attacker.br_cash + level.br_kill_cash;
        attacker br_snd( "target_down" );
        attacker br_snd( "reward_cash" );
        attacker br_vo( "vo_wanted_killed" );
    }
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

    if ( !isdefined( self.br_kills ) )
        self.br_kills = 0;

    // (The client's Warzone HUD draws the circle, squad, cash, plates, gas
    // and redeploy from `self.bo2mp_br`: these text lines stay hidden.)
    if ( getdvarint( "scr_br_wzhud" ) )
    {
        self.br_hud_circle.alpha = 0;
        self.br_hud_squad.alpha = 0;
        self.br_hud_cash.alpha = 0;
        self.br_hud_armor.alpha = 0;
        self.br_hud_note.y = 150;
        self.br_wzhud = 1;
    }

    self.br_ev_hit = 0;
    self.br_ev_break = 0;
    self.br_ev_kill = 0;
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

// The Warzone HUD's values (the client draws them; published to his
// client as `bo2mp_br`): `key=value` fields joined by "|", then a
// `m=<state>,<health %>,<armor>,<name>` field for each squadmate (state:
// a alive, r redeploying, d dead, o out).
br_hud_publish()
{
    if ( !isdefined( self.br_squad ) || !isdefined( level.br_phases ) )
        return;

    now = br_elapsed();
    st = "wait";
    ms = 0;

    if ( !isdefined( level.br_started ) )
        st = "lobby";
    else if ( level.br_phase < 0 )
    {
        st = "infil";
        ms = level.br_infil_ms - now;
    }
    else if ( level.br_phase >= level.br_phases.size )
        st = "final";
    else
    {
        p = level.br_phases[level.br_phase];

        if ( now < p.close_from )
            ms = p.close_from - now;
        else
        {
            st = "close";
            ms = p.closed - now;
        }
    }

    if ( ms < 0 )
        ms = 0;

    players = 0;

    foreach ( pl in level.players )
    {
        if ( isdefined( pl.br_squad ) && !br_squad_out( pl.br_squad ) && ( isalive( pl ) || pl.br_pending ) )
            players++;
    }

    redeploy = -1;

    if ( !isalive( self ) && self.br_pending && isdefined( self.br_redeploy_at ) )
        redeploy = self.br_redeploy_at - gettime();

    gas = 0;
    gas_d = 0;

    if ( isalive( self ) && level.br_phase >= 0 && isdefined( level.br_center ) && !br_inside( self.origin, 0 ) )
    {
        gas = 1;
        gas_d = int( distance2d( self.origin, level.br_center ) - level.br_radius );
    }

    contract = "";
    contract_ms = -1;

    if ( isdefined( level.br_contract[self.br_squad] ) )
    {
        foreach ( c in level.br_contract_kinds )
        {
            if ( c.key == level.br_contract[self.br_squad] )
                contract = c.label;
        }

        if ( isdefined( level.br_contract_end[self.br_squad] ) )
            contract_ms = level.br_contract_end[self.br_squad] - gettime();
    }

    circle = "";

    if ( isdefined( level.br_center ) && isdefined( level.br_radius ) )
    {
        circle = int( level.br_center[0] ) + "," + int( level.br_center[1] ) + "," + int( level.br_radius );

        if ( isdefined( level.br_next_center ) )
            circle = circle + "," + int( level.br_next_center[0] ) + "," + int( level.br_next_center[1] ) + "," + int( level.br_next_radius );
    }

    health = 0;

    if ( isalive( self ) && self.maxhealth > 0 )
        health = int( self.health * 100 / self.maxhealth );

    s = "v=1|st=" + st + "|ph=" + ( level.br_phase + 1 ) + "|np=" + level.br_phases.size + "|ms=" + ms;
    s = s + "|sq=" + br_squads_standing() + "|pl=" + players + "|my=" + ( self.br_squad + 1 );
    s = s + "|ca=" + self.br_cash + "|ki=" + self.br_kills + "|hp=" + health;
    s = s + "|ar=" + self.br_armor + "|ah=" + level.br_plate_hp + "|ps=" + level.br_plate_slots + "|pc=" + self.br_plates_carried + "|pm=" + level.br_plates_max;
    s = s + "|rd=" + br_redeploy_wait() + "|rt=" + redeploy + "|out=" + self.br_out + "|sr=" + self.br_selfrevive;
    s = s + "|gas=" + gas + "|gd=" + gas_d + "|mask=" + self.br_mask_on + "|mm=" + self.br_mask_ms;
    s = s + "|ct=" + contract + "|cm=" + contract_ms + "|circle=" + circle;
    s = s + "|eh=" + self.br_ev_hit + "|eb=" + self.br_ev_break + "|ek=" + self.br_ev_kill;

    if ( isdefined( self.br_ev_victim ) )
        s = s + "|ev=" + self.br_ev_victim;

    foreach ( mate in br_squad_members( self.br_squad ) )
    {
        if ( mate == self )
            continue;

        state = "a";

        if ( !isalive( mate ) )
        {
            if ( mate.br_pending )
                state = "r";
            else if ( mate.br_out || br_squad_out( mate.br_squad ) )
                state = "o";
            else
                state = "d";
        }

        mh = 0;

        if ( isalive( mate ) && mate.maxhealth > 0 )
            mh = int( mate.health * 100 / mate.maxhealth );

        s = s + "|m=" + state + "," + mh + "," + mate.br_armor + "," + mate.name;
    }

    self.bo2mp_br = s;
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

            if ( isdefined( self.br_wzhud ) )
                self.br_hud_gas.alpha = 0;
        }
        else
            self.br_hud_gas.alpha = 0;

        self br_hud_publish();

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
