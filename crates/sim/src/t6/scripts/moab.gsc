// bo2mp: an optional Tactical Nuke (Modern Warfare 2's) for Black Ops II
// multiplayer, called in with Ghosts' KEM Strike voice lines. OUR CODE,
// after Modern Warfare 2's maps/mp/killstreaks/_nuke.gsc (read from MW2's
// common_mp): a ten second countdown with its ticking timer, the incoming
// sound 3.3 s before the blast, the blast and its shock wave, the flash
// vision, every enemy dead 1.5 s after (by the caller's hand) and the
// aftermath vision. The match goes on.
//
// Earned by kills in one life, killstreak kills too (as MW2 counts them);
// its own kills count toward nothing: not the next nuke, not his
// scorestreaks.
//
// (The script keeps its old name, `bo2mp/moab`, and its dvars.)
// Off unless `scr_moab 1`. Settings (dvars):
//   scr_moab              1 to allow it (default 0: native Black Ops II)
//   scr_moab_kills        kills in one life to earn it (default 25)
//   scr_moab_timer        countdown seconds (default 7)
//   scr_moab_endsgame     1: the blast ends the match, as MW2's (default 0)
// Earned once per kill streak; held until used: hold [Use] for a second.

main()
{
    if ( getdvarint( "scr_moab" ) == 0 )
        return;

    level.moab_kills = moab_dvar_int( "scr_moab_kills", 25 );
    level.moab_timer = moab_dvar_int( "scr_moab_timer", 7 );
    level.moab_endsgame = moab_dvar_int( "scr_moab_endsgame", 0 );
    level.nukevisionset = "mpnuke_aftermath";
    level thread moab_onconnect();

    foreach ( player in level.players )
        player thread moab_watchstreak();
}

moab_dvar_int( name, fallback )
{
    if ( getdvar( name ) == "" )
        return fallback;

    return getdvarint( name );
}

moab_onconnect()
{
    level endon( "game_ended" );

    for (;;)
    {
        level waittill( "connected", player );
        player thread moab_watchstreak();
    }
}

// Ghosts' announcer (its KEM Strike lines), else Modern Warfare 2's.
moab_voice( line )
{
    self playlocalsound( "iw6:US_1mc_" + line );
}

// This life's kills: every kill since he spawned (killstreak kills too),
// less what a nuke of his killed.
moab_life_kills()
{
    if ( !isdefined( self.pers["kills"] ) || !isdefined( self.moab_base ) )
        return 0;

    return self.pers["kills"] - self.moab_base;
}

moab_onspawn()
{
    self endon( "disconnect" );
    level endon( "game_ended" );

    for (;;)
    {
        self waittill( "spawned_player" );
        self.moab_base = 0;

        if ( isdefined( self.pers["kills"] ) )
            self.moab_base = self.pers["kills"];

        self.moab_earned = 0;
    }
}

// Earned when this life's kills reach the count; used by holding [Use].
moab_watchstreak()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    self.moab_ready = 0;
    self.moab_earned = 0;
    self.moab_base = 0;

    if ( isdefined( self.pers["kills"] ) )
        self.moab_base = self.pers["kills"];

    self thread moab_onspawn();

    for (;;)
    {
        wait 0.25;

        if ( !self.moab_ready && !self.moab_earned && isalive( self ) && self moab_life_kills() >= level.moab_kills )
        {
            self.moab_earned = 1;
            self.moab_ready = 1;
        }

        // (Earned, or given: `give killstreak nuke` in the console.)
        if ( self.moab_ready && !isdefined( self.moab_announced ) )
        {
            self.moab_announced = 1;
            self iprintlnbold( "TACTICAL NUKE READY: HOLD [USE] TO CALL IT IN" );
            self moab_voice( "achieve_kem" );
        }

        // (`give killstreak nuke` while he has one: called in at once.)
        if ( self.moab_ready && isalive( self ) && isdefined( self.moab_call_now ) && !isdefined( level.moab_incoming ) )
        {
            self.moab_call_now = undefined;
            self.moab_ready = 0;
            self.moab_announced = undefined;
            level thread moab_launch( self );
            continue;
        }

        if ( self.moab_ready && isalive( self ) && self usebuttonpressed() )
        {
            held = 0;

            while ( held < 1.0 && isalive( self ) && self usebuttonpressed() )
            {
                wait 0.05;
                held = held + 0.05;
            }

            if ( held >= 1.0 && !isdefined( level.moab_incoming ) )
            {
                self.moab_ready = 0;
                self.moab_announced = undefined;
                level thread moab_launch( self );
            }
        }
    }
}

