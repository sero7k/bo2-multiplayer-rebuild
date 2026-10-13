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
//   scr_moab_timer        countdown seconds (default 10)
//   scr_moab_endsgame     1: the blast ends the match, as MW2's (default 0)
// Earned once per kill streak; held until used: hold [Use] for a second.

main()
{
    if ( getdvarint( "scr_moab" ) == 0 )
        return;

    level.moab_kills = moab_dvar_int( "scr_moab_kills", 25 );
    level.moab_timer = moab_dvar_int( "scr_moab_timer", 10 );
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
            self iprintlnbold( "TACTICAL NUKE READY: HOLD [USE] TO CALL IT IN" );
            self moab_voice( "achieve_kem" );
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

    foreach ( player in level.players )
    {
        if ( player == owner )
        {
            player iprintlnbold( "TACTICAL NUKE INBOUND" );
            player moab_voice( "use_tnuke" );
        }
        else if ( level.teambased && player.pers["team"] == level.moab_team )
        {
            player iprintlnbold( "FRIENDLY TACTICAL NUKE INBOUND" );
            player moab_voice( "use_tnuke" );
        }
        else
        {
            player iprintlnbold( "ENEMY TACTICAL NUKE INBOUND" );
            player moab_voice( "enemy_tnuke" );
        }
    }

    timer = maps\mp\gametypes\_hud_util::createservertimer( "objective", 2.0 );
    timer maps\mp\gametypes\_hud_util::setpoint( "CENTER", "TOP", 0, 60 );
    timer.color = ( 1, 0.35, 0.1 );
    timer settimer( level.moab_timer );
    level thread moab_ticks();
    level thread moab_incoming_sound( level.moab_timer - 3.3 );
    wait level.moab_timer;
    timer destroy();
    level.moab_detonated = 1;

    foreach ( player in level.players )
    {
        player playlocalsound( "iw4:nuke_explosion" );
        player playlocalsound( "iw4:nuke_wave" );
        // (Black Ops II's biggest blast under it.)
        player playlocalsound( "mpl_sd_exp_suitcase_bomb_main" );
    }

    earthquake( 0.6, 10, owner.origin, 100000 );
    wait 0.25;
    visionsetnaked( "mpnuke", 3 );
    wait 1.25;
    moab_deaths( owner );
    level.nukedetonated = 1;
    visionsetnaked( "mpnuke_aftermath", 5 );

    if ( level.moab_endsgame )
    {
        wait 2;

        if ( level.teambased )
            thread maps\mp\gametypes\_globallogic::endgame( level.moab_team, "TACTICAL NUKE" );
        else if ( isdefined( owner ) )
            thread maps\mp\gametypes\_globallogic::endgame( owner, "TACTICAL NUKE" );

        return;
    }

    level.moab_incoming = undefined;
}

moab_ticks()
{
    while ( !isdefined( level.moab_detonated ) )
    {
        foreach ( player in level.players )
            player playlocalsound( "iw4:ui_mp_nukebomb_timer" );

        wait 1.0;
    }

    level.moab_detonated = undefined;
}

moab_incoming_sound( delay )
{
    if ( delay > 0 )
        wait delay;

    foreach ( player in level.players )
        player playlocalsound( "iw4:nuke_incoming" );
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
