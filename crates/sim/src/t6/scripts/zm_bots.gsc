// bo2mp: Custom Games' Zombies bots. OUR CODE (Black Ops II has no Zombies
// bots): teammates who play for the team, not just for kills.
//
// Each bot, every 0.2 s, picks the most pressing thing to do:
//   1. revive a downed teammate when no zombie is on top of him;
//   2. fight: the nearest zombie it can see - knifed in the first rounds
//      (ammo saved) and while Insta-Kill runs, shot otherwise - leaving
//      alone the zombies a player is gathering into a train (they are let
//      be unless they come for the bot);
//   3. rebuild a broken window nearby when the area is quiet;
//   4. spend its points: a perk (Juggernog first, then Quick Revive, Speed
//      Cola, Double Tap...) once the power is on, the Pack-a-Punch for its
//      gun, a door or debris near the team when it has points to spare;
//   5. otherwise stay near the host, a few steps off.
// It keeps a reserve of points so it can always revive and rebuild.
//
// The count comes from the lobby (`bo2mp_zm_bots`, 0-3; none by default).
// The bots join before the first round, as players of the host's team.

main()
{
    count = getdvarint( "bo2mp_zm_bots" );
    if ( count <= 0 )
        return;

    level.bo2mp_bot_perks = [];
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_armorvest";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_quickrevive";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_fastreload";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_rof";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_longersprint";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_additionalprimaryweapon";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_deadshot";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_flakjacket";
    level.bo2mp_bot_perks[ level.bo2mp_bot_perks.size ] = "specialty_grenadepulldeath";

    for ( i = 0; i < count; i++ )
    {
        bot = addtestclient();
        if ( !isdefined( bot ) )
            break;

        bot.pers[ "isBot" ] = 1;
        bot thread bot_life();
        wait 0.1;
    }
}

// What each perk costs (Black Ops II's prices; a solo Quick Revive is
// 500).
perk_cost( perk )
{
    switch ( perk )
    {
        case "specialty_armorvest":
            return 2500;
        case "specialty_quickrevive":
            if ( getplayers().size == 1 )
                return 500;
            return 1500;
        case "specialty_fastreload":
            return 3000;
        case "specialty_rof":
            return 2000;
        case "specialty_longersprint":
            return 2000;
        case "specialty_additionalprimaryweapon":
            return 4000;
        case "specialty_deadshot":
            return 1500;
        case "specialty_flakjacket":
            return 2000;
        case "specialty_grenadepulldeath":
            return 2000;
    }
    return 2500;
}

bot_life()
{
    self endon( "disconnect" );
    level endon( "end_game" );

    self.bot_reserve = 1000;
    self.bot_busy_until = 0;

    for (;;)
    {
        wait 0.2;

        if ( !isalive( self ) || self.sessionstate != "playing" )
            continue;

        // Downed: nothing to do but wait for help.
        if ( isdefined( self.revivetrigger ) )
        {
            self allowattack( 0 );
            continue;
        }

        if ( self bot_revive() )
            continue;

        if ( self bot_fight() )
            continue;

        if ( gettime() < self.bot_busy_until )
            continue;

        if ( self bot_rebuild() )
            continue;

        if ( self bot_spend() )
            continue;

        self bot_follow();
    }
}

// The zombies, alive.
zombies()
{
    team = "axis";
    if ( isdefined( level.zombie_team ) )
        team = level.zombie_team;

    return getaispeciesarray( team, "all" );
}

// The nearest zombie to a point, and how far (squared): [ zombie, d2 ].
nearest_zombie( origin, zs )
{
    best = undefined;
    best_d = 99999999;
    foreach ( z in zs )
    {
        if ( !isalive( z ) )
            continue;

        d = distancesquared( origin, z.origin );
        if ( d < best_d )
        {
            best_d = d;
            best = z;
        }
    }
    out = [];
    out[ 0 ] = best;
    out[ 1 ] = best_d;
    return out;
}