moab_launch( owner )
{
    level.moab_incoming = 1;
    level.moab_owner = owner;
    level.moab_team = owner.pers["team"];
    owner notify( "used_moab" );

    // (The HUD's KEM STRIKE INBOUND banner: who called it, when, for how
    // long; the score bar and clock hide while it is up.)
    level.moab_banner = "" + owner.name;
    level.moab_banner_id = gettime();
    level.moab_banner_ms = level.moab_timer * 1000;

    foreach ( player in level.players )
    {
        if ( player == owner || ( level.teambased && player.pers["team"] == level.moab_team ) )
            player moab_voice( "use_tnuke" );
        else
            player moab_voice( "enemy_tnuke" );
    }

    foreach ( player in level.players )
        player playlocalsound( "iw6:kem_launch" );

    level thread moab_ticks();
    level thread moab_incoming_sound( level.moab_timer - 3.3 );
    wait level.moab_timer;
    level.moab_detonated = 1;

    foreach ( player in level.players )
    {
        player playlocalsound( "iw6:kem_explosion" );
        player playlocalsound( "iw6:kem_explosion_boom" );
        player playlocalsound( "iw6:kem_explosion_debris" );
        player playlocalsound( "iw4:nuke_wave" );
        // (Black Ops II's biggest blast under it.)
        player playlocalsound( "mpl_sd_exp_suitcase_bomb_main" );
    }

    // (Ghosts' KEM: the light has swelled for the last seconds; the
    // blast goes all white, the enemies die in it, and the white burns down
    // to the dark after.)
    earthquake( 0.7, 6, owner.origin, 100000 );
    visionsetnaked( "kem_white", 0 );
    wait 1.2;
    moab_deaths( owner );
    level.nukedetonated = 1;

    for ( k = 1; k <= 8; k++ )
    {
        visionsetnaked( "kem_burn_" + k, 0 );
        wait 0.3;
    }
    level thread moab_vision_back();

    if ( level.moab_endsgame )
    {
        wait 2;
        level.moab_banner = undefined;

        if ( level.teambased )
            thread maps\mp\gametypes\_globallogic::endgame( level.moab_team, "TACTICAL NUKE" );
        else if ( isdefined( owner ) )
            thread maps\mp\gametypes\_globallogic::endgame( owner, "TACTICAL NUKE" );

        return;
    }

    // (The score bar and clock come back once the blast has settled.)
    wait 4;
    level.moab_banner = undefined;
    level.moab_incoming = undefined;
}

moab_ticks()
{
    // (Ghosts' KEM Strike alarm, a 1.3 s sound, back to back.)
    while ( !isdefined( level.moab_detonated ) )
    {
        foreach ( player in level.players )
            player playlocalsound( "iw6:ui_mp_kem_timer" );

        wait 1.3;
    }

    level.moab_detonated = undefined;
}

moab_incoming_sound( delay )
{
    if ( delay > 0 )
        wait delay;

    foreach ( player in level.players )
        player playlocalsound( "iw6:kem_incoming" );

    // (The glare swells over the last seconds, the ground shaking more.)
    earthquake( 0.15, 3.3, level.moab_owner.origin, 100000 );

    for ( k = 1; k <= 8; k++ )
    {
        visionsetnaked( "kem_glare_" + k, 0 );
        wait 0.4;
    }
}

// The map's own look again once the dust has settled.
moab_vision_back()
{
    level endon( "game_ended" );
    wait 20;

    if ( isdefined( level.moab_incoming ) )
        return;

    visionsetnaked( getdvar( "mapname" ), 8 );
}

// Every enemy dies by the caller's hand (in free-for-all, everyone else).
// The kills count toward nothing: his kill count for the next nuke, his
// kill streak and his scorestreak momentum are put back as they were.
moab_deaths( owner )
{
    kills = undefined;
    streak = undefined;
    momentum = undefined;

    if ( isdefined( owner ) )
    {
        kills = owner.pers["kills"];
        streak = owner.pers["cur_kill_streak"];
        momentum = owner.pers["momentum"];
    }

    foreach ( player in level.players )
    {
        if ( !isalive( player ) || player == owner )
            continue;

        if ( level.teambased && player.pers["team"] == level.moab_team )
            continue;

        player dodamage( player.health + 100000, player.origin, owner, owner, "none", "MOD_EXPLOSIVE" );
    }

    wait 0.1;

    if ( !isdefined( owner ) )
        return;

    if ( isdefined( kills ) && isdefined( owner.pers["kills"] ) && isdefined( owner.moab_base ) )
        owner.moab_base = owner.moab_base + ( owner.pers["kills"] - kills );

    if ( isdefined( streak ) )
        owner.pers["cur_kill_streak"] = streak;

    if ( isdefined( momentum ) )
        owner.pers["momentum"] = momentum;
}
