// bo2mp: an optional M.O.A.B. (Modern Warfare 3's Massive Ordnance Air
// Blast) for Black Ops II multiplayer. OUR CODE, not the original: no
// Modern Warfare 3 script is readable on a PC install (they ship compiled),
// so it follows Modern Warfare 2's tactical nuke script (maps/mp/
// killstreaks/_nuke.gsc, read from MW2's common_mp: a ten second
// countdown with a ticking timer, the incoming sound 3.3 s before the
// blast, the blast's sound, the flash vision, the deaths 1.5 s after,
// then the aftermath vision) with Modern Warfare 3's rules: the match goes
// on, every enemy dies, and the enemy team is hit by an EMP (Black Ops
// II's own `_emp::emp_use`, whose aftermath already restores
// `level.nukevisionset`).
//
// Off unless `scr_moab 1`. Settings (dvars):
//   scr_moab              1 to allow it (default 0: native Black Ops II)
//   scr_moab_kills        kills in one life to earn it (default 25)
//   scr_moab_timer        countdown seconds (default 10)
//   scr_moab_emp          1: EMP the enemy team after the blast (default 1)
//   scr_moab_endsgame     1: the blast ends the match, as MW2's nuke (default 0)
// Earned once per kill streak; held until used: hold [Use] for a second.

main()
{
    if ( getdvarint( "scr_moab" ) == 0 )
        return;

    level.moab_kills = moab_dvar_int( "scr_moab_kills", 25 );
    level.moab_timer = moab_dvar_int( "scr_moab_timer", 10 );
    level.moab_emp = moab_dvar_int( "scr_moab_emp", 1 );
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

// Earned when this life's kills reach the count; used by holding [Use].
moab_watchstreak()
{
    self endon( "disconnect" );
    level endon( "game_ended" );
    self.moab_ready = 0;
    earned_at = -1;

    for (;;)
    {
        wait 0.25;

        if ( !isdefined( self.pers["cur_kill_streak"] ) )
            continue;

        streak = self.pers["cur_kill_streak"];

        if ( streak < earned_at )
            earned_at = -1;

        if ( !self.moab_ready && earned_at < 0 && streak >= level.moab_kills )
        {
            earned_at = streak;
            self.moab_ready = 1;
            self iprintlnbold( "M.O.A.B. READY: HOLD [USE] TO CALL IT IN" );
            self playlocalsound( "mpl_killstreak_emp_activate" );
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
            player iprintlnbold( "M.O.A.B. INBOUND" );
        else if ( level.teambased && player.pers["team"] == level.moab_team )
            player iprintlnbold( "FRIENDLY M.O.A.B. INBOUND" );
        else
            player iprintlnbold( "ENEMY M.O.A.B. INBOUND" );
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
        // (Modern Warfare's own when its sounds are installed, and Black
        // Ops II's biggest blast under it.)
        player playlocalsound( "nuke_explosion" );
        player playlocalsound( "nuke_wave" );
        player playlocalsound( "mpl_sd_exp_suitcase_bomb_main" );
        player playlocalsound( "mpl_killstreak_emp_blast_front" );
    }

    earthquake( 0.6, 5, owner.origin, 100000 );
    wait 0.25;
    visionsetnaked( "mpnuke", 3 );
    wait 1.25;
    moab_deaths( owner );
    level.nukedetonated = 1;
    visionsetnaked( "mpnuke_aftermath", 5 );

    if ( level.moab_endsgame )
    {
        if ( level.teambased )
            thread maps\mp\gametypes\_globallogic::endgame( level.moab_team, "M.O.A.B." );
        else if ( isdefined( owner ) )
            thread maps\mp\gametypes\_globallogic::endgame( owner, "M.O.A.B." );

        return;
    }

    if ( level.moab_emp && isdefined( owner ) )
        owner maps\mp\killstreaks\_emp::emp_use( 0 );

    level.moab_incoming = undefined;
}

moab_ticks()
{
    while ( !isdefined( level.moab_detonated ) )
    {
        foreach ( player in level.players )
        {
            player playlocalsound( "ui_mp_nukebomb_timer" );
            player playlocalsound( "mpl_sab_ui_suitcasebomb_timer" );
        }

        wait 1.0;
    }

    level.moab_detonated = undefined;
}

moab_incoming_sound( delay )
{
    if ( delay > 0 )
        wait delay;

    foreach ( player in level.players )
    {
        player playlocalsound( "nuke_incoming" );
        player playlocalsound( "mpl_lightning_bomb_incoming" );
    }
}

// Every enemy dies by the owner's hand (in free-for-all, everyone else).
moab_deaths( owner )
{
    foreach ( player in level.players )
    {
        if ( !isalive( player ) || player == owner )
            continue;

        if ( level.teambased && player.pers["team"] == level.moab_team )
            continue;

        player dodamage( player.health + 100000, player.origin, owner, owner, "none", "MOD_EXPLOSIVE" );
    }
}