// 1. Revive.
bot_revive()
{
    zs = zombies();
    target = undefined;
    target_d = 99999999;
    foreach ( p in getplayers() )
    {
        if ( p == self || !isdefined( p.revivetrigger ) )
            continue;

        // (Not into a crowd: a zombie on him first.)
        near = nearest_zombie( p.origin, zs );
        if ( isdefined( near[ 0 ] ) && near[ 1 ] < 150 * 150 )
            continue;

        d = distancesquared( self.origin, p.origin );
        if ( d < target_d )
        {
            target_d = d;
            target = p;
        }
    }
    if ( !isdefined( target ) )
    {
        self cancelgoal( "revive" );
        return 0;
    }

    self allowattack( 0 );
    self addgoal( target.origin, 32, 4, "revive" );
    if ( target_d < 70 * 70 )
    {
        self lookat( target.origin + ( 0, 0, 20 ) );
        self pressusebutton( 0.5 );
    }
    return 1;
}

// A zombie a player is gathering into a train: one of five or more after
// the same player, out in the open, not near the bot.
trained( z, zs )
{
    if ( !isdefined( z.favoriteenemy ) || !isplayer( z.favoriteenemy ) || z.favoriteenemy == self || z.favoriteenemy istestclient() )
        return 0;

    if ( distancesquared( z.origin, self.origin ) < 250 * 250 )
        return 0;

    n = 0;
    foreach ( other in zs )
    {
        if ( isalive( other ) && isdefined( other.favoriteenemy ) && other.favoriteenemy == z.favoriteenemy )
            n++;
    }
    return n >= 5;
}

instakill_on()
{
    if ( !isdefined( level.zombie_vars ) )
        return 0;

    team = self.team;
    if ( isdefined( team ) && isdefined( level.zombie_vars[ team ] ) && isdefined( level.zombie_vars[ team ][ "zombie_insta_kill" ] ) )
        return level.zombie_vars[ team ][ "zombie_insta_kill" ];

    if ( isdefined( level.zombie_vars[ "zombie_insta_kill" ] ) )
        return level.zombie_vars[ "zombie_insta_kill" ];

    return 0;
}

// 2. Fight.
bot_fight()
{
    zs = zombies();
    eye = self geteye();
    best = undefined;
    best_d = 1100 * 1100;
    foreach ( z in zs )
    {
        if ( !isalive( z ) )
            continue;

        d = distancesquared( self.origin, z.origin );
        if ( d >= best_d )
            continue;

        if ( self trained( z, zs ) )
            continue;

        if ( !sighttracepassed( eye, z.origin + ( 0, 0, 50 ), 0, z ) )
            continue;

        best = z;
        best_d = d;
    }
    if ( !isdefined( best ) )
    {
        self allowattack( 0 );
        self clearlookat();
        self cancelgoal( "fight" );
        return 0;
    }

    head = best.origin + ( 0, 0, 58 );
    self lookat( head );
    round = 1;
    if ( isdefined( level.round_number ) )
        round = level.round_number;

    knife = ( round <= 2 || self instakill_on() ) && best_d < 300 * 300;
    if ( knife )
    {
        // Close in and knife (the first rounds save ammo; Insta-Kill makes
        // every knife a kill).
        self allowattack( 0 );
        self addgoal( best.origin, 40, 3, "fight" );
        if ( best_d < 72 * 72 )
            self pressmelee();
        return 1;
    }

    self cancelgoal( "fight" );
    self allowattack( 1 );
    self pressattackbutton( 2 );
    // (Backs off a zombie on top of it.)
    if ( best_d < 90 * 90 )
    {
        away = vectornormalize( self.origin - best.origin );
        self addgoal( self.origin + away * 150, 32, 3, "fight" );
    }
    return 1;
}

// 3. Rebuild a window near it when the area is quiet.
bot_rebuild()
{
    if ( !isdefined( level.exterior_goals ) )
        return 0;

    near = nearest_zombie( self.origin, zombies() );
    if ( isdefined( near[ 0 ] ) && near[ 1 ] < 500 * 500 )
        return 0;

    foreach ( g in level.exterior_goals )
    {
        if ( !isdefined( g.zbarrier ) || distancesquared( self.origin, g.origin ) > 600 * 600 )
            continue;

        broken = 0;
        pieces = g.zbarrier getnumzbarrierpieces();
        for ( i = 0; i < pieces; i++ )
        {
            if ( g.zbarrier getzbarrierpiecestate( i ) != "closed" )
            {
                broken = 1;
                break;
            }
        }
        if ( !broken )
            continue;

        self allowattack( 0 );
        self addgoal( g.origin, 40, 2, "rebuild" );
        if ( distancesquared( self.origin, g.origin ) < 80 * 80 )
        {
            self lookat( g.zbarrier.origin + ( 0, 0, 40 ) );
            self pressusebutton( 0.6 );
        }
        return 1;
    }
    self cancelgoal( "rebuild" );
    return 0;
}

power_on()
{
    return isdefined( level.flag ) && isdefined( level.flag[ "power_on" ] ) && level.flag[ "power_on" ];
}

// Walks to a trigger and holds Use on it; busy for a few seconds after.
buy_at( trig )
{
    self allowattack( 0 );
    self addgoal( trig.origin, 36, 2, "buy" );
    if ( distancesquared( self.origin, trig.origin ) < 90 * 90 )
    {
        self lookat( trig.origin + ( 0, 0, 40 ) );
        self pressusebutton( 0.3 );
        self.bot_busy_until = gettime() + 2500;
        self cancelgoal( "buy" );
    }
    return 1;
}

// Whether the Pack-a-Punch takes this gun: a custom gun from the box only
// with its upgraded weapon (bo2mp/zm_guest_guns); the map's own as before.
can_upgrade( gun )
{
    if ( !isdefined( level.bo2mp_guest_gun ) || !isdefined( level.bo2mp_guest_gun[ gun ] ) )
        return 1;

    if ( !isdefined( level.zombie_weapons ) || !isdefined( level.zombie_weapons[ gun ] ) )
        return 0;

    return isdefined( level.zombie_weapons[ gun ].upgrade_name );
}

// 4. Spend points: a perk, the Pack-a-Punch, a door near the team.
bot_spend()
{
    near = nearest_zombie( self.origin, zombies() );
    if ( isdefined( near[ 0 ] ) && near[ 1 ] < 600 * 600 )
        return 0;

    score = self.score;
    if ( !isdefined( score ) )
        return 0;

    if ( power_on() || getdvar( "mapname" ) == "zm_nuked" )
    {
        machines = getentarray( "zombie_vending", "targetname" );
        foreach ( perk in level.bo2mp_bot_perks )
        {
            if ( self hasperk( perk ) )
                continue;

            if ( score < perk_cost( perk ) + 500 )
                break;

            foreach ( m in machines )
            {
                if ( isdefined( m.script_noteworthy ) && m.script_noteworthy == perk && distancesquared( self.origin, m.origin ) < 2500 * 2500 )
                    return self buy_at( m );
            }
        }

        // The Pack-a-Punch, for a gun not yet upgraded.
        if ( score >= 5000 + self.bot_reserve )
        {
            gun = self getcurrentweapon();
            // (A custom gun from the box is upgraded as the map's own are,
            // unless it has no upgraded weapon.)
            if ( isdefined( gun ) && !issubstr( gun, "upgraded" ) && can_upgrade( gun ) )
            {
                pap = getentarray( "specialty_weapupgrade", "script_noteworthy" );
                foreach ( m in pap )
                {
                    if ( distancesquared( self.origin, m.origin ) < 2500 * 2500 )
                        return self buy_at( m );
                }
            }
        }
    }

    // A door or debris the team stands at, with points to spare.
    host = host_player();
    if ( isdefined( host ) && isdefined( level.round_number ) && level.round_number >= 3 )
    {
        names = [];
        names[ 0 ] = "zombie_door";
        names[ 1 ] = "zombie_debris";
        names[ 2 ] = "zombie_airlock_buy";
        foreach ( name in names )
        {
            foreach ( d in getentarray( name, "targetname" ) )
            {
                cost = d.zombie_cost;
                if ( !isdefined( cost ) || score < cost + 1500 )
                    continue;

                if ( distancesquared( d.origin, host.origin ) > 500 * 500 )
                    continue;

                return self buy_at( d );
            }
        }
    }
    return 0;
}

// The first player who is not a bot (the host).
host_player()
{
    foreach ( p in getplayers() )
    {
        if ( !p istestclient() )
            return p;
    }
    return undefined;
}

// 5. Near the host, a few steps off.
bot_follow()
{
    self allowattack( 0 );
    self clearlookat();
    host = host_player();
    if ( !isdefined( host ) || host == self )
        return;

    if ( distancesquared( self.origin, host.origin ) > 260 * 260 )
        self addgoal( host.origin, 180, 1, "follow" );
    else
        self cancelgoal( "follow" );
}
